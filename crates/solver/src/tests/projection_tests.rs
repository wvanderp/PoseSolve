use crate::projection::{project_point, rotation_enu_to_cam, CameraIntrinsics};

fn default_intrinsics() -> CameraIntrinsics {
    CameraIntrinsics {
        focal_px: 1000.0,
        cx: 500.0,
        cy: 400.0,
        k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
    }
}

// ── Base-case projection (yaw=0, pitch=0, roll=0 → camera looks North) ─────

#[test]
fn point_directly_ahead_projects_to_center() {
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(0.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];
    let pt = [0.0, 100.0, 0.0]; // 100 m North
    let (u, v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!(
        (u - intr.cx).abs() < 0.01,
        "u should be cx: u={}, cx={}",
        u,
        intr.cx
    );
    assert!(
        (v - intr.cy).abs() < 0.01,
        "v should be cy: v={}, cy={}",
        v,
        intr.cy
    );
}

#[test]
fn point_to_east_projects_right_of_center() {
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(0.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];
    let pt = [10.0, 100.0, 0.0]; // 100 m N, 10 m E
    let (u, v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!(u > intr.cx, "point to the east should have u > cx: {}", u);
    assert!(
        (v - intr.cy).abs() < 0.5,
        "v should be ~cy: v={}, cy={}",
        v,
        intr.cy
    );
}

#[test]
fn point_above_projects_above_center() {
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(0.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];
    let pt = [0.0, 100.0, 10.0]; // 100 m N, 10 m Up
    let (u, v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!(
        (u - intr.cx).abs() < 0.5,
        "u should be ~cx: u={}",
        u
    );
    assert!(v < intr.cy, "point above should have v < cy: v={}", v);
}

#[test]
fn point_below_projects_below_center() {
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(0.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];
    let pt = [0.0, 100.0, -10.0]; // 100 m N, 10 m Down
    let (u, v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!(v > intr.cy, "point below should have v > cy: v={}", v);
}

#[test]
fn point_behind_camera_returns_none() {
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(0.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];
    let pt = [0.0, -100.0, 0.0]; // 100 m South (behind)
    assert!(project_point(pt, cam, &rot, &intr).is_none());
}

// ── Yaw tests ───────────────────────────────────────────────────────────────

#[test]
fn yaw_90_facing_east_point_east_projects_to_center() {
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(90.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];
    let pt = [100.0, 0.0, 0.0]; // 100 m East
    let (u, v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!(
        (u - intr.cx).abs() < 0.01,
        "u should be cx: u={}",
        u
    );
    assert!(
        (v - intr.cy).abs() < 0.01,
        "v should be cy: v={}",
        v
    );
}

#[test]
fn yaw_180_facing_south_point_south_projects_to_center() {
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(180.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];
    let pt = [0.0, -100.0, 0.0]; // 100 m South
    let (u, v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!((u - intr.cx).abs() < 0.01, "u={}", u);
    assert!((v - intr.cy).abs() < 0.01, "v={}", v);
}

#[test]
fn yaw_270_facing_west_point_west_projects_to_center() {
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(270.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];
    let pt = [-100.0, 0.0, 0.0]; // 100 m West
    let (u, v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!((u - intr.cx).abs() < 0.01, "u={}", u);
    assert!((v - intr.cy).abs() < 0.01, "v={}", v);
}

#[test]
fn yaw_90_point_south_projects_right() {
    // Facing East, South is to the right
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(90.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];
    let pt = [100.0, -10.0, 0.0]; // mostly East, slightly South
    let (u, _v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!(
        u > intr.cx,
        "facing east, south should be right of center: u={}",
        u
    );
}

// ── Pitch tests ─────────────────────────────────────────────────────────────

#[test]
fn pitch_down_point_below_center_moves_to_center() {
    let intr = default_intrinsics();
    // Point at (0, 100, -17.6) = 10° below horizontal
    let pt = [0.0, 100.0, -17.633];
    let cam = [0.0, 0.0, 0.0];

    // Without pitch: point should be below center
    let rot_no_pitch = rotation_enu_to_cam(0.0, 0.0, 0.0);
    let (_, v_no) = project_point(pt, cam, &rot_no_pitch, &intr).unwrap();
    assert!(v_no > intr.cy, "without pitch, point should be below cy");

    // With pitch = -10° (looking down): point should be near centre
    let rot_pitch = rotation_enu_to_cam(0.0, -10.0, 0.0);
    let (u, v) = project_point(pt, cam, &rot_pitch, &intr).unwrap();
    assert!(
        (v - intr.cy).abs() < 5.0,
        "with -10° pitch, point 10° below should be near cy: v={}, cy={}",
        v, intr.cy
    );
    assert!((u - intr.cx).abs() < 1.0);
}

#[test]
fn pitch_up_point_above_center_moves_to_center() {
    let intr = default_intrinsics();
    let pt = [0.0, 100.0, 17.633]; // 10° above horizontal
    let cam = [0.0, 0.0, 0.0];

    let rot = rotation_enu_to_cam(0.0, 10.0, 0.0);
    let (_, v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!(
        (v - intr.cy).abs() < 5.0,
        "with +10° pitch, point 10° above should be near cy: v={}",
        v
    );
}

// ── Roll tests ──────────────────────────────────────────────────────────────

#[test]
fn roll_rotates_image() {
    let intr = default_intrinsics();
    let cam = [0.0, 0.0, 0.0];
    let pt = [10.0, 100.0, 0.0]; // slightly to the right

    // Without roll
    let rot0 = rotation_enu_to_cam(0.0, 0.0, 0.0);
    let (u0, v0) = project_point(pt, cam, &rot0, &intr).unwrap();

    // With 10° roll (clockwise from behind)
    let rot10 = rotation_enu_to_cam(0.0, 0.0, 10.0);
    let (u10, v10) = project_point(pt, cam, &rot10, &intr).unwrap();

    // Point to the right should move slightly down with clockwise roll
    assert!(v10 > v0, "roll should shift right point downward: v0={}, v10={}", v0, v10);
    // And slightly left
    assert!(u10 < u0, "roll should shift right point leftward: u0={}, u10={}", u0, u10);
}

// ── Projection consistency ──────────────────────────────────────────────────

#[test]
fn projection_scale_is_correct() {
    // A point 10m East at 100m distance should subtend atan(10/100) ≈ 5.71°
    // In pixels: f * 10/100 = 1000 * 0.1 = 100
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(0.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];
    let pt = [10.0, 100.0, 0.0];
    let (u, v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!(
        ((u - intr.cx) - 100.0).abs() < 0.01,
        "pixel offset should be 100: got {}",
        u - intr.cx
    );
    assert!((v - intr.cy).abs() < 0.01);
}

#[test]
fn camera_offset_works_correctly() {
    // Camera at (50, 0, 0), point at (50, 100, 0) → straight ahead
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(0.0, 0.0, 0.0);
    let cam = [50.0, 0.0, 0.0];
    let pt = [50.0, 100.0, 0.0];
    let (u, v) = project_point(pt, cam, &rot, &intr).unwrap();
    assert!((u - intr.cx).abs() < 0.01);
    assert!((v - intr.cy).abs() < 0.01);
}

#[test]
fn project_multiple_points_left_right_ordering() {
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(0.0, 0.0, 0.0);
    let cam = [0.0, 0.0, 0.0];

    let pt_left = [-10.0, 100.0, 0.0];
    let pt_right = [10.0, 100.0, 0.0];

    let (u_left, _) = project_point(pt_left, cam, &rot, &intr).unwrap();
    let (u_right, _) = project_point(pt_right, cam, &rot, &intr).unwrap();

    assert!(u_right > u_left, "right point should have larger u");
}

#[test]
fn combined_yaw_and_pitch() {
    // Camera facing East (yaw=90), pitched down 45°
    // A point directly below and east should project near center
    let intr = default_intrinsics();
    let rot = rotation_enu_to_cam(90.0, -45.0, 0.0);
    let cam = [0.0, 0.0, 100.0]; // camera at 100m altitude
    // Point 100m east and 100m below camera (at ground level)
    let pt = [100.0, 0.0, 0.0];
    let result = project_point(pt, cam, &rot, &intr);
    assert!(result.is_some(), "point should be in front of camera");
    let (u, v) = result.unwrap();
    // Should be near center (the point is along the optical axis direction)
    assert!(
        (u - intr.cx).abs() < 5.0,
        "u should be near cx: u={}",
        u
    );
    assert!(
        (v - intr.cy).abs() < 5.0,
        "v should be near cy: v={}",
        v
    );
}
