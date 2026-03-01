use crate::geo::{enu_to_lla, lla_to_enu};
use crate::optimizer::{
    estimate_scene_distance, initialize_pose, levenberg_marquardt, EnuCorrespondence, OptResult,
    NUM_PARAMS,
};
use crate::projection::{project_point, rotation_enu_to_cam, CameraIntrinsics};
use crate::types::{Corr, Covariance, Diagnostics, Intrinsics, Pose, SolveRequest, SolveResponse};

// ── Public entry point ──────────────────────────────────────────────────────

pub fn solve_impl(req: &SolveRequest) -> Result<SolveResponse, String> {
    let active: Vec<&Corr> = req
        .correspondences
        .iter()
        .filter(|c| c.enabled.unwrap_or(true))
        .collect();

    if active.is_empty() {
        return Err("Need at least one enabled correspondence to estimate pose".to_string());
    }

    let focal_px = estimate_focal(req);
    let cx = req.image.width / 2.0;
    let cy = req.image.height / 2.0;
    // Base intrinsics: focal length + principal point are fixed during
    // optimisation.  Distortion starts from the provided prior (or 0).
    let intr = CameraIntrinsics {
        focal_px,
        cx,
        cy,
        k1: 0.0,
        k2: 0.0,
        p1: 0.0,
        p2: 0.0,
    };

    // Extract distortion initial-guess from priors (e.g. from EXIF / DNG).
    let dist_prior: [f64; 4] = req
        .priors
        .as_ref()
        .and_then(|p| p.distortion.as_ref())
        .map(|d| {
            [
                d.k1.unwrap_or(0.0),
                d.k2.unwrap_or(0.0),
                d.p1.unwrap_or(0.0),
                d.p2.unwrap_or(0.0),
            ]
        })
        .unwrap_or([0.0; 4]);

    // ── Single-point fallback (no geometric information) ────────────────
    if active.len() == 1 {
        return solve_single_point(req, active[0], &intr, dist_prior);
    }

    // ── Reference LLA (centroid of world points) ────────────────────────
    let n = active.len() as f64;
    let ref_lat = active.iter().map(|c| c.world.lat).sum::<f64>() / n;
    let ref_lon = active.iter().map(|c| c.world.lon).sum::<f64>() / n;
    let ref_alt = active
        .iter()
        .map(|c| c.world.alt.unwrap_or(0.0))
        .sum::<f64>()
        / n;

    // ── Build ENU correspondences ───────────────────────────────────────
    let enu_corrs: Vec<EnuCorrespondence> = active
        .iter()
        .map(|c| EnuCorrespondence {
            enu: lla_to_enu(
                c.world.lat,
                c.world.lon,
                c.world.alt.unwrap_or(0.0),
                ref_lat,
                ref_lon,
                ref_alt,
            ),
            pixel: [c.pixel.u, c.pixel.v],
            sigma: c.pixel.sigma_px.unwrap_or(1.0).max(1e-6),
        })
        .collect();

    // Altitude prior in ENU metres
    let alt_prior_enu = req
        .priors
        .as_ref()
        .and_then(|p| p.camera_alt.as_ref())
        .map(|a| a.mean - ref_alt);

    // ── Multi-start optimisation ────────────────────────────────────────
    let np = active.len();
    let result = multi_start_optimise(&enu_corrs, &intr, alt_prior_enu, dist_prior, np);

    // ── Convert result back to LLA ──────────────────────────────────────
    let cam_enu = [result.params[0], result.params[1], result.params[2]];
    let (mut cam_lat, mut cam_lon, _) = enu_to_lla(cam_enu, ref_lat, ref_lon, ref_alt);
    let cam_alt = ref_alt + result.params[2];

    // Bounds clamping
    if let Some(b) = req.priors.as_ref().and_then(|p| p.bounds.as_ref()) {
        cam_lat = cam_lat.clamp(b.lat_min, b.lat_max);
        cam_lon = cam_lon.clamp(b.lon_min, b.lon_max);
    }

    let yaw_deg = ((result.params[3] % 360.0) + 360.0) % 360.0;

    // ── Diagnostics ─────────────────────────────────────────────────────
    let (residuals_px, rmse_px) = reprojection_errors(&result.params, &enu_corrs, &intr);
    let covariance = build_covariance(&result, &enu_corrs, ref_lat);

    let mut warnings = Vec::new();
    if !result.converged {
        warnings.push(format!(
            "Optimizer did not fully converge after {} iterations.",
            result.iterations
        ));
    }
    if rmse_px > 50.0 {
        warnings.push(format!(
            "High reprojection error (RMSE = {:.1} px). Solution may be inaccurate.",
            rmse_px
        ));
    }
    if active.len() < 3 {
        warnings.push("Fewer than 3 correspondences: solution is underdetermined.".to_string());
    }

    Ok(SolveResponse {
        pose: Pose {
            lat: cam_lat,
            lon: cam_lon,
            alt: cam_alt,
            yaw_deg,
            pitch_deg: result.params[4],
            roll_deg: result.params[5],
        },
        intrinsics: Intrinsics {
            focal_px,
            cx,
            cy,
            k1: result.params[6],
            k2: result.params[7],
            p1: result.params[8],
            p2: result.params[9],
        },
        covariance,
        diagnostics: Diagnostics {
            rmse_px,
            inlier_ratio: 1.0,
            residuals_px,
            inlier_ids: active.iter().map(|c| c.id.clone()).collect(),
            warnings,
        },
    })
}

// ── Single-point fallback ───────────────────────────────────────────────────

fn solve_single_point(
    req: &SolveRequest,
    corr: &Corr,
    intr: &CameraIntrinsics,
    dist_prior: [f64; 4],
) -> Result<SolveResponse, String> {
    let lat = corr.world.lat;
    let lon = corr.world.lon;
    let alt = req
        .priors
        .as_ref()
        .and_then(|p| p.camera_alt.as_ref().map(|c| c.mean))
        .unwrap_or_else(|| corr.world.alt.map(|g| (g - 2.0).max(0.0)).unwrap_or(2.0));

    Ok(SolveResponse {
        pose: Pose {
            lat,
            lon,
            alt,
            yaw_deg: 0.0,
            pitch_deg: 0.0,
            roll_deg: 0.0,
        },
        intrinsics: Intrinsics {
            focal_px: intr.focal_px,
            cx: intr.cx,
            cy: intr.cy,
            k1: dist_prior[0],
            k2: dist_prior[1],
            p1: dist_prior[2],
            p2: dist_prior[3],
        },
        covariance: fallback_covariance(),
        diagnostics: Diagnostics {
            rmse_px: 0.0,
            inlier_ratio: 1.0,
            residuals_px: vec![0.0],
            inlier_ids: vec![corr.id.clone()],
            warnings: vec![
                "Only one correspondence: camera placed at world point location.".to_string(),
            ],
        },
    })
}

// ── Multi-start wrapper ─────────────────────────────────────────────────────

fn multi_start_optimise(
    corrs: &[EnuCorrespondence],
    intr: &CameraIntrinsics,
    alt_prior: Option<f64>,
    dist_prior: [f64; 4],
    np: usize,
) -> OptResult {
    let base = initialize_pose(corrs, intr, alt_prior, dist_prior);
    let dist = estimate_scene_distance(corrs, intr);

    let me: f64 = corrs.iter().map(|c| c.enu[0]).sum::<f64>() / corrs.len() as f64;
    let mn: f64 = corrs.iter().map(|c| c.enu[1]).sum::<f64>() / corrs.len() as f64;
    let min_u: f64 = corrs.iter().map(|c| c.enu[2]).fold(f64::INFINITY, f64::min);
    let mu: f64 = corrs.iter().map(|c| c.enu[2]).sum::<f64>() / corrs.len() as f64;

    // Altitude candidates (in ENU metres).
    // ref_alt is the centroid altitude, so u=0 means "at centroid height",
    // and u=-ref_alt would be ~0 m MSL.  We try several altitudes to
    // cover ground-level cameras looking up at buildings as well as
    // elevated viewpoints.
    // Also include -ref_alt + 10.0 to represent camera at ~10m above MSL,
    // which is common for ground-level photography.
    let alt_candidates = [
        base[2],                               // heuristic (min_u − 10)
        min_u - 50.0,                          // well below lowest point
        -(mu - min_u).abs().max(50.0) - min_u, // approximately ground level
        min_u - 5.0,                           // just below lowest point
        mu - 100.0,                            // well below mean altitude
    ];

    // Pitch candidates: base pitch  ± offsets to cover ground-level cameras
    // looking up at tall buildings and elevated/aerial viewpoints.
    // Extended range to cover more scenarios including upward-looking cameras.
    let pitch_offsets = [0.0, 10.0, 20.0, -10.0, -20.0, 30.0, -30.0, 40.0, -40.0];

    // Try multiple yaw offsets × distance scales × altitude levels × pitch offsets
    let yaw_offsets = [
        0.0, 22.5, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0, 337.5,
    ];
    let dist_scales = [0.2, 0.5, 1.0, 3.0, 8.0];
    let mut best: Option<OptResult> = None;

    for &off in &yaw_offsets {
        for &ds in &dist_scales {
            for &alt in &alt_candidates {
                for &poff in &pitch_offsets {
                    let mut init = base;
                    init[3] = ((init[3] + off) % 360.0 + 360.0) % 360.0;
                    init[4] = (init[4] + poff).clamp(-80.0, 80.0);
                    let yr = init[3].to_radians();
                    let d = dist * ds;
                    init[0] = me - d * yr.sin();
                    init[1] = mn - d * yr.cos();
                    init[2] = alt;

                    let r = levenberg_marquardt(init, corrs, intr, alt_prior, dist_prior, np, 30);
                    if best.is_none() || r.cost < best.as_ref().unwrap().cost {
                        best = Some(r);
                    }
                }
            }
        }
    }

    // Refine the best candidate with many iterations
    let b = best.unwrap();
    levenberg_marquardt(b.params, corrs, intr, alt_prior, dist_prior, np, 300)
}

// ── Focal-length estimation ─────────────────────────────────────────────────

fn estimate_focal(req: &SolveRequest) -> f64 {
    req.priors
        .as_ref()
        .and_then(|p| p.focal_px.as_ref().map(|f| f.mean))
        .unwrap_or(req.image.width * 0.9)
}

// ── Reprojection-error diagnostics ──────────────────────────────────────────

fn reprojection_errors(
    params: &[f64; NUM_PARAMS],
    corrs: &[EnuCorrespondence],
    base_intr: &CameraIntrinsics,
) -> (Vec<f64>, f64) {
    let cam = [params[0], params[1], params[2]];
    let rot = rotation_enu_to_cam(params[3], params[4], params[5]);
    // Build full intrinsics including optimised distortion coefficients.
    let intr = CameraIntrinsics {
        focal_px: base_intr.focal_px,
        cx: base_intr.cx,
        cy: base_intr.cy,
        k1: params[6],
        k2: params[7],
        p1: params[8],
        p2: params[9],
    };
    let mut res = Vec::with_capacity(corrs.len());
    let mut ss = 0.0;
    for c in corrs {
        let r = match project_point(c.enu, cam, &rot, &intr) {
            Some((u, v)) => (u - c.pixel[0]).hypot(v - c.pixel[1]),
            None => 1000.0,
        };
        ss += r * r;
        res.push(r);
    }
    let rmse = (ss / corrs.len() as f64).sqrt();
    (res, rmse)
}

// ── Covariance ──────────────────────────────────────────────────────────────

fn build_covariance(result: &OptResult, corrs: &[EnuCorrespondence], ref_lat: f64) -> Covariance {
    let n_obs = corrs.len() * 2;
    let sigma2 = if n_obs > NUM_PARAMS {
        result.cost / (n_obs - NUM_PARAMS) as f64
    } else {
        result.cost.max(1.0)
    };

    let cov_raw = invert_nxn(&result.jtj, sigma2);

    // Re-order [e,n,u,yaw,pitch,roll,k1,k2,p1,p2]
    //       → [lat(n),lon(e),alt(u),yaw,pitch,roll,k1,k2,p1,p2]
    let m_lat = 111_320.0;
    let m_lon = 111_320.0 * ref_lat.to_radians().cos();
    // map[i] = which optimizer param corresponds to output param i
    let map: [usize; NUM_PARAMS] = [1, 0, 2, 3, 4, 5, 6, 7, 8, 9];
    let scl: [f64; NUM_PARAMS] = [
        1.0 / m_lat,
        1.0 / m_lon,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
    ];

    let mut out = [0.0f64; NUM_PARAMS * NUM_PARAMS];
    for i in 0..NUM_PARAMS {
        for j in 0..NUM_PARAMS {
            out[i * NUM_PARAMS + j] = cov_raw[map[i] * NUM_PARAMS + map[j]] * scl[i] * scl[j];
        }
    }

    Covariance {
        labels: vec![
            "lat".into(),
            "lon".into(),
            "alt".into(),
            "yawDeg".into(),
            "pitchDeg".into(),
            "rollDeg".into(),
            "k1".into(),
            "k2".into(),
            "p1".into(),
            "p2".into(),
        ],
        matrix: out.to_vec(),
    }
}

fn invert_nxn(mat: &[f64; NUM_PARAMS * NUM_PARAMS], scale: f64) -> [f64; NUM_PARAMS * NUM_PARAMS] {
    const N: usize = NUM_PARAMS;
    // Augmented matrix [A | I], stored as N rows × 2N cols.
    // Use a flat Vec to avoid issues with non-literal const expressions in
    // fixed-size stack arrays.
    let mut a = vec![0.0f64; N * 2 * N];
    for i in 0..N {
        for j in 0..N {
            a[i * 2 * N + j] = mat[i * N + j];
        }
        a[i * 2 * N + (N + i)] = 1.0;
    }

    for col in 0..N {
        // Find pivot row
        let (mut mx, mut mr) = (a[col * 2 * N + col].abs(), col);
        for row in (col + 1)..N {
            let v = a[row * 2 * N + col].abs();
            if v > mx {
                mx = v;
                mr = row;
            }
        }
        if mx < 1e-30 {
            return fallback_cov_matrix();
        }
        if mr != col {
            // Swap rows col and mr
            for k in 0..(2 * N) {
                a.swap(col * 2 * N + k, mr * 2 * N + k);
            }
        }
        let piv = a[col * 2 * N + col];
        for j in 0..(2 * N) {
            a[col * 2 * N + j] /= piv;
        }
        for row in 0..N {
            if row != col {
                let f = a[row * 2 * N + col];
                for j in 0..(2 * N) {
                    let delta = f * a[col * 2 * N + j];
                    a[row * 2 * N + j] -= delta;
                }
            }
        }
    }

    let mut r = [0.0f64; N * N];
    for i in 0..N {
        for j in 0..N {
            r[i * N + j] = a[i * 2 * N + (N + j)] * scale;
        }
    }
    r
}

fn fallback_cov_matrix() -> [f64; NUM_PARAMS * NUM_PARAMS] {
    let mut m = [0.0f64; NUM_PARAMS * NUM_PARAMS];
    // Diagonal heuristics: [lat, lon, alt, yaw, pitch, roll, k1, k2, p1, p2]
    m[0 * NUM_PARAMS + 0] = 1e-8; // lat  variance
    m[1 * NUM_PARAMS + 1] = 1e-8; // lon  variance
    m[2 * NUM_PARAMS + 2] = 4.0; // alt  (m²)
    m[3 * NUM_PARAMS + 3] = 25.0; // yaw  (deg²)
    m[4 * NUM_PARAMS + 4] = 9.0; // pitch(deg²)
    m[5 * NUM_PARAMS + 5] = 9.0; // roll (deg²)
    m[6 * NUM_PARAMS + 6] = 0.25; // k1
    m[7 * NUM_PARAMS + 7] = 0.0625; // k2
    m[8 * NUM_PARAMS + 8] = 2.5e-3; // p1
    m[9 * NUM_PARAMS + 9] = 2.5e-3; // p2
    m
}

fn fallback_covariance() -> Covariance {
    Covariance {
        labels: vec![
            "lat".into(),
            "lon".into(),
            "alt".into(),
            "yawDeg".into(),
            "pitchDeg".into(),
            "rollDeg".into(),
            "k1".into(),
            "k2".into(),
            "p1".into(),
            "p2".into(),
        ],
        matrix: fallback_cov_matrix().to_vec(),
    }
}
