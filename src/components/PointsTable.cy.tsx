import React from "react";
import { mount } from "cypress/react";
import PointsTable from "./PointsTable";
import { useStore } from "../state/store";

describe("PointsList", () => {
  beforeEach(() => {
    // Reset store state before each test
    useStore.setState({
      points: [],
      activePointId: null,
    });
  });

  it("renders empty state when no points", () => {
    mount(<PointsTable />);
    cy.contains("No points created yet").should("be.visible");
    cy.contains("Points List").should("be.visible");
  });

  it("displays points with image and world coordinates", () => {
    // Add test points to the store
    const { addPoint, updatePointImage, updatePointWorld } =
      useStore.getState();

    const pointId1 = addPoint({ id: "test-point-1" });
    updatePointImage(pointId1, 100, 200);
    updatePointWorld(pointId1, 51.9225, 4.47917);

    const pointId2 = addPoint({ id: "test-point-2" });
    updatePointImage(pointId2, 300, 400);

    mount(<PointsTable />);

    cy.contains("Points List (2)").should("be.visible");

    // Check first point (linked)
    cy.contains("Point test-poi...").should("be.visible");
    cy.contains("u: 100.0 px").should("be.visible");
    cy.contains("v: 200.0 px").should("be.visible");
    cy.contains("lat: 51.9225°").should("be.visible");
    cy.contains("lon: 4.4792°").should("be.visible");
    cy.contains("Linked").should("be.visible");

    // Check second point (image only)
    cy.contains("u: 300.0 px").should("be.visible");
    cy.contains("v: 400.0 px").should("be.visible");
    cy.contains("Image").should("be.visible");
  });

  it("shows active point highlighting", () => {
    const { addPoint, setActivePoint } = useStore.getState();

    const pointId = addPoint({ id: "test-active-point" });
    setActivePoint(pointId);

    mount(<PointsTable />);

    cy.contains("● Active").should("be.visible");
    cy.get('[data-cy="point-item"]').should("have.class", "bg-blue-100");
  });

  it("allows clicking to select points", () => {
    const { addPoint } = useStore.getState();
    const pointId = addPoint({ id: "test-clickable-point" });

    mount(<PointsTable />);

    cy.contains("Point test-cli...").click();

    // Verify the point becomes active
    cy.then(() => {
      expect(useStore.getState().activePointId).to.equal(pointId);
    });
  });

  it("updates when store data changes", () => {
    mount(<PointsTable />);

    // Initially empty
    cy.contains("No points created yet").should("be.visible");

    // Add a point programmatically
    cy.then(() => {
      const { addPoint } = useStore.getState();
      addPoint({ id: "dynamic-point" });
    });

    // Should now show the point
    cy.contains("Points List (1)").should("be.visible");
    cy.contains("Point dynamic-...").should("be.visible");
  });
});
