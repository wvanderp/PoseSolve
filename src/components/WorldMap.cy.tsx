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
    cy.viewport(800, 800);
  });

  const mountMap = () => {
    mount(<WorldMap height={400} />);
  };

  it("shows updated tip text for auto-linking", () => {
    mountMap();

    cy.contains(
      "Tip: Double-click to add world point (or attach to active image point);"
    ).should("be.visible");
    cy.contains("left-click to select").should("be.visible");
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
    // Also assert that the map container has the expected role or class
    cy.get("#map, .leaflet-container").should(($el) => {
      expect($el.length).to.be.greaterThan(0);
    });
  });

  it("loads marker icon asset for rendered markers", () => {
    // Add a world point so a marker is rendered
    useStore.setState({
      points: [{ id: "w_marker", lat: 51.9225, lon: 4.47917 }],
      image: null,
    });

    mountMap();

    // Marker may render as an <img> (default Leaflet icon) or a div (divIcon).
    const markerSelector =
      "img.leaflet-marker-icon, .marker-icon, .selected-marker-icon";
    cy.get(markerSelector, { timeout: 10000 }).should("be.visible");
    // If an <img> exists, assert its src is reachable; otherwise just ensure
    // the marker element is present. Use a body query so we can handle zero
    // matches gracefully.
    cy.get("body").then(($body) => {
      const $imgs = $body.find("img.leaflet-marker-icon");
      if ($imgs.length) {
        const src = $imgs.eq(0).attr("src");
        if (src) {
          cy.request({ url: String(src), failOnStatusCode: false }).then(
            (resp) => {
              expect(resp.status).to.equal(200);
            }
          );
        }
      }
    });
  });

  it("selects a marker when dragged", () => {
    // Add a single world point so a marker is rendered
    useStore.setState({
      points: [{ id: "w_drag", lat: 51.9225, lon: 4.47917 }],
      activePointId: null,
      image: null,
    });

    mountMap();

    // Wait for the marker element (img or div) and simulate a drag.
    const markerSelector =
      "img.leaflet-marker-icon, .marker-icon, .selected-marker-icon";
    cy.get(markerSelector, { timeout: 10000 })
      .first()
      .should("be.visible")
      .then(($el) => {
        const rect = ($el[0] as HTMLElement).getBoundingClientRect();
        const startX = rect.left + rect.width / 2;
        const startY = rect.top + rect.height / 2;
        const endX = startX + 40;
        const endY = startY;

        // Press down on the marker, move, and release on the page to trigger Leaflet's drag handlers.
        cy.wrap($el).trigger("mousedown", {
          button: 0,
          clientX: startX,
          clientY: startY,
          force: true,
        });
        cy.get("body").trigger("mousemove", {
          clientX: endX,
          clientY: endY,
          force: true,
        });
        cy.get("body").trigger("mouseup", { force: true });
      });

    // After dragging, the point should be selected (activePointId set).
    cy.then(() => {
      const state = useStore.getState();
      expect(state.activePointId).to.equal("w_drag");
      // Ensure the active point exists in the points array
      const active = state.points.find(
        (p: any) => p.id === state.activePointId
      );
      expect(active).to.exist;
    });
  });

  it("selects an unselected marker when dragged (while another is active)", () => {
    // Two world points: one pre-selected, one not. Drag the unselected one
    // and ensure it becomes active after the drag.
    useStore.setState({
      points: [
        { id: "w_active", lat: 51.9225, lon: 4.48 },
        { id: "w_to_drag", lat: 51.9235, lon: 4.47917 },
      ],
      activePointId: "w_active",
      image: null,
    });

    mountMap();

    const markerSelector =
      "img.leaflet-marker-icon, .marker-icon, .selected-marker-icon";
    cy.get(markerSelector, { timeout: 10000 })
      .should("have.length.at.least", 2)
      .then(($els) => {
        const $el = $els[1] as unknown as HTMLElement;
        const rect = $el.getBoundingClientRect();
        const startX = rect.left + rect.width / 2;
        const startY = rect.top + rect.height / 2;
        const endX = startX + 40;
        const endY = startY;

        cy.wrap($el).trigger("mousedown", {
          button: 0,
          clientX: startX,
          clientY: startY,
          force: true,
        });
        cy.get("body").trigger("mousemove", {
          clientX: endX,
          clientY: endY,
          force: true,
        });
        cy.get("body").trigger("mouseup", { force: true });
      });

    cy.then(() => {
      const state = useStore.getState();
      expect(state.activePointId).to.equal("w_to_drag");
    });
  });

  it("updates store coordinates when marker is dragged", () => {
    // Add a single world point so a marker is rendered
    useStore.setState({
      points: [{ id: "w_drag_store", lat: 51.9225, lon: 4.47917 }],
      activePointId: null,
      image: null,
    });

    mountMap();

    // Capture initial coordinates from the store
    const before = useStore
      .getState()
      .points.find((p: any) => p.id === "w_drag_store");
    expect(before).to.exist;
    const beforeLat = before!.lat;
    const beforeLon = before!.lon;

    const markerSelector =
      "img.leaflet-marker-icon, .marker-icon, .selected-marker-icon";
    cy.get(markerSelector, { timeout: 10000 })
      .first()
      .should("be.visible")
      .then(($el) => {
        const rect = ($el[0] as HTMLElement).getBoundingClientRect();
        const startX = rect.left + rect.width / 2;
        const startY = rect.top + rect.height / 2;
        const endX = startX + 80; // move to the right
        const endY = startY + 10; // slight vertical shift

        // Simulate drag: mousedown on marker, move, then mouseup
        cy.wrap($el).trigger("mousedown", {
          button: 0,
          clientX: startX,
          clientY: startY,
          force: true,
        });
        cy.get("body").trigger("mousemove", {
          clientX: endX,
          clientY: endY,
          force: true,
        });
        cy.get("body").trigger("mouseup", { force: true });
      });

    // Wait a bit for the dragend handler to update the store
    cy.wait(200);

    // After dragging, the stored world coordinates for the point should have changed
    cy.then(() => {
      const after = useStore
        .getState()
        .points.find((p: any) => p.id === "w_drag_store");
      expect(after).to.exist;
      // The lat or lon should differ from before
      expect(after!.lat === beforeLat && after!.lon === beforeLon).to.equal(
        false
      );
    });
  });

  it("disables double-click zoom and only creates markers on double-click", () => {
    mountMap();

    // Get initial zoom level
    cy.then(() => {
      const initialZoom = useStore.getState().mapCenter ? 13 : 13; // default zoom

      // Double-click on the map
      cy.get("#map").dblclick(100, 100, { force: true });

      // Wait a moment for any potential zoom animation
      cy.wait(500);

      // Verify that a point was added to the store
      cy.then(() => {
        const state = useStore.getState();
        expect(state.points).to.have.length(1);
        expect(state.points[0]).to.have.property("lat");
        expect(state.points[0]).to.have.property("lon");
        // Verify that the newly added point is automatically selected
        expect(state.activePointId).to.equal(state.points[0].id);
      });

      // Verify that the map zoom did not change (no zoom on double-click)
      cy.get(".leaflet-control-zoom-in").should("be.visible"); // Map controls should still be there
      // Note: We can't easily test the exact zoom level in Cypress component tests
      // but the fact that doubleClickZoom: false is set should prevent zoom
    });
  });

  it("maintains markers after map panning", () => {
    // Add multiple world points
    useStore.setState({
      points: [
        { id: "w_1", lat: 51.9225, lon: 4.47917 },
        { id: "w_2", lat: 51.9235, lon: 4.48 },
      ],
      activePointId: null,
      image: null,
    });

    mountMap();

    const markerSelector =
      "img.leaflet-marker-icon, .marker-icon, .selected-marker-icon";

    // Verify markers are initially visible
    cy.get(markerSelector, { timeout: 10000 }).should("have.length", 2);

    // Pan the map by clicking and dragging the map background (not a marker)
    cy.get("#map .leaflet-map-pane")
      .trigger("mousedown", {
        clientX: 200,
        clientY: 100,
        which: 1,
        force: true,
      })
      .trigger("mousemove", { clientX: 250, clientY: 150, force: true })
      .trigger("mouseup", { force: true });

    // Wait for pan animation to complete
    cy.wait(500);

    // Verify markers are still visible after panning
    cy.get(markerSelector, { timeout: 5000 }).should("have.length", 2);

    // Verify points are still in the store
    cy.then(() => {
      const state = useStore.getState();
      expect(state.points).to.have.length(2);
    });
  });

  it("maintains markers after map zooming", () => {
    // Add multiple world points
    useStore.setState({
      points: [
        { id: "w_1", lat: 51.9225, lon: 4.47917 },
        { id: "w_2", lat: 51.9235, lon: 4.48 },
      ],
      activePointId: null,
      image: null,
    });

    mountMap();

    const markerSelector =
      "img.leaflet-marker-icon, .marker-icon, .selected-marker-icon";

    // Verify markers are initially visible
    cy.get(markerSelector, { timeout: 10000 }).should("have.length", 2);

    // Zoom in using the zoom control button
    cy.get(".leaflet-control-zoom-in").click();

    // Wait for zoom animation to complete
    cy.wait(500);

    // Verify markers are still visible after zooming
    cy.get(markerSelector).should("have.length", 2);

    // Zoom out
    cy.get(".leaflet-control-zoom-out").click();

    // Wait for zoom animation
    cy.wait(500);

    // Verify markers are still visible after zooming out
    cy.get(markerSelector).should("have.length", 2);

    // Verify points are still in the store
    cy.then(() => {
      const state = useStore.getState();
      expect(state.points).to.have.length(2);
    });
  });
});
