# Product & Technical Specification — **CamPose** (working title)

## 1) Problem Statement

Given a single photo and a set of user-identified world landmarks (lat/lon\[/alt]) tagged to pixel coordinates, estimate the camera **pose** (position + orientation) and relevant **intrinsics** (focal length; optionally principal point & distortion). Output should be **as precise as possible** but **probabilistic** (with uncertainty).

## 2) Target Users & Goals

* **Advanced users** (OSINT, survey/photo sleuths, researchers, hobby photogrammetrists).
* **Primary goals**

  * Rapidly add/control correspondences (pixel ↔ world).
  * Get a **best-estimate camera pose** with **uncertainty**.
  * Visualize pose on a map and reproject landmarks/field-of-view to sanity-check.

## 3) Non-Goals / Out of Scope

* Automatic landmark detection or geocoding (user provides locations).
* Multi-image bundle adjustment (future enhancement; spec leaves hooks).
* Server compute (must run entirely in-browser, offline-capable after load).

---

## 4) System Overview

### 4.1 Architecture

* **Frontend**: React (TypeScript) + Tailwind
* **Map**: Leaflet (OSM tiles)
* **Computation core**: **Rust → WebAssembly (WASM)**; isolates numerics from UI
* **Worker model**: WASM runs inside a **Web Worker** (via Comlink) to keep UI smooth
* **State**: Zustand (global app state) + React Query (lazy-loading & caching)
* **Build**: Vite

### 4.2 Data Flow

1. User loads image (drag/drop or file picker). EXIF parsed in UI.
2. User adds **correspondences**: (pixel u,v) ↔ (world lat,lon\[,alt]) with optional **uncertainty**.
3. UI packages inputs → Web Worker → WASM solver.
4. Solver returns:

   * **Pose**: lat/lon/alt, yaw/pitch/roll (or quaternion).
   * **Intrinsics**: focal length (in px), optional principal point & distortion.
   * **Uncertainty**: covariance & bootstrap stats; confidence ellipsoid for position; angular std devs.
   * **Diagnostics**: inlier set (RANSAC), residuals, reprojection RMSE.
5. UI renders:

   * Camera frustum on map.
   * Reprojected landmarks overlaid on the image.
   * Numeric results + error bars.

---

## 5) Functional Requirements

### 5.1 Projects & Files

* **Create/Open/Save Project**: JSON file (`.campose.json`)

  * Includes image (as file ref or embedded base64), EXIF summary, correspondences, solver settings, results.
* **Import/Export**

  * **CSV** for correspondences (world + pixel + uncertainties).
  * **GeoJSON** export of camera pose (Point at camera location, properties with orientation and intrinsics).
  * **PNG/JPEG** export of annotated image (optional).

### 5.2 Image Panel (Left)

* Zoom/pan canvas.
* Click-to-add pixel point; drag to adjust; snap to subpixel (optional).
* Per-point UI:

  * ID/name
  * Pixel (u,v)
  * Linked world point (lat, lon, \[alt])
  * Pixel uncertainty (σ\_px, default 1.0 px)
  * Visibility toggle / lock
* Overlays:

  * Reprojected world landmarks (after solve) with residual vectors.
  * Vanishing line helper (optional; see 7.5).

### 5.3 Map Panel (Right)

* Leaflet map with OSM tiles.
* Add world points by clicking map or entering lat/lon manually.
* Altitude input:

  * Manual meters above ellipsoid (default 0)
  * “Lock altitude” checkbox for points with known heights.
* Move markers; show per-point uncertainty radius (meters).
* Show **camera icon** with **FOV frustum** and **bearing** after solve.
* Draw **reprojection rays** and **coverage wedge**.

### 5.4 Correspondence Management

* Two ways to link:

  * Add pixel point → “Link to world point” picker (and vice versa).
  * Batch link in a table (ID ↔ ID).
* Validation:

  * Warn on duplicate links, too few correspondences, near-collinearity, etc.
* Required minimums:

  * If intrinsics known: **≥3** points (P3P).
  * If focal length unknown: **≥4** (EPnP); recommend **6–12** for stability.

### 5.5 Solver Controls

* **Model preset**

  * *Basic*: Unknown pose + focal length (principal point ≈ image center; no distortion).
  * *Calibrated-ish*: Use EXIF focal if available (as prior with uncertainty).
  * *Advanced*: Estimate k1,k2 (radial) + optionally p1,p2 (tangential).
* **RANSAC** options:

  * Max iterations
  * Inlier threshold (pixels)
  * Probability target (e.g., 0.999)
* **Refinement**:

  * Nonlinear least squares (Gauss-Newton / Levenberg-Marquardt).
  * Robust loss (Huber/Cauchy) toggle.
* **Uncertainty**

  * **Covariance from Hessian inverse** (fast).
  * **Bootstrap** N samples (resample correspondences) for empirical posteriors.
* **Constraints & Priors**

  * Altitude prior for camera (e.g., ground level ± σ).
  * Pose prior region (optional bounding box / area).
  * Focal prior (from EXIF) with σ.

### 5.6 Results & Diagnostics

* Numeric:

  * Camera **lat/lon/alt** (+σ)
  * Orientation: yaw/pitch/roll (deg) (+σ)
  * Focal length (px; and equivalent mm at given sensor, if EXIF has sensor width)
  * Principal point offset (px) if estimated
  * Distortion coefficients if estimated
  * Reprojection RMSE (px), inliers %, median residual
* Graphics:

  * Residual plot (image overlay)
  * Inlier/outlier coloring
  * Map: confidence **error ellipsoid** (position) & angular error cones
* Logs:

  * RANSAC stats, iteration counts, convergence report
  * Warnings (degenerate geometry, weak conditioning)

---

## 6) Technical Design — Solver

### 6.1 Coordinate Systems

* **World**: WGS84 (lat, lon, alt) → converted internally to **ECEF** then **local ENU** frame anchored at the **median of world points** (reduces conditioning issues).
* **Camera**: Right-handed pinhole model; rotation **R** (world→camera) + translation **t**.
* **Image**: Pixel coordinates (u,v) with origin at top-left; normalized camera coords for math.

### 6.2 Camera Model

* **Pinhole** with intrinsics **K**:

  * focal length **f** (pixels)
  * principal point (cx, cy) (default image center; optionally estimated)
* **Distortion** (optional/advanced):

  * Radial: k1, k2 (optionally k3)
  * Tangential: p1, p2
    (Brown–Conrady; distortion applied in normalized coords)

### 6.3 Algorithm Pipeline

1. **Preprocess**

   * Build local ENU for world points.
   * Normalize pixel coordinates (optional isotropic scaling).
   * If EXIF focal present: set as prior (mean ± σ).
2. **Minimal Pose Hypotheses (RANSAC)**

   * If focal length **known/prior**: **P3P** hypotheses from 3 correspondences (plus disambiguation with 4th).
   * If focal **unknown**: **EPnP** (efficient PnP) or **UPnP** variant supporting unknown focal.
   * Score with **reprojection error**; mark inliers under threshold.
3. **Nonlinear Refinement**

   * Optimize over parameters θ = {R, t, f\[, cx, cy, k1, k2, p1, p2]}
     using **Levenberg-Marquardt**, robust loss (Huber).
   * Constraints/Priors as soft penalties:

     * (f − f₀)²/σ\_f², altitude prior, etc.
4. **Uncertainty Estimation**

   * Compute **approximate covariance** Σ ≈ (JᵀWJ)⁻¹ from final Jacobian J (with weights W).
   * **Bootstrap** B times (B=200 default; user adjustable): resample inliers with replacement, re-fit quickly; compute empirical std devs & quantiles for key outputs.
5. **Diagnostics & Outputs**

   * Inlier mask, residuals, RMSE.
   * Pose + intrinsics + Σ.
   * Bootstrap distributions (for UI histograms/intervals).

### 6.4 Numerical Stack (Rust)

* **Linear algebra**: `nalgebra`
* **Optimization**: `argmin` or `levenberg_marquardt` crate
* **Random**: `rand`
* **WASM bindgen**: `wasm-bindgen`, `wasm-bindgen-futures`
* **Optional**: Implement **EPnP/P3P** directly (lightweight) instead of pulling full OpenCV into WASM (keeps bundle small & portable).

> **Why not OpenCV in WASM?** It’s possible but heavy. Implementing P3P/EPnP + LM ourselves keeps load times small, avoids Emscripten complexity, and is more controllable.

### 6.5 WASM API (Rust ↔ TS)

All functions are **pure** (no global state), deterministic given seeds.

```ts
// Types (TS)
type Pixel = { u: number; v: number; sigmaPx?: number };
type WorldLLA = { lat: number; lon: number; alt?: number; sigmaM?: number };
type Corr = { id: string; pixel: Pixel; world: WorldLLA; enabled?: boolean };

type SolverModel = {
  estimateFocal: boolean;
  estimatePrincipalPoint: boolean;
  estimateDistortion: boolean; // k1,k2,(p1,p2)
};

type Priors = {
  focalPx?: { mean: number; sigma: number };
  cameraAlt?: { mean: number; sigma: number };
  bounds?: { latMin: number; latMax: number; lonMin: number; lonMax: number };
};

type RansacCfg = { maxIters: number; inlierPx: number; targetProb: number };
type RefineCfg = { maxIters: number; robustLoss: "none"|"huber"; huberDelta?: number };
type UncertaintyCfg = { bootstrap: { enabled: boolean; samples: number; seed?: number } };

type SolveRequest = {
  image: { width: number; height: number };
  correspondences: Corr[];
  model: SolverModel;
  priors?: Priors;
  ransac: RansacCfg;
  refine: RefineCfg;
  uncertainty: UncertaintyCfg;
};

type Pose = {
  lat: number; lon: number; alt: number;
  yawDeg: number; pitchDeg: number; rollDeg: number; // ENU yaw=azimuth
};

type Intrinsics = {
  focalPx: number;
  cx: number; cy: number;
  k1?: number; k2?: number; p1?: number; p2?: number;
};

type Covariance = { // flattened row-major cov for [position(3), angles(3), focal(1)]
  matrix: number[]; labels: string[];
};

type Diagnostics = {
  rmsePx: number;
  inlierRatio: number;
  residualsPx: number[]; // per inlier
  inlierIds: string[];
  warnings: string[];
};

type Bootstrap = {
  positionSamples: number[][];   // [B][3]
  orientationSamples: number[][];// [B][3]
  focalSamples?: number[];       // [B]
};

type SolveResponse = {
  pose: Pose;
  intrinsics: Intrinsics;
  covariance: Covariance;
  bootstrap?: Bootstrap;
  diagnostics: Diagnostics;
};
```

**WASM functions**

```rust
#[wasm_bindgen]
pub fn solve(req_json: String) -> Result<String, JsValue>; // JSON in/out

#[wasm_bindgen]
pub fn reproject_points(req_json: String) -> Result<String, JsValue>; 
// Given pose+intrinsics+world points → pixel projections (for UI overlays)
```

---

## 7) Technical Design — Frontend

### 7.1 UI Layout

* **Top bar**: Load/Save project, Undo/Redo, Settings, Help.
* **Left pane (Image)**: Canvas with tool palette (Add point, Move, Delete, Vanishing line).
* **Right pane (Map)**: Leaflet map, layer controls, toggle confidence ellipsoid, FOV wedge.
* **Bottom drawer**: Tabs:

  * **Points** (tabular editor with filter/sort)
  * **Solver** (model, RANSAC, priors, refine, uncertainty)
  * **Results** (numbers, charts)
  * **Logs**

### 7.2 Interactions

* **Add pixel point**: Click image → point card pops; “Link to world point…”
* **Add world point**: Click map → enter name/alt/uncertainty → “Link to pixel…”
* **Quick link**: Select a pixel point, UI highlights nearest unlinked world point candidates by name.
* **Solve**: Button triggers worker job; progress indicator (RANSAC iters).
* **Iterate**: Tweak inlier threshold, include/exclude points, hit “Refine only”.
* **Overlay toggles**: Residual vectors, inlier coloring, reprojected markers.

### 7.3 Error Handling & Guidance

* “Too few non-collinear points” → explain minimums & why.
* “Weak geometry” → suggest adding points with diverse bearings/depths.
* “Altitude ambiguity” → recommend adding at least one point with known altitude or add a camera altitude prior.

### 7.4 Performance

* WASM < 1.5 MB gzipped target.
* Solve under 200 ms for 12–20 points on mid-tier laptop; bootstrap can run async and stream partial results.

### 7.5 (Optional) Geometry Helpers

* **Horizon/Vanishing** line: User draws horizon; converts to pitch/roll constraint (soft prior).
* **Known orientation hint**: If the photo has EXIF yaw/pitch/roll, treat as weak prior.

---

## 8) Data Persistence

### 8.1 Project JSON schema (sketch)

```json
{
  "version": 1,
  "image": { "name": "foo.jpg", "width": 4032, "height": 3024, "exif": { "focal_mm": 6.8, "sensor_mm": 6.4 } },
  "points": {
    "pixel": [{ "id": "p1", "u": 1234.5, "v": 987.6, "sigmaPx": 1.0 }],
    "world": [{ "id": "w1", "lat": 52.37, "lon": 4.90, "alt": 3.0, "sigmaM": 5.0 }],
    "links": [{ "pixelId": "p1", "worldId": "w1" }]
  },
  "solver": { "model": {...}, "ransac": {...}, "refine": {...}, "priors": {...}, "uncertainty": {...} },
  "result": { "pose": {...}, "intrinsics": {...}, "covariance": {...}, "bootstrap": {...}, "diagnostics": {...} }
}
```

---

## 9) Acceptance Criteria

### 9.1 Functional

* Can add ≥10 correspondences in < 2 minutes with minimal friction.
* Produces a pose and focal estimate with residual RMSE reported.
* Reprojection overlay aligns visually with landmarks (by eye) when residuals < \~2 px median.
* Exports GeoJSON with camera pose & orientation fields.

### 9.2 Numerical/Quality

* **Synthetic test** (unit tests):

  * Generate a random pose + focal, sample 12 points, add 1 px noise → solver recovers:

    * Position within **±3 m** (at 1 km range scenario) and yaw within **±0.5°** (median over 100 runs).
* **Robustness test**:

  * 20% outliers → RANSAC identifies ≥80% of true inliers with suitable threshold.
* **Uncertainty**:

  * Bootstrap std devs scale sensibly with injected noise; covariance and bootstrap agree within 20%.

---

## 10) Security & Privacy

* Runs fully client-side; no image/upload leaves the browser.
* Optional PWA mode with offline cache of app bundles.
* No third-party analytics by default.

---

## 11) Accessibility & i18n

* Keyboard shortcuts for add/move/delete points.
* High-contrast mode toggle.
* UI copy ready for i18n (English default).

---

## 12) Implementation Plan & Milestones

### M1 — Foundation (1–2 weeks)

* Project scaffold (Vite, React, Tailwind, Zustand).
* Image canvas with zoom/pan; map panel (Leaflet).
* Point creation/editing and linking table.

### M2 — Core Solver (2–3 weeks)

* Rust WASM skeleton (`wasm-bindgen`), worker wiring (Comlink).
* Implement **EPnP** + **P3P**, RANSAC, LM refinement, basic covariance.
* Reprojection function for UI overlay.

### M3 — Uncertainty & UX Polish (1–2 weeks)

* Bootstrap uncertainty (as async job with progress).
* Residual overlays, inlier visualization, frustum on map.
* Save/Load project; CSV/GeoJSON export.

### M4 — Advanced Options (1–2 weeks)

* Distortion estimation (k1,k2; opt p1,p2).
* Priors & soft constraints (focal, altitude, horizon).
* Performance tuning & bundle size budget.

---

## 13) Testing Strategy

### 13.1 Unit Tests (Rust)

* EPnP/P3P correctness versus analytic projections (zero-noise check).
* Jacobian finite-difference verification.
* RANSAC: outlier handling & reproducible seeds.

### 13.2 Integration (TS+WASM)

* End-to-end synthetic scene runner: generate world points → render to pixels → add noise → verify recovery.

### 13.3 Manual Scenarios

* Known webcams/landmarks with surveyed positions.
* Photos with/without EXIF focal length.

---

## 14) Future Extensions (nice-to-have)

* Multi-image **bundle adjustment** (shared camera across frames or multi-cam rigs).
* Terrain elevation via local DEM (SRTM/MapLibre, if ever brought in).
* Line/edge constraints (not just points).
* MCMC posterior sampling (more exact uncertainty) if performance allows.

---

## 15) Open Technical Choices (pre-decided defaults)

* **Optimization**: Levenberg-Marquardt with Huber loss (δ=1.0 px default).
* **Default priors**:

  * Focal prior from EXIF if present (σ = 10% of value); else wide (σ = imageWidth).
  * Camera altitude prior disabled by default.
* **RANSAC**:

  * inlier threshold = **2.0 px**
  * maxIters = **5000**
  * targetProb = **0.999**
* **Minimum correspondences**:

  * If `estimateFocal=false`: 4+ (prefer 6–10)
  * If `estimateFocal=true`: 6+ (prefer 10–20)

---

## 16) Deliverables

* Source repo with:

  * `/app` (React TS)
  * `/solver` (Rust → WASM)
  * `/worker` (TS wrapper with Comlink)
  * `/schemas` (TS types & JSON schema)
  * `/tests` (Rust + TS integration)
* Build scripts, CI (lint, typecheck, unit tests).
* README with setup, dev, and QA instructions.
* Sample projects (2–3 with expected outputs).
