import React, { useEffect, useMemo, useState } from 'react';
import * as Comlink from 'comlink';
import type { SolveRequest, SolveResponse } from './types/solver';

// Worker setup
const createWorker = () => new Worker(new URL('./worker/solverWorker.ts', import.meta.url), { type: 'module' });

export default function App() {
  const [status, setStatus] = useState<string>('Idle');
  const [result, setResult] = useState<SolveResponse | null>(null);
  const workerApi = useMemo(() => {
    const w = createWorker();
    return Comlink.wrap<any>(w);
  }, []);

  useEffect(() => {
    return () => {
      // terminate worker when unmounting
      // @ts-ignore
      workerApi?.[Comlink.releaseProxy]?.();
    };
  }, [workerApi]);

  const onSolve = async () => {
    setStatus('Solving...');
    try {
      const req: SolveRequest = {
        image: { width: 1000, height: 750 },
        correspondences: [],
        model: { estimateFocal: true, estimatePrincipalPoint: false, estimateDistortion: false },
        ransac: { maxIters: 1000, inlierPx: 3.0, targetProb: 0.999 },
        refine: { maxIters: 50, robustLoss: 'huber', huberDelta: 1.0 },
        uncertainty: { bootstrap: { enabled: false, samples: 0 } },
      };
      const res = await workerApi.solve(req);
      setResult(res);
      setStatus('Done');
    } catch (err: any) {
      console.error(err);
      setStatus('Error: ' + (err?.message ?? String(err)));
    }
  };

  return (
    <div style={{ fontFamily: 'system-ui, sans-serif', padding: 16 }}>
      <h1>CamPose (prototype)</h1>
      <p>Status: {status}</p>
      <button onClick={onSolve} style={{ padding: '8px 12px', border: '1px solid #ccc', borderRadius: 6 }}>Solve (demo)</button>
      {result && (
        <pre style={{ marginTop: 16, background: '#111', color: '#ddd', padding: 12, borderRadius: 8, overflow: 'auto' }}>
{JSON.stringify(result, null, 2)}
        </pre>
      )}
      <p style={{ marginTop: 24, color: '#666' }}>Next: wire image/map UI and real correspondences.</p>
    </div>
  );
}
