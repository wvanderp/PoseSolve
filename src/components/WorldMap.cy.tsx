import React from "react";
import WorldMap from "./WorldMap";
import { useStore } from "../state/store";
import { mount } from "cypress/react";

describe("WorldMap (component)", () => {
  beforeEach(() => {
    // Reset store to a minimal state using the new unified store shape
    useStore.setState({
      points: [],
      activePointId: null,
      image: null,
    });
  });

  const mountMap = () => {
    mount(<WorldMap height={200} />);
  };

  it("shows updated tip text for auto-linking", () => {
    mountMap();

    cy.contains(
      "Click to add world point and auto-link to image center"
    ).should("be.visible");
    cy.contains("Selecting points shows cross-selection").should("be.visible");
  });

  it("integrates with store for auto-linking", () => {
    // Set up store with image for auto-linking
    useStore.setState({
      image: { url: "test.png", width: 200, height: 200, name: "test.png" },
      points: [],
    });

    mountMap();

    // Verify that the component has access to the required store functions
    cy.then(() => {
      const {
        setImage,
        addPoint,
        updatePointImage,
        updatePointWorld,
        selectLinkedPoint,
      } = useStore.getState();
      expect(setImage).to.be.a("function");
      expect(addPoint).to.be.a("function");
      expect(updatePointImage).to.be.a("function");
      expect(updatePointWorld).to.be.a("function");
      expect(selectLinkedPoint).to.be.a("function");
    });
  });

  it("handles cross-selection functionality", () => {
    // Set up store with linked points
    const worldPoint = { id: "w_1", lat: 52.0, lon: 4.0 };
    const pixelPoint = {
      id: "px_1",
      u: 100,
      v: 100,
      sigmaPx: 1,
      enabled: true,
      height: 0,
    };
    // The new store keeps a single points array. Links are not stored separately in this model.
    useStore.setState({
      points: [worldPoint, pixelPoint],
      activePointId: null,
      image: { url: "test.png", width: 200, height: 200, name: "test.png" },
    });

    mountMap();

    // Verify store has cross-selection function
    cy.then(() => {
      const { selectLinkedPoint } = useStore.getState();
      expect(selectLinkedPoint).to.be.a("function");

      // Test selection: in the unified store selecting a point makes it active
      selectLinkedPoint("w_1", "world");

      const state = useStore.getState();
      expect(state.activePointId).to.equal("w_1");
    });
  });

  it("renders map container", () => {
    mountMap();

    // Should render the map container div
    cy.get("div").should("exist");
  });

  it("loads marker icon asset for rendered markers", () => {
    // Add a world point so a marker is rendered
    useStore.setState({
      points: [{ id: "w_marker", lat: 51.9225, lon: 4.47917 }],
      image: null,
    });

    mountMap();

    // Leaflet renders an <img> element with class 'leaflet-marker-icon'
    cy.get("img.leaflet-marker-icon", { timeout: 10000 }).should("be.visible");
    cy.get("img.leaflet-marker-icon")
      .first()
      .should("have.attr", "src")
      .then((src) => {
        const url = String(src);
        // Request the image URL to ensure the asset is served (status 200)
        // Use failOnStatusCode=false to capture the status and assert explicitly
        cy.request({ url, failOnStatusCode: false }).then((resp) => {
          expect(resp.status).to.equal(200);
        });
      });
  });
});
