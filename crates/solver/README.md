# solver (WASM)

Rust/WebAssembly camera-pose solver used by the PoseSolve web app.

This crate:

- Accepts 2D↔3D correspondences (pixel points + world lat/lon/alt points).
- Estimates a baseline camera pose and camera intrinsics.
- Returns diagnostics and a covariance matrix for downstream UI feedback.
- Compiles to WebAssembly and runs in-browser through a Web Worker.

## Project overview

The repository has a React + TypeScript frontend and this Rust solver core.

- Frontend gathers image points and map/world points.
- Frontend sends a JSON `SolveRequest` to this crate (via wasm-bindgen).
- Rust computes a first-pass estimate and returns a JSON `SolveResponse`.
- Worker boundary keeps the UI responsive while solving.

Current implementation status:

- `solve` is implemented and stable for baseline/fuzzy tests.
- `reproject_points` is a placeholder and currently returns `"Not implemented yet"`.

## Algorithm (current baseline)

The current solver is intentionally a **heuristic first-pass estimator**, not yet a full nonlinear geometric optimizer.

High-level flow in `solve_impl`:

1. Filter enabled correspondences.
2. Compute summary statistics (min/max/mean of world and pixel coordinates).
3. Estimate camera position:
	- 1 point: camera lat/lon = that point.
	- 2+ points: use a span-based heuristic around the correspondence bounds.
	- Clamp to optional geographic bounds prior if provided.
4. Estimate altitude:
	- Use `priors.camera_alt.mean` when supplied.
	- Otherwise use first world altitude minus 2 m (floored at 0), fallback 2 m.
5. Estimate intrinsics:
	- Use `priors.focal_px.mean` when supplied.
	- Otherwise default to `0.9 * image.width`.
	- Principal point is image center (`cx = width/2`, `cy = height/2`).
6. Estimate orientation:
	- `yaw` from bearing between estimated camera location and mean world point.
	- `pitch = -2`, `roll = 0` (fixed baseline defaults).
7. Diagnostics:
	- Per-point residuals in pixel units (sigma-aware).
	- RMSE and inlier IDs (all enabled points currently treated as inliers).
	- Baseline warning messages indicating heuristic mode.

## Build

From repository root:

```bash
npm run build:wasm
```

Equivalent direct command:

```bash
wasm-pack build crates/solver --target web --release
```

Generated output:

- `crates/solver/pkg/solver.js`
- `crates/solver/pkg/solver_bg.wasm`

## Running tests

### Rust solver tests (recommended for this crate)

From repository root:

```bash
cd crates/solver
cargo test
```

Useful variants:

```bash
cargo test estimation
cargo test diagnostics
cargo test fixture_case
```

### Full repository test pipeline

From repository root:

```bash
npm run test
```

This runs TypeScript checks + Vitest + Cypress component tests. In network-restricted environments, Cypress setup can fail unless installed with `CYPRESS_INSTALL_BINARY=0`.

## Adding new tests

Solver tests live under `crates/solver/src/tests/`.

- `estimation.rs`: pose/intrinsics behavior.
- `diagnostics.rs`: residuals, warnings, covariance, shape checks.
- `input_validation.rs`: expected failure paths.
- `fixture_case.rs`: end-to-end fixture-driven scenarios.

### Add a direct unit test

1. Open the most relevant test module (for example `estimation.rs`).
2. Use helpers from `helpers.rs` (`sample_corr`, `base_request`, `solve_to_response`).
3. Construct a `SolveRequest` with your new correspondences.
4. Assert the expected solver outputs (pose/intrinsics/diagnostics).

### Add a new fixture test (new image + world points)

When you have a real image and a new set of world points:

1. Prepare fixture JSON with points containing:
	- `id`, `enabled`
	- pixel: `u`, `v`, `sigmaPx`
	- world: `lat`, `lon`, `height` (mapped to altitude)
2. Store the fixture JSON in the repository (existing pattern uses root-level JSON files).
3. In `fixture_case.rs`, load it with `include_str!(...)`.
4. Deserialize into `FixtureRoot` (from `helpers.rs`) and map to solver `Corr` values.
5. Build a `SolveRequest` with image dimensions and optional priors (for example focal prior from EXIF).
6. Run `solve_impl` and assert acceptance criteria (for example max haversine distance from a known camera position).

Tip: keep one test focused on a single quality gate (e.g., position tolerance) so failures are easy to diagnose.

## Input/output model summary

- Input: `SolveRequest`
  - `image { width, height }`
  - `correspondences[]` with pixel + world data
  - optional `priors` (`focalPx`, `cameraAlt`, `bounds`)
- Output: `SolveResponse`
  - `pose` (`lat`, `lon`, `alt`, `yawDeg`, `pitchDeg`, `rollDeg`)
  - `intrinsics` (`focalPx`, `cx`, `cy`)
  - `covariance` (6×6 flattened matrix + labels)
  - `diagnostics` (RMSE, residuals, inliers, warnings)

## Notes for contributors

- Keep this crate deterministic and fast; it runs in-browser.
- If changing solver behavior, update/add tests in `src/tests/` in the same PR.
- If changing serialized fields, keep TypeScript and Rust types aligned.
