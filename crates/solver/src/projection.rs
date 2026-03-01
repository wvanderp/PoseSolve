/// 3×3 rotation matrix, stored row-major.
pub(crate) type Mat3 = [[f64; 3]; 3];

/// Camera intrinsic parameters.
///
/// Distortion follows the Brown-Conrady model (same convention as OpenCV):
///
/// ```text
/// r² = xn² + yn²
/// xd = xn · (1 + k1·r² + k2·r⁴) + 2·p1·xn·yn + p2·(r² + 2·xn²)
/// yd = yn · (1 + k1·r² + k2·r⁴) + p1·(r² + 2·yn²) + 2·p2·xn·yn
/// u  = f · xd + cx
/// v  = f · yd + cy
/// ```
///
/// Set k1 = k2 = p1 = p2 = 0 for a pure pinhole model.
#[derive(Clone, Debug)]
pub(crate) struct CameraIntrinsics {
    pub focal_px: f64,
    pub cx: f64,
    pub cy: f64,
    /// Radial distortion – first order (negative = barrel).
    pub k1: f64,
    /// Radial distortion – second order.
    pub k2: f64,
    /// Tangential distortion 1.
    pub p1: f64,
    /// Tangential distortion 2.
    pub p2: f64,
}

// ── Rotation matrix construction ────────────────────────────────────────────

/// Build the rotation matrix that transforms a vector in the local ENU frame
/// into the camera coordinate frame.
///
/// Conventions
/// -----------
/// * **Camera frame**: X-right, Y-down, Z-forward (standard computer-vision).
/// * **Yaw** (heading): 0 = looking North, 90 = East, clockwise positive.
/// * **Pitch**: 0 = horizontal, negative = looking down, positive = looking up.
/// * **Roll**: 0 = level horizon, positive = clockwise seen from behind camera.
///
/// The full rotation is  `R = R_roll · R_pitch · R_base · R_yaw` where:
///
/// * `R_yaw`   – rotates ENU around the Up axis to align North with the
///               camera heading.
/// * `R_base`  – swaps axes from heading-aligned ENU to camera frame
///               (cam-X = East, cam-Y = −Up, cam-Z = North).
/// * `R_pitch` – tilts the camera frame around its X (right) axis.
/// * `R_roll`  – banks the camera frame around its Z (forward) axis.
pub(crate) fn rotation_enu_to_cam(yaw_deg: f64, pitch_deg: f64, roll_deg: f64) -> Mat3 {
    let yaw = yaw_deg.to_radians();
    let pitch = pitch_deg.to_radians();
    let roll = roll_deg.to_radians();

    // R_yaw — rotate ENU around Up (Z) by yaw
    let (sy, cy) = (yaw.sin(), yaw.cos());
    let r_yaw: Mat3 = [[cy, -sy, 0.0], [sy, cy, 0.0], [0.0, 0.0, 1.0]];

    // R_base — ENU → camera (when yaw=pitch=roll=0, camera looks North)
    //   cam-X = E (1,0,0)
    //   cam-Y = -U (0,0,-1)
    //   cam-Z = N  (0,1,0)
    let r_base: Mat3 = [[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]];

    // R_pitch — rotate camera frame around X by −pitch
    //   (positive pitch = looking up → rotate −pitch around X so Z tilts up)
    let p = -pitch;
    let (sp, cp) = (p.sin(), p.cos());
    let r_pitch: Mat3 = [[1.0, 0.0, 0.0], [0.0, cp, -sp], [0.0, sp, cp]];

    // R_roll — rotate camera frame around Z by roll
    let (sr, cr) = (roll.sin(), roll.cos());
    let r_roll: Mat3 = [[cr, -sr, 0.0], [sr, cr, 0.0], [0.0, 0.0, 1.0]];

    // full = R_roll · R_pitch · R_base · R_yaw
    let t1 = mat3_mul(&r_base, &r_yaw);
    let t2 = mat3_mul(&r_pitch, &t1);
    mat3_mul(&r_roll, &t2)
}

// ── Projection ──────────────────────────────────────────────────────────────

/// Project a 3-D ENU point to pixel coordinates through a camera described
/// by `CameraIntrinsics` (pinhole + optional Brown-Conrady distortion).
///
/// Returns `None` when the point is behind the camera (Z_cam ≤ 0).
pub(crate) fn project_point(
    point_enu: [f64; 3],
    cam_enu: [f64; 3],
    rot: &Mat3,
    intr: &CameraIntrinsics,
) -> Option<(f64, f64)> {
    let dp = [
        point_enu[0] - cam_enu[0],
        point_enu[1] - cam_enu[1],
        point_enu[2] - cam_enu[2],
    ];
    let p = mat3_vec(rot, &dp);
    if p[2] <= 0.0 {
        return None;
    }

    // Normalised (metric) image coordinates
    let xn = p[0] / p[2];
    let yn = p[1] / p[2];

    // Brown-Conrady distortion -------------------------------------------
    let r2 = xn * xn + yn * yn;
    let radial = 1.0 + intr.k1 * r2 + intr.k2 * r2 * r2;
    let xd = xn * radial + 2.0 * intr.p1 * xn * yn + intr.p2 * (r2 + 2.0 * xn * xn);
    let yd = yn * radial + intr.p1 * (r2 + 2.0 * yn * yn) + 2.0 * intr.p2 * xn * yn;
    // -------------------------------------------------------------------

    let u = intr.focal_px * xd + intr.cx;
    let v = intr.focal_px * yd + intr.cy;
    Some((u, v))
}

// ── Small linear-algebra helpers ────────────────────────────────────────────

pub(crate) fn mat3_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut c = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            c[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    c
}

pub(crate) fn mat3_vec(m: &Mat3, v: &[f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}
