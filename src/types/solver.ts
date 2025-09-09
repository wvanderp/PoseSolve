export type Pixel = { u: number; v: number; sigmaPx?: number };
export type WorldLLA = { lat: number; lon: number; alt?: number; sigmaM?: number };
export type Corr = { id: string; pixel: Pixel; world: WorldLLA; enabled?: boolean };

export type SolverModel = {
  estimateFocal: boolean;
  estimatePrincipalPoint: boolean;
  estimateDistortion: boolean; // k1,k2,(p1,p2)
};

export type Priors = {
  focalPx?: { mean: number; sigma: number };
  cameraAlt?: { mean: number; sigma: number };
  bounds?: { latMin: number; latMax: number; lonMin: number; lonMax: number };
};

export type RansacCfg = { maxIters: number; inlierPx: number; targetProb: number };
export type RefineCfg = { maxIters: number; robustLoss: 'none'|'huber'; huberDelta?: number };
export type UncertaintyCfg = { bootstrap: { enabled: boolean; samples: number; seed?: number } };

export type SolveRequest = {
  image: { width: number; height: number };
  correspondences: Corr[];
  model: SolverModel;
  priors?: Priors;
  ransac: RansacCfg;
  refine: RefineCfg;
  uncertainty: UncertaintyCfg;
};

export type Pose = {
  lat: number; lon: number; alt: number;
  yawDeg: number; pitchDeg: number; rollDeg: number;
};

export type Intrinsics = {
  focalPx: number;
  cx: number; cy: number;
  k1?: number; k2?: number; p1?: number; p2?: number;
};

export type Covariance = {
  matrix: number[]; labels: string[];
};

export type Diagnostics = {
  rmsePx: number;
  inlierRatio: number;
  residualsPx: number[];
  inlierIds: string[];
  warnings: string[];
};

export type Bootstrap = {
  positionSamples: number[][];
  orientationSamples: number[][];
  focalSamples?: number[];
};

export type SolveResponse = {
  pose: Pose;
  intrinsics: Intrinsics;
  covariance: Covariance;
  bootstrap?: Bootstrap;
  diagnostics: Diagnostics;
};
