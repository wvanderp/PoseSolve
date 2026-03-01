# Solver Improvement Plan: PnP via Nonlinear Optimization

## Problem Statement

The current `solve_impl` estimator is a naive heuristic:

- **Single point**: camera position = the world point itself
- **Multi-point**: camera lat/lon = centroid + hardcoded offset (`+0.1*span_lat`, `-0.5*span_lon`)
- **Orientation**: yaw from bearing to centroid, pitch/roll hardcoded (`-2°`, `0°`)

This produces results ~300 m off on the Pathé fixture (5 correspondences, expected ≤10 m).
The core issue is that **pixel observations are never used geometrically** — only the world-point centroid and bounding box are considered.

## Chosen Approach: Iterative Reprojection-Error Minimization

This is the classic **Perspective-n-Point (PnP)** problem. Given:

- `n` pixel ↔ world-LLA correspondences
- Camera intrinsics (focal length, principal point)

Find the camera's 6-DOF pose (lat, lon, alt, yaw, pitch, roll).

**Algorithm**: Levenberg-Marquardt (LM) nonlinear least-squares, minimizing total reprojection error. This is the standard approach in photogrammetry and visual SLAM.

### Why this approach?

| Alternative                   | Verdict                                                                                        |
| ----------------------------- | ---------------------------------------------------------------------------------------------- |
| DLT (Direct Linear Transform) | Needs ≥6 points; our fixtures have 3–5                                                         |
| P3P (closed-form 3-point)     | Exactly 3 points, up to 4 ambiguous solutions; fragile                                         |
| EPnP                          | Needs ≥4 points and is complex to implement correctly                                          |
| **LM on reprojection error**  | **Works with ≥3 points, handles priors naturally, gives covariance for free, well-understood** |

### Dependencies

- **nalgebra** (already commented out in `Cargo.toml`): needed for matrix operations (SVD, linear solves). The `libm` feature makes it `no_std`-compatible for WASM.
- No other new dependencies required.

---

## Implementation Increments

### Increment 1: ENU Coordinate Conversions (`geo.rs`)

**Goal**: Convert between LLA (lat/lon/alt) and a local East-North-Up (ENU) Cartesian frame.

**What to do**:

- Add `lla_to_enu(lat, lon, alt, ref_lat, ref_lon, ref_alt) → (e, n, u)` using WGS-84 ellipsoid
- Add `enu_to_lla(e, n, u, ref_lat, ref_lon, ref_alt) → (lat, lon, alt)` (inverse)
- The reference point will be the centroid of the world points

**Why first**: Every subsequent step needs Cartesian coordinates. LLA is non-Euclidean; we can't do linear algebra with degrees directly.

**Tests**: Unit-test round-trip `lla_to_enu → enu_to_lla` for known coordinates.

**Estimated size**: ~60 lines of code + tests.

---

### Increment 2: Pinhole Camera Projection Model (`projection.rs`)

**Goal**: Implement the forward projection: given camera pose + intrinsics + a 3D ENU point → predicted pixel (u, v).

**What to do**:

- Build rotation matrix `R(yaw, pitch, roll)` from Euler angles (yaw = heading, pitch = tilt, roll = bank)
- Transform world-ENU point to camera frame: `p_cam = R * (p_world - t_cam)`
- Apply pinhole projection: `u = fx * (x/z) + cx`, `v = fy * (y/z) + cy`
- Return predicted pixel coordinates

**Convention**: Define the camera coordinate system (Z forward, X right, Y down — standard CV convention) and the mapping from ENU to camera initial orientation (camera looking North at horizon → yaw=0, pitch=0, roll=0).

**Tests**: Verify that a point directly in front of the camera projects to the principal point. Verify that a point to the right projects to a higher `u`.

**Estimated size**: ~80 lines + tests. Depends on Increment 1.

---

### Increment 3: Reprojection Error & Jacobians

**Goal**: Compute the residual vector and its Jacobian for the LM optimizer.

**What to do**:

- `reprojection_residuals(params, correspondences, intrinsics, ref_lla) → Vec<f64>` where params = `[lat_offset, lon_offset, alt, yaw, pitch, roll]` (6 values)
- Each correspondence contributes 2 residuals: `(u_predicted - u_observed) / sigma`, `(v_predicted - v_observed) / sigma`
- Jacobian: 2n × 6 matrix. Use **numerical differentiation** (central differences) initially for correctness; can switch to analytic later if needed for performance.

**Tests**: Check that residuals are zero when the camera is placed at the true position with perfect correspondences. Check Jacobian against finite differences.

**Estimated size**: ~60 lines + tests.

---

### Increment 4: Levenberg-Marquardt Optimizer

**Goal**: Implement LM to iteratively refine the 6-DOF camera pose.

**What to do**:

- Standard LM loop:
  1. Compute residuals `r` and Jacobian `J`
  2. Solve `(J^T J + λ I) δ = -J^T r` for step `δ`
  3. If cost decreases, accept step, reduce `λ`; else increase `λ`
  4. Repeat until convergence (small step or small gradient)
- Use nalgebra for the 6×6 linear solve (Cholesky or LU)
- Cap iterations (e.g., 50) and check for convergence

**Tests**: Optimize a synthetic case where the true pose is known. Verify convergence within a few iterations.

**Estimated size**: ~100 lines + tests.

---

### Increment 5: Initialization Heuristic

**Goal**: Provide a good starting point for the LM optimizer so it converges to the correct local minimum.

**What to do**:

- **Position**: Use bearing intersection. For each pair of correspondences, the pixel direction in camera frame constrains the camera to a half-plane. A rough intersection gives a position estimate.
  - Simpler alternative: Start at `(centroid_lat - offset, centroid_lon - offset)` where offset is estimated from the average angular span of the pixel points vs the geographic span.
- **Yaw**: Compute the bearing from the camera estimate to the centroid of world points; align with the horizontal centroid of pixel coordinates.
- **Pitch**: Estimate from the vertical pixel centroid offset from the principal point.
- **Roll**: Start at 0 (assume level camera).
- **Altitude**: Use prior if available, otherwise `mean_world_alt - 2m` (existing logic).

**Tests**: Verify the initialization is within the basin of convergence for the Pathé fixture.

**Estimated size**: ~80 lines + tests.

---

### Increment 6: Integration into `solve_impl`

**Goal**: Wire up increments 1–5 to replace the heuristic estimator.

**What to do**:

- In `solve_impl`:
  1. Collect active correspondences (existing logic)
  2. Compute reference LLA (centroid of world points)
  3. Convert all world points to ENU
  4. Compute initial guess (Increment 5)
  5. Run LM optimizer (Increment 4)
  6. Convert optimized camera ENU position back to LLA
  7. Populate `SolveResponse` with optimized pose + intrinsics
- Handle edge cases:
  - 1 point: Keep existing behavior (camera at the point location)
  - 2 points: LM with constrained DOF or fallback heuristic
  - ≥3 points: Full LM optimization

**Tests**: All 17 existing tests must pass, including:

- `coolhaven_pipeline_native_types_within_100m` (3 points, ≤100 m)
- `Pathe` (5 points, ≤10 m) ← **currently failing**

**Estimated size**: ~40 lines (mostly wiring).

---

### Increment 7: Update Existing Tests

**Goal**: Adapt unit tests that assert on the old heuristic behavior.

**What to do**:

- `multipoint_solution_uses_span_heuristic` — This test asserts the exact heuristic offsets (`mean + 0.1*span`). Rewrite to check that the solution is geometrically reasonable (e.g., camera is behind the points relative to the pixel layout).
- `pose_orientation_defaults_and_yaw_range` — The hardcoded pitch=-2°, roll=0° assertions need to accept optimizer-derived values. Keep yaw ∈ [0, 360) check.
- `diagnostics_include_baseline_warning` — Remove or update the "heuristic estimator" warning text.
- `bounds_prior_clamps_position` — Bounds need to be enforced as box constraints in the optimizer or as a post-clamp.

**Tests**: All 17 tests green.

---

### Increment 8: Priors as Soft Constraints

**Goal**: Incorporate priors (focal, altitude, bounds) into the optimization.

**What to do**:

- **Altitude prior**: Add a penalty term `(alt - prior_mean) / sigma_alt` to the residual vector
- **Bounds prior**: Clamp position after each LM step (box constraint) or add barrier penalty terms
- **Focal prior**: If focal is a free parameter in future, add prior penalty. For now, focal is fixed from input.

**Estimated size**: ~30 lines.

---

### Increment 9: Meaningful Covariance & Diagnostics

**Goal**: Replace the hardcoded covariance with one derived from the optimization.

**What to do**:

- After LM converges, compute `Cov = σ² (J^T J)^{-1}` where `σ² = cost / (2n - 6)`
- Populate the 6×6 covariance matrix
- Compute true reprojection residuals (in pixels) for each correspondence
- Compute RMSE from actual reprojection errors

**Estimated size**: ~30 lines.

---

## Summary

| Increment | Description                   | Depends on | ~Lines   |
| --------- | ----------------------------- | ---------- | -------- |
| 1         | ENU coordinate conversions    | —          | 60       |
| 2         | Pinhole projection model      | 1          | 80       |
| 3         | Reprojection error + Jacobian | 1, 2       | 60       |
| 4         | Levenberg-Marquardt optimizer | 3          | 100      |
| 5         | Initialization heuristic      | 1, 2       | 80       |
| 6         | Integration into `solve_impl` | 1–5        | 40       |
| 7         | Update existing tests         | 6          | 30       |
| 8         | Priors as soft constraints    | 6          | 30       |
| 9         | Covariance & diagnostics      | 6          | 30       |
| **Total** |                               |            | **~510** |

## Risks & Mitigations

- **LM convergence**: Bad initialization → wrong local minimum. Mitigated by Increment 5 (good init) and optionally multi-start with a few random perturbations.
- **Numerical stability near poles or with tiny baselines**: Use ENU (Increment 1) to keep numbers in reasonable ranges (meters, not degrees).
- **Test breakage**: Increments 6+7 are coupled. Implement 1–5 as pure additions with their own tests, then swap in Increment 6 and fix tests in 7 together.
- **WASM size**: nalgebra with `libm` + `no_std` keeps the binary small. The `wasm-opt = false` setting remains.
