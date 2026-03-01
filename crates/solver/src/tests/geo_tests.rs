use crate::geo::{bearing_degrees, ecef_to_lla, enu_to_lla, lla_to_ecef, lla_to_enu};

// ── Round-trip tests ────────────────────────────────────────────────────────

#[test]
fn lla_ecef_round_trip_rotterdam() {
    let (lat, lon, alt) = (51.9225, 4.4792, 0.0);
    let ecef = lla_to_ecef(lat, lon, alt);
    let (lat2, lon2, alt2) = ecef_to_lla(ecef);
    assert!((lat2 - lat).abs() < 1e-10, "lat: {} vs {}", lat2, lat);
    assert!((lon2 - lon).abs() < 1e-10, "lon: {} vs {}", lon2, lon);
    assert!((alt2 - alt).abs() < 1e-3, "alt: {} vs {}", alt2, alt);
}

#[test]
fn lla_ecef_round_trip_with_altitude() {
    let (lat, lon, alt) = (51.9225, 4.4792, 150.0);
    let ecef = lla_to_ecef(lat, lon, alt);
    let (lat2, lon2, alt2) = ecef_to_lla(ecef);
    assert!((lat2 - lat).abs() < 1e-10);
    assert!((lon2 - lon).abs() < 1e-10);
    assert!((alt2 - alt).abs() < 1e-3);
}

#[test]
fn lla_ecef_round_trip_equator() {
    let (lat, lon, alt) = (0.0, 0.0, 0.0);
    let ecef = lla_to_ecef(lat, lon, alt);
    let (lat2, lon2, alt2) = ecef_to_lla(ecef);
    assert!((lat2 - lat).abs() < 1e-10);
    assert!((lon2 - lon).abs() < 1e-10);
    assert!((alt2 - alt).abs() < 1e-3);
}

#[test]
fn lla_ecef_round_trip_high_latitude() {
    // Near the North Pole
    let (lat, lon, alt) = (89.999, 45.0, 100.0);
    let ecef = lla_to_ecef(lat, lon, alt);
    let (lat2, lon2, alt2) = ecef_to_lla(ecef);
    assert!((lat2 - lat).abs() < 1e-8);
    assert!((lon2 - lon).abs() < 1e-6); // lon less accurate near poles
    assert!((alt2 - alt).abs() < 1e-2);
}

#[test]
fn lla_ecef_round_trip_southern_hemisphere() {
    let (lat, lon, alt) = (-33.8688, 151.2093, 50.0); // Sydney
    let ecef = lla_to_ecef(lat, lon, alt);
    let (lat2, lon2, alt2) = ecef_to_lla(ecef);
    assert!((lat2 - lat).abs() < 1e-10);
    assert!((lon2 - lon).abs() < 1e-10);
    assert!((alt2 - alt).abs() < 1e-3);
}

// ── ENU conversion tests ───────────────────────────────────────────────────

#[test]
fn enu_round_trip_same_point() {
    let (lat, lon, alt) = (51.9225, 4.4792, 10.0);
    let enu = lla_to_enu(lat, lon, alt, lat, lon, alt);
    assert!(enu[0].abs() < 1e-6, "e = {}", enu[0]);
    assert!(enu[1].abs() < 1e-6, "n = {}", enu[1]);
    assert!(enu[2].abs() < 1e-6, "u = {}", enu[2]);
}

#[test]
fn enu_round_trip_offset_point() {
    let ref_lat = 51.9225;
    let ref_lon = 4.4792;
    let ref_alt = 0.0;
    let pt_lat = 51.923;
    let pt_lon = 4.480;
    let pt_alt = 50.0;

    let enu = lla_to_enu(pt_lat, pt_lon, pt_alt, ref_lat, ref_lon, ref_alt);
    let (lat2, lon2, alt2) = enu_to_lla(enu, ref_lat, ref_lon, ref_alt);

    assert!(
        (lat2 - pt_lat).abs() < 1e-8,
        "lat: {} vs {}",
        lat2,
        pt_lat
    );
    assert!(
        (lon2 - pt_lon).abs() < 1e-8,
        "lon: {} vs {}",
        lon2,
        pt_lon
    );
    assert!(
        (alt2 - pt_alt).abs() < 0.01,
        "alt: {} vs {}",
        alt2,
        pt_alt
    );
}

#[test]
fn enu_point_to_east_has_positive_e() {
    let ref_lat = 51.9225;
    let ref_lon = 4.4792;
    let ref_alt = 0.0;
    // A point 0.001° east of reference
    let enu = lla_to_enu(ref_lat, ref_lon + 0.001, ref_alt, ref_lat, ref_lon, ref_alt);
    assert!(enu[0] > 0.0, "east should be positive: {}", enu[0]);
    assert!(enu[1].abs() < 1.0, "north should be ~0: {}", enu[1]);
    assert!(enu[2].abs() < 1.0, "up should be ~0: {}", enu[2]);
}

#[test]
fn enu_point_to_north_has_positive_n() {
    let ref_lat = 51.9225;
    let ref_lon = 4.4792;
    let ref_alt = 0.0;
    let enu = lla_to_enu(ref_lat + 0.001, ref_lon, ref_alt, ref_lat, ref_lon, ref_alt);
    assert!(enu[0].abs() < 1.0, "east should be ~0: {}", enu[0]);
    assert!(enu[1] > 0.0, "north should be positive: {}", enu[1]);
    assert!(enu[2].abs() < 1.0, "up should be ~0: {}", enu[2]);
}

#[test]
fn enu_point_above_has_positive_u() {
    let ref_lat = 51.9225;
    let ref_lon = 4.4792;
    let ref_alt = 0.0;
    let enu = lla_to_enu(ref_lat, ref_lon, ref_alt + 100.0, ref_lat, ref_lon, ref_alt);
    assert!(enu[0].abs() < 1.0, "east should be ~0: {}", enu[0]);
    assert!(enu[1].abs() < 1.0, "north should be ~0: {}", enu[1]);
    assert!(
        (enu[2] - 100.0).abs() < 0.1,
        "up should be ~100: {}",
        enu[2]
    );
}

#[test]
fn enu_known_distance_1km_east() {
    let ref_lat = 52.0;
    let ref_lon = 5.0;
    let ref_alt = 0.0;
    // 1 km East ≈ 0.01449° longitude at lat 52°
    let delta_lon = 1000.0 / (111_320.0 * 52.0_f64.to_radians().cos());
    let enu = lla_to_enu(ref_lat, ref_lon + delta_lon, ref_alt, ref_lat, ref_lon, ref_alt);
    assert!(
        (enu[0] - 1000.0).abs() < 5.0,
        "1 km east should give e≈1000: {}",
        enu[0]
    );
}

#[test]
fn enu_known_distance_1km_north() {
    let ref_lat = 52.0;
    let ref_lon = 5.0;
    let ref_alt = 0.0;
    // 1 km North ≈ 0.00899° latitude
    let delta_lat = 1000.0 / 111_320.0;
    let enu = lla_to_enu(ref_lat + delta_lat, ref_lon, ref_alt, ref_lat, ref_lon, ref_alt);
    assert!(
        (enu[1] - 1000.0).abs() < 2.0,
        "1 km north should give n≈1000: {}",
        enu[1]
    );
}

#[test]
fn enu_inverse_is_exact_inverse() {
    let ref_lat = 48.8566;
    let ref_lon = 2.3522;
    let ref_alt = 35.0;
    // arbitrary ENU offset
    let e = 500.0;
    let n = -300.0;
    let u = 80.0;
    let (lat, lon, alt) = enu_to_lla([e, n, u], ref_lat, ref_lon, ref_alt);
    let enu2 = lla_to_enu(lat, lon, alt, ref_lat, ref_lon, ref_alt);
    assert!((enu2[0] - e).abs() < 0.01, "e: {} vs {}", enu2[0], e);
    assert!((enu2[1] - n).abs() < 0.01, "n: {} vs {}", enu2[1], n);
    assert!((enu2[2] - u).abs() < 0.01, "u: {} vs {}", enu2[2], u);
}

// ── Bearing tests ───────────────────────────────────────────────────────────

#[test]
fn bearing_due_north() {
    let b = bearing_degrees(51.0, 4.0, 52.0, 4.0);
    assert!((b - 0.0).abs() < 0.1, "expected ~0, got {}", b);
}

#[test]
fn bearing_due_east() {
    let b = bearing_degrees(51.0, 4.0, 51.0, 5.0);
    assert!((b - 90.0).abs() < 1.0, "expected ~90, got {}", b);
}

#[test]
fn bearing_due_south() {
    let b = bearing_degrees(52.0, 4.0, 51.0, 4.0);
    assert!((b - 180.0).abs() < 0.1, "expected ~180, got {}", b);
}

#[test]
fn bearing_due_west() {
    let b = bearing_degrees(51.0, 5.0, 51.0, 4.0);
    assert!((b - 270.0).abs() < 1.0, "expected ~270, got {}", b);
}

#[test]
fn bearing_always_in_0_360() {
    let b = bearing_degrees(0.0, 0.0, -1.0, -1.0);
    assert!(b >= 0.0 && b < 360.0, "bearing {} out of range", b);
}
