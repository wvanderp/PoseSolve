import React from "react";
import Toolbar from "./Toolbar";
import { useStore } from "../state/store";
import { mount } from "cypress/react";

describe("Toolbar (component)", () => {
  beforeEach(() => {
    // reset minimal store state used by toolbar
    useStore.setState({
      points: [],
      activePointId: null,
      image: null,
    });
  });

  it("renders status and counts and shows disabled delete when no active point", () => {
    const counts = { image: 0, world: 0, linked: 0 };
    const onDelete = cy.stub().as("onDelete");
    const onSolve = cy.stub().as("onSolve");

    cy.mount(
      <Toolbar
        activePointId={null}
        onDelete={onDelete}
        onSolve={onSolve}
        status={"Idle"}
        counts={counts}
      />
    );

    cy.contains("Status: Idle").should("exist");
    cy.contains("Points: 0 px, 0 world, linked: 0").should("exist");
    cy.get('[data-testid="delete-point"]').should("be.disabled");
  });

  it("enables delete when active point exists and calls handler", () => {
    const counts = { image: 1, world: 0, linked: 0 };
    const onDelete = cy.stub().as("onDelete");
    const onSolve = cy.stub().as("onSolve");

    // simulate a selected point in the store
    useStore.setState({
      points: [{ id: "p1", u: 10, v: 20 }],
      activePointId: "p1",
    });

    cy.mount(
      <Toolbar
        activePointId={useStore.getState().activePointId}
        onDelete={onDelete}
        onSolve={onSolve}
        status={"Ready"}
        counts={counts}
      />
    );

    cy.get('[data-testid="delete-point"]').should("not.be.disabled").click();
    cy.get("@onDelete").should("have.been.calledOnce");
  });

  it("calls solve handler when Solve button clicked", () => {
    const counts = { image: 2, world: 1, linked: 1 };
    const onDelete = cy.stub().as("onDelete");
    const onSolve = cy.stub().as("onSolve");

    cy.mount(
      <Toolbar
        activePointId={null}
        onDelete={onDelete}
        onSolve={onSolve}
        status={"Ready"}
        counts={counts}
      />
    );

    // Solve button should be present and enabled
    cy.contains("Solve").should("be.visible").and("not.be.disabled").click();
    cy.get("@onSolve").should("have.been.calledOnce");

    // Update counts prop and ensure UI updates accordingly
    const newCounts = { image: 3, world: 2, linked: 2 };
    cy.mount(
      <Toolbar
        activePointId={null}
        onDelete={onDelete}
        onSolve={onSolve}
        status={"Ready"}
        counts={newCounts}
      />
    );
    cy.contains("Points: 3 px, 2 world, linked: 2").should("exist");
  });
});
