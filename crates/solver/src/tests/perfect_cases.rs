use crate::geo::lla_to_enu;
use crate::projection::{project_point, rotation_enu_to_cam, CameraIntrinsics};
use crate::solve_impl;
use crate::types::{Corr, GaussianPrior, Image, Pixel, Priors, SolveRequest, WorldLla};

use super::helpers::haversine_m;

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Generate a full-pipeline `SolveRequest` with **perfect** synthetic data.
///
/// Given a known camera pose (lat/lon/alt + yaw/pitch/roll), camera intrinsics,
/// and a set of world points (lat/lon/alt), this function:
///   1. Converts everything to a local ENU frame (centroid of world points as reference).
///   2. Forward-projects each world point through the camera model to get exact pixel coords.
///   3. Builds a `SolveRequest` with a matching focal-length prior.
///
/// The returned pixel coordinates are mathematically perfect — zero noise.
fn build_perfect_request(
    cam_lat: f64,
    cam_lon: f64,
    cam_alt: f64,
    yaw_deg: f64,
    pitch_deg: f64,
    roll_deg: f64,
    focal_px: f64,
    image_width: f64,
    image_height: f64,
    world_pts: &[(f64, f64, f64)], // (lat, lon, alt)
) -> SolveRequest {
    let cx = image_width / 2.0;
    let cy = image_height / 2.0;
    let intr = CameraIntrinsics {
        focal_px,
        cx,
        cy,
        k1: 0.0,
        k2: 0.0,
        p1: 0.0,
        p2: 0.0,
    };

    // Reference point = centroid of world points (mirrors solve_impl behaviour)
    let n = world_pts.len() as f64;
    let ref_lat = world_pts.iter().map(|p| p.0).sum::<f64>() / n;
    let ref_lon = world_pts.iter().map(|p| p.1).sum::<f64>() / n;
    let ref_alt = world_pts.iter().map(|p| p.2).sum::<f64>() / n;

    // Camera in ENU
    let cam_enu = lla_to_enu(cam_lat, cam_lon, cam_alt, ref_lat, ref_lon, ref_alt);
    let rot = rotation_enu_to_cam(yaw_deg, pitch_deg, roll_deg);

    // Project each world point → pixel
    let correspondences: Vec<Corr> = world_pts
        .iter()
        .enumerate()
        .filter_map(|(i, &(lat, lon, alt))| {
            let enu = lla_to_enu(lat, lon, alt, ref_lat, ref_lon, ref_alt);
            project_point(enu, cam_enu, &rot, &intr).map(|(u, v)| Corr {
                id: format!("pt_{}", i),
                enabled: Some(true),
                pixel: Pixel {
                    u,
                    v,
                    sigma_px: Some(1.0),
                },
                world: WorldLla {
                    lat,
                    lon,
                    alt: Some(alt),
                },
            })
        })
        .collect();

    assert!(
        correspondences.len() == world_pts.len(),
        "Not all world points are visible to the camera ({} of {}). \
         Adjust the scene geometry.",
        correspondences.len(),
        world_pts.len(),
    );

    SolveRequest {
        image: Image {
            width: image_width,
            height: image_height,
        },
        correspondences,
        priors: Some(Priors {
            focal_px: Some(GaussianPrior { mean: focal_px }),
            camera_alt: None,
            bounds: None,
            distortion: None,
        }),
    }
}

// ── Assertion helpers ───────────────────────────────────────────────────────

fn assert_pose_close(
    response: &crate::types::SolveResponse,
    expected_lat: f64,
    expected_lon: f64,
    expected_alt: f64,
    expected_yaw: f64,
    expected_pitch: f64,
    expected_roll: f64,
    pos_tol_m: f64,
    angle_tol_deg: f64,
    label: &str,
) {
    let dist = haversine_m(response.pose.lat, response.pose.lon, expected_lat, expected_lon);
    assert!(
        dist <= pos_tol_m,
        "[{}] Position off by {:.2} m (tolerance {:.0} m). \
         Expected ({}, {}), got ({}, {})",
        label, dist, pos_tol_m,
        expected_lat, expected_lon,
        response.pose.lat, response.pose.lon,
    );

    let alt_err = (response.pose.alt - expected_alt).abs();
    assert!(
        alt_err <= pos_tol_m,
        "[{}] Altitude off by {:.2} m (tolerance {:.0} m). \
         Expected {}, got {}",
        label, alt_err, pos_tol_m,
        expected_alt, response.pose.alt,
    );

    let yaw_err = angular_distance(response.pose.yaw_deg, expected_yaw);
    assert!(
        yaw_err <= angle_tol_deg,
        "[{}] Yaw off by {:.2}° (tolerance {:.0}°). Expected {}, got {}",
        label, yaw_err, angle_tol_deg,
        expected_yaw, response.pose.yaw_deg,
    );

    let pitch_err = (response.pose.pitch_deg - expected_pitch).abs();
    assert!(
        pitch_err <= angle_tol_deg,
        "[{}] Pitch off by {:.2}° (tolerance {:.0}°). Expected {}, got {}",
        label, pitch_err, angle_tol_deg,
        expected_pitch, response.pose.pitch_deg,
    );

    let roll_err = (response.pose.roll_deg - expected_roll).abs();
    assert!(
        roll_err <= angle_tol_deg,
        "[{}] Roll off by {:.2}° (tolerance {:.0}°). Expected {}, got {}",
        label, roll_err, angle_tol_deg,
        expected_roll, response.pose.roll_deg,
    );
}

/// Shortest angular distance between two headings (handles 0°/360° wrap).
fn angular_distance(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    d.min(360.0 - d)
}

// ═══════════════════════════════════════════════════════════════════════════
// Test cases
// ═══════════════════════════════════════════════════════════════════════════

/// **Case 1 – Natural spread-out scene facing north.**
///
/// Camera stands on the ground south of a scene, facing north.  Six world
/// points are spread across ~300 m east-west and ~200 m north-south at
/// various altitudes, giving a well-conditioned geometry with good angular
/// diversity in the image.
#[test]
fn perfect_spread_out_facing_north() {
    let cam_lat = 51.9080;
    let cam_lon = 4.4700;
    let cam_alt = 2.0;
    let yaw = 0.0; // facing north
    let pitch = -5.0;
    let roll = 0.0;

    let focal = 3000.0;
    let w = 4000.0;
    let h = 3000.0;

    // Points roughly 300-600 m to the north, spread east-west
    let world_pts = vec![
        (51.9110, 4.4660, 40.0), // NW, low building
        (51.9120, 4.4700, 80.0), // N,  tall tower
        (51.9115, 4.4740, 60.0), // NE, medium building
        (51.9105, 4.4680, 25.0), // N-NW, low structure
        (51.9130, 4.4720, 100.0), // far N, very tall
        (51.9108, 4.4710, 35.0), // nearby, low
    ];

    let req = build_perfect_request(
        cam_lat, cam_lon, cam_alt, yaw, pitch, roll, focal, w, h, &world_pts,
    );

    let response = solve_impl(&req).expect("solve must succeed");

    eprintln!(
        "[spread_north] pos=({}, {}), alt={:.1}, yaw={:.1}, pitch={:.1}, roll={:.1}, \
         RMSE={:.4}px, warnings={:?}",
        response.pose.lat, response.pose.lon, response.pose.alt,
        response.pose.yaw_deg, response.pose.pitch_deg, response.pose.roll_deg,
        response.diagnostics.rmse_px, response.diagnostics.warnings,
    );

    assert_pose_close(
        &response,
        cam_lat, cam_lon, cam_alt, yaw, pitch, roll,
        10.0,  // position within 10 m
        2.0,   // angles within 2°
        "spread_north",
    );
}

/// **Case 2 – High covariance / tightly clustered points.**
///
/// All world points are within a ~30 m cluster at similar altitudes and
/// directions from the camera.  This yields high covariance in the pose
/// estimate (poor conditioning) because the angular spread is very small.
/// The solver may struggle, which is expected.
#[test]
fn perfect_high_covariance_clustered() {
    let cam_lat = 51.9080;
    let cam_lon = 4.4700;
    let cam_alt = 2.0;
    let yaw = 45.0; // facing northeast
    let pitch = -3.0;
    let roll = 0.0;

    let focal = 2500.0;
    let w = 4000.0;
    let h = 3000.0;

    // All points tightly clustered ~500 m to the NE, within a ~30 m cube
    let world_pts = vec![
        (51.9112, 4.4760, 50.0),
        (51.9113, 4.4762, 55.0),
        (51.9111, 4.4758, 48.0),
        (51.9114, 4.4761, 52.0),
        (51.9112, 4.4759, 53.0),
        (51.9113, 4.4760, 51.0),
    ];

    let req = build_perfect_request(
        cam_lat, cam_lon, cam_alt, yaw, pitch, roll, focal, w, h, &world_pts,
    );

    let response = solve_impl(&req).expect("solve must succeed");

    eprintln!(
        "[high_covariance] pos=({}, {}), alt={:.1}, yaw={:.1}, pitch={:.1}, roll={:.1}, \
         RMSE={:.4}px, warnings={:?}",
        response.pose.lat, response.pose.lon, response.pose.alt,
        response.pose.yaw_deg, response.pose.pitch_deg, response.pose.roll_deg,
        response.diagnostics.rmse_px, response.diagnostics.warnings,
    );

    // Generous tolerances – clustered points make accurate recovery hard.
    assert_pose_close(
        &response,
        cam_lat, cam_lon, cam_alt, yaw, pitch, roll,
        50.0,  // position within 50 m
        5.0,   // angles within 5°
        "high_covariance",
    );
}

/// **Case 3 – Elevated viewpoint looking down at buildings.**
///
/// Camera is on a high-rise roof (100 m altitude) looking down at ground-
/// level structures with steep pitch.  Tests solver behaviour with large
/// negative pitch and vertical relief.
#[test]
fn perfect_elevated_looking_down() {
    let cam_lat = 51.9200;
    let cam_lon = 4.4800;
    let cam_alt = 100.0;
    let yaw = 180.0; // facing south
    let pitch = -25.0; // looking down steeply
    let roll = 0.0;

    let focal = 2000.0;
    let w = 4000.0;
    let h = 3000.0;

    // Ground-level features 200-500 m to the south at low altitudes
    let world_pts = vec![
        (51.9170, 4.4780, 5.0),  // SW ground
        (51.9175, 4.4820, 8.0),  // SE ground
        (51.9180, 4.4800, 12.0), // S, slightly elevated
        (51.9165, 4.4790, 3.0),  // far SW ground
        (51.9172, 4.4810, 15.0), // S, rooftop
        (51.9185, 4.4795, 20.0), // near S, taller building
    ];

    let req = build_perfect_request(
        cam_lat, cam_lon, cam_alt, yaw, pitch, roll, focal, w, h, &world_pts,
    );

    let response = solve_impl(&req).expect("solve must succeed");

    eprintln!(
        "[elevated_down] pos=({}, {}), alt={:.1}, yaw={:.1}, pitch={:.1}, roll={:.1}, \
         RMSE={:.4}px, warnings={:?}",
        response.pose.lat, response.pose.lon, response.pose.alt,
        response.pose.yaw_deg, response.pose.pitch_deg, response.pose.roll_deg,
        response.diagnostics.rmse_px, response.diagnostics.warnings,
    );

    assert_pose_close(
        &response,
        cam_lat, cam_lon, cam_alt, yaw, pitch, roll,
        15.0, // position within 15 m
        3.0,  // angles within 3°
        "elevated_down",
    );
}

/// **Case 4 – Long-range telephoto-like observation.**
///
/// Camera is ~2 km away from the scene with a long focal length (5000 px),
/// observing a cluster of buildings.  The narrow field of view and large
/// distance test depth recovery.
#[test]
fn perfect_long_range_telephoto() {
    let cam_lat = 51.9000;
    let cam_lon = 4.4700;
    let cam_alt = 5.0;
    let yaw = 0.0; // facing north
    let pitch = -2.0;
    let roll = 0.0;

    let focal = 5000.0; // telephoto
    let w = 4000.0;
    let h = 3000.0;

    // Buildings ~2 km to the north
    let world_pts = vec![
        (51.9180, 4.4680, 80.0),
        (51.9185, 4.4710, 120.0),
        (51.9175, 4.4720, 60.0),
        (51.9190, 4.4700, 150.0),
        (51.9182, 4.4690, 90.0),
        (51.9178, 4.4715, 70.0),
    ];

    let req = build_perfect_request(
        cam_lat, cam_lon, cam_alt, yaw, pitch, roll, focal, w, h, &world_pts,
    );

    let response = solve_impl(&req).expect("solve must succeed");

    eprintln!(
        "[telephoto] pos=({}, {}), alt={:.1}, yaw={:.1}, pitch={:.1}, roll={:.1}, \
         RMSE={:.4}px, warnings={:?}",
        response.pose.lat, response.pose.lon, response.pose.alt,
        response.pose.yaw_deg, response.pose.pitch_deg, response.pose.roll_deg,
        response.diagnostics.rmse_px, response.diagnostics.warnings,
    );

    // Telephoto at 2 km is inherently harder for depth estimation,
    // but with perfect data the solver should still converge well.
    assert_pose_close(
        &response,
        cam_lat, cam_lon, cam_alt, yaw, pitch, roll,
        50.0, // position within 50 m
        3.0,  // angles within 3°
        "telephoto",
    );
}

/// **Case 5 – Non-zero roll, facing west.**
///
/// Camera has a significant roll angle (tilted horizon) and is facing west,
/// testing the solver's ability to recover all six pose degrees of freedom
/// simultaneously.
#[test]
fn perfect_rolled_facing_west() {
    let cam_lat = 51.9100;
    let cam_lon = 4.4800;
    let cam_alt = 8.0;
    let yaw = 270.0; // facing west
    let pitch = -4.0;
    let roll = 12.0; // noticeable horizon tilt

    let focal = 2800.0;
    let w = 4000.0;
    let h = 3000.0;

    // Points to the west at various positions
    let world_pts = vec![
        (51.9095, 4.4730, 30.0), // WSW
        (51.9105, 4.4720, 50.0), // WNW
        (51.9100, 4.4710, 70.0), // W, tall
        (51.9090, 4.4740, 20.0), // SW-ish
        (51.9110, 4.4725, 45.0), // NW-ish
        (51.9098, 4.4715, 60.0), // W, medium
        (51.9102, 4.4735, 35.0), // W, close
    ];

    let req = build_perfect_request(
        cam_lat, cam_lon, cam_alt, yaw, pitch, roll, focal, w, h, &world_pts,
    );

    let response = solve_impl(&req).expect("solve must succeed");

    eprintln!(
        "[rolled_west] pos=({}, {}), alt={:.1}, yaw={:.1}, pitch={:.1}, roll={:.1}, \
         RMSE={:.4}px, warnings={:?}",
        response.pose.lat, response.pose.lon, response.pose.alt,
        response.pose.yaw_deg, response.pose.pitch_deg, response.pose.roll_deg,
        response.diagnostics.rmse_px, response.diagnostics.warnings,
    );

    assert_pose_close(
        &response,
        cam_lat, cam_lon, cam_alt, yaw, pitch, roll,
        15.0, // position within 15 m
        3.0,  // angles within 3°
        "rolled_west",
    );
}
