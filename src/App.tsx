import React, { useEffect, useRef, useState } from "react";
import * as Comlink from "comlink";
import type { SolveRequest, SolveResponse } from "./types/solver";
import ImageCanvas from "./components/ImageCanvas";
import WorldMap from "./components/WorldMap";
import Toolbar from "./components/Toolbar";
import PointsTable from "./components/PointsTable";
import SolveResult from "./components/SolveResult";
import { useStore } from "./state/store";

// Worker setup
const createWorker = () =>
  new Worker(new URL("./worker/solverWorker.ts", import.meta.url), {
    type: "module",
  });

export default function App() {
  const [status, setStatus] = useState<string>("Idle");
  const [result, setResult] = useState<SolveResponse | null>(null);

  // Keep a stable ref to the current proxy so onSolve always uses the live one.
  // The effect below recreates both the raw Worker and the Comlink proxy whenever
  // the component mounts (including the Strict Mode double-mount cycle).
  const workerApiRef = useRef<Comlink.Remote<{
    solve: (req: SolveRequest) => Promise<SolveResponse>;
    reproject_points: (req: unknown) => Promise<unknown>;
  }> | null>(null);

  useEffect(() => {
    const w = createWorker();
    workerApiRef.current = Comlink.wrap<{
      solve: (req: SolveRequest) => Promise<SolveResponse>;
      reproject_points: (req: unknown) => Promise<unknown>;
    }>(w);

    return () => {
      workerApiRef.current?.[Comlink.releaseProxy]();
      workerApiRef.current = null;
      w.terminate();
    };
  }, []);

  const points = useStore((s) => s.points);
  const image = useStore((s) => s.image);
  const activePointId = useStore((s) => s.activePointId);
  const removePoint = useStore((s) => s.removePoint);
  const imagePoints = points.filter(
    (p) => typeof p.u === "number" && typeof p.v === "number"
  );
  const worldPoints = points.filter(
    (p) => typeof p.lat === "number" && typeof p.lon === "number"
  );
  const linkedPoints = points.filter(
    (p) =>
      typeof p.u === "number" &&
      typeof p.v === "number" &&
      typeof p.lat === "number" &&
      typeof p.lon === "number"
  );

  const onSolve = async () => {
    if (!image) {
      setStatus("Please upload an image");
      return;
    }
    if (!workerApiRef.current) {
      setStatus("Worker not ready");
      return;
    }
    setStatus("Solving...");
    try {
      const correspondences = linkedPoints.map((pt) => ({
        id: pt.id,
        pixel: { u: pt.u as number, v: pt.v as number, sigmaPx: pt.sigmaPx },
        world: {
          lat: pt.lat as number,
          lon: pt.lon as number,
          alt: pt.alt,
          sigmaM: pt.sigmaM,
        },
        enabled: true,
      }));
      const req: SolveRequest = {
        image: { width: image.width, height: image.height },
        correspondences,
        model: {
          estimateFocal: true,
          estimatePrincipalPoint: false,
          estimateDistortion: false,
        },
        ransac: { maxIters: 5000, inlierPx: 2.0, targetProb: 0.999 },
        refine: { maxIters: 50, robustLoss: "huber", huberDelta: 1.0 },
        uncertainty: { bootstrap: { enabled: false, samples: 0 } },
      };
      const res = await workerApiRef.current.solve(req);
      setResult(res);
      setStatus("Done");
    } catch (err: any) {
      console.error(err);
      setStatus("Error: " + (err?.message ?? String(err)));
    }
  };

  return (
    <div
      style={{
        fontFamily: "system-ui, sans-serif",
        padding: 12,
        height: "100%",
        boxSizing: "border-box",
      }}
    >
      <h1 style={{ margin: "8px 0 12px" }}>CamPose</h1>
      <Toolbar
        activePointId={activePointId}
        onDelete={() => {
          if (activePointId) removePoint(activePointId);
        }}
        onSolve={onSolve}
        status={status}
        counts={{
          image: imagePoints.length,
          world: worldPoints.length,
          linked: linkedPoints.length,
        }}
      />
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 12,
          alignItems: "start",
        }}
      >
        <div>
          <h3>Image</h3>
          <ImageCanvas height={520} showLocalDelete={false} />
        </div>
        <div>
          <h3>Map</h3>
          <WorldMap height={520} cameraPosition={result?.pose ?? null} />
        </div>
      </div>

      {/* Points List */}
      <div style={{ marginTop: 12 }}>
        <PointsTable />
      </div>

      {/* toolbar contains global action buttons and status */}
      {result && <SolveResult result={result} />}
    </div>
  );
}
