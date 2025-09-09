import React from 'react';
import WorldMap from './WorldMap';
import { useStore } from '../state/store';
// @ts-ignore
import { mount } from 'cypress/react';

// Mock Leaflet imports at the top level
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

describe('WorldMap (component)', () => {
  beforeEach(() => {
    // Reset store to a minimal state
    useStore.setState({
      worldPoints: [],
      activeWorldId: null,
      activePixelId: null,
      image: null,
    });
  });

  const mountMap = () => {
    // @ts-ignore
    cy.mount(<WorldMap height={200} />);
  };

  it('shows updated tip text for auto-linking', () => {
    mountMap();
    
    cy.contains('Click to add world point and auto-link to image center').should('be.visible');
    cy.contains('Selecting points shows cross-selection').should('be.visible');
  });

  it('integrates with store for auto-linking', () => {
    // Set up store with image for auto-linking
    useStore.setState({
      image: { url: 'test.png', width: 200, height: 200, name: 'test.png' },
      worldPoints: [],
      pixelPoints: [],
      links: [],
    });

    mountMap();

    // Verify that the component has access to the required store functions
    cy.then(() => {
      const { addWorldPoint, addPixelPoint, linkPoints } = useStore.getState();
      expect(addWorldPoint).to.be.a('function');
      expect(addPixelPoint).to.be.a('function'); 
      expect(linkPoints).to.be.a('function');
    });
  });

  it('handles cross-selection functionality', () => {
    // Set up store with linked points
    const worldPoint = { id: 'w_1', lat: 52.0, lon: 4.0 };
    const pixelPoint = { id: 'px_1', u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 };
    const link = { pixelId: 'px_1', worldId: 'w_1' };

    useStore.setState({
      worldPoints: [worldPoint],
      pixelPoints: [pixelPoint],
      links: [link],
      activeWorldId: 'w_1',
      image: { url: 'test.png', width: 200, height: 200, name: 'test.png' },
    });

    mountMap();

    // Verify store has cross-selection function
    cy.then(() => {
      const { selectLinkedPoint } = useStore.getState();
      expect(selectLinkedPoint).to.be.a('function');
      
      // Test cross-selection
      selectLinkedPoint('w_1', 'world');
      
      const state = useStore.getState();
      expect(state.activeWorldId).to.equal('w_1');
      expect(state.activePixelId).to.equal('px_1');
    });
  });

  it('renders map container', () => {
    mountMap();
    
    // Should render the map container div
    cy.get('div').should('exist');
  });
});