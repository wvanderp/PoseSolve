import React from "react";
import App from "./App";
import { useStore } from "./state/store";
import { mount } from "cypress/react";

// Mock worker for App component
// Avoid calling Cypress commands at module scope (causes "outside a running test" errors).
const mockWorker = {
  solve: async () => ({ pose: {}, intrinsics: {}, diagnostics: {} }),
};

describe("App Integration - Complete Workflow", () => {
  beforeEach(() => {
    // Reset store to clean state
    useStore.setState({
      image: {
        url: "test.png",
        width: 400,
        height: 300,
        name: "test-image.png",
      },
      points: [],
      activePointId: null,
    });
  });

  it("demonstrates complete workflow from start to finish", () => {
    cy.mount(<App />);

    // Verify initial state
    cy.contains("Status: Idle").should("be.visible");
    cy.contains("Points: 0 px, 0 world, linked: 0").should("be.visible");

    // Test workflow steps using the unified store API
    cy.then(() => {
      const { addPoint, selectors } = useStore.getState();

      // Create a single point that has both image and world coordinates (represents a linked point)
      const id = addPoint({
        lat: 52.0,
        lon: 4.0,
        u: 200,
        v: 150,
        sigmaPx: 1,
        enabled: true,
        height: 0,
      });

      // Verify points were created and linked (i.e. one point with both image & world fields)
      const state = useStore.getState();
      const imagePoints = selectors.getImagePoints(state);
      const worldPoints = selectors.getWorldPoints(state);
      const linkedPoints = state.points.filter(
        (p) =>
          typeof p.u === "number" &&
          typeof p.v === "number" &&
          typeof p.lat === "number" &&
          typeof p.lon === "number"
      );

      expect(imagePoints).to.have.length(1);
      expect(worldPoints).to.have.length(1);
      expect(linkedPoints).to.have.length(1);
      expect(linkedPoints[0].id).to.equal(id);
    });

    // Verify UI updates
    cy.contains("Points: 1 px, 1 world, linked: 1").should("be.visible");

    // Test cross-selection
    cy.then(() => {
      const { selectLinkedPoint } = useStore.getState();
      const id = useStore.getState().points[0].id;
      selectLinkedPoint(id, "world");

      const state = useStore.getState();
      expect(state.activePointId).to.not.be.null;
    });

    // Test height editing functionality
    cy.then(() => {
      const { updatePointHeight } = useStore.getState();
      const pt = useStore.getState().points[0];

      updatePointHeight(pt.id, 15.5);

      const updatedState = useStore.getState();
      const updatedPoint = updatedState.points.find((p: any) => p.id === pt.id);
      expect(updatedPoint?.height).to.equal(15.5);
    });

    // Verify solve button is available
    cy.get("button").contains("Solve").should("be.visible");
  });

  it("handles multiple point pairs with proper linking", () => {
    cy.mount(<App />);

    cy.then(() => {
      const { addPoint, selectors } = useStore.getState();

      // Add multiple linked points (each has both world and pixel fields)
      const pairs = [
        { world: { lat: 52.0, lon: 4.0 }, pixel: { u: 100, v: 100 } },
        { world: { lat: 52.1, lon: 4.1 }, pixel: { u: 200, v: 200 } },
        { world: { lat: 52.2, lon: 4.2 }, pixel: { u: 300, v: 100 } },
      ];

      pairs.forEach((pair) => {
        addPoint({
          ...pair.world,
          ...pair.pixel,
          sigmaPx: 1,
          enabled: true,
          height: Math.random() * 20,
        });
      });

      const state = useStore.getState();
      const imagePoints = selectors.getImagePoints(state);
      const worldPoints = selectors.getWorldPoints(state);
      const linkedPoints = state.points.filter(
        (p) =>
          typeof p.u === "number" &&
          typeof p.v === "number" &&
          typeof p.lat === "number" &&
          typeof p.lon === "number"
      );

      expect(state.points).to.have.length(3);
      expect(imagePoints).to.have.length(3);
      expect(worldPoints).to.have.length(3);
      expect(linkedPoints).to.have.length(3);

      // Verify 1-to-1 uniqueness: each point is its own linked pair
      const ids = state.points.map((p: any) => p.id);
      expect(new Set(ids).size).to.equal(3);
    });

    cy.contains("Points: 3 px, 3 world, linked: 3").should("be.visible");
  });

  it("maintains data integrity during point operations", () => {
    cy.mount(<App />);

    cy.then(() => {
      const { addPoint, updatePointImage, updatePointHeight, removePoint } =
        useStore.getState();

      // Create a single linked point (image + world on one object)
      const id = addPoint({
        u: 100,
        v: 100,
        lat: 52.0,
        lon: 4.0,
        sigmaPx: 1,
        enabled: true,
        height: 10,
      });

      // Test update operation preserves data on the single point
      updatePointImage(id, 150, 150);
      updatePointHeight(id, 20);

      let state = useStore.getState();
      const movedPoint = state.points.find((p: any) => p.id === id);
      expect(movedPoint?.u).to.equal(150);
      expect(movedPoint?.v).to.equal(150);
      expect(movedPoint?.height).to.equal(20);
      expect(state.points).to.have.length(1);

      // Test removal deletes the unified point
      removePoint(id);

      state = useStore.getState();
      expect(state.points).to.have.length(0);
    });
  });
});
