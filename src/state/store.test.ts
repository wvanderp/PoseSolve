import { describe, it, expect, beforeEach } from 'vitest';
import { useStore } from './store';

const PERSIST_KEY = 'posesolve-assistant-store';

const clearPersistedStore = () => {
  useStore.persist.clearStorage();
};

type PersistedAssistantState = {
  state: {
    points: Array<{ id: string }>;
    activePointId: string | null;
  };
  version: number;
};

describe('Store - Selection functionality (unified points)', () => {
  beforeEach(() => {
    clearPersistedStore();
    // Reset store to initial state
    useStore.setState({ image: null, points: [], activePointId: null });
  });

  it('selecting an image point sets activePointId', () => {
    const { addPoint, selectLinkedPoint } = useStore.getState();

    const ptId = addPoint({ u: 100, v: 200 });
    selectLinkedPoint(ptId, 'pixel');

    const state = useStore.getState();
    expect(state.activePointId).toBe(ptId);
  });

  it('selecting a world point sets activePointId', () => {
    const { addPoint, selectLinkedPoint } = useStore.getState();

    const ptId = addPoint({ lat: 52.0, lon: 4.0 });
    selectLinkedPoint(ptId, 'world');

    const state = useStore.getState();
    expect(state.activePointId).toBe(ptId);
  });

  it('selecting a missing point clears activePointId', () => {
    const { selectLinkedPoint } = useStore.getState();

    selectLinkedPoint('nonexistent', 'pixel');
    const state = useStore.getState();
    expect(state.activePointId).toBeNull();
  });
});

describe('Store - Height functionality', () => {
  beforeEach(() => {
    clearPersistedStore();
    useStore.setState({ points: [] });
  });

  it('should add points with default height of 0', () => {
    const { addPoint } = useStore.getState();
    const id = addPoint({ u: 10, v: 20 });

    const state = useStore.getState();
    const pt = state.points.find(p => p.id === id);
    expect(pt).toBeTruthy();
    expect(pt?.height).toBe(0);
  });

  it('should update point height', () => {
    const { addPoint, updatePointHeight } = useStore.getState();
    const id = addPoint({ u: 10, v: 20 });
    updatePointHeight(id, 15.5);

    const state = useStore.getState();
    const pt = state.points.find(p => p.id === id);
    expect(pt?.height).toBe(15.5);
  });

  it('should preserve height when moving point (image update)', () => {
    const { addPoint, updatePointHeight, updatePointImage } = useStore.getState();
    const id = addPoint({ u: 10, v: 20 });
    updatePointHeight(id, 10);
    updatePointImage(id, 200, 300);

    const state = useStore.getState();
    const pt = state.points.find(p => p.id === id);
    expect(pt?.height).toBe(10);
    expect(pt?.u).toBe(200);
    expect(pt?.v).toBe(300);
  });
});

describe('Store - Removal and active clearing', () => {
  beforeEach(() => {
    clearPersistedStore();
    useStore.setState({ points: [], activePointId: null });
  });

  it('removing a point deletes it and clears active if needed', () => {
    const { addPoint, removePoint, setActivePoint } = useStore.getState();
    const id = addPoint({ u: 1, v: 2 });
    setActivePoint(id);

    let state = useStore.getState();
    expect(state.activePointId).toBe(id);

    removePoint(id);
    state = useStore.getState();
    expect(state.points.find(p => p.id === id)).toBeUndefined();
    expect(state.activePointId).toBeNull();
  });
});

describe('Store - Selectors', () => {
  beforeEach(() => {
    clearPersistedStore();
    useStore.setState({ points: [] });
  });

  it('getPointById returns the correct point', () => {
    const { addPoint, selectors } = useStore.getState();
    const id = addPoint({ u: 5, v: 6, lat: 52, lon: 4 });

    const state = useStore.getState();
    const pt = state.selectors.getPointById(state, id);
    expect(pt).toBeTruthy();
    expect(pt?.id).toBe(id);
  });

  it('getImagePoints returns only image points', () => {
    const { addPoint, selectors } = useStore.getState();
    const imgId = addPoint({ u: 10, v: 20 });
    addPoint({ lat: 52, lon: 4 });

    const state = useStore.getState();
    const imgs = state.selectors.getImagePoints(state);
    expect(imgs.some(p => p.id === imgId)).toBe(true);
    expect(imgs.every(p => typeof p.u === 'number' && typeof p.v === 'number')).toBe(true);
  });

  it('getWorldPoints returns only world points', () => {
    const { addPoint } = useStore.getState();
    const worldId = addPoint({ lat: 52.1, lon: 4.1 });
    addPoint({ u: 1, v: 2 });

    const state = useStore.getState();
    const worlds = state.selectors.getWorldPoints(state);
    expect(worlds.some(p => p.id === worldId)).toBe(true);
    expect(worlds.every(p => typeof p.lat === 'number' && typeof p.lon === 'number')).toBe(true);
  });
});

describe('Store - Persistence', () => {
  beforeEach(() => {
    clearPersistedStore();
    useStore.setState({ points: [], activePointId: null });
  });

  it('persists points and active point id to storage', async () => {
    const { addPoint, setActivePoint } = useStore.getState();
    const id = addPoint({ u: 100, v: 200, lat: 52.0, lon: 4.0 });
    setActivePoint(id);

    const storage = useStore.persist.getOptions().storage;
    expect(storage).toBeTruthy();
    const persisted = await storage?.getItem(PERSIST_KEY) as PersistedAssistantState | null;

    expect(persisted?.state.activePointId).toBe(id);
    expect(persisted?.state.points.some((p: { id: string }) => p.id === id)).toBe(true);
  });

  it('rehydrates points and active point id from storage', async () => {
    const storage = useStore.persist.getOptions().storage;
    expect(storage).toBeTruthy();

    const persistedState = {
      state: {
        points: [{ id: 'pt_persisted', u: 10, v: 20, lat: 51.9, lon: 4.4, height: 0, enabled: true, sigmaPx: 1 }],
        activePointId: 'pt_persisted',
      },
      version: 0,
    };
    useStore.setState({ points: [], activePointId: null });
    await storage?.setItem(PERSIST_KEY, persistedState);
    await useStore.persist.rehydrate();

    const state = useStore.getState();
    expect(state.points).toHaveLength(1);
    expect(state.points[0].id).toBe('pt_persisted');
    expect(state.activePointId).toBe('pt_persisted');
  });
});
