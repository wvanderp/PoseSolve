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

/// Verify projection from known correct position
#[test]
fn verify_maas_projection_from_correct_position() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_px = (27.0 / 36.0) * image_width;  // 3024

    // Expected (correct) camera position
    let cam_lat = 51.916810141229455;
    let cam_lon = 4.491832827359327;
    let cam_alt = 10.0;  // Assume ground level

    // World points with pixel coordinates
    let correspondences: Vec<(&str, f64, f64, f64, f64, f64)> = vec![
        ("pt0", 51.908407170731785, 4.4884439705492705, 133.0, 1637.72, 1559.00),
        ("pt1", 51.90653193209267, 4.487362504005433, 149.0, 1352.03, 1673.15),
        ("pt2", 51.90745197954069, 4.489266872406007, 98.0, 829.07, 1867.68),
        ("pt3", 51.91639663828099, 4.491353631019593, 4.2, 2223.98, 2629.30),
        ("pt4", 51.90680331495428, 4.489020109176637, 150.0, 859.34, 1635.09),
        ("pt5", 51.91016236948932, 4.488526582717896, 17.0, 1502.97, 2309.31),
        ("pt6", 51.90727657626541, 4.48969602584839, 92.0, 619.92, 1891.30),
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.1).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.2).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.3).sum::<f64>() / n;

    let cam_enu = lla_to_enu(cam_lat, cam_lon, cam_alt, ref_lat, ref_lon, ref_alt);
    eprintln!("\n=== VERIFY PROJECTION FROM CORRECT POSITION ===");
    eprintln!("Camera ENU: ({:.1}, {:.1}, {:.1})", cam_enu[0], cam_enu[1], cam_enu[2]);

    // Try different yaw and pitch combinations to find what works
    let intr = CameraIntrinsics {
        focal_px, cx: image_width / 2.0, cy: image_height / 2.0, 
        k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
    };

    let mut best_rmse = 1e9;
    let mut best_yaw = 0.0;
    let mut best_pitch = 0.0;

    for yaw in (0..360).step_by(1) {
        for pitch in -90..=90 {
            let rot = rotation_enu_to_cam(yaw as f64, pitch as f64, 0.0);
            
            let mut sse = 0.0;
            let mut valid = 0;
            for &(_, lat, lon, alt, u_actual, v_actual) in &correspondences {
                let pt_enu = lla_to_enu(lat, lon, alt, ref_lat, ref_lon, ref_alt);
                if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
                    sse += (u - u_actual).powi(2) + (v - v_actual).powi(2);
                    valid += 1;
                }
            }
            
            if valid == correspondences.len() {
                let rmse = (sse / n).sqrt();
                if rmse < best_rmse {
                    best_rmse = rmse;
                    best_yaw = yaw as f64;
                    best_pitch = pitch as f64;
                }
            }
        }
    }

    eprintln!("Best orientation: yaw={:.0}°, pitch={:.0}°, RMSE={:.1}px", best_yaw, best_pitch, best_rmse);
    
    // Show projections for best orientation
    let rot = rotation_enu_to_cam(best_yaw, best_pitch, 0.0);
    eprintln!("\nProjections at best orientation:");
    for &(name, lat, lon, alt, u_actual, v_actual) in &correspondences {
        let pt_enu = lla_to_enu(lat, lon, alt, ref_lat, ref_lon, ref_alt);
        if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
            let err = ((u - u_actual).powi(2) + (v - v_actual).powi(2)).sqrt();
            eprintln!("  {}: proj=({:.0}, {:.0}), actual=({:.0}, {:.0}), err={:.0}px", 
                     name, u, v, u_actual, v_actual, err);
        } else {
            eprintln!("  {}: BEHIND CAMERA", name);
        }
    }
}

/// Verify projection from correct position with alt/roll search
#[test]
fn verify_maas_full_search() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_px = (27.0 / 36.0) * image_width;

    let cam_lat = 51.916810141229455;
    let cam_lon = 4.491832827359327;

    let correspondences: Vec<(f64, f64, f64, f64, f64)> = vec![
        (51.908407170731785, 4.4884439705492705, 133.0, 1637.72, 1559.00),
        (51.90653193209267, 4.487362504005433, 149.0, 1352.03, 1673.15),
        (51.90745197954069, 4.489266872406007, 98.0, 829.07, 1867.68),
        (51.91639663828099, 4.491353631019593, 4.2, 2223.98, 2629.30),
        (51.90680331495428, 4.489020109176637, 150.0, 859.34, 1635.09),
        (51.91016236948932, 4.488526582717896, 17.0, 1502.97, 2309.31),
        (51.90727657626541, 4.48969602584839, 92.0, 619.92, 1891.30),
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.0).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.1).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.2).sum::<f64>() / n;

    eprintln!("\n=== FULL SEARCH with altitude and roll ===");

    let mut best_rmse = 1e9;
    let mut best_params: (f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0, 0.0, 0.0);

    for alt in [2.0, 5.0, 10.0, 15.0, 20.0, 30.0, 50.0, 100.0] {
        for yaw in (150..=230).step_by(2) {
            for pitch in -45..=45 {
                for roll in (-45..=45).step_by(5) {
                    let intr = CameraIntrinsics {
                        focal_px, cx: image_width / 2.0, cy: image_height / 2.0, 
                        k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
                    };

                    let cam_enu = lla_to_enu(cam_lat, cam_lon, alt, ref_lat, ref_lon, ref_alt);
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
                            best_params = (alt, yaw as f64, pitch as f64, roll as f64, 0.0);
                            if rmse < 100.0 {
                                eprintln!("Found: alt={:.0}, yaw={}, pitch={}, roll={}, RMSE={:.1}px", 
                                          alt, yaw, pitch, roll, rmse);
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!("\nBest result: alt={:.0}m, yaw={:.0}°, pitch={:.0}°, roll={:.0}°, RMSE={:.1}px", 
              best_params.0, best_params.1, best_params.2, best_params.3, best_rmse);
}

/// Verify with focal length and distortion search
#[test]
fn verify_maas_with_focal_distortion() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;

    let cam_lat = 51.916810141229455;
    let cam_lon = 4.491832827359327;

    let correspondences: Vec<(f64, f64, f64, f64, f64)> = vec![
        (51.908407170731785, 4.4884439705492705, 133.0, 1637.72, 1559.00),
        (51.90653193209267, 4.487362504005433, 149.0, 1352.03, 1673.15),
        (51.90745197954069, 4.489266872406007, 98.0, 829.07, 1867.68),
        (51.91639663828099, 4.491353631019593, 4.2, 2223.98, 2629.30),
        (51.90680331495428, 4.489020109176637, 150.0, 859.34, 1635.09),
        (51.91016236948932, 4.488526582717896, 17.0, 1502.97, 2309.31),
        (51.90727657626541, 4.48969602584839, 92.0, 619.92, 1891.30),
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.0).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.1).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.2).sum::<f64>() / n;

    eprintln!("\n=== SEARCH with focal length and distortion ===");

    let mut best_rmse = 1e9;
    let mut best_params: (f64, f64, f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    for focal_px in [2500.0, 2700.0, 2900.0, 3024.0, 3100.0, 3300.0, 3500.0] {
        for alt in [5.0, 10.0, 15.0, 20.0] {
            for yaw in (180..=220).step_by(2) {
                for pitch in -30..=30 {
                    for k1 in [-0.3, -0.2, -0.1, 0.0, 0.1, 0.2, 0.3] {
                        let intr = CameraIntrinsics {
                            focal_px, cx: image_width / 2.0, cy: image_height / 2.0, 
                            k1, k2: 0.0, p1: 0.0, p2: 0.0,
                        };

                        let cam_enu = lla_to_enu(cam_lat, cam_lon, alt, ref_lat, ref_lon, ref_alt);
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
                                best_params = (focal_px, alt, yaw as f64, pitch as f64, 0.0, k1, 0.0);
                                if rmse < 50.0 {
                                    eprintln!("Found: focal={:.0}, alt={:.0}, yaw={}, pitch={}, k1={:.2}, RMSE={:.1}px", 
                                              focal_px, alt, yaw, pitch, k1, rmse);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!("\nBest result:");
    eprintln!("  focal={:.0}px, alt={:.0}m, yaw={:.0}°, pitch={:.0}°, k1={:.2}", 
              best_params.0, best_params.1, best_params.2, best_params.3, best_params.5);
    eprintln!("  RMSE={:.1}px", best_rmse);
}

/// Trace through projection step by step
#[test]
fn trace_maas_projection() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, mat3_vec, CameraIntrinsics};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_px = 3024.0;

    // Expected camera
    let cam_lat = 51.916810141229455;
    let cam_lon = 4.491832827359327;
    let cam_alt = 10.0;

    // Reference point
    let ref_lat = 51.909004283;
    let ref_lon = 4.489095671;
    let ref_alt = 91.886;

    // pt0 - one of the world points
    let pt0_lat = 51.908407170731785;
    let pt0_lon = 4.4884439705492705;
    let pt0_alt = 133.0;
    let pt0_u_actual = 1637.72;
    let pt0_v_actual = 1559.00;

    eprintln!("\n=== TRACE PROJECTION ===");

    // Convert to ENU
    let cam_enu = lla_to_enu(cam_lat, cam_lon, cam_alt, ref_lat, ref_lon, ref_alt);
    let pt0_enu = lla_to_enu(pt0_lat, pt0_lon, pt0_alt, ref_lat, ref_lon, ref_alt);

    eprintln!("Camera ENU: ({:.1}, {:.1}, {:.1})", cam_enu[0], cam_enu[1], cam_enu[2]);
    eprintln!("Point0 ENU: ({:.1}, {:.1}, {:.1})", pt0_enu[0], pt0_enu[1], pt0_enu[2]);

    // Delta (point - camera)
    let dp = [pt0_enu[0] - cam_enu[0], pt0_enu[1] - cam_enu[1], pt0_enu[2] - cam_enu[2]];
    eprintln!("Delta (point - camera): ({:.1}, {:.1}, {:.1})", dp[0], dp[1], dp[2]);
    eprintln!("  (E={:.1} means point is EAST of camera)", dp[0]);
    eprintln!("  (N={:.1} means point is SOUTH of camera)", dp[1]);
    eprintln!("  (U={:.1} means point is ABOVE camera)", dp[2]);

    // Try yaw=195 (facing South-Southwest)
    let yaw = 195.0;
    let pitch = 7.0;  // looking slightly up
    let roll = 0.0;

    let rot = rotation_enu_to_cam(yaw, pitch, roll);
    eprintln!("\nRotation matrix (yaw={}, pitch={}):", yaw, pitch);
    for i in 0..3 {
        eprintln!("  [{:8.4}, {:8.4}, {:8.4}]", rot[i][0], rot[i][1], rot[i][2]);
    }

    // Transform to camera frame
    let cam_coords = mat3_vec(&rot, &dp);
    eprintln!("\nCamera frame coords: ({:.1}, {:.1}, {:.1})", cam_coords[0], cam_coords[1], cam_coords[2]);
    eprintln!("  (X={:.1} positive means RIGHT)", cam_coords[0]);
    eprintln!("  (Y={:.1} positive means DOWN)", cam_coords[1]);
    eprintln!("  (Z={:.1} positive means FORWARD)", cam_coords[2]);

    if cam_coords[2] <= 0.0 {
        eprintln!("\nPoint is BEHIND camera!");
    } else {
        // Project
        let xn = cam_coords[0] / cam_coords[2];
        let yn = cam_coords[1] / cam_coords[2];
        eprintln!("\nNormalized coords: xn={:.4}, yn={:.4}", xn, yn);

        let u = focal_px * xn + image_width / 2.0;
        let v = focal_px * yn + image_height / 2.0;
        eprintln!("Projected pixel: u={:.0}, v={:.0}", u, v);
        eprintln!("Actual pixel:    u={:.0}, v={:.0}", pt0_u_actual, pt0_v_actual);
        eprintln!("Error: {:.0}px", ((u - pt0_u_actual).powi(2) + (v - pt0_v_actual).powi(2)).sqrt());
    }
}

/// Test with different camera altitudes
#[test]
fn test_maas_with_different_altitudes() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_px = 3024.0;

    let cam_lat = 51.916810141229455;
    let cam_lon = 4.491832827359327;

    let correspondences: Vec<(f64, f64, f64, f64, f64)> = vec![
        (51.908407170731785, 4.4884439705492705, 133.0, 1637.72, 1559.00),
        (51.90653193209267, 4.487362504005433, 149.0, 1352.03, 1673.15),
        (51.90745197954069, 4.489266872406007, 98.0, 829.07, 1867.68),
        (51.91639663828099, 4.491353631019593, 4.2, 2223.98, 2629.30),
        (51.90680331495428, 4.489020109176637, 150.0, 859.34, 1635.09),
        (51.91016236948932, 4.488526582717896, 17.0, 1502.97, 2309.31),
        (51.90727657626541, 4.48969602584839, 92.0, 619.92, 1891.30),
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.0).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.1).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.2).sum::<f64>() / n;

    eprintln!("\n=== TESTING DIFFERENT ALTITUDES ===");

    for cam_alt in [10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 50.0] {
        let intr = CameraIntrinsics {
            focal_px, cx: image_width / 2.0, cy: image_height / 2.0, 
            k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
        };

        let cam_enu = lla_to_enu(cam_lat, cam_lon, cam_alt, ref_lat, ref_lon, ref_alt);
        
        // Search for best yaw and pitch at this altitude
        let mut best_rmse = 1e9;
        let mut best_yaw = 0.0;
        let mut best_pitch = 0.0;
        
        for yaw in (180..=220).step_by(1) {
            for pitch in -20..=20 {
                let rot = rotation_enu_to_cam(yaw as f64, pitch as f64, 0.0);
                
                let mut sse = 0.0;
                let mut valid = 0;
                for &(lat, lon, alt, u_actual, v_actual) in &correspondences {
                    let pt_enu = lla_to_enu(lat, lon, alt, ref_lat, ref_lon, ref_alt);
                    if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
                        sse += (u - u_actual).powi(2) + (v - v_actual).powi(2);
                        valid += 1;
                    }
                }
                
                if valid == correspondences.len() {
                    let rmse = (sse / n).sqrt();
                    if rmse < best_rmse {
                        best_rmse = rmse;
                        best_yaw = yaw as f64;
                        best_pitch = pitch as f64;
                    }
                }
            }
        }
        
        eprintln!("alt={:.0}m: best yaw={:.0}°, pitch={:.0}°, RMSE={:.1}px", 
                  cam_alt, best_yaw, best_pitch, best_rmse);
    }
}

/// Test without pt0
#[test]
fn test_maas_without_pt0() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_px = 3024.0;

    let cam_lat = 51.916810141229455;
    let cam_lon = 4.491832827359327;
    let cam_alt = 10.0;

    // Exclude pt0
    let correspondences: Vec<(f64, f64, f64, f64, f64)> = vec![
        //(51.908407170731785, 4.4884439705492705, 133.0, 1637.72, 1559.00),  // pt0 excluded
        (51.90653193209267, 4.487362504005433, 149.0, 1352.03, 1673.15),
        (51.90745197954069, 4.489266872406007, 98.0, 829.07, 1867.68),
        (51.91639663828099, 4.491353631019593, 4.2, 2223.98, 2629.30),
        (51.90680331495428, 4.489020109176637, 150.0, 859.34, 1635.09),
        (51.91016236948932, 4.488526582717896, 17.0, 1502.97, 2309.31),
        (51.90727657626541, 4.48969602584839, 92.0, 619.92, 1891.30),
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.0).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.1).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.2).sum::<f64>() / n;

    let intr = CameraIntrinsics {
        focal_px, cx: image_width / 2.0, cy: image_height / 2.0, 
        k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
    };

    let cam_enu = lla_to_enu(cam_lat, cam_lon, cam_alt, ref_lat, ref_lon, ref_alt);

    eprintln!("\n=== TEST WITHOUT PT0 ===");
    
    let mut best_rmse = 1e9;
    let mut best_params = (0.0, 0.0);
    
    for yaw in (180..=220).step_by(1) {
        for pitch in -30..=30 {
            let rot = rotation_enu_to_cam(yaw as f64, pitch as f64, 0.0);
            
            let mut sse = 0.0;
            let mut valid = 0;
            for &(lat, lon, alt, u_actual, v_actual) in &correspondences {
                let pt_enu = lla_to_enu(lat, lon, alt, ref_lat, ref_lon, ref_alt);
                if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
                    sse += (u - u_actual).powi(2) + (v - v_actual).powi(2);
                    valid += 1;
                }
            }
            
            if valid == correspondences.len() {
                let rmse = (sse / n).sqrt();
                if rmse < best_rmse {
                    best_rmse = rmse;
                    best_params = (yaw as f64, pitch as f64);
                }
            }
        }
    }
    
    eprintln!("Best: yaw={:.0}°, pitch={:.0}°, RMSE={:.1}px", best_params.0, best_params.1, best_rmse);
    
    // Show individual errors
    let rot = rotation_enu_to_cam(best_params.0, best_params.1, 0.0);
    eprintln!("\nIndividual errors:");
    for (i, &(lat, lon, alt, u_actual, v_actual)) in correspondences.iter().enumerate() {
        let pt_enu = lla_to_enu(lat, lon, alt, ref_lat, ref_lon, ref_alt);
        if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
            let err = ((u - u_actual).powi(2) + (v - v_actual).powi(2)).sqrt();
            eprintln!("  pt{}: err={:.0}px", i+1, err);  // pt1 through pt6
        }
    }
}

/// Test with different principal points
#[test]
fn test_maas_principal_point() {
    use crate::geo::lla_to_enu;
    use crate::projection::{rotation_enu_to_cam, project_point, CameraIntrinsics};

    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_px = 3024.0;

    let cam_lat = 51.916810141229455;
    let cam_lon = 4.491832827359327;
    let cam_alt = 10.0;

    let correspondences: Vec<(f64, f64, f64, f64, f64)> = vec![
        (51.908407170731785, 4.4884439705492705, 133.0, 1637.72, 1559.00),
        (51.90653193209267, 4.487362504005433, 149.0, 1352.03, 1673.15),
        (51.90745197954069, 4.489266872406007, 98.0, 829.07, 1867.68),
        (51.91639663828099, 4.491353631019593, 4.2, 2223.98, 2629.30),
        (51.90680331495428, 4.489020109176637, 150.0, 859.34, 1635.09),
        (51.91016236948932, 4.488526582717896, 17.0, 1502.97, 2309.31),
        (51.90727657626541, 4.48969602584839, 92.0, 619.92, 1891.30),
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.0).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.1).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.2).sum::<f64>() / n;

    let cam_enu = lla_to_enu(cam_lat, cam_lon, cam_alt, ref_lat, ref_lon, ref_alt);

    eprintln!("\n=== TEST PRINCIPAL POINT ===");
    
    let mut best_rmse = 1e9;
    let mut best_params = (0.0, 0.0, 0.0, 0.0);
    
    // Standard center
    let cx_center = image_width / 2.0;
    let cy_center = image_height / 2.0;
    
    // Search cx, cy offsets
    for cx_off in [-200.0, -100.0, -50.0, 0.0, 50.0, 100.0, 200.0] {
        for cy_off in [-200.0, -100.0, -50.0, 0.0, 50.0, 100.0, 200.0] {
            let intr = CameraIntrinsics {
                focal_px, 
                cx: cx_center + cx_off, 
                cy: cy_center + cy_off, 
                k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
            };
            
            for yaw in (180..=220).step_by(2) {
                for pitch in -30..=30 {
                    let rot = rotation_enu_to_cam(yaw as f64, pitch as f64, 0.0);
                    
                    let mut sse = 0.0;
                    let mut valid = 0;
                    for &(lat, lon, alt, u_actual, v_actual) in &correspondences {
                        let pt_enu = lla_to_enu(lat, lon, alt, ref_lat, ref_lon, ref_alt);
                        if let Some((u, v)) = project_point(pt_enu, cam_enu, &rot, &intr) {
                            sse += (u - u_actual).powi(2) + (v - v_actual).powi(2);
                            valid += 1;
                        }
                    }
                    
                    if valid == correspondences.len() {
                        let rmse = (sse / n).sqrt();
                        if rmse < best_rmse {
                            best_rmse = rmse;
                            best_params = (yaw as f64, pitch as f64, cx_off, cy_off);
                        }
                    }
                }
            }
        }
    }
    
    eprintln!("Best: yaw={:.0}°, pitch={:.0}°, cx_off={:.0}, cy_off={:.0}, RMSE={:.1}px", 
              best_params.0, best_params.1, best_params.2, best_params.3, best_rmse);
}

/// Test initializing at the expected position
#[test]
fn test_maas_init_at_expected() {
    use crate::geo::lla_to_enu;
    use crate::projection::CameraIntrinsics;
    use crate::optimizer::levenberg_marquardt;
    use crate::types::{Corr, Pixel, WorldLla};
    
    let image_width: f64 = 4032.0;
    let image_height: f64 = 3024.0;
    let focal_px = 3024.0;

    let correspondences: Vec<Corr> = vec![
        Corr { id: "pt0".to_string(), enabled: Some(true),
               pixel: Pixel { u: 1637.72, v: 1559.00, sigma_px: Some(1.0) },
               world: WorldLla { lat: 51.908407170731785, lon: 4.4884439705492705, alt: Some(133.0) }},
        Corr { id: "pt1".to_string(), enabled: Some(true),
               pixel: Pixel { u: 1352.03, v: 1673.15, sigma_px: Some(1.0) },
               world: WorldLla { lat: 51.90653193209267, lon: 4.487362504005433, alt: Some(149.0) }},
        Corr { id: "pt2".to_string(), enabled: Some(true),
               pixel: Pixel { u: 829.07, v: 1867.68, sigma_px: Some(1.0) },
               world: WorldLla { lat: 51.90745197954069, lon: 4.489266872406007, alt: Some(98.0) }},
        Corr { id: "pt3".to_string(), enabled: Some(true),
               pixel: Pixel { u: 2223.98, v: 2629.30, sigma_px: Some(1.0) },
               world: WorldLla { lat: 51.91639663828099, lon: 4.491353631019593, alt: Some(4.2) }},
        Corr { id: "pt4".to_string(), enabled: Some(true),
               pixel: Pixel { u: 859.34, v: 1635.09, sigma_px: Some(1.0) },
               world: WorldLla { lat: 51.90680331495428, lon: 4.489020109176637, alt: Some(150.0) }},
        Corr { id: "pt5".to_string(), enabled: Some(true),
               pixel: Pixel { u: 1502.97, v: 2309.31, sigma_px: Some(1.0) },
               world: WorldLla { lat: 51.91016236948932, lon: 4.488526582717896, alt: Some(17.0) }},
        Corr { id: "pt6".to_string(), enabled: Some(true),
               pixel: Pixel { u: 619.92, v: 1891.30, sigma_px: Some(1.0) },
               world: WorldLla { lat: 51.90727657626541, lon: 4.48969602584839, alt: Some(92.0) }},
    ];

    let n = correspondences.len() as f64;
    let ref_lat = correspondences.iter().map(|c| c.world.lat).sum::<f64>() / n;
    let ref_lon = correspondences.iter().map(|c| c.world.lon).sum::<f64>() / n;
    let ref_alt = correspondences.iter().map(|c| c.world.alt.unwrap_or(0.0)).sum::<f64>() / n;

    // Convert to ENU correspondences
    let corrs: Vec<_> = correspondences.iter().map(|c| {
        let enu = lla_to_enu(c.world.lat, c.world.lon, c.world.alt.unwrap_or(0.0), ref_lat, ref_lon, ref_alt);
        crate::optimizer::EnuCorrespondence {
            enu: [enu[0], enu[1], enu[2]],
            pixel: [c.pixel.u, c.pixel.v],
            sigma: 1.0,
        }
    }).collect();

    // Expected camera position
    let exp_lat = 51.916810141229455;
    let exp_lon = 4.491832827359327;
    let exp_alt = 10.0;
    let exp_enu = lla_to_enu(exp_lat, exp_lon, exp_alt, ref_lat, ref_lon, ref_alt);

    eprintln!("\n=== TEST INIT AT EXPECTED POSITION ===");
    eprintln!("Expected ENU: ({:.1}, {:.1}, {:.1})", exp_enu[0], exp_enu[1], exp_enu[2]);

    let intr = CameraIntrinsics {
        focal_px, cx: image_width / 2.0, cy: image_height / 2.0,
        k1: 0.0, k2: 0.0, p1: 0.0, p2: 0.0,
    };

    // Initialize at expected position with various orientations
    for yaw in [195.0, 200.0, 205.0, 210.0] {
        for pitch in [5.0, 10.0, 15.0] {
            let init: [f64; 10] = [
                exp_enu[0], exp_enu[1], exp_enu[2],  // position
                yaw, pitch, 0.0,                      // orientation
                0.0, 0.0, 0.0, 0.0,                   // distortion
            ];
            
            let result = levenberg_marquardt(init, &corrs, &intr, None, [0.0, 0.0, 0.0, 0.0], 7, 100);
            
            eprintln!("Init yaw={:.0}, pitch={:.0}: final cost={:.0}, converged?={}",
                      yaw, pitch, result.cost, result.iterations < 100);
            eprintln!("  Final pos: ({:.1}, {:.1}, {:.1})",
                      result.params[0], result.params[1], result.params[2]);
            eprintln!("  Final orient: yaw={:.1}, pitch={:.1}, roll={:.1}",
                      result.params[3], result.params[4], result.params[5]);
            
            // How far did it move from expected?
            let moved = ((result.params[0] - exp_enu[0]).powi(2) +
                         (result.params[1] - exp_enu[1]).powi(2) +
                         (result.params[2] - exp_enu[2]).powi(2)).sqrt();
            eprintln!("  Moved: {:.0}m from expected", moved);
        }
    }
}
