use crate::reproject_points;

use super::helpers::{base_request, sample_corr, solve_to_response};

#[test]
fn diagnostics_include_only_enabled_points() {
    let mut p2 = sample_corr("p2", 300.0, 400.0, 51.92, 4.50, 20.0);
    p2.enabled = Some(false);
    let req = base_request(vec![
        sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 20.0),
        p2,
        sample_corr("p3", 500.0, 600.0, 51.91, 4.48, 20.0),
    ]);

    let response = solve_to_response(req);

    assert_eq!(response.diagnostics.inlier_ids.len(), 2);
    assert!(response.diagnostics.inlier_ids.contains(&"p1".to_string()));
    assert!(response.diagnostics.inlier_ids.contains(&"p3".to_string()));
    assert_eq!(response.diagnostics.residuals_px.len(), 2);
    assert!(response.diagnostics.inlier_ratio > 0.0);
}

#[test]
fn tiny_sigma_is_clamped_and_residuals_remain_finite() {
    let mut p1 = sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 20.0);
    p1.pixel.sigma_px = Some(0.0);
    let req = base_request(vec![
        p1,
        sample_corr("p2", 300.0, 400.0, 51.92, 4.50, 20.0),
    ]);

    let response = solve_to_response(req);
    assert!(response.diagnostics.rmse_px.is_finite());
    assert!(response.diagnostics.residuals_px.iter().all(|v| v.is_finite()));
}

#[test]
fn covariance_shape_and_labels_are_stable() {
    let req = base_request(vec![sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 20.0)]);
    let response = solve_to_response(req);

    assert_eq!(response.covariance.labels.len(), 10);
    assert_eq!(response.covariance.matrix.len(), 100);
    assert_eq!(response.covariance.labels[0], "lat");
    assert_eq!(response.covariance.labels[1], "lon");
}

#[test]
fn pose_orientation_yaw_range_and_finite_angles() {
    let req = base_request(vec![
        sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 20.0),
        sample_corr("p2", 300.0, 400.0, 51.92, 4.50, 20.0),
    ]);
    let response = solve_to_response(req);

    assert!(response.pose.yaw_deg >= 0.0 && response.pose.yaw_deg < 360.0,
        "yaw should be in [0,360): {}", response.pose.yaw_deg);
    assert!(response.pose.pitch_deg.is_finite(),
        "pitch should be finite: {}", response.pose.pitch_deg);
    assert!(response.pose.roll_deg.is_finite(),
        "roll should be finite: {}", response.pose.roll_deg);
    // Pitch should be in a reasonable range (−90°..90°)
    assert!(response.pose.pitch_deg.abs() < 90.0,
        "pitch should be < 90°: {}", response.pose.pitch_deg);
    // Roll should be reasonable (regularised toward 0)
    assert!(response.pose.roll_deg.abs() < 90.0,
        "roll should be < 90°: {}", response.pose.roll_deg);
}

#[test]
fn single_point_warns_about_single_correspondence() {
    let req = base_request(vec![sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 20.0)]);
    let response = solve_to_response(req);

    assert!(response
        .diagnostics
        .warnings
        .iter()
        .any(|w| w.contains("one correspondence")),
        "expected single-point warning, got {:?}", response.diagnostics.warnings);
}

#[test]
fn two_point_warns_about_underdetermined() {
    let req = base_request(vec![
        sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 20.0),
        sample_corr("p2", 300.0, 400.0, 51.92, 4.50, 20.0),
    ]);
    let response = solve_to_response(req);

    assert!(response
        .diagnostics
        .warnings
        .iter()
        .any(|w| w.contains("underdetermined")),
        "expected underdetermined warning, got {:?}", response.diagnostics.warnings);
}

#[test]
fn reproject_points_returns_not_implemented_warning() {
    let out = reproject_points("{}".to_string()).expect("reproject must succeed");
    let value: serde_json::Value = serde_json::from_str(&out).expect("output must be json");
    assert_eq!(value["pixels"], serde_json::json!([]));
    assert_eq!(value["warnings"][0], serde_json::json!("Not implemented yet"));
}
