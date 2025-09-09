use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SolveRequest {
    image: Image,
    // Other fields omitted in stub
}

#[derive(Deserialize)]
struct Image { width: f64, height: f64 }

#[derive(Serialize)]
struct Pose { lat: f64, lon: f64, alt: f64, yawDeg: f64, pitchDeg: f64, rollDeg: f64 }

#[derive(Serialize)]
struct Intrinsics { focalPx: f64, cx: f64, cy: f64 }

#[derive(Serialize)]
struct Covariance { matrix: Vec<f64>, labels: Vec<String> }

#[derive(Serialize)]
struct Diagnostics { rmsePx: f64, inlierRatio: f64, residualsPx: Vec<f64>, inlierIds: Vec<String>, warnings: Vec<String> }

#[derive(Serialize)]
struct SolveResponse { pose: Pose, intrinsics: Intrinsics, covariance: Covariance, diagnostics: Diagnostics }

#[wasm_bindgen]
pub fn solve(req_json: String) -> Result<String, JsValue> {
    let req: SolveRequest = serde_json::from_str(&req_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid request JSON: {e}")))?;

    let resp = SolveResponse {
        pose: Pose { lat: 0.0, lon: 0.0, alt: 0.0, yawDeg: 0.0, pitchDeg: 0.0, rollDeg: 0.0 },
        intrinsics: Intrinsics { focalPx: 1000.0, cx: req.image.width/2.0, cy: req.image.height/2.0 },
        covariance: Covariance { matrix: vec![], labels: vec![] },
        diagnostics: Diagnostics {
            rmsePx: 0.0,
            inlierRatio: 0.0,
            residualsPx: vec![],
            inlierIds: vec![],
            warnings: vec!["Stub solver response from WASM".to_string()],
        },
    };

    serde_json::to_string(&resp).map_err(|e| JsValue::from_str(&format!("Serialize error: {e}")))
}

#[wasm_bindgen]
pub fn reproject_points(_req_json: String) -> Result<String, JsValue> {
    Ok("{\"pixels\": [], \"warnings\": [\"Not implemented\"]}".to_string())
}
