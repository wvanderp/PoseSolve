use crate::optimizer::{
    estimate_scene_distance, initialize_pose, levenberg_marquardt, EnuCorrespondence, NUM_PARAMS,
};
use crate::projection::{project_point, rotation_enu_to_cam, CameraIntrinsics};

/// Helper: build correspondences by projecting known 3-D points through
/// a known camera pose (the "ground truth").
fn synthetic_corrs(
    cam_enu: [f64; 3],
    yaw: f64,
    pitch: f64,
    roll: f64,
    intr: &CameraIntrinsics,
    world_pts: &[[f64; 3]],
) -> Vec<EnuCorrespondence> {
    let rot = rotation_enu_to_cam(yaw, pitch, roll);
    world_pts
        .iter()
        .filter_map(|&pt| {
            project_point(pt, cam_enu, &rot, intr).map(|(u, v)| EnuCorrespondence {
                enu: pt,
                pixel: [u, v],
                sigma: 1.0,
            })
        })
        .collect()
}

fn default_intr() -> CameraIntrinsics {
    CameraIntrinsics {
        focal_px: 2000.0,
        cx: 1500.0,
        cy: 1000.0,
        k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
    }
}

// ── Scene distance estimation ───────────────────────────────────────────────

#[test]
fn scene_distance_estimate_is_reasonable() {
    let intr = default_intr();
    // Camera 500m south of the scene
    let cam = [0.0, -500.0, 0.0];
    let pts: Vec<[f64; 3]> = vec![
        [-50.0, 0.0, 50.0],
        [50.0, 0.0, 50.0],
        [0.0, 0.0, 100.0],
    ];
    let corrs = synthetic_corrs(cam, 0.0, 0.0, 0.0, &intr, &pts);
    let d = estimate_scene_distance(&corrs, &intr);
    assert!(
        (d - 500.0).abs() < 150.0,
        "estimated distance should be ~500m, got {}",
        d
    );
}

// ── Initialization ──────────────────────────────────────────────────────────

#[test]
fn initialization_produces_finite_values() {
    let intr = default_intr();
    let cam = [0.0, -300.0, -10.0];
    let pts: Vec<[f64; 3]> = vec![
        [-40.0, 0.0, 50.0],
        [40.0, 0.0, 50.0],
        [0.0, 0.0, 100.0],
    ];
    let corrs = synthetic_corrs(cam, 0.0, -5.0, 0.0, &intr, &pts);
    let init = initialize_pose(&corrs, &intr, None, [0.0; 4]);
    for (i, v) in init.iter().enumerate() {
        assert!(v.is_finite(), "init param {} is not finite: {}", i, v);
    }
}

#[test]
fn initialization_yaw_roughly_correct() {
    let intr = default_intr();
    // Camera 500m south, looking north → yaw ≈ 0°
    let cam = [0.0, -500.0, 0.0];
    let pts: Vec<[f64; 3]> = vec![
        [-50.0, 0.0, 50.0],
        [50.0, 0.0, 50.0],
        [0.0, 0.0, 100.0],
    ];
    let corrs = synthetic_corrs(cam, 0.0, 0.0, 0.0, &intr, &pts);
    let init = initialize_pose(&corrs, &intr, None, [0.0; 4]);
    let yaw = init[3];
    // Should be roughly north (0° or close to 360°)
    let yaw_err = (yaw - 0.0).abs().min((yaw - 360.0).abs());
    assert!(
        yaw_err < 45.0,
        "yaw should be roughly north (~0°), got {}°",
        yaw
    );
}

// ── LM convergence from known truth ────────────────────────────────────────

#[test]
fn lm_converges_from_true_pose() {
    let intr = default_intr();
    let true_cam = [0.0, -500.0, 5.0];
    let true_yaw = 10.0;
    let true_pitch = -3.0;
    let true_roll = 0.0;
    let pts: Vec<[f64; 3]> = vec![
        [-50.0, 0.0, 40.0],
        [50.0, 0.0, 60.0],
        [0.0, 20.0, 80.0],
        [-30.0, -10.0, 30.0],
    ];
    let corrs = synthetic_corrs(true_cam, true_yaw, true_pitch, true_roll, &intr, &pts);
    assert_eq!(corrs.len(), 4, "all points should project");

    let init = [true_cam[0], true_cam[1], true_cam[2], true_yaw, true_pitch, true_roll, 0.0, 0.0, 0.0, 0.0];
    let result = levenberg_marquardt(init, &corrs, &intr, None, [0.0; 4], corrs.len(), 50);

    assert!(
        result.cost < 1e-10,
        "cost should be ~0 from true pose, got {}",
        result.cost
    );
}

#[test]
fn lm_converges_from_perturbed_pose() {
    let intr = default_intr();
    let true_cam = [100.0, -400.0, 10.0];
    let true_yaw = 5.0;
    let true_pitch = -5.0;
    let true_roll = 1.0;
    let pts: Vec<[f64; 3]> = vec![
        [50.0, 0.0, 40.0],
        [150.0, 0.0, 60.0],
        [100.0, 20.0, 80.0],
        [80.0, -10.0, 30.0],
        [120.0, 10.0, 50.0],
    ];
    let corrs = synthetic_corrs(true_cam, true_yaw, true_pitch, true_roll, &intr, &pts);
    assert_eq!(corrs.len(), 5);

    // Start 20m off in position, 5° off in yaw
    let init = [
        true_cam[0] + 20.0,
        true_cam[1] - 15.0,
        true_cam[2] + 5.0,
        true_yaw + 5.0,
        true_pitch - 2.0,
        true_roll + 1.0,
        0.0, 0.0, 0.0, 0.0,
    ];
    let result = levenberg_marquardt(init, &corrs, &intr, None, [0.0; 4], corrs.len(), 100);

    assert!(
        result.cost < 1.0,
        "cost should be very small after convergence, got {}",
        result.cost
    );
    assert!(
        (result.params[0] - true_cam[0]).abs() < 2.0,
        "e should be close: {} vs {}",
        result.params[0],
        true_cam[0]
    );
    assert!(
        (result.params[1] - true_cam[1]).abs() < 2.0,
        "n should be close: {} vs {}",
        result.params[1],
        true_cam[1]
    );
    assert!(
        (result.params[2] - true_cam[2]).abs() < 2.0,
        "u should be close: {} vs {}",
        result.params[2],
        true_cam[2]
    );
}

#[test]
fn lm_recovers_yaw_correctly() {
    let intr = default_intr();
    let true_cam = [0.0, 0.0, 0.0];
    let true_yaw = 45.0;
    let true_pitch = -2.0;
    let true_roll = 0.0;
    let pts: Vec<[f64; 3]> = vec![
        [200.0, 200.0, 50.0],
        [250.0, 250.0, 60.0],
        [300.0, 200.0, 70.0],
        [150.0, 250.0, 40.0],
    ];
    let corrs = synthetic_corrs(true_cam, true_yaw, true_pitch, true_roll, &intr, &pts);
    
    let init = [5.0, -10.0, 2.0, true_yaw + 3.0, true_pitch + 1.0, 0.5, 0.0, 0.0, 0.0, 0.0];
    let result = levenberg_marquardt(init, &corrs, &intr, None, [0.0; 4], corrs.len(), 100);

    let yaw_err = (result.params[3] - true_yaw).abs();
    let yaw_err = yaw_err.min(360.0 - yaw_err);
    assert!(
        yaw_err < 2.0,
        "yaw should converge: got {}° vs true {}°",
        result.params[3],
        true_yaw
    );
}

#[test]
fn lm_with_altitude_prior() {
    let intr = default_intr();
    let true_cam = [0.0, -500.0, 5.0];
    let pts: Vec<[f64; 3]> = vec![
        [-50.0, 0.0, 50.0],
        [50.0, 0.0, 50.0],
        [0.0, 0.0, 100.0],
    ];
    let corrs = synthetic_corrs(true_cam, 0.0, 0.0, 0.0, &intr, &pts);

    let init = [10.0, -450.0, 20.0, 5.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let result = levenberg_marquardt(init, &corrs, &intr, Some(5.0), [0.0; 4], corrs.len(), 100);

    // Altitude should be close to the prior
    assert!(
        (result.params[2] - 5.0).abs() < 10.0,
        "altitude should be near prior 5.0, got {}",
        result.params[2]
    );
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn lm_with_3_points_minimum() {
    let intr = default_intr();
    let true_cam = [0.0, -800.0, 0.0];
    let pts: Vec<[f64; 3]> = vec![
        [-100.0, 0.0, 50.0],
        [100.0, 0.0, 50.0],
        [0.0, 0.0, 100.0],
    ];
    let corrs = synthetic_corrs(true_cam, 0.0, -3.0, 0.0, &intr, &pts);
    assert_eq!(corrs.len(), 3);

    let init = [10.0, -750.0, 5.0, 5.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let result = levenberg_marquardt(init, &corrs, &intr, None, [0.0; 4], corrs.len(), 100);

    assert!(
        result.cost < 10.0,
        "should converge with 3 points, cost={}",
        result.cost
    );
}

#[test]
fn lm_with_2_points_uses_regularisation() {
    let intr = default_intr();
    let true_cam = [0.0, -500.0, 0.0];
    let pts: Vec<[f64; 3]> = vec![
        [-50.0, 0.0, 50.0],
        [50.0, 0.0, 50.0],
    ];
    let corrs = synthetic_corrs(true_cam, 0.0, 0.0, 0.0, &intr, &pts);
    assert_eq!(corrs.len(), 2);

    let init = [10.0, -450.0, 5.0, 5.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let result = levenberg_marquardt(init, &corrs, &intr, None, [0.0; 4], corrs.len(), 100);

    // With 2 points + regularisation, should still produce a reasonable result
    assert!(result.params.iter().all(|v| v.is_finite()));
    // Roll should be close to 0 due to regularisation
    assert!(
        result.params[5].abs() < 20.0,
        "roll should be regularised near 0, got {}",
        result.params[5]
    );
}

// ── Synthetic full-pipeline recovery tests ──────────────────────────────────

#[test]
fn recover_pose_facing_northeast() {
    let intr = CameraIntrinsics {
        focal_px: 3000.0,
        cx: 2016.0,
        cy: 1512.0,
        k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
    };
    let true_cam = [-200.0, -200.0, 2.0];
    let true_yaw = 45.0;
    let true_pitch = -5.0;
    let true_roll = 0.0;

    let pts: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 50.0],
        [50.0, 50.0, 80.0],
        [-20.0, 40.0, 30.0],
        [30.0, -10.0, 60.0],
        [10.0, 20.0, 100.0],
    ];
    let corrs = synthetic_corrs(true_cam, true_yaw, true_pitch, true_roll, &intr, &pts);
    assert_eq!(corrs.len(), 5);

    // Start from the initialiser
    let init = initialize_pose(&corrs, &intr, None, [0.0; 4]);
    let result = levenberg_marquardt(init, &corrs, &intr, None, [0.0; 4], corrs.len(), 150);

    assert!(
        result.cost < 1.0,
        "should converge, cost = {}",
        result.cost
    );
    assert!(
        (result.params[0] - true_cam[0]).abs() < 5.0,
        "e: {} vs {}",
        result.params[0],
        true_cam[0]
    );
    assert!(
        (result.params[1] - true_cam[1]).abs() < 5.0,
        "n: {} vs {}",
        result.params[1],
        true_cam[1]
    );
}

#[test]
fn recover_pose_facing_south() {
    let intr = CameraIntrinsics {
        focal_px: 2500.0,
        cx: 1500.0,
        cy: 1000.0,
        k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
    };
    let true_cam = [0.0, 500.0, 10.0];
    let true_yaw = 180.0;
    let true_pitch = -8.0;
    let true_roll = 2.0;

    let pts: Vec<[f64; 3]> = vec![
        [-30.0, 0.0, 40.0],
        [30.0, 0.0, 40.0],
        [0.0, -20.0, 70.0],
        [-20.0, 10.0, 50.0],
        [20.0, 10.0, 60.0],
    ];
    let corrs = synthetic_corrs(true_cam, true_yaw, true_pitch, true_roll, &intr, &pts);

    let init = initialize_pose(&corrs, &intr, None, [0.0; 4]);
    let result = levenberg_marquardt(init, &corrs, &intr, None, [0.0; 4], corrs.len(), 150);

    assert!(result.cost < 5.0, "should converge, cost={}", result.cost);
}

#[test]
fn recover_pose_with_large_distance() {
    // Camera 2 km away from scene
    let intr = CameraIntrinsics {
        focal_px: 3000.0,
        cx: 2000.0,
        cy: 1500.0,
        k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
    };
    let true_cam = [0.0, -2000.0, 5.0];
    let true_yaw = 0.0;
    let true_pitch = -2.0;
    let true_roll = 0.0;

    let pts: Vec<[f64; 3]> = vec![
        [-100.0, 0.0, 80.0],
        [100.0, 0.0, 80.0],
        [-50.0, 0.0, 120.0],
        [50.0, 0.0, 120.0],
        [0.0, 0.0, 150.0],
    ];
    let corrs = synthetic_corrs(true_cam, true_yaw, true_pitch, true_roll, &intr, &pts);

    let init = initialize_pose(&corrs, &intr, None, [0.0; 4]);
    let result = levenberg_marquardt(init, &corrs, &intr, None, [0.0; 4], corrs.len(), 150);

    assert!(
        result.cost < 5.0,
        "should converge at 2km distance, cost={}",
        result.cost
    );
    assert!(
        (result.params[1] - true_cam[1]).abs() < 50.0,
        "n should be ~-2000: {}",
        result.params[1]
    );
}

#[test]
fn lm_result_has_valid_jtj() {
    let intr = default_intr();
    let true_cam = [0.0, -500.0, 5.0];
    let pts: Vec<[f64; 3]> = vec![
        [-50.0, 0.0, 50.0],
        [50.0, 0.0, 50.0],
        [0.0, 0.0, 100.0],
        [20.0, 10.0, 60.0],
    ];
    let corrs = synthetic_corrs(true_cam, 10.0, -3.0, 0.0, &intr, &pts);
    let init = [0.0, -500.0, 5.0, 10.0, -3.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let result = levenberg_marquardt(init, &corrs, &intr, None, [0.0; 4], corrs.len(), 50);

    // The diagonal of JtJ should be positive
    for i in 0..NUM_PARAMS {
        let diag = result.jtj[i * NUM_PARAMS + i];
        assert!(diag >= 0.0, "JtJ diagonal[{}] should be ≥ 0: {}", i, diag);
    }
}
