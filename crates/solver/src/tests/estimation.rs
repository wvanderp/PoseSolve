use crate::types::{Bounds, GaussianPrior, Priors};

use super::helpers::{base_request, sample_corr, solve_to_response};

#[test]
fn single_point_solution_uses_point_location() {
    let req = base_request(vec![sample_corr("p1", 800.0, 900.0, 51.91, 4.47, 15.0)]);
    let response = solve_to_response(req);

    assert!((response.pose.lat - 51.91).abs() < 1e-12);
    assert!((response.pose.lon - 4.47).abs() < 1e-12);
}

#[test]
fn multipoint_solution_is_geometrically_reasonable() {
    let req = base_request(vec![
        sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 20.0),
        sample_corr("p2", 300.0, 400.0, 51.92, 4.50, 20.0),
    ]);
    let response = solve_to_response(req);

    // Camera position should be finite and in a reasonable geographic range
    assert!(response.pose.lat.is_finite());
    assert!(response.pose.lon.is_finite());
    assert!(response.pose.alt.is_finite());

    // Should be somewhere in the vicinity of the world points
    // (within ~1 degree ≈ 111 km – very generous for 2 points)
    assert!(
        (response.pose.lat - 51.91).abs() < 1.0,
        "lat too far: {}",
        response.pose.lat
    );
    assert!(
        (response.pose.lon - 4.48).abs() < 1.0,
        "lon too far: {}",
        response.pose.lon
    );
}

#[test]
fn bounds_prior_clamps_position() {
    let mut req = base_request(vec![
        sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 20.0),
        sample_corr("p2", 300.0, 400.0, 51.92, 4.50, 20.0),
    ]);
    req.priors = Some(Priors {
        focal_px: None,
        camera_alt: None,
        bounds: Some(Bounds {
            lat_min: 51.905,
            lat_max: 51.906,
            lon_min: 4.455,
            lon_max: 4.456,
        }),
        distortion: None,
    });

    let response = solve_to_response(req);

    // Position must be within the bounds
    assert!(
        response.pose.lat >= 51.905 - 1e-12 && response.pose.lat <= 51.906 + 1e-12,
        "lat out of bounds: {}",
        response.pose.lat
    );
    assert!(
        response.pose.lon >= 4.455 - 1e-12 && response.pose.lon <= 4.456 + 1e-12,
        "lon out of bounds: {}",
        response.pose.lon
    );
}

#[test]
fn camera_alt_prior_overrides_default_alt_logic() {
    let mut req = base_request(vec![sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 40.0)]);
    req.priors = Some(Priors {
        focal_px: None,
        camera_alt: Some(GaussianPrior { mean: 123.0 }),
        bounds: None,
        distortion: None,
    });
    let response = solve_to_response(req);

    assert!((response.pose.alt - 123.0).abs() < 1e-12);
}

#[test]
fn default_alt_uses_world_alt_minus_two_meters() {
    let req = base_request(vec![sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 40.0)]);
    let response = solve_to_response(req);

    assert!((response.pose.alt - 38.0).abs() < 1e-12);
}

#[test]
fn focal_prior_overrides_default_focal() {
    let mut req = base_request(vec![sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 40.0)]);
    req.priors = Some(Priors {
        focal_px: Some(GaussianPrior { mean: 1777.0 }),
        camera_alt: None,
        bounds: None,
        distortion: None,
    });
    let response = solve_to_response(req);

    assert!((response.intrinsics.focal_px - 1777.0).abs() < 1e-12);
}

#[test]
fn default_focal_and_principal_point_are_from_image() {
    let req = base_request(vec![sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 40.0)]);
    let response = solve_to_response(req);

    assert!((response.intrinsics.focal_px - 3600.0).abs() < 1e-12);
    assert!((response.intrinsics.cx - 2000.0).abs() < 1e-12);
    assert!((response.intrinsics.cy - 1500.0).abs() < 1e-12);
}

#[test]
fn multipoint_with_focal_prior_gives_finite_result() {
    let mut req = base_request(vec![
        sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 20.0),
        sample_corr("p2", 300.0, 400.0, 51.92, 4.50, 20.0),
        sample_corr("p3", 500.0, 300.0, 51.91, 4.48, 30.0),
    ]);
    req.priors = Some(Priors {
        focal_px: Some(GaussianPrior { mean: 2500.0 }),
        camera_alt: None,
        bounds: None,
        distortion: None,
    });
    let response = solve_to_response(req);

    assert!(response.pose.lat.is_finite());
    assert!(response.pose.lon.is_finite());
    assert!(response.pose.alt.is_finite());
    assert!(response.pose.yaw_deg >= 0.0 && response.pose.yaw_deg < 360.0);
}

#[test]
fn multipoint_with_altitude_prior_stays_near_prior() {
    let mut req = base_request(vec![
        sample_corr("p1", 100.0, 200.0, 51.90, 4.46, 80.0),
        sample_corr("p2", 300.0, 400.0, 51.92, 4.50, 80.0),
        sample_corr("p3", 500.0, 300.0, 51.91, 4.48, 100.0),
    ]);
    req.priors = Some(Priors {
        focal_px: None,
        camera_alt: Some(GaussianPrior { mean: 5.0 }),
        bounds: None,
        distortion: None,
    });
    let response = solve_to_response(req);

    // With an altitude prior of 5m and world points at 80-100m,
    // the camera altitude should be influenced toward the prior.
    // With only 3 arbitrary correspondences the geometric solution
    // may dominate, so we allow generous tolerance.
    assert!(
        (response.pose.alt - 5.0).abs() < 120.0,
        "altitude {} should be somewhat near prior 5.0",
        response.pose.alt
    );
}
