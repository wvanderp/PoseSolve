mod estimator;
mod geo;
mod optimizer;
mod projection;
pub mod types;

use types::SolveRequest;
use wasm_bindgen::prelude::*;

pub use estimator::solve_impl;

#[wasm_bindgen]
pub fn solve(req_json: String) -> Result<String, JsValue> {
    let req: SolveRequest = serde_json::from_str(&req_json)
        .map_err(|e| to_js_error(format!("Invalid request JSON: {e}")))?;
    let resp = solve_impl(&req).map_err(to_js_error)?;
    serde_json::to_string(&resp).map_err(|e| to_js_error(format!("Serialize error: {e}")))
}

#[cfg(target_arch = "wasm32")]
fn to_js_error(message: String) -> JsValue {
    JsValue::from_str(&message)
}

#[cfg(not(target_arch = "wasm32"))]
fn to_js_error(_message: String) -> JsValue {
    JsValue::NULL
}

#[wasm_bindgen]
pub fn reproject_points(_req_json: String) -> Result<String, JsValue> {
    Ok("{\"pixels\": [], \"warnings\": [\"Not implemented yet\"]}".to_string())
}

#[cfg(test)]
mod tests;
