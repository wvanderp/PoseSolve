use crate::projection::{project_point, rotation_enu_to_cam, CameraIntrinsics};

/// Parameter vector layout:
/// ```text
/// [0] e       – camera East  (m, ENU)
/// [1] n       – camera North (m, ENU)
/// [2] u       – camera Up    (m, ENU)
/// [3] yaw°    – heading (0 = North, clockwise)
/// [4] pitch°  – elevation (-90 = straight down)
/// [5] roll°   – bank (0 = level)
/// [6] k1      – radial distortion, 1st order
/// [7] k2      – radial distortion, 2nd order
/// [8] p1      – tangential distortion 1
/// [9] p2      – tangential distortion 2
/// ```
pub(crate) const NUM_PARAMS: usize = 10;

// ── Data types ──────────────────────────────────────────────────────────────

/// A single pixel ↔ world correspondence expressed in ENU.
pub(crate) struct EnuCorrespondence {
    pub enu: [f64; 3],
    pub pixel: [f64; 2],
    pub sigma: f64,
}

/// Result returned by the Levenberg-Marquardt optimiser.
pub(crate) struct OptResult {
    pub params: [f64; NUM_PARAMS],
    pub cost: f64,
    pub iterations: usize,
    pub converged: bool,
    /// Flattened 6×6 J^T·J (row-major) at the final iterate – used for
    /// covariance estimation.
    pub jtj: [f64; NUM_PARAMS * NUM_PARAMS],
}

// ── Residuals ───────────────────────────────────────────────────────────────

/// Compute the full residual vector (reprojection + regularisation terms).
///
/// Layout: `[du₁/σ₁, dv₁/σ₁, …, duₙ/σₙ, dvₙ/σₙ, <prior terms>]`
///
/// `base_intr` carries the fixed focal length and principal point.
/// Distortion coefficients (k1, k2, p1, p2) are taken from `params[6..10]`
/// and regularised toward `dist_prior`.
fn residuals(
    params: &[f64; NUM_PARAMS],
    corrs: &[EnuCorrespondence],
    base_intr: &CameraIntrinsics,
    alt_prior_enu: Option<f64>,
    dist_prior: [f64; 4],
    n_points: usize,
) -> Vec<f64> {
    let cam = [params[0], params[1], params[2]];
    let rot = rotation_enu_to_cam(params[3], params[4], params[5]);

    // When there are too few correspondences to reliably separate distortion
    // from camera pose, freeze the distortion at the prior values so that the
    // effective model is a pure pinhole (or a pinhole with the user-supplied
    // priors as fixed offsets).  Five correspondences gives 10 equations for
    // 10 unknowns – the minimum for a well-determined system.
    let enough_for_distortion = n_points >= 5;
    // Build full intrinsics: fixed focal/principal point + optimised/frozen distortion.
    let intr = CameraIntrinsics {
        focal_px: base_intr.focal_px,
        cx: base_intr.cx,
        cy: base_intr.cy,
        k1: if enough_for_distortion {
            params[6]
        } else {
            dist_prior[0]
        },
        k2: if enough_for_distortion {
            params[7]
        } else {
            dist_prior[1]
        },
        p1: if enough_for_distortion {
            params[8]
        } else {
            dist_prior[2]
        },
        p2: if enough_for_distortion {
            params[9]
        } else {
            dist_prior[3]
        },
    };

    let mut res = Vec::with_capacity(corrs.len() * 2 + 8);
    for c in corrs {
        match project_point(c.enu, cam, &rot, &intr) {
            Some((u, v)) => {
                res.push((u - c.pixel[0]) / c.sigma);
                res.push((v - c.pixel[1]) / c.sigma);
            }
            None => {
                // Behind camera → large smooth penalty that grows with negative Z
                let dp = [c.enu[0] - cam[0], c.enu[1] - cam[1], c.enu[2] - cam[2]];
                let rm = rotation_enu_to_cam(params[3], params[4], params[5]);
                let z_cam = rm[2][0] * dp[0] + rm[2][1] * dp[1] + rm[2][2] * dp[2];
                let penalty = 1000.0 + (-z_cam).max(0.0) * 10.0;
                res.push(penalty);
                res.push(penalty);
            }
        }
    }

    // Altitude prior (soft constraint)
    if let Some(ap) = alt_prior_enu {
        res.push((params[2] - ap) / 10.0); // σ_alt = 10 m
    }

    // Roll regularisation (prefer level horizon)
    let roll_sigma = if n_points < 3 { 5.0 } else { 45.0 };
    res.push(params[5] / roll_sigma);

    // Pitch regularisation for under-determined cases
    if n_points < 3 {
        res.push(params[4] / 30.0);
    }

    // Distortion regularisation (or identity constraints when frozen).
    //
    // When distortion is estimated (n_points ≥ 5), add Gaussian soft
    // constraints toward the prior.
    //
    // When distortion is frozen (n_points < 5) we do NOT add distortion
    // residuals here at all.  The Levenberg-Marquardt loop adds a unit
    // identity block directly to the J^T J diagonal for those parameters,
    // keeping the normal-equations matrix non-singular while leaving the
    // pose sub-block bit-identical to the old 6-DOF solver.  This avoids
    // any floating-point accumulation differences that could shift the
    // optimizer into a different basin of attraction.
    if enough_for_distortion {
        // Σ chosen to allow realistic smartphone distortion.
        //   k1: 0.5  – strong barrel common on wide-angle lenses
        //   k2: 0.25 – second-order term, smaller in practice
        //   p1: 0.05 – tangential distortion is small on most lenses
        //   p2: 0.05
        res.push((params[6] - dist_prior[0]) / 0.5);
        res.push((params[7] - dist_prior[1]) / 0.25);
        res.push((params[8] - dist_prior[2]) / 0.05);
        res.push((params[9] - dist_prior[3]) / 0.05);
    }
    // (frozen case handled by the LM caller)

    res
}

// ── Numerical Jacobian ──────────────────────────────────────────────────────

fn jacobian(
    params: &[f64; NUM_PARAMS],
    corrs: &[EnuCorrespondence],
    base_intr: &CameraIntrinsics,
    alt_prior: Option<f64>,
    dist_prior: [f64; 4],
    n_points: usize,
) -> Vec<[f64; NUM_PARAMS]> {
    // Step sizes: position in metres, angles in degrees, distortion dimensionless
    const H: [f64; NUM_PARAMS] = [1e-3, 1e-3, 1e-3, 1e-4, 1e-4, 1e-4, 1e-6, 1e-7, 1e-7, 1e-7];

    let n_res = residuals(params, corrs, base_intr, alt_prior, dist_prior, n_points).len();
    let mut jac = vec![[0.0; NUM_PARAMS]; n_res];

    for j in 0..NUM_PARAMS {
        let mut pp = *params;
        let mut pm = *params;
        pp[j] += H[j];
        pm[j] -= H[j];
        let rp = residuals(&pp, corrs, base_intr, alt_prior, dist_prior, n_points);
        let rm = residuals(&pm, corrs, base_intr, alt_prior, dist_prior, n_points);
        let inv2h = 1.0 / (2.0 * H[j]);
        for i in 0..n_res {
            jac[i][j] = (rp[i] - rm[i]) * inv2h;
        }
    }
    jac
}

// ── Levenberg-Marquardt ─────────────────────────────────────────────────────

pub(crate) fn levenberg_marquardt(
    initial: [f64; NUM_PARAMS],
    corrs: &[EnuCorrespondence],
    base_intr: &CameraIntrinsics,
    alt_prior: Option<f64>,
    dist_prior: [f64; 4],
    n_points: usize,
    max_iter: usize,
) -> OptResult {
    let mut params = initial;
    let mut lambda: f64 = 1.0;

    let r0 = residuals(&params, corrs, base_intr, alt_prior, dist_prior, n_points);
    let mut cost: f64 = r0.iter().map(|v| v * v).sum();

    let mut converged = false;
    let mut iterations: usize = 0;
    let mut last_jtj = [[0.0f64; NUM_PARAMS]; NUM_PARAMS];

    for _iter in 0..max_iter {
        iterations = _iter + 1;

        let r = residuals(&params, corrs, base_intr, alt_prior, dist_prior, n_points);
        let jac = jacobian(&params, corrs, base_intr, alt_prior, dist_prior, n_points);
        let n_res = r.len();

        // Normal equations: J^T J and J^T r
        let mut jtj = [[0.0f64; NUM_PARAMS]; NUM_PARAMS];
        let mut jtr = [0.0f64; NUM_PARAMS];
        for i in 0..n_res {
            for a in 0..NUM_PARAMS {
                jtr[a] += jac[i][a] * r[i];
                for b in a..NUM_PARAMS {
                    jtj[a][b] += jac[i][a] * jac[i][b];
                }
            }
        }
        // Mirror upper triangle
        for a in 0..NUM_PARAMS {
            for b in (a + 1)..NUM_PARAMS {
                jtj[b][a] = jtj[a][b];
            }
        }

        // When distortion is frozen, add a unit identity block to the
        // distortion sub-diagonal so the normal-equations matrix remains
        // non-singular.  The corresponding jtr entries are already 0.
        let enough_for_distortion_norm = n_points >= 5;
        if !enough_for_distortion_norm {
            for k in 6..NUM_PARAMS {
                jtj[k][k] += 1.0;
            }
        }
        last_jtj = jtj;

        // Try damped steps (inner loop adjusts λ)
        let mut step_accepted = false;
        for _ in 0..30 {
            let mut a = jtj;
            for i in 0..NUM_PARAMS {
                a[i][i] += lambda * a[i][i].max(1e-12);
            }
            let mut b = [0.0; NUM_PARAMS];
            for i in 0..NUM_PARAMS {
                b[i] = -jtr[i];
            }

            if let Some(delta) = solve_nxn(&mut a, &mut b) {
                let mut np = params;
                for i in 0..NUM_PARAMS {
                    np[i] += delta[i];
                }
                // Normalise yaw to [0, 360)
                np[3] = ((np[3] % 360.0) + 360.0) % 360.0;

                let nr = residuals(&np, corrs, base_intr, alt_prior, dist_prior, n_points);
                let nc: f64 = nr.iter().map(|v| v * v).sum();

                if nc < cost {
                    params = np;
                    cost = nc;
                    lambda = (lambda * 0.3).max(1e-12);
                    step_accepted = true;
                    break;
                }
            }
            lambda = (lambda * 3.0).min(1e16);
        }

        if !step_accepted {
            break;
        }

        // Convergence checks
        if cost < 1e-20 {
            converged = true;
            break;
        }
    }

    // Flatten jtj
    let mut jtj_flat = [0.0; NUM_PARAMS * NUM_PARAMS];
    for i in 0..NUM_PARAMS {
        for j in 0..NUM_PARAMS {
            jtj_flat[i * NUM_PARAMS + j] = last_jtj[i][j];
        }
    }

    OptResult {
        params,
        cost,
        iterations,
        converged,
        jtj: jtj_flat,
    }
}

// ── Initialisation heuristic ────────────────────────────────────────────────

/// Produce a starting guess for the 10-DOF camera parameter vector
/// `[e, n, u, yaw°, pitch°, roll°, k1, k2, p1, p2]`.
///
/// `dist_prior` seeds the distortion coefficients; pass `[0.0; 4]` when no
/// prior is available.
pub(crate) fn initialize_pose(
    corrs: &[EnuCorrespondence],
    intr: &CameraIntrinsics,
    alt_prior_enu: Option<f64>,
    dist_prior: [f64; 4],
) -> [f64; NUM_PARAMS] {
    let n = corrs.len() as f64;

    // Centroids
    let me: f64 = corrs.iter().map(|c| c.enu[0]).sum::<f64>() / n;
    let mn: f64 = corrs.iter().map(|c| c.enu[1]).sum::<f64>() / n;
    let mu: f64 = corrs.iter().map(|c| c.enu[2]).sum::<f64>() / n;
    let min_u: f64 = corrs.iter().map(|c| c.enu[2]).fold(f64::INFINITY, f64::min);

    let mpx: f64 = corrs.iter().map(|c| c.pixel[0]).sum::<f64>() / n;
    let mpv: f64 = corrs.iter().map(|c| c.pixel[1]).sum::<f64>() / n;

    // ── Yaw from pixel–ENU correlation ──────────────────────────────────
    let (mut cov_ue, mut cov_un, mut var_e, mut var_n) = (0.0, 0.0, 0.0, 0.0);
    for c in corrs {
        let de = c.enu[0] - me;
        let dn = c.enu[1] - mn;
        let du = c.pixel[0] - mpx;
        cov_ue += du * de;
        cov_un += du * dn;
        var_e += de * de;
        var_n += dn * dn;
    }
    let yaw_deg = if var_e > 1e-10 || var_n > 1e-10 {
        let r = (-cov_un / var_n.max(1e-10)).atan2(cov_ue / var_e.max(1e-10));
        ((r.to_degrees() % 360.0) + 360.0) % 360.0
    } else {
        0.0
    };

    // ── Distance from camera to scene ───────────────────────────────────
    let dist = estimate_scene_distance(corrs, intr);

    let yaw_rad = yaw_deg.to_radians();
    let cam_e = me - dist * yaw_rad.sin();
    let cam_n = mn - dist * yaw_rad.cos();

    // ── Altitude ────────────────────────────────────────────────────────
    let cam_u = alt_prior_enu.unwrap_or_else(|| (min_u - 10.0).min(mu - 5.0));

    // ── Pitch from vertical pixel offset ────────────────────────────────
    let pitch_deg = -((mpv - intr.cy) / intr.focal_px).atan().to_degrees();

    [
        cam_e,
        cam_n,
        cam_u,
        yaw_deg,
        pitch_deg,
        0.0,
        dist_prior[0],
        dist_prior[1],
        dist_prior[2],
        dist_prior[3],
    ]
}

/// Median of pair-wise  `geo_distance_3d × focal / pixel_distance`.
pub(crate) fn estimate_scene_distance(corrs: &[EnuCorrespondence], intr: &CameraIntrinsics) -> f64 {
    let mut dists = Vec::new();
    for i in 0..corrs.len() {
        for j in (i + 1)..corrs.len() {
            let de = corrs[i].enu[0] - corrs[j].enu[0];
            let dn = corrs[i].enu[1] - corrs[j].enu[1];
            let du_enu = corrs[i].enu[2] - corrs[j].enu[2];
            let geo = (de * de + dn * dn + du_enu * du_enu).sqrt(); // full 3-D
            let du = corrs[i].pixel[0] - corrs[j].pixel[0];
            let dv = corrs[i].pixel[1] - corrs[j].pixel[1];
            let px = (du * du + dv * dv).sqrt();
            if px > 10.0 && geo > 1.0 {
                dists.push(geo * intr.focal_px / px);
            }
        }
    }
    if dists.is_empty() {
        return 500.0;
    }
    dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
    dists[dists.len() / 2]
}

// ── N×N linear solve ─────────────────────────────────────────────────────────

/// Solve A·x = b (NUM_PARAMS × NUM_PARAMS) with Gaussian elimination +
/// partial pivoting.  Returns `None` when the matrix is singular.
fn solve_nxn(
    a: &mut [[f64; NUM_PARAMS]; NUM_PARAMS],
    b: &mut [f64; NUM_PARAMS],
) -> Option<[f64; NUM_PARAMS]> {
    for col in 0..NUM_PARAMS {
        // Pivot
        let (mut mx, mut mr) = (a[col][col].abs(), col);
        for row in (col + 1)..NUM_PARAMS {
            let v = a[row][col].abs();
            if v > mx {
                mx = v;
                mr = row;
            }
        }
        if mx < 1e-30 {
            return None;
        }
        if mr != col {
            a.swap(col, mr);
            b.swap(col, mr);
        }
        // Eliminate
        for row in (col + 1)..NUM_PARAMS {
            let f = a[row][col] / a[col][col];
            for k in col..NUM_PARAMS {
                a[row][k] -= f * a[col][k];
            }
            b[row] -= f * b[col];
        }
    }
    // Back-substitute
    let mut x = [0.0; NUM_PARAMS];
    for i in (0..NUM_PARAMS).rev() {
        let mut s = b[i];
        for j in (i + 1)..NUM_PARAMS {
            s -= a[i][j] * x[j];
        }
        x[i] = s / a[i][i];
    }
    Some(x)
}
