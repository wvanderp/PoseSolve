use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolveRequest {
    pub image: Image,
    #[serde(default)]
    pub correspondences: Vec<Corr>,
    #[serde(default)]
    pub priors: Option<Priors>,
}

#[derive(Deserialize)]
pub struct Image {
    pub width: f64,
    pub height: f64,
}

#[derive(Deserialize)]
pub struct Corr {
    pub id: String,
    pub pixel: Pixel,
    pub world: WorldLla,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pixel {
    pub u: f64,
    pub v: f64,
    #[serde(default)]
    pub sigma_px: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldLla {
    pub lat: f64,
    pub lon: f64,
    #[serde(default)]
    pub alt: Option<f64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Priors {
    #[serde(default)]
    pub focal_px: Option<GaussianPrior>,
    #[serde(default)]
    pub camera_alt: Option<GaussianPrior>,
    #[serde(default)]
    pub bounds: Option<Bounds>,
    /// Optional initial guess for lens-distortion coefficients (e.g. from EXIF /
    /// calibration data).  Helps the solver converge for wide-angle / smartphone
    /// lenses.  Any omitted field defaults to 0.
    #[serde(default)]
    pub distortion: Option<DistortionPrior>,
}

/// Initial-guess (and weak Gaussian prior) for Brown-Conrady distortion
/// coefficients.  These match the values stored in EXIF / DNG / OpenCV
/// camera-calibration files.
///
/// * `k1`, `k2` – radial distortion.  Negative values = barrel (typical for
///   wide-angle smartphone lenses).
/// * `p1`, `p2` – tangential (decentring) distortion.  Usually ≈ 0 for
///   modern lenses.
#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DistortionPrior {
    #[serde(default)]
    pub k1: Option<f64>,
    #[serde(default)]
    pub k2: Option<f64>,
    #[serde(default)]
    pub p1: Option<f64>,
    #[serde(default)]
    pub p2: Option<f64>,
}

#[derive(Deserialize)]
pub struct GaussianPrior {
    pub mean: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bounds {
    pub lat_min: f64,
    pub lat_max: f64,
    pub lon_min: f64,
    pub lon_max: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pose {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    pub yaw_deg: f64,
    pub pitch_deg: f64,
    pub roll_deg: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Intrinsics {
    pub focal_px: f64,
    pub cx: f64,
    pub cy: f64,
    /// Radial distortion coefficient (first order).
    pub k1: f64,
    /// Radial distortion coefficient (second order).
    pub k2: f64,
    /// Tangential distortion coefficient 1.
    pub p1: f64,
    /// Tangential distortion coefficient 2.
    pub p2: f64,
}

#[derive(Serialize)]
pub struct Covariance {
    pub matrix: Vec<f64>,
    pub labels: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub rmse_px: f64,
    pub inlier_ratio: f64,
    pub residuals_px: Vec<f64>,
    pub inlier_ids: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct SolveResponse {
    pub pose: Pose,
    pub intrinsics: Intrinsics,
    pub covariance: Covariance,
    pub diagnostics: Diagnostics,
}
