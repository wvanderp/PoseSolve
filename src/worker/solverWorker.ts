import * as Comlink from 'comlink';
import type { SolveRequest, SolveResponse } from '../types/solver';

// Static import of the wasm-pack generated JS glue.
// Vite handles `new URL('./solver_bg.wasm', import.meta.url)` inside solver.js
// and emits the WASM as a hashed asset in both dev and production builds.
// @ts-ignore: TS can't resolve the wasm-pack generated module; types come from solver.d.ts
import init, { solve as wasmSolve, reproject_points as wasmReproject } from '../../crates/solver/pkg/solver.js';

// Initialise once; re-use the same promise so concurrent callers don't double-init.
const initPromise = init().catch((err: unknown) => {
  console.error('[solverWorker] WASM init failed:', err);
  throw err;
});

const api = {
  async solve(req: SolveRequest): Promise<SolveResponse> {
    try {
      await initPromise;
      const out = wasmSolve(JSON.stringify(req));
      return JSON.parse(out) as SolveResponse;
    } catch (err: any) {
      console.error('[solverWorker] solve error:', err);
      return {
        pose: { lat: 0, lon: 0, alt: 0, yawDeg: 0, pitchDeg: 0, rollDeg: 0 },
        intrinsics: { focalPx: 1000, cx: req.image.width / 2, cy: req.image.height / 2 },
        covariance: { matrix: [], labels: [] },
        diagnostics: {
          rmsePx: 0,
          inlierRatio: 0,
          residualsPx: [],
          inlierIds: [],
          warnings: ['Solver error: ' + (err?.message ?? String(err))],
        },
      };
    }
  },

  async reproject_points(req: unknown): Promise<unknown> {
    try {
      await initPromise;
      const out = wasmReproject(JSON.stringify(req));
      return JSON.parse(out);
    } catch (err: any) {
      return { pixels: [], warnings: ['Solver error: ' + (err?.message ?? String(err))] };
    }
  },
};

Comlink.expose(api);
