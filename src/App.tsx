import React, { useEffect, useMemo, useState } from 'react';
import * as Comlink from 'comlink';
import type { SolveRequest, SolveResponse } from './types/solver';
import ImageCanvas from './components/ImageCanvas';
import WorldMap from './components/WorldMap';
import LinkingPanel from './components/LinkingPanel';
import { useStore } from './state/store';

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

  const pixelPoints = useStore(s => s.pixelPoints);
  const worldPoints = useStore(s => s.worldPoints);
  const links = useStore(s => s.links);
  const image = useStore(s => s.image);

  const onSolve = async () => {
    if (!image) { setStatus('Please upload an image'); return; }
    setStatus('Solving...');
    try {
      const correspondences = links.map(l => {
        const p = pixelPoints.find(pp => pp.id === l.pixelId)!;
        const w = worldPoints.find(wp => wp.id === l.worldId)!;
        return { id: l.pixelId, pixel: { u: p.u, v: p.v, sigmaPx: p.sigmaPx }, world: { lat: w.lat, lon: w.lon, alt: w.alt, sigmaM: w.sigmaM }, enabled: true };
      });
      const req: SolveRequest = {
        image: { width: image.width, height: image.height },
        correspondences,
        model: { estimateFocal: true, estimatePrincipalPoint: false, estimateDistortion: false },
        ransac: { maxIters: 5000, inlierPx: 2.0, targetProb: 0.999 },
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
    <div style={{ fontFamily: 'system-ui, sans-serif', padding: 12, height: '100%', boxSizing: 'border-box' }}>
      <h1 style={{ margin: '8px 0 12px' }}>CamPose</h1>
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, alignItems: 'start' }}>
        <div>
          <h3>Image</h3>
          <ImageCanvas height={520} />
        </div>
        <div>
          <h3>Map</h3>
          <WorldMap height={520} />
        </div>
      </div>
  <div style={{ marginTop: 12, display: 'flex', gap: 12, alignItems: 'center', flexWrap: 'wrap' }}>
        <button onClick={onSolve} style={{ padding: '8px 12px', border: '1px solid #ccc', borderRadius: 6 }}>Solve</button>
        <span>Status: {status}</span>
        <span>| Points: {pixelPoints.length} px, {worldPoints.length} world, links: {links.length}</span>
      </div>
  <LinkingPanel />
      {result && (
        <pre style={{ marginTop: 12, background: '#111', color: '#ddd', padding: 12, borderRadius: 8, overflow: 'auto', maxHeight: 240 }}>
{JSON.stringify(result, null, 2)}
        </pre>
      )}
    </div>
  );
}
