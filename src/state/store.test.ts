import { describe, it, expect, beforeEach } from 'vitest';
import { useStore } from './store';

describe('Store - Cross-selection functionality', () => {
  beforeEach(() => {
    // Reset store to initial state
    useStore.setState({
      image: null,
      pixelPoints: [],
      worldPoints: [],
      links: [],
      activePixelId: null,
      activeWorldId: null,
    });
  });

  it('should select linked world point when pixel point is selected', () => {
    const { addPixelPoint, addWorldPoint, linkPoints, selectLinkedPoint } = useStore.getState();
    
    // Add points and link them
    const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    const worldId = addWorldPoint({ lat: 52.0, lon: 4.0 });
    linkPoints(pixelId, worldId);
    
    // Select pixel point should select linked world point
    selectLinkedPoint(pixelId, 'pixel');
    
    const state = useStore.getState();
    expect(state.activePixelId).toBe(pixelId);
    expect(state.activeWorldId).toBe(worldId);
  });

  it('should select linked pixel point when world point is selected', () => {
    const { addPixelPoint, addWorldPoint, linkPoints, selectLinkedPoint } = useStore.getState();
    
    // Add points and link them
    const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    const worldId = addWorldPoint({ lat: 52.0, lon: 4.0 });
    linkPoints(pixelId, worldId);
    
    // Select world point should select linked pixel point
    selectLinkedPoint(worldId, 'world');
    
    const state = useStore.getState();
    expect(state.activeWorldId).toBe(worldId);
    expect(state.activePixelId).toBe(pixelId);
  });

  it('should handle selection of unlinked points', () => {
    const { addPixelPoint, selectLinkedPoint } = useStore.getState();
    
    // Add unlinked pixel point
    const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    
    // Select unlinked pixel point
    selectLinkedPoint(pixelId, 'pixel');
    
    const state = useStore.getState();
    expect(state.activePixelId).toBe(pixelId);
    expect(state.activeWorldId).toBeNull();
  });
});

describe('Store - Height functionality', () => {
  beforeEach(() => {
    useStore.setState({
      pixelPoints: [],
    });
  });

  it('should add pixel points with default height of 0', () => {
    const { addPixelPoint } = useStore.getState();
    
    const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true });
    
    const state = useStore.getState();
    const point = state.pixelPoints.find(p => p.id === pixelId);
    expect(point?.height).toBe(0);
  });

  it('should update pixel point height', () => {
    const { addPixelPoint, updatePixelPointHeight } = useStore.getState();
    
    const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    updatePixelPointHeight(pixelId, 15.5);
    
    const state = useStore.getState();
    const point = state.pixelPoints.find(p => p.id === pixelId);
    expect(point?.height).toBe(15.5);
  });

  it('should preserve height when moving points', () => {
    const { addPixelPoint, updatePixelPointHeight, movePixelPoint } = useStore.getState();
    
    const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    updatePixelPointHeight(pixelId, 10);
    movePixelPoint(pixelId, 200, 200);
    
    const state = useStore.getState();
    const point = state.pixelPoints.find(p => p.id === pixelId);
    expect(point?.height).toBe(10);
    expect(point?.u).toBe(200);
    expect(point?.v).toBe(200);
  });
});

describe('Store - Linking functionality', () => {
  beforeEach(() => {
    useStore.setState({
      pixelPoints: [],
      worldPoints: [],
      links: [],
      activePixelId: null,
      activeWorldId: null,
    });
  });

  it('should create 1-to-1 links and clear active selections', () => {
    const { addPixelPoint, addWorldPoint, linkPoints } = useStore.getState();
    
    const pixelId1 = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    const pixelId2 = addPixelPoint({ u: 200, v: 200, sigmaPx: 1, enabled: true, height: 0 });
    const worldId1 = addWorldPoint({ lat: 52.0, lon: 4.0 });
    const worldId2 = addWorldPoint({ lat: 52.1, lon: 4.1 });
    
    // Link first pair
    linkPoints(pixelId1, worldId1);
    
    let state = useStore.getState();
    expect(state.links).toHaveLength(1);
    expect(state.activePixelId).toBeNull();
    expect(state.activeWorldId).toBeNull();
    
    // Link second pair should not interfere
    linkPoints(pixelId2, worldId2);
    
    state = useStore.getState();
    expect(state.links).toHaveLength(2);
  });

  it('should enforce 1-to-1 constraint by removing existing links', () => {
    const { addPixelPoint, addWorldPoint, linkPoints } = useStore.getState();
    
    const pixelId1 = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    const pixelId2 = addPixelPoint({ u: 200, v: 200, sigmaPx: 1, enabled: true, height: 0 });
    const worldId1 = addWorldPoint({ lat: 52.0, lon: 4.0 });
    
    // Link first pair
    linkPoints(pixelId1, worldId1);
    
    // Try to link second pixel to same world point
    linkPoints(pixelId2, worldId1);
    
    const state = useStore.getState();
    expect(state.links).toHaveLength(1);
    expect(state.links[0].pixelId).toBe(pixelId2);
    expect(state.links[0].worldId).toBe(worldId1);
  });

  it('should remove links when points are deleted', () => {
    const { addPixelPoint, addWorldPoint, linkPoints, removePixelPoint, removeWorldPoint } = useStore.getState();
    
    const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    const worldId = addWorldPoint({ lat: 52.0, lon: 4.0 });
    linkPoints(pixelId, worldId);
    
    // Remove pixel point should remove link
    removePixelPoint(pixelId);
    
    let state = useStore.getState();
    expect(state.links).toHaveLength(0);
    expect(state.pixelPoints).toHaveLength(0);
    
    // Add them back and test world point removal
    const newPixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    const newWorldId = addWorldPoint({ lat: 52.0, lon: 4.0 });
    linkPoints(newPixelId, newWorldId);
    
    removeWorldPoint(newWorldId);
    
    state = useStore.getState();
    expect(state.links).toHaveLength(0);
    expect(state.worldPoints).toHaveLength(0);
  });
});

describe('Store - Selectors', () => {
  beforeEach(() => {
    useStore.setState({
      pixelPoints: [],
      worldPoints: [],
      links: [],
    });
  });

  it('should find linked world point for pixel point', () => {
    const { addPixelPoint, addWorldPoint, linkPoints, selectors } = useStore.getState();
    
    const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    const worldId = addWorldPoint({ lat: 52.0, lon: 4.0 });
    linkPoints(pixelId, worldId);
    
    const state = useStore.getState();
    const linkedWorld = selectors.getLinkedWorldForPixel(state, pixelId);
    
    expect(linkedWorld).toBeTruthy();
    expect(linkedWorld?.id).toBe(worldId);
    expect(linkedWorld?.lat).toBe(52.0);
    expect(linkedWorld?.lon).toBe(4.0);
  });

  it('should find linked pixel point for world point', () => {
    const { addPixelPoint, addWorldPoint, linkPoints, selectors } = useStore.getState();
    
    const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    const worldId = addWorldPoint({ lat: 52.0, lon: 4.0 });
    linkPoints(pixelId, worldId);
    
    const state = useStore.getState();
    const linkedPixel = selectors.getLinkedPixelForWorld(state, worldId);
    
    expect(linkedPixel).toBeTruthy();
    expect(linkedPixel?.id).toBe(pixelId);
    expect(linkedPixel?.u).toBe(100);
    expect(linkedPixel?.v).toBe(100);
  });

  it('should return null for unlinked points', () => {
    const { addPixelPoint, addWorldPoint, selectors } = useStore.getState();
    
    const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 });
    const worldId = addWorldPoint({ lat: 52.0, lon: 4.0 });
    
    const state = useStore.getState();
    
    expect(selectors.getLinkedWorldForPixel(state, pixelId)).toBeNull();
    expect(selectors.getLinkedPixelForWorld(state, worldId)).toBeNull();
  });
});