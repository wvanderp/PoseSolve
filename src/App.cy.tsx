import React from 'react';
import App from '../App';
import { useStore } from '../state/store';
// @ts-ignore
import { mount } from 'cypress/react';

// Mock worker for App component
const mockWorker = {
  solve: cy.stub().resolves({ pose: {}, intrinsics: {}, diagnostics: {} }),
};

// Mock Comlink
// @ts-ignore
global.Comlink = {
  wrap: () => mockWorker,
  releaseProxy: cy.stub(),
};

// Mock Leaflet for WorldMap
// @ts-ignore
global.L = {
  map: cy.stub().returns({
    on: cy.stub(),
    addTo: cy.stub(),
  }),
  tileLayer: cy.stub().returns({
    addTo: cy.stub(),
  }),
  layerGroup: cy.stub().returns({
    addTo: cy.stub(),
    clearLayers: cy.stub(),
  }),
  marker: cy.stub().returns({
    addTo: cy.stub(),
    on: cy.stub(),
    bindTooltip: cy.stub().returns({
      openTooltip: cy.stub(),
    }),
  }),
  Icon: {
    Default: {
      mergeOptions: cy.stub(),
    },
  },
};

describe('App Integration - Complete Workflow', () => {
  beforeEach(() => {
    // Reset store to clean state
    useStore.setState({
      image: { url: 'test.png', width: 400, height: 300, name: 'test-image.png' },
      pixelPoints: [],
      worldPoints: [],
      links: [],
      activePixelId: null,
      activeWorldId: null,
    });
  });

  it('demonstrates complete workflow from start to finish', () => {
    // @ts-ignore
    cy.mount(<App />);

    // Verify initial state
    cy.contains('Status: Idle').should('be.visible');
    cy.contains('Points: 0 px, 0 world, links: 0').should('be.visible');

    // Test workflow steps
    cy.then(() => {
      const { addWorldPoint, addPixelPoint, linkPoints } = useStore.getState();
      
      // Simulate map click creating world point and auto-linked pixel point
      const worldId = addWorldPoint({ lat: 52.0, lon: 4.0 });
      const pixelId = addPixelPoint({ 
        u: 200, // image center x
        v: 150, // image center y  
        sigmaPx: 1,
        enabled: true,
        height: 0
      });
      linkPoints(pixelId, worldId);
      
      // Verify points were created and linked
      const state = useStore.getState();
      expect(state.pixelPoints).to.have.length(1);
      expect(state.worldPoints).to.have.length(1);
      expect(state.links).to.have.length(1);
      expect(state.links[0].pixelId).to.equal(pixelId);
      expect(state.links[0].worldId).to.equal(worldId);
    });

    // Verify UI updates
    cy.contains('Points: 1 px, 1 world, links: 1').should('be.visible');

    // Test cross-selection
    cy.then(() => {
      const { selectLinkedPoint } = useStore.getState();
      selectLinkedPoint('w_mfd50gzw_0', 'world');
      
      const state = useStore.getState();
      expect(state.activeWorldId).to.not.be.null;
    });

    // Test height editing functionality
    cy.then(() => {
      const { updatePixelPointHeight } = useStore.getState();
      const pixelPoint = useStore.getState().pixelPoints[0];
      
      updatePixelPointHeight(pixelPoint.id, 15.5);
      
      const updatedState = useStore.getState();
      const updatedPoint = updatedState.pixelPoints.find(p => p.id === pixelPoint.id);
      expect(updatedPoint?.height).to.equal(15.5);
    });

    // Verify solve button is available
    cy.get('button').contains('Solve').should('be.visible');
  });

  it('handles multiple point pairs with proper linking', () => {
    // @ts-ignore  
    cy.mount(<App />);

    cy.then(() => {
      const { addWorldPoint, addPixelPoint, linkPoints } = useStore.getState();
      
      // Add multiple point pairs
      const pairs = [
        { world: { lat: 52.0, lon: 4.0 }, pixel: { u: 100, v: 100 } },
        { world: { lat: 52.1, lon: 4.1 }, pixel: { u: 200, v: 200 } },
        { world: { lat: 52.2, lon: 4.2 }, pixel: { u: 300, v: 100 } },
      ];

      pairs.forEach(pair => {
        const worldId = addWorldPoint(pair.world);
        const pixelId = addPixelPoint({ 
          ...pair.pixel, 
          sigmaPx: 1, 
          enabled: true, 
          height: Math.random() * 20  // Random height for variety
        });
        linkPoints(pixelId, worldId);
      });

      const state = useStore.getState();
      expect(state.pixelPoints).to.have.length(3);
      expect(state.worldPoints).to.have.length(3);
      expect(state.links).to.have.length(3);

      // Verify 1-to-1 linking constraint
      const pixelIds = state.links.map(l => l.pixelId);
      const worldIds = state.links.map(l => l.worldId);
      expect(new Set(pixelIds).size).to.equal(3); // All unique
      expect(new Set(worldIds).size).to.equal(3); // All unique
    });

    cy.contains('Points: 3 px, 3 world, links: 3').should('be.visible');
  });

  it('maintains data integrity during point operations', () => {
    // @ts-ignore
    cy.mount(<App />);

    cy.then(() => {
      const { 
        addWorldPoint, 
        addPixelPoint, 
        linkPoints, 
        movePixelPoint, 
        updatePixelPointHeight,
        removePixelPoint 
      } = useStore.getState();
      
      // Create and link points
      const worldId = addWorldPoint({ lat: 52.0, lon: 4.0 });
      const pixelId = addPixelPoint({ u: 100, v: 100, sigmaPx: 1, enabled: true, height: 10 });
      linkPoints(pixelId, worldId);

      // Test move operation preserves data
      movePixelPoint(pixelId, 150, 150);
      updatePixelPointHeight(pixelId, 20);

      let state = useStore.getState();
      const movedPoint = state.pixelPoints.find(p => p.id === pixelId);
      expect(movedPoint?.u).to.equal(150);
      expect(movedPoint?.v).to.equal(150);
      expect(movedPoint?.height).to.equal(20);
      expect(state.links).to.have.length(1); // Link preserved

      // Test removal cleans up links
      removePixelPoint(pixelId);
      
      state = useStore.getState();
      expect(state.pixelPoints).to.have.length(0);
      expect(state.links).to.have.length(0); // Link removed
      expect(state.worldPoints).to.have.length(1); // World point remains
    });
  });
});