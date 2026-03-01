import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';

export type Point = {
  id: string;
  // image / pixel coords
  u?: number;
  v?: number;
  sigmaPx?: number;
  enabled?: boolean;
  height?: number;
  // world coords
  lat?: number;
  lon?: number;
  alt?: number;
  sigmaM?: number;
};

export type ImageInfo = { url?: string; width: number; height: number; name?: string };
export type Selectors = {
  getPointById: (state: { points: Point[] }, id: string) => Point | null;
  getImagePoints: (state: { points: Point[] }) => Point[];
  getWorldPoints: (state: { points: Point[] }) => Point[];
};

type State = {
  image: ImageInfo | null;
  // center of the image canvas in pixel coordinates (u,v)
  imageCenter: { u: number; v: number };
  // center of the world/map in lat/lon; may be null until map initializes
  mapCenter: { lat: number; lon: number } | null;
  points: Point[];
  activePointId: string | null;
  // UI view state (moved from component-local state)
  zoom: number;
  panOffset: { x: number; y: number };
  editingHeightId: string | null;
  selectors: Selectors;
  // actions
  setImage: (img: ImageInfo | null) => void;
  setZoom: (z: number) => void;
  setPanOffset: (p: { x: number; y: number }) => void;
  setEditingHeight: (id: string | null) => void;
  setImageCenter: (u: number, v: number) => void;
  setMapCenter: (lat: number, lon: number) => void;
  addPoint: (p: Partial<Point> & { id?: string }) => string;
  updatePointImage: (id: string, u: number, v: number) => void;
  updatePointWorld: (id: string, lat: number, lon: number) => void;
  updatePointHeight: (id: string, height: number) => void;
  updatePointFields: (id: string, fields: Partial<Point>) => void;
  removePoint: (id: string) => void;
  setActivePoint: (id: string | null) => void;
  selectLinkedPoint: (pointId: string, pointType: 'pixel' | 'world') => void;
};

let _idCounter = 0;
const genId = (prefix: string) => `${prefix}_${Date.now().toString(36)}_${(_idCounter++).toString(36)}`;
export const selectors: Selectors = {
  getPointById: (state, id: string): Point | null => state.points.find(p => p.id === id) ?? null,
  getImagePoints: (state) => state.points.filter(p => typeof p.u === 'number' && typeof p.v === 'number'),
  getWorldPoints: (state) => state.points.filter(p => typeof p.lat === 'number' && typeof p.lon === 'number'),
};

const createMemoryStorage = (): Storage => {
  const map = new Map<string, string>();
  return {
    get length() {
      return map.size;
    },
    clear() {
      map.clear();
    },
    getItem(key: string) {
      return map.has(key) ? map.get(key)! : null;
    },
    key(index: number) {
      return Array.from(map.keys())[index] ?? null;
    },
    removeItem(key: string) {
      map.delete(key);
    },
    setItem(key: string, value: string) {
      map.set(key, value);
    },
  };
};

const fallbackStorage = createMemoryStorage();

const getSafeStorage = (): Storage => {
  const candidate = (globalThis as { localStorage?: unknown }).localStorage as Partial<Storage> | undefined;
  if (
    candidate
    && typeof candidate.getItem === 'function'
    && typeof candidate.setItem === 'function'
    && typeof candidate.removeItem === 'function'
  ) {
    return candidate as Storage;
  }
  return fallbackStorage;
};

export const useStore = create<State>()(persist((set, get) => ({
  // Default image (requested): Coolhaven / Erasmus medical center Rotterdam skyline
  image: {
    url: 'https://upload.wikimedia.org/wikipedia/commons/9/9e/Coolhaven_Erasmus_medical_center_Rotterdam_skyline.jpg',
    // placeholder dimensions; ImageCanvas will update these to the real natural size when the image loads
    width: 3264,
    height: 2448,
    name: 'Coolhaven Rotterdam (default)'
  },
  // initialize image center to middle of placeholder image
  imageCenter: { u: 3264 / 2, v: 2448 / 2 },
  // map center unknown until WorldMap sets it
  mapCenter: null,
  points: [],
  activePointId: null,
  // default view state
  zoom: 1,
  panOffset: { x: 0, y: 0 },
  editingHeightId: null,
  setImage: (img) => set({ image: img }),
  setImageCenter: (u, v) => set({ imageCenter: { u, v } }),
  setMapCenter: (lat, lon) => set({ mapCenter: { lat, lon } }),
  setZoom: (z) => set({ zoom: z }),
  setPanOffset: (p) => set({ panOffset: p }),
  setEditingHeight: (id) => set({ editingHeightId: id }),
  addPoint: (p) => {
    const id = p.id ?? genId('pt');
    const defaults: Partial<Point> = { enabled: true, sigmaPx: 1, height: 0 };
    // populate missing side from known centers where possible
    const state = get();
    const filled: Partial<Point> = { ...defaults, ...p };

    const hasImage = typeof p.u === 'number' && typeof p.v === 'number';
    const hasWorld = typeof p.lat === 'number' && typeof p.lon === 'number';

    // Compute centroid of existing points for each side (fallback to viewport center)
    const imagePoints = state.points.filter(pt => typeof pt.u === 'number' && typeof pt.v === 'number');
    const worldPoints = state.points.filter(pt => typeof pt.lat === 'number' && typeof pt.lon === 'number');

    const imageCentroid = imagePoints.length > 0
      ? {
          u: imagePoints.reduce((sum, pt) => sum + pt.u!, 0) / imagePoints.length,
          v: imagePoints.reduce((sum, pt) => sum + pt.v!, 0) / imagePoints.length,
        }
      : state.imageCenter;

    const worldCentroid = worldPoints.length > 0
      ? {
          lat: worldPoints.reduce((sum, pt) => sum + pt.lat!, 0) / worldPoints.length,
          lon: worldPoints.reduce((sum, pt) => sum + pt.lon!, 0) / worldPoints.length,
        }
      : state.mapCenter;

    // If user supplied image coords but not world coords, use centroid of world points
    if (hasImage && !hasWorld && worldCentroid) {
      filled.lat = worldCentroid.lat;
      filled.lon = worldCentroid.lon;
    }
    // If user supplied world coords but not image coords, use centroid of image points
    if (hasWorld && !hasImage && imageCentroid) {
      filled.u = imageCentroid.u;
      filled.v = imageCentroid.v;
    }
    // If neither side supplied, fall back to centroids (image centroid always exists)
    if (!hasImage && !hasWorld) {
      filled.u = filled.u ?? imageCentroid.u;
      filled.v = filled.v ?? imageCentroid.v;
      if (worldCentroid) {
        filled.lat = filled.lat ?? worldCentroid.lat;
        filled.lon = filled.lon ?? worldCentroid.lon;
      }
    }

    set((s) => ({ points: [...s.points, { id, ...filled }] }));
    return id;
  },
  updatePointImage: (id, u, v) => set((s) => ({
    points: s.points.map(pt => pt.id === id ? { ...pt, u, v } : pt)
  })),
  updatePointWorld: (id, lat, lon) => set((s) => ({
    points: s.points.map(pt => pt.id === id ? { ...pt, lat, lon } : pt)
  })),
  updatePointHeight: (id, height) => set((s) => ({
    points: s.points.map(pt => pt.id === id ? { ...pt, height } : pt)
  })),
  updatePointFields: (id, fields) => set((s) => ({
    points: s.points.map(pt => pt.id === id ? { ...pt, ...fields } : pt)
  })),
  removePoint: (id) => set((s) => ({
    points: s.points.filter(pt => pt.id !== id),
    activePointId: s.activePointId === id ? null : s.activePointId,
  })),
  setActivePoint: (id) => set({ activePointId: id }),
  selectLinkedPoint: (pointId, pointType) => {
    // In unified model, selecting any point simply makes it active.
    const state = get();
    const pt = state.points.find(p => p.id === pointId);
    set({ activePointId: pt ? pt.id : null });
  },
  // expose selectors for tests
  selectors,
}), {
  name: 'posesolve-assistant-store',
  storage: createJSONStorage(getSafeStorage),
  partialize: (state) => ({
    points: state.points,
    activePointId: state.activePointId,
  }),
}));
