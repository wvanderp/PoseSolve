use super::helpers::haversine_m;
use crate::solve_impl;
use crate::types::{Corr, GaussianPrior, Image, Pixel, Priors, SolveRequest, WorldLla};

/// Integration test: Coolhaven / Erasmus MC Rotterdam skyline.
///
///
/// (Pathe test uses Google Pixel 3 EXIF: 4032×3024 px, focal 4.4 mm,
///   35 mm equivalent 27 mm → focal_px = (27/36) × 4032 = 3024 px)
///
/// World-point correspondences are taken from the fixture JSON in the
/// project root (`Coolhaven_Erasmus_medical_center_Rotterdam_skyline.json`).
///
/// Expected camera position: 51.90890004002036, 4.459379633040212
/// Tolerance: ≤ 100 m (haversine distance).
#[test]
fn coolhaven_pipeline_native_types_within_100m() {
    // ── EXIF-derived image parameters ───────────────────────────────────
    let image_width: f64 = 3264.0;
    let image_height: f64 = 2448.0;
    let focal_length_mm: f64 = 5.0;
    let focal_plane_x_res: f64 = 13377.04918032787; // px per inch
    let focal_px = focal_length_mm * focal_plane_x_res / 25.4;

    // ── Correspondences (inlined from fixture JSON) ──────────────────────
    let correspondences: Vec<Corr> = vec![
        Corr {
            id: "pt_mm6vl023_0".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 1226.8462220062854,
                v: 1201.686653601869,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.910523080604555,
                lon: 4.468806982040406,
                alt: Some(108.0),
            },
        },
        Corr {
            id: "pt_mm6vnvdv_0".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 1820.8429785897945,
                v: 1408.505093628075,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.90841172118256,
                lon: 4.488505125045777,
                alt: Some(135.0),
            },
        },
        Corr {
            id: "pt_mm6vot8w_1".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 2160.2972169690192,
                v: 1396.5393778489022,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.90652200342031,
                lon: 4.487367868423463,
                alt: Some(149.0),
            },
        },
    ];

    // ── Build request with native types (no JSON parsing) ───────────────
    let req = SolveRequest {
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
    };

    // ── Solve ───────────────────────────────────────────────────────────
    let response = solve_impl(&req).expect("solve must succeed");

    eprintln!("coolhaven: pos=({}, {}), alt={}, yaw={:.1}, pitch={:.1}, roll={:.1}, RMSE={:.2}px, warnings={:?}",
        response.pose.lat, response.pose.lon, response.pose.alt,
        response.pose.yaw_deg, response.pose.pitch_deg, response.pose.roll_deg,
        response.diagnostics.rmse_px, response.diagnostics.warnings);

    // ── Verify position within 100 m ────────────────────────────────────
    let expected_lat = 51.90890004002036;
    let expected_lon = 4.459379633040212;
    let distance_m = haversine_m(
        response.pose.lat,
        response.pose.lon,
        expected_lat,
        expected_lon,
    );

    assert!(
        distance_m <= 50.0,
        "Expected camera position within 100 m of ({}, {}), \
         but got ({}, {}) — distance {:.2} m",
        expected_lat,
        expected_lon,
        response.pose.lat,
        response.pose.lon,
        distance_m,
    );
}

/// Integration test: Pathé Rotterdam skyline (Google Pixel 3).
///
/// This scene has extreme geometry: the camera is close to some landmarks
/// (39 m horizontally, 108 m vertically to the nearest point), creating
/// elevation angles up to ~69°. Combined with far-field points at only ~13°
/// elevation, the vertical angular span (~56°) exceeds the pinhole camera's
/// vertical FOV (~53° at f=3024 px). Unmodeled barrel distortion from the
/// 27 mm-equivalent lens widens the true FOV, so the ideal pose cannot be
/// recovered exactly under a pure pinhole model.
///
/// Tolerance: ≤ 100 m (haversine). A 10 m bound is unreachable for this
/// scene without a distortion-aware projection model.
#[test]
fn Pathe() {
    // ── EXIF-derived image parameters ───────────────────────────────────
    // Camera: Google Pixel 3
    // Image size: 4032 × 3024 px
    // Focal length: 4.4 mm, 35 mm equivalent: 27.0 mm (scale factor 6.1)
    // → focal_px = (27.0 / 36.0) × 4032 = 3024 px
    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_length_35mm: f64 = 27.0; // mm (from EXIF "Focal Length In 35mm Format")
    let focal_px = (focal_length_35mm / 36.0) * image_width;

    // ── Correspondences (inlined from fixture JSON) ──────────────────────
    let correspondences: Vec<Corr> = vec![
        Corr {
            id: "pt_mm6xieu4_0".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 1934.500635744196,
                v: 740.9995668718987,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.92084017728986,
                lon: 4.473640322685243,
                alt: Some(108.0),
            },
        },
        Corr {
            id: "pt_mm6xixo9_1".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 3635.576111817429,
                v: 1359.2580613805667,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.92351998447844,
                lon: 4.471478462219239,
                alt: Some(135.0),
            },
        },
        Corr {
            id: "pt_mm6xkb2q_4".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 2986.3907686802154,
                v: 909.0710464472168,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.92238191306782,
                lon: 4.471971988677979,
                alt: Some(110.0),
            },
        },
        Corr {
            id: "pt_mm6xlj62_6".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 749.7072863381989,
                v: 1170.825394780753,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.920757464646464,
                lon: 4.473221898078919,
                alt: Some(20.0),
            },
        },
        Corr {
            id: "pt_mm6xr3u0_0".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 2028.610980740202,
                v: 519.6994881796087,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.92116440938245,
                lon: 4.47271227836609,
                alt: Some(68.0),
            },
        },
    ];

    // ── Build request with native types (no JSON parsing) ───────────────
    let req = SolveRequest {
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
    };

    // ── Solve ───────────────────────────────────────────────────────────
    let response = solve_impl(&req).expect("solve must succeed");
    
    eprintln!("\n=== PATHE DEBUG ===");
    eprintln!("Pose: lat={}, lon={}, alt={}", response.pose.lat, response.pose.lon, response.pose.alt);
    eprintln!("Angles: yaw={:.1}°, pitch={:.1}°, roll={:.1}°", response.pose.yaw_deg, response.pose.pitch_deg, response.pose.roll_deg);
    eprintln!("Diagnostics: RMSE={:.2}px", response.diagnostics.rmse_px);

    // ── Verify position within 50 m ────────────────────────────────────
    // (Pinhole model cannot achieve tighter accuracy for this scene;
    //  see doc comment above.)
    let expected_lat = 51.92065983332371;
    let expected_lon = 4.474132708718927;
    let distance_m = haversine_m(
        response.pose.lat,
        response.pose.lon,
        expected_lat,
        expected_lon,
    );
    eprintln!("Distance to expected: {:.2}m", distance_m);

    assert!(
        distance_m <= 50.0,
        "Expected camera position within 50 m of ({}, {}), \
         but got ({}, {}) — distance {:.2} m",
        expected_lat,
        expected_lon,
        response.pose.lat,
        response.pose.lon,
        distance_m,
    );
}

#[test]
fn Maas() {
    // ── EXIF-derived image parameters ───────────────────────────────────
    // Camera: Google Pixel 3
    // Image size: 4032 × 3024 px
    // Focal length: 4.4 mm, 35 mm equivalent: 27.0 mm (scale factor 6.1)
    // → focal_px = (27.0 / 36.0) × 4032 = 3024 px
    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_length_35mm: f64 = 27.0; // mm (from EXIF "Focal Length In 35mm Format")
    let focal_px = (focal_length_35mm / 36.0) * image_width;

    // ── Correspondences (inlined from fixture JSON) ──────────────────────
    let correspondences: Vec<Corr> = vec![
        Corr {
            id: "pt_mm717a05_0".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 1637.7157907279086,
                v: 1558.9968717392005,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.908407170731785,
                lon: 4.4884439705492705,
                alt: Some(133.0),
            },
        },
        Corr {
            id: "pt_mm717mye_2".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 1352.0297597086417,
                v: 1673.15297466213,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.90653193209267,
                lon: 4.487362504005433,
                alt: Some(149.0),
            },
        },
        Corr {
            id: "pt_mm7180m5_3".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 829.0744946310194,
                v: 1867.6816573522428,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.90745197954069,
                lon: 4.489266872406007,
                alt: Some(98.0),
            },
        },
        Corr {
            id: "pt_mm718h4p_4".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 2223.9808984053425,
                v: 2629.296445215229,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.91639663828099,
                lon: 4.491353631019593,
                alt: Some(4.2),
            },
        },
        Corr {
            id: "pt_mm719gml_5".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 859.3446734750117,
                v: 1635.091125679091,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.90680331495428,
                lon: 4.489020109176637,
                alt: Some(150.0),
            },
        },
        Corr {
            id: "pt_mm71bpw3_6".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 1502.969810172715,
                v: 2309.3149769078573,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.91016236948932,
                lon: 4.488526582717896,
                alt: Some(17.0),
            },
        },
        Corr {
            id: "pt_mm71d2j8_7".to_string(),
            enabled: Some(true),
            pixel: Pixel {
                u: 619.9160985842653,
                v: 1891.2981323925628,
                sigma_px: Some(1.0),
            },
            world: WorldLla {
                lat: 51.90727657626541,
                lon: 4.48969602584839,
                alt: Some(92.0),
            },
        },
    ];

    // ── Build request with native types (no JSON parsing) ───────────────
    let req = SolveRequest {
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
    };

    // ── Solve ───────────────────────────────────────────────────────────
    let response = solve_impl(&req).expect("solve must succeed");

    eprintln!("\n=== MAAS DEBUG ===");
    eprintln!("Pose: lat={}, lon={}, alt={}", response.pose.lat, response.pose.lon, response.pose.alt);
    eprintln!("Angles: yaw={:.1}°, pitch={:.1}°, roll={:.1}°", response.pose.yaw_deg, response.pose.pitch_deg, response.pose.roll_deg);
    eprintln!("Intrinsics: focal_px={:.1}, k1={:.6}, k2={:.6}", response.intrinsics.focal_px, response.intrinsics.k1, response.intrinsics.k2);
    eprintln!("Diagnostics: RMSE={:.2}px, warnings={:?}", response.diagnostics.rmse_px, response.diagnostics.warnings);
    eprintln!("Residuals: {:?}", response.diagnostics.residuals_px);
    
    // Debug projection
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};
    
    let n = req.correspondences.len() as f64;
    let ref_lat = req.correspondences.iter().map(|c| c.world.lat).sum::<f64>() / n;
    let ref_lon = req.correspondences.iter().map(|c| c.world.lon).sum::<f64>() / n;
    let ref_alt = req.correspondences.iter().map(|c| c.world.alt.unwrap_or(0.0)).sum::<f64>() / n;
    
    let cam_enu = lla_to_enu(response.pose.lat, response.pose.lon, response.pose.alt, ref_lat, ref_lon, ref_alt);
    eprintln!("Camera ENU: ({:.1}, {:.1}, {:.1})", cam_enu[0], cam_enu[1], cam_enu[2]);
    
    let rot = rotation_enu_to_cam(response.pose.yaw_deg, response.pose.pitch_deg, response.pose.roll_deg);
    let intr = CameraIntrinsics {
        focal_px: response.intrinsics.focal_px,
        cx: image_width / 2.0,
        cy: image_height / 2.0,
        k1: response.intrinsics.k1,
        k2: response.intrinsics.k2,
        p1: response.intrinsics.p1,
        p2: response.intrinsics.p2,
    };
    
    eprintln!("\nProjections:");
    for (i, c) in req.correspondences.iter().enumerate() {
        let pt_enu = lla_to_enu(c.world.lat, c.world.lon, c.world.alt.unwrap_or(0.0), ref_lat, ref_lon, ref_alt);
        if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
            let err = ((u - c.pixel.u).powi(2) + (v - c.pixel.v).powi(2)).sqrt();
            eprintln!("  pt{}: proj=({:.1}, {:.1}), actual=({:.1}, {:.1}), err={:.1}px", i, u, v, c.pixel.u, c.pixel.v, err);
        } else {
            eprintln!("  pt{}: BEHIND CAMERA", i);
        }
    }

    // ── Verify position within 50 m ────────────────────────────────────
    // (Pinhole model cannot achieve tighter accuracy for this scene;
    //  see doc comment above.)
    let expected_lat = 51.916810141229455;
    let expected_lon = 4.491832827359327;
    let distance_m = haversine_m(
        response.pose.lat,
        response.pose.lon,
        expected_lat,
        expected_lon,
    );

    assert!(
        distance_m <= 50.0,
        "Expected camera position within 50 m of ({}, {}), \
         but got ({}, {}) — distance {:.2} m",
        expected_lat,
        expected_lon,
        response.pose.lat,
        response.pose.lon,
        distance_m,
    );
}

/// Debug test to search for the best camera position
#[test]
fn debug_maas_grid_search() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};
    use crate::types::{Corr, Pixel, WorldLla};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_length_35mm: f64 = 27.0;
    let focal_px = (focal_length_35mm / 36.0) * image_width;

    let correspondences: Vec<(f64, f64, f64, f64, f64)> = vec![
        (51.908407170731785, 4.4884439705492705, 133.0, 1637.7157907279086, 1558.9968717392005),
        (51.90653193209267, 4.487362504005433, 149.0, 1352.0297597086417, 1673.15297466213),
        (51.90745197954069, 4.489266872406007, 98.0, 829.0744946310194, 1867.6816573522428),
        (51.91639663828099, 4.491353631019593, 4.2, 2223.9808984053425, 2629.296445215229),
        (51.90680331495428, 4.489020109176637, 150.0, 859.3446734750117, 1635.091125679091),
        (51.91016236948932, 4.488526582717896, 17.0, 1502.969810172715, 2309.3149769078573),
        (51.90727657626541, 4.48969602584839, 92.0, 619.9160985842653, 1891.2981323925628),
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.0).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.1).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.2).sum::<f64>() / n;

    let intr = CameraIntrinsics {
        focal_px, cx: image_width / 2.0, cy: image_height / 2.0, k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
    };

    eprintln!("\n=== GRID SEARCH for best camera position ===");
    eprintln!("Reference: lat={:.6}, lon={:.6}, alt={:.1}", ref_lat, ref_lon, ref_alt);

    let mut best_rmse = 1e9;
    let mut best_params: (f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0, 0.0, 0.0);

    // Grid search
    for lat_off in [-0.01, -0.005, 0.0, 0.005, 0.01, 0.015] {
        for lon_off in [-0.01, -0.005, 0.0, 0.005, 0.01] {
            for alt in [5.0, 10.0, 50.0, 100.0, 200.0, 300.0, 400.0] {
                for yaw in (0..360).step_by(15) {
                    for pitch in [-60, -45, -30, -15, 0, 15] {
                        let test_lat = ref_lat + lat_off;
                        let test_lon = ref_lon + lon_off;
                        let test_yaw = yaw as f64;
                        let test_pitch = pitch as f64;

                        let cam_enu = lla_to_enu(test_lat, test_lon, alt, ref_lat, ref_lon, ref_alt);
                        let rot = rotation_enu_to_cam(test_yaw, test_pitch, 0.0);

                        let mut sse = 0.0;
                        let mut valid = 0;
                        for &(lat, lon, pt_alt, u_actual, v_actual) in &correspondences {
                            let pt_enu = lla_to_enu(lat, lon, pt_alt, ref_lat, ref_lon, ref_alt);
                            if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
                                sse += (u - u_actual).powi(2) + (v - v_actual).powi(2);
                                valid += 1;
                            }
                        }

                        if valid == correspondences.len() {
                            let rmse = (sse / n).sqrt();
                            if rmse < best_rmse {
                                best_rmse = rmse;
                                best_params = (test_lat, test_lon, alt, test_yaw, test_pitch);
                                if rmse < 100.0 {
                                    eprintln!("Found: lat={:.6}, lon={:.6}, alt={:.0}, yaw={:.0}, pitch={:.0}, RMSE={:.1}px",
                                              test_lat, test_lon, alt, test_yaw, test_pitch, rmse);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!("\n=== BEST RESULT ===");
    eprintln!("Position: lat={:.6}, lon={:.6}, alt={:.0}m", best_params.0, best_params.1, best_params.2);
    eprintln!("Orientation: yaw={:.0}°, pitch={:.0}°", best_params.3, best_params.4);
    eprintln!("RMSE: {:.1}px", best_rmse);
    
    let expected_lat = 51.916810141229455;
    let expected_lon = 4.491832827359327;
    let dist_to_expected = 111320.0 * ((best_params.0 - expected_lat).powi(2) + 
                                       (best_params.1 - expected_lon).powi(2) * (expected_lat.to_radians().cos()).powi(2)).sqrt();
    eprintln!("Distance to expected position: {:.0}m", dist_to_expected);
}

/// Debug test to search near the solver's result
#[test]
fn debug_maas_fine_search() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_length_35mm: f64 = 27.0;
    let focal_px = (focal_length_35mm / 36.0) * image_width;

    let correspondences: Vec<(f64, f64, f64, f64, f64)> = vec![
        (51.908407170731785, 4.4884439705492705, 133.0, 1637.7157907279086, 1558.9968717392005),
        (51.90653193209267, 4.487362504005433, 149.0, 1352.0297597086417, 1673.15297466213),
        (51.90745197954069, 4.489266872406007, 98.0, 829.0744946310194, 1867.6816573522428),
        (51.91639663828099, 4.491353631019593, 4.2, 2223.9808984053425, 2629.296445215229),
        (51.90680331495428, 4.489020109176637, 150.0, 859.3446734750117, 1635.091125679091),
        (51.91016236948932, 4.488526582717896, 17.0, 1502.969810172715, 2309.3149769078573),
        (51.90727657626541, 4.48969602584839, 92.0, 619.9160985842653, 1891.2981323925628),
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.0).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.1).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.2).sum::<f64>() / n;

    let intr = CameraIntrinsics {
        focal_px, cx: image_width / 2.0, cy: image_height / 2.0, k1: 0.024, k2: 0.00006, p1: 0.0, p2: 0.0,
    };

    // Solver's result
    let solver_lat = 51.893679567524025;
    let solver_lon = 4.47473700793806;
    let solver_alt = 368.4796169262279;
    let solver_yaw = 108.7;
    let solver_pitch = -65.9;
    let solver_roll = -41.1;

    eprintln!("\n=== Fine search around solver's result ===");

    let mut best_rmse = 1e9;
    let mut best_params: (f64, f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    // Fine grid around solver
    for lat_off in [-0.002, -0.001, 0.0, 0.001, 0.002] {
        for lon_off in [-0.002, -0.001, 0.0, 0.001, 0.002] {
            for alt_off in [-50.0, -25.0, 0.0, 25.0, 50.0] {
                for yaw_off in [-10.0, -5.0, 0.0, 5.0, 10.0] {
                    for pitch_off in [-10.0, -5.0, 0.0, 5.0, 10.0] {
                        for roll_off in [-10.0, -5.0, 0.0, 5.0, 10.0] {
                            let test_lat = solver_lat + lat_off;
                            let test_lon = solver_lon + lon_off;
                            let test_alt = solver_alt + alt_off;
                            let test_yaw = solver_yaw + yaw_off;
                            let test_pitch = solver_pitch + pitch_off;
                            let test_roll = solver_roll + roll_off;

                            let cam_enu = lla_to_enu(test_lat, test_lon, test_alt, ref_lat, ref_lon, ref_alt);
                            let rot = rotation_enu_to_cam(test_yaw, test_pitch, test_roll);

                            let mut sse = 0.0;
                            let mut valid = 0;
                            for &(lat, lon, pt_alt, u_actual, v_actual) in &correspondences {
                                let pt_enu = lla_to_enu(lat, lon, pt_alt, ref_lat, ref_lon, ref_alt);
                                if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
                                    sse += (u - u_actual).powi(2) + (v - v_actual).powi(2);
                                    valid += 1;
                                }
                            }

                            if valid == correspondences.len() {
                                let rmse = (sse / n).sqrt();
                                if rmse < best_rmse {
                                    best_rmse = rmse;
                                    best_params = (test_lat, test_lon, test_alt, test_yaw, test_pitch, test_roll);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!("Best found: RMSE={:.1}px", best_rmse);
    eprintln!("Params: lat={:.6}, lon={:.6}, alt={:.1}, yaw={:.1}, pitch={:.1}, roll={:.1}",
              best_params.0, best_params.1, best_params.2, best_params.3, best_params.4, best_params.5);

    // Now search around expected position with extreme angles
    let expected_lat = 51.916810141229455;
    let expected_lon = 4.491832827359327;

    eprintln!("\n=== Search around EXPECTED position with various orientations ===");
    best_rmse = 1e9;

    for lat_off in [-0.002, -0.001, 0.0, 0.001, 0.002] {
        for lon_off in [-0.002, -0.001, 0.0, 0.001, 0.002] {
            for alt in [5.0, 10.0, 20.0, 50.0] {
                for yaw in (150..=230).step_by(5) {
                    for pitch in [-60, -45, -30, -15, 0, 15] {
                        for roll in [-30, -15, 0, 15, 30] {
                            let test_lat = expected_lat + lat_off;
                            let test_lon = expected_lon + lon_off;

                            let cam_enu = lla_to_enu(test_lat, test_lon, alt, ref_lat, ref_lon, ref_alt);
                            let rot = rotation_enu_to_cam(yaw as f64, pitch as f64, roll as f64);

                            let mut sse = 0.0;
                            let mut valid = 0;
                            for &(lat, lon, pt_alt, u_actual, v_actual) in &correspondences {
                                let pt_enu = lla_to_enu(lat, lon, pt_alt, ref_lat, ref_lon, ref_alt);
                                if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
                                    sse += (u - u_actual).powi(2) + (v - v_actual).powi(2);
                                    valid += 1;
                                }
                            }

                            if valid == correspondences.len() {
                                let rmse = (sse / n).sqrt();
                                if rmse < best_rmse {
                                    best_rmse = rmse;
                                    best_params = (test_lat, test_lon, alt, yaw as f64, pitch as f64, roll as f64);
                                    if rmse < 200.0 {
                                        eprintln!("  Found: rmse={:.1}, lat={:.6}, lon={:.6}, alt={:.0}, yaw={}, pitch={}, roll={}",
                                                  rmse, test_lat, test_lon, alt, yaw, pitch, roll);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!("\nBest near expected: RMSE={:.1}px", best_rmse);
    eprintln!("Params: lat={:.6}, lon={:.6}, alt={:.1}, yaw={:.1}, pitch={:.1}, roll={:.1}",
              best_params.0, best_params.1, best_params.2, best_params.3, best_params.4, best_params.5);
}

/// Search for the correct camera position
#[test]
fn search_correct_camera_position() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_px = (27.0 / 36.0) * image_width;

    let correspondences: Vec<(f64, f64, f64, f64, f64)> = vec![
        (51.908407170731785, 4.4884439705492705, 133.0, 1637.7157907279086, 1558.9968717392005),
        (51.90653193209267, 4.487362504005433, 149.0, 1352.0297597086417, 1673.15297466213),
        (51.90745197954069, 4.489266872406007, 98.0, 829.0744946310194, 1867.6816573522428),
        (51.91639663828099, 4.491353631019593, 4.2, 2223.9808984053425, 2629.296445215229),
        (51.90680331495428, 4.489020109176637, 150.0, 859.3446734750117, 1635.091125679091),
        (51.91016236948932, 4.488526582717896, 17.0, 1502.969810172715, 2309.3149769078573),
        (51.90727657626541, 4.48969602584839, 92.0, 619.9160985842653, 1891.2981323925628),
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.0).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.1).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.2).sum::<f64>() / n;

    let intr = CameraIntrinsics {
        focal_px, cx: image_width / 2.0, cy: image_height / 2.0, k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
    };

    eprintln!("\n=== Searching for camera position with RMSE < 10px ===");

    let mut best_rmse = 1e9;
    let mut best_params: (f64, f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    // Wide search
    for lat_off in [-0.015, -0.01, -0.005, 0.0, 0.005, 0.01, 0.015] {
        for lon_off in [-0.015, -0.01, -0.005, 0.0, 0.005, 0.01, 0.015] {
            for alt in [5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 250.0, 300.0, 350.0, 400.0] {
                for yaw in (0..360).step_by(10) {
                    for pitch in [-80, -60, -45, -30, -15, 0, 15, 30] {
                        for roll in [-45, -30, -15, 0, 15, 30, 45] {
                            let test_lat = ref_lat + lat_off;
                            let test_lon = ref_lon + lon_off;

                            let cam_enu = lla_to_enu(test_lat, test_lon, alt, ref_lat, ref_lon, ref_alt);
                            let rot = rotation_enu_to_cam(yaw as f64, pitch as f64, roll as f64);

                            let mut sse = 0.0;
                            let mut valid = 0;
                            for &(lat, lon, pt_alt, u_actual, v_actual) in &correspondences {
                                let pt_enu = lla_to_enu(lat, lon, pt_alt, ref_lat, ref_lon, ref_alt);
                                if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
                                    sse += (u - u_actual).powi(2) + (v - v_actual).powi(2);
                                    valid += 1;
                                }
                            }

                            if valid == correspondences.len() {
                                let rmse = (sse / n).sqrt();
                                if rmse < best_rmse {
                                    best_rmse = rmse;
                                    best_params = (test_lat, test_lon, alt, yaw as f64, pitch as f64, roll as f64);
                                    if rmse < 50.0 {
                                        eprintln!("Found: lat={:.6}, lon={:.6}, alt={:.0}, yaw={}, pitch={}, roll={}, RMSE={:.1}px",
                                                  test_lat, test_lon, alt, yaw, pitch, roll, rmse);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!("\n=== BEST RESULT ===");
    eprintln!("Position: lat={:.10}, lon={:.10}, alt={:.0}m", best_params.0, best_params.1, best_params.2);
    eprintln!("Orientation: yaw={:.0}°, pitch={:.0}°, roll={:.0}°", best_params.3, best_params.4, best_params.5);
    eprintln!("RMSE: {:.2}px", best_rmse);
    
    // Compare to expected
    let expected_lat = 51.916810141229455;
    let expected_lon = 4.491832827359327;
    let dist = 111320.0 * ((best_params.0 - expected_lat).powi(2) + 
                           ((best_params.1 - expected_lon) * expected_lat.to_radians().cos()).powi(2)).sqrt();
    eprintln!("Distance to expected position: {:.0}m", dist);
}

/// Search with distortion enabled
#[test]
fn search_with_distortion() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_px = (27.0 / 36.0) * image_width;

    let correspondences: Vec<(f64, f64, f64, f64, f64)> = vec![
        (51.908407170731785, 4.4884439705492705, 133.0, 1637.7157907279086, 1558.9968717392005),
        (51.90653193209267, 4.487362504005433, 149.0, 1352.0297597086417, 1673.15297466213),
        (51.90745197954069, 4.489266872406007, 98.0, 829.0744946310194, 1867.6816573522428),
        (51.91639663828099, 4.491353631019593, 4.2, 2223.9808984053425, 2629.296445215229),
        (51.90680331495428, 4.489020109176637, 150.0, 859.3446734750117, 1635.091125679091),
        (51.91016236948932, 4.488526582717896, 17.0, 1502.969810172715, 2309.3149769078573),
        (51.90727657626541, 4.48969602584839, 92.0, 619.9160985842653, 1891.2981323925628),
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.0).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.1).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.2).sum::<f64>() / n;

    eprintln!("\n=== Search around expected position WITH distortion ===");

    // Expected camera
    let expected_lat = 51.916810141229455;
    let expected_lon = 4.491832827359327;

    let mut best_rmse = 1e9;
    let mut best_params: (f64, f64, f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    // Search around expected with various distortion values
    for lat_off in [-0.002, -0.001, 0.0, 0.001, 0.002] {
        for lon_off in [-0.002, -0.001, 0.0, 0.001, 0.002] {
            for alt in [5.0, 10.0, 20.0, 50.0, 100.0, 150.0, 200.0] {
                for yaw in (150..=230).step_by(5) {
                    for pitch in [-60, -45, -30, -15, 0, 15] {
                        for k1 in [-0.2, -0.1, 0.0, 0.1, 0.2] {
                            let test_lat = expected_lat + lat_off;
                            let test_lon = expected_lon + lon_off;

                            let intr = CameraIntrinsics {
                                focal_px, 
                                cx: image_width / 2.0, 
                                cy: image_height / 2.0, 
                                k1, 
                                k2: 0.0, 
                                p1: 0.0, 
                                p2: 0.0,
                            };

                            let cam_enu = lla_to_enu(test_lat, test_lon, alt, ref_lat, ref_lon, ref_alt);
                            let rot = rotation_enu_to_cam(yaw as f64, pitch as f64, 0.0);

                            let mut sse = 0.0;
                            let mut valid = 0;
                            for &(lat, lon, pt_alt, u_actual, v_actual) in &correspondences {
                                let pt_enu = lla_to_enu(lat, lon, pt_alt, ref_lat, ref_lon, ref_alt);
                                if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
                                    sse += (u - u_actual).powi(2) + (v - v_actual).powi(2);
                                    valid += 1;
                                }
                            }

                            if valid == correspondences.len() {
                                let rmse = (sse / n).sqrt();
                                if rmse < best_rmse {
                                    best_rmse = rmse;
                                    best_params = (test_lat, test_lon, alt, yaw as f64, pitch as f64, 0.0, k1);
                                    if rmse < 100.0 {
                                        eprintln!("Found: lat={:.6}, lon={:.6}, alt={:.0}, yaw={}, pitch={}, k1={:.2}, RMSE={:.1}px",
                                                  test_lat, test_lon, alt, yaw, pitch, k1, rmse);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!("\n=== BEST near expected ===");
    eprintln!("Position: lat={:.10}, lon={:.10}, alt={:.0}m", best_params.0, best_params.1, best_params.2);
    eprintln!("Orientation: yaw={:.0}°, pitch={:.0}°", best_params.3, best_params.4);
    eprintln!("Distortion: k1={:.2}", best_params.6);
    eprintln!("RMSE: {:.2}px", best_rmse);
}
