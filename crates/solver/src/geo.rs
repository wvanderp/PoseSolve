// ── WGS-84 constants ────────────────────────────────────────────────────────
const WGS84_A: f64 = 6_378_137.0; // semi-major axis (m)
const WGS84_F: f64 = 1.0 / 298.257_223_563; // flattening
const WGS84_E2: f64 = 2.0 * WGS84_F - WGS84_F * WGS84_F; // first eccentricity squared

/// Radius of curvature in the prime vertical.
fn prime_vertical_radius(sin_lat: f64) -> f64 {
    WGS84_A / (1.0 - WGS84_E2 * sin_lat * sin_lat).sqrt()
}

// ── Coordinate conversions ──────────────────────────────────────────────────

/// Convert geodetic LLA (lat/lon degrees, alt metres above ellipsoid) → ECEF.
pub(crate) fn lla_to_ecef(lat_deg: f64, lon_deg: f64, alt: f64) -> [f64; 3] {
    let lat = lat_deg.to_radians();
    let lon = lon_deg.to_radians();
    let (slat, clat) = (lat.sin(), lat.cos());
    let n = prime_vertical_radius(slat);
    [
        (n + alt) * clat * lon.cos(),
        (n + alt) * clat * lon.sin(),
        (n * (1.0 - WGS84_E2) + alt) * slat,
    ]
}

/// Convert ECEF → geodetic LLA (Bowring iterative, 10 iterations).
pub(crate) fn ecef_to_lla(ecef: [f64; 3]) -> (f64, f64, f64) {
    let [x, y, z] = ecef;
    let lon = y.atan2(x);
    let p = (x * x + y * y).sqrt();
    let mut lat = (z / p).atan();
    for _ in 0..10 {
        let slat = lat.sin();
        let n = prime_vertical_radius(slat);
        lat = (z + WGS84_E2 * n * slat).atan2(p);
    }
    let (slat, clat) = (lat.sin(), lat.cos());
    let n = prime_vertical_radius(slat);
    let alt = if clat.abs() > 1e-10 {
        p / clat - n
    } else {
        z / slat - n * (1.0 - WGS84_E2)
    };
    (lat.to_degrees(), lon.to_degrees(), alt)
}

/// Convert geodetic LLA → local East-North-Up relative to a reference point.
pub(crate) fn lla_to_enu(
    lat_deg: f64,
    lon_deg: f64,
    alt: f64,
    ref_lat: f64,
    ref_lon: f64,
    ref_alt: f64,
) -> [f64; 3] {
    let p = lla_to_ecef(lat_deg, lon_deg, alt);
    let r = lla_to_ecef(ref_lat, ref_lon, ref_alt);
    let (dx, dy, dz) = (p[0] - r[0], p[1] - r[1], p[2] - r[2]);
    let rl = ref_lat.to_radians();
    let rn = ref_lon.to_radians();
    let (slat, clat) = (rl.sin(), rl.cos());
    let (slon, clon) = (rn.sin(), rn.cos());
    [
        -slon * dx + clon * dy,
        -slat * clon * dx - slat * slon * dy + clat * dz,
        clat * clon * dx + clat * slon * dy + slat * dz,
    ]
}

/// Convert local ENU → geodetic LLA.
pub(crate) fn enu_to_lla(
    enu: [f64; 3],
    ref_lat: f64,
    ref_lon: f64,
    ref_alt: f64,
) -> (f64, f64, f64) {
    let [e, n, u] = enu;
    let rl = ref_lat.to_radians();
    let rn = ref_lon.to_radians();
    let (slat, clat) = (rl.sin(), rl.cos());
    let (slon, clon) = (rn.sin(), rn.cos());
    let dx = -slon * e - slat * clon * n + clat * clon * u;
    let dy = clon * e - slat * slon * n + clat * slon * u;
    let dz = clat * n + slat * u;
    let r = lla_to_ecef(ref_lat, ref_lon, ref_alt);
    ecef_to_lla([r[0] + dx, r[1] + dy, r[2] + dz])
}

// ── Bearing ─────────────────────────────────────────────────────────────────

/// Bearing in degrees (0 = N, 90 = E) from point 1 to point 2.
pub(crate) fn bearing_degrees(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let y = dlon.sin() * phi2.cos();
    let x = phi1.cos() * phi2.sin() - phi1.sin() * phi2.cos() * dlon.cos();
    let mut brng = y.atan2(x).to_degrees();
    if brng < 0.0 {
        brng += 360.0;
    }
    brng
}
