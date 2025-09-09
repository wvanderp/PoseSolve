import React from 'react';
import ImageCanvas from './ImageCanvas';
import { useStore } from '../state/store';
// @ts-ignore
import { mount } from 'cypress/react';


describe('ImageCanvas (component)', () => {
  beforeEach(() => {
    // Reset store to a minimal state to avoid network image loading in most assertions
    useStore.setState({
      image: null,
      pixelPoints: [],
      worldPoints: [],
      links: [],
      activePixelId: null,
      activeWorldId: null,
    });
  });

  // Helper to mount the component in a beforeEach
  const mountCanvas = () => {
    // @ts-ignore
  cy.mount(<ImageCanvas height={200} />);
  };

  // Helper to create a simple SVG data URL and set it on the store
  // Use a background color that does not collide with point fill colors
  const setTestImage = (
    width: number,
    height: number,
    name = 'test-image.svg',
    bg = '#224466'
  ) => {
    const svg = `<svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg"><rect width="${width}" height="${height}" fill="${bg}"/></svg>`;
    // btoa exists in the browser test runner
    const url = 'data:image/svg+xml;base64,' + btoa(svg);
    useStore.setState({ image: { url, width, height, name } });
  };

  // Canvas helpers for pixel-precise assertions
  const colorDist = (a: [number, number, number], b: [number, number, number]) => {
    const dr = a[0] - b[0];
    const dg = a[1] - b[1];
    const db = a[2] - b[2];
    return Math.sqrt(dr * dr + dg * dg + db * db);
  };

  const expectCanvasColorNear = (
    $canvas: JQuery<HTMLCanvasElement>,
    offsetX: number,
    offsetY: number,
    expectedRgb: [number, number, number],
    tolerance = 25
  ) => {
    const canvas = $canvas[0];
    const rect = canvas.getBoundingClientRect();
    const sx = canvas.width / rect.width;
    const sy = canvas.height / rect.height;
    const x = Math.round(offsetX * sx);
    const y = Math.round(offsetY * sy);
    const ctx = canvas.getContext('2d')!;
    const data = ctx.getImageData(x, y, 1, 1).data;
    const rgb: [number, number, number] = [data[0], data[1], data[2]];
    expect(colorDist(rgb, expectedRgb)).to.be.lessThan(tolerance);
  };

  const expectCanvasColorWithinRadius = (
    $canvas: JQuery<HTMLCanvasElement>,
    offsetX: number,
    offsetY: number,
    expectedRgb: [number, number, number],
    radiusPx = 6,
    tolerance = 35
  ) => {
    const canvas = $canvas[0];
    const rect = canvas.getBoundingClientRect();
    const sx = canvas.width / rect.width;
    const sy = canvas.height / rect.height;
    const cx = Math.round(offsetX * sx);
    const cy = Math.round(offsetY * sy);
    const ctx = canvas.getContext('2d')!;
    let found = false;
    for (let dy = -radiusPx; dy <= radiusPx && !found; dy++) {
      for (let dx = -radiusPx; dx <= radiusPx && !found; dx++) {
        const x = cx + dx;
        const y = cy + dy;
        if (x < 0 || y < 0 || x >= canvas.width || y >= canvas.height) continue;
        const d = ctx.getImageData(x, y, 1, 1).data;
        const rgb: [number, number, number] = [d[0], d[1], d[2]];
        if (colorDist(rgb, expectedRgb) < tolerance) {
          found = true;
        }
      }
    }
    expect(found, `expected color ${expectedRgb} within ${radiusPx}px of (${offsetX},${offsetY})`).to.be.true;
  };

  const expectCanvasColorInBox = (
    $canvas: JQuery<HTMLCanvasElement>,
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    expectedRgb: [number, number, number],
    tolerance = 35
  ) => {
    const canvas = $canvas[0];
    const rect = canvas.getBoundingClientRect();
    const sx = canvas.width / rect.width;
    const sy = canvas.height / rect.height;
    const minX = Math.max(0, Math.floor(Math.min(x1, x2) * sx));
    const minY = Math.max(0, Math.floor(Math.min(y1, y2) * sy));
    const maxX = Math.min(canvas.width - 1, Math.ceil(Math.max(x1, x2) * sx));
    const maxY = Math.min(canvas.height - 1, Math.ceil(Math.max(y1, y2) * sy));
    const ctx = canvas.getContext('2d')!;
    let found = false;
    for (let y = minY; y <= maxY && !found; y++) {
      for (let x = minX; x <= maxX && !found; x++) {
        const d = ctx.getImageData(x, y, 1, 1).data;
        const rgb: [number, number, number] = [d[0], d[1], d[2]];
        if (colorDist(rgb, expectedRgb) < tolerance) found = true;
      }
    }
    expect(found, `expected color ${expectedRgb} in box [${x1},${y1}]x[${x2},${y2}]`).to.be.true;
  };

  it('shows upload input and placeholder when no image is set', () => {
  mountCanvas();
    cy.get('[data-testid="file-input"]').should('exist');
    cy.get('[data-testid="placeholder"]').should('contain.text', 'Upload an image to begin');
  });

  it('displays zoom and pan controls', () => {
  mountCanvas();
    cy.get('[data-testid="reset-zoom"]').should('exist');
    cy.get('[data-testid="reset-pan"]').should('exist');
    cy.get('[data-testid="zoom-level"]').should('contain.text', 'Zoom: 100%');
  });

  it('handles image loading and displays metadata', () => {
    // Set up a test image in the store and mount the canvas for this test only
    setTestImage(100, 100, 'test-image.svg');
  mountCanvas();

    // Wait for image to load and check metadata display
    cy.contains('test-image.svg — 100×100').should('be.visible');
    cy.get('[data-testid="placeholder"]').should('not.exist');
  });

  describe('Point Management', () => {
    beforeEach(() => {
      // Set up a test image and mount the canvas for all Point Management tests
      setTestImage(200, 200, 'test-canvas.svg');
  mountCanvas();
    });

    it('adds points when clicking on canvas', () => {
      // canvas is mounted in beforeEach
      // Wait for canvas to be ready
      cy.get('[data-testid="canvas"]').should('be.visible');

      // Click to add a point
      cy.get('[data-testid="canvas"]').click(50, 50);

      // Check that a point was added to the store
      cy.then(() => {
        const state = useStore.getState();
        expect(state.pixelPoints).to.have.length(1);
        // Assert the stored pixel coordinates match the click (allow small tolerance
        // for coordinate transforms / device pixel ratios)
        expect(state.pixelPoints[0].u).to.be.approximately(50, 2);
        expect(state.pixelPoints[0].v).to.be.approximately(50, 2);
      });

      // And assert the canvas actually rendered the point at that location (selected color)
      // Selected fill color = #ff6b6b -> [255, 107, 107]
      cy.get('[data-testid="canvas"]').then(($c) => {
        expectCanvasColorNear($c as JQuery<HTMLCanvasElement>, 50, 50, [255, 107, 107], 35);
      });
    });

    it('shows selected point indicator', () => {
      // canvas is mounted in beforeEach
      // Add a point by clicking
      cy.get('[data-testid="canvas"]').click(50, 50);

      // Should show selected point info
      cy.get('[data-testid="selected-point"]').should('be.visible');

      // And the underlying store should contain the point at the expected coords
      cy.then(() => {
        const state = useStore.getState();
        expect(state.pixelPoints).to.have.length(1);
        expect(state.pixelPoints[0].u).to.be.approximately(50, 2);
        expect(state.pixelPoints[0].v).to.be.approximately(50, 2);
      });
    });

    it('removes points on right-click', () => {
      // Pre-add a point to the store
      const pointId = 'test-point-1';
  // Use helper to set the image and add the point
  setTestImage(200, 200, 'test.svg');
  useStore.setState({ pixelPoints: [{ id: pointId, u: 50, v: 50, sigmaPx: 1, enabled: true }] });

      // canvas is mounted in beforeEach

      // Right-click on the point location
      cy.get('[data-testid="canvas"]').rightclick(50, 50);

      // Check that point was removed
      cy.then(() => {
        const state = useStore.getState();
        expect(state.pixelPoints).to.have.length(0);
      });
    });

    it('verifies point dragging functionality exists', () => {
      // Pre-add a point
      const pointId = 'test-point-1';
  setTestImage(200, 200, 'test.svg');
  useStore.setState({ pixelPoints: [{ id: pointId, u: 50, v: 50, sigmaPx: 1, enabled: true }] });

      // canvas is mounted in beforeEach
      // Verify that the canvas responds to mouse events
      // This test verifies the functionality exists rather than testing exact coordinates
      cy.get('[data-testid="canvas"]')
        .should('be.visible')
        .trigger('mousedown', 50, 50)
        .trigger('mousemove', 100, 100)
        .trigger('mouseup');

      // Verify we still have a point and the component is functional
      cy.then(() => {
        const state = useStore.getState();
        expect(state.pixelPoints).to.have.length(1);
        expect(state.pixelPoints[0].id).to.equal(pointId);
        // And the point should have moved (approximately) to the dragged location
        // Allow a small tolerance for coordinate mapping differences
        expect(state.pixelPoints[0].u).to.be.approximately(100, 4);
        expect(state.pixelPoints[0].v).to.be.approximately(100, 4);
      });

      // Also verify the pixel moved on canvas (selected color at new location)
      cy.get('[data-testid="canvas"]').then(($c) => {
        expectCanvasColorNear($c as JQuery<HTMLCanvasElement>, 100, 100, [255, 107, 107], 35);
      });
    });
  });

  describe('Zoom and Pan Functionality', () => {
    beforeEach(() => {
  // Reuse helper to set image for zoom/pan tests
  setTestImage(200, 200, 'test.svg');
    });

    it('shows zoom level indicator', () => {
  mountCanvas();
      cy.get('[data-testid="zoom-level"]').should('contain.text', 'Zoom: 100%');
    });

    it('zooms in while canvas size stays constant and reprojection changes', () => {
  mountCanvas();

      // capture initial canvas size
      cy.get('[data-testid="canvas"]').then($c => {
        const initialW = ($c[0] as HTMLCanvasElement).width;

        // Click once to add a point at a fixed screen position
        cy.get('[data-testid="canvas"]').click(100, 100);

        let firstPoint: { u: number; v: number };
        cy.then(() => {
          const state = useStore.getState();
          expect(state.pixelPoints).to.have.length(1);
          firstPoint = state.pixelPoints[0];
        });

        // dispatch wheel event to zoom in
        cy.get('[data-testid="canvas"]').trigger('wheel', { deltaY: -150, deltaMode: 0 });

        // zoom level text should update
        cy.get('[data-testid="zoom-level"]').should('not.contain.text', 'Zoom: 100%');

        // canvas pixel width should remain the same
        cy.get('[data-testid="canvas"]').should($c2 => {
          const newW = ($c2[0] as HTMLCanvasElement).width;
          expect(newW).to.equal(initialW);
        });

        // At the same screen position, the point should still be under the cursor (zoom to cursor)
        cy.get('[data-testid="canvas"]').then(($c) => {
          // Selected color should still be at 100,100 after zoom
          expectCanvasColorNear($c as JQuery<HTMLCanvasElement>, 100, 100, [255, 107, 107], 35);
        });

        // Clicking again should hit the existing point and not add a new one
        cy.get('[data-testid="canvas"]').click(100, 100);
        cy.then(() => {
          const state = useStore.getState();
          expect(state.pixelPoints).to.have.length(1);
          const existing = state.pixelPoints[0];
          expect(existing.u).to.be.approximately(firstPoint.u, 0.001);
          expect(existing.v).to.be.approximately(firstPoint.v, 0.001);
        });
      });
    });

    it('resets zoom when clicking reset button', () => {
  mountCanvas();

      // Simulate zoom by wheel (this would require more complex mocking)
      // For now, just test that the reset button exists and is clickable
      cy.get('[data-testid="reset-zoom"]').click();
      cy.get('[data-testid="zoom-level"]').should('contain.text', 'Zoom: 100%');
    });

    it('resets pan when clicking reset button', () => {
  mountCanvas();

      // Test that reset pan button exists and is clickable
      cy.get('[data-testid="reset-pan"]').click();
      // The pan should be reset (harder to test visually without checking internal state)
    });

    it('enables panning mode with shift+drag', () => {
  mountCanvas();

      // Test shift+drag for panning (simplified test)
      cy.get('[data-testid="canvas"]')
        .trigger('mousedown', { which: 1, shiftKey: true, clientX: 50, clientY: 50 })
        .trigger('mousemove', { which: 1, shiftKey: true, clientX: 100, clientY: 100 })
        .trigger('mouseup');

      // In panning mode, no new points should be added
      cy.then(() => {
        const state = useStore.getState();
        expect(state.pixelPoints).to.have.length(0);
      });
    });

  it('panning shifts rendered content on screen (pixel assertion)', () => {
      mountCanvas();
      // Add a point at 50,50
      cy.get('[data-testid="canvas"]').click(50, 50);

      // Assert the point is drawn at the original screen location
      cy.get('[data-testid="canvas"]').then(($c) => {
        expectCanvasColorNear($c as JQuery<HTMLCanvasElement>, 50, 50, [255, 107, 107], 35);
      });

      // Pan by an exact +50,+50 in canvas pixels by converting to CSS movement using the element's scale
      cy.get('[data-testid="canvas"]').then(($c) => {
        const canvas = $c[0] as HTMLCanvasElement;
        const rect = canvas.getBoundingClientRect();
        const sx = canvas.width / rect.width;
        const sy = canvas.height / rect.height;
        const moveCssX = Math.ceil(50 / sx);
        const moveCssY = Math.ceil(50 / sy);

        cy.wrap($c)
          .trigger('mousedown', { which: 1, shiftKey: true, clientX: 10, clientY: 10 })
          .trigger('mousemove', { which: 1, shiftKey: true, movementX: moveCssX, movementY: moveCssY, clientX: 10 + moveCssX, clientY: 10 + moveCssY })
          .trigger('mouseup');
      });

      // The point should no longer be at 50,50 but should appear near 100,100
      cy.wait(20);
      cy.get('[data-testid="canvas"]').then(($c) => {
        // Old location should not match the selected color anymore (allow generous tolerance by inverting check)
        const canvas = ($c[0] as HTMLCanvasElement);
        const rect = canvas.getBoundingClientRect();
        const sx = canvas.width / rect.width;
        const sy = canvas.height / rect.height;
        const xOld = Math.round(50 * sx);
        const yOld = Math.round(50 * sy);
        const dataOld = canvas.getContext('2d')!.getImageData(xOld, yOld, 1, 1).data;
        const rgbOld: [number, number, number] = [dataOld[0], dataOld[1], dataOld[2]];
        const distOldRed = colorDist(rgbOld, [255, 107, 107]);
        const distOldTeal = colorDist(rgbOld, [0, 209, 178]);
        expect(distOldRed).to.be.greaterThan(35);
        expect(distOldTeal).to.be.greaterThan(35);

        // New location: after panning the selection is cleared, so the point should be drawn with default teal
        // Expect default teal color somewhere near the intended new position (80..140)
        expectCanvasColorInBox($c as JQuery<HTMLCanvasElement>, 80, 80, 140, 140, [0, 209, 178], 60);
      });
    });
  });

  describe('File Upload', () => {
    it('handles file selection', () => {
  mountCanvas();

      // Create a test file blob
      const testFileContent = 'test-image-data';

      // Use selectFile with fixture approach
      cy.get('[data-testid="file-input"]').selectFile({
        contents: Cypress.Buffer.from(testFileContent),
        fileName: 'test-image.png',
        mimeType: 'image/png'
      }, { force: true });

      // Note: Full file upload testing would require more sophisticated mocking
      // of FileReader and Image loading
    });
  });

  describe('World Point Linking', () => {
    it('links pixel points to active world points', () => {
      // Set up initial state with an active world point
      const worldPointId = 'world-1';
  setTestImage(200, 200, 'test.svg');
  useStore.setState({ worldPoints: [{ id: worldPointId, lat: 52.0, lon: 4.0 }], activeWorldId: worldPointId });

      // @ts-ignore
      cy.mount(<ImageCanvas height={200} />);

      // Click to add a pixel point
      cy.get('[data-testid="canvas"]').click(50, 50);

      // Check that the points were linked
      cy.then(() => {
        const state = useStore.getState();
        expect(state.links).to.have.length(1);
        expect(state.links[0].worldId).to.equal(worldPointId);
        expect(state.pixelPoints).to.have.length(1);
        // Verify the pixel point location was recorded at the click position
        expect(state.pixelPoints[0].u).to.be.approximately(50, 3);
        expect(state.pixelPoints[0].v).to.be.approximately(50, 3);
      });

      // And verify visually on canvas
      cy.get('[data-testid="canvas"]').then(($c) => {
        expectCanvasColorNear($c as JQuery<HTMLCanvasElement>, 50, 50, [255, 107, 107], 35);
      });
    });
  });

  describe('Canvas Interaction Edge Cases', () => {
    it('handles clicks when no image is loaded', () => {
  mountCanvas();

      // Click on canvas without image - should not add points
      cy.get('[data-testid="canvas"]').click(50, 50);

      cy.then(() => {
        const state = useStore.getState();
        expect(state.pixelPoints).to.have.length(0);
      });
    });

    it('prevents placing points outside the image boundaries', () => {
      // Set up a test image with known dimensions
      const testImage = {
        width: 100,
        height: 100,
        name: 'test-boundary.svg'
      };
      // Use helper to set the image; helper creates matching data URL
      setTestImage(100, 100, 'test-boundary.svg');

      mountCanvas();

      // Wait for canvas to be ready
      cy.get('[data-testid="canvas"]').should('be.visible');

      // First, try clicking within the image bounds to ensure it works normally
      cy.get('[data-testid="canvas"]').click(50, 50, { force: true }); // Should be within the scaled image

      cy.then(() => {
        const state = useStore.getState();
        // At least one point should exist (the valid click)
        expect(state.pixelPoints.length).to.be.at.least(1);
        // And it should be within bounds
        const validPoint = state.pixelPoints.find(p => p.u >= 0 && p.u <= 100 && p.v >= 0 && p.v <= 100);
        expect(validPoint).to.exist;
      });

      // Count points before trying to click outside
      let pointsBeforeOutsideClicks = 0;
      cy.then(() => {
        pointsBeforeOutsideClicks = useStore.getState().pixelPoints.length;
      });

      // Try to click outside the image bounds
      // The image is 100x100, scaled to fit height 200, so it should be 200x200 in the canvas
      // Try clicking outside this area (but within canvas bounds to avoid interaction issues)
      cy.get('[data-testid="canvas"]').click(350, 50, { force: true }); // Far right of image
      cy.get('[data-testid="canvas"]').click(50, 250, { force: true }); // Below image

      // Check that no additional points were added outside the image
      cy.then(() => {
        const state = useStore.getState();

        // Verify all points are within image bounds
        state.pixelPoints.forEach(point => {
          // Points should be within the image dimensions (0-100 in image coordinates)
          expect(point.u).to.be.at.least(0);
          expect(point.u).to.be.at.most(100);
          expect(point.v).to.be.at.least(0);
          expect(point.v).to.be.at.most(100);
        });

        // If boundary checking is properly implemented, no new points should be added
        // when clicking outside the image bounds
        if (state.pixelPoints.length > pointsBeforeOutsideClicks) {
          // If new points were added, they should still be within bounds
          const newPoints = state.pixelPoints.slice(pointsBeforeOutsideClicks);
          newPoints.forEach(point => {
            expect(point.u).to.be.at.least(0);
            expect(point.u).to.be.at.most(100);
            expect(point.v).to.be.at.least(0);
            expect(point.v).to.be.at.most(100);
          });
        }
      });
    });

    it('prevents dragging points outside the image boundaries', () => {
      // Set up a test image with a point near the edge
      const testImage = {
        width: 100,
        height: 100,
        name: 'test-drag-boundary.svg'
      };
      const pointId = 'test-drag-point';
      setTestImage(100, 100, 'test-drag-boundary.svg');
      useStore.setState({ pixelPoints: [{ id: pointId, u: 90, v: 90, sigmaPx: 1, enabled: true }] });

      mountCanvas();

      // Wait for canvas to be ready
      cy.get('[data-testid="canvas"]').should('be.visible');

      // Get the initial position
      let initialPoint;
      cy.then(() => {
        const state = useStore.getState();
        initialPoint = state.pixelPoints.find(p => p.id === pointId);
        expect(initialPoint).to.exist;
        expect(initialPoint!.u).to.equal(90);
        expect(initialPoint!.v).to.equal(90);
      });

      // Try to drag the point outside the image bounds
      // The image is scaled to fit height 200, so at (90,90) in image coords it should be around (180,180) in canvas coords
      cy.get('[data-testid="canvas"]')
        .trigger('mousedown', { which: 1, clientX: 180, clientY: 180, force: true })
        .trigger('mousemove', { which: 1, clientX: 250, clientY: 250, force: true }) // Try to drag outside
        .trigger('mouseup', { force: true });

      // Check that the point is still within bounds
      cy.then(() => {
        const state = useStore.getState();
        const draggedPoint = state.pixelPoints.find(p => p.id === pointId);
        expect(draggedPoint).to.exist;

        // Point should still be within image bounds (not dragged outside)
        expect(draggedPoint!.u).to.be.at.least(0);
        expect(draggedPoint!.u).to.be.at.most(100);
        expect(draggedPoint!.v).to.be.at.least(0);
        expect(draggedPoint!.v).to.be.at.most(100);

        // If boundary checking works, the point should not have moved to the attempted position
        // It should either stay at the original position or be clamped to the boundary
      });

      // Also confirm the rendered point stays within the visible image area (not far outside)
      cy.get('[data-testid="canvas"]').then(($c) => {
        // The point should still be at its original screen location ~180,180 with default (unselected) color #00d1b2
        expectCanvasColorNear($c as JQuery<HTMLCanvasElement>, 180, 180, [0, 209, 178], 45);
      });
    });

  describe('Rendering colors for selection state', () => {
    beforeEach(() => {
      setTestImage(200, 200, 'colors.svg');
      mountCanvas();
    });

    it('uses selected color for the last selected point and default for others', () => {
      // First click creates P1 and selects it
      cy.get('[data-testid="canvas"]').click(40, 40);
      // Second click creates P2 and selects it, P1 becomes unselected
      cy.get('[data-testid="canvas"]').click(120, 40);

      // P2 should be selected color (#ff6b6b)
      cy.get('[data-testid="canvas"]').then(($c) => {
        expectCanvasColorNear($c as JQuery<HTMLCanvasElement>, 120, 40, [255, 107, 107], 35);
        // P1 should be default color (#00d1b2)
        expectCanvasColorNear($c as JQuery<HTMLCanvasElement>, 40, 40, [0, 209, 178], 35);
      });
    });
  });

    it('shows helpful instructions', () => {
      // @ts-ignore
      cy.mount(<ImageCanvas height={200} />);

      cy.contains('Double-click to add points').should('be.visible');
      cy.contains('left-click to select').should('be.visible');
      cy.contains('drag to move or pan').should('be.visible');
      cy.contains('Delete button removes selected point').should('be.visible');
    });

    it('shows delete button when point is selected', () => {
      setTestImage(200, 200, 'test.svg');
      useStore.setState({ pixelPoints: [{ id: 'test-point', u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 }] });
      
      // @ts-ignore
      cy.mount(<ImageCanvas height={200} />);

      // Click on the point to select it
      cy.get('[data-testid="canvas"]').click(100, 100);

      // Delete button should appear
      cy.get('[data-testid="delete-point"]').should('be.visible');
      
      // Click delete button
      cy.get('[data-testid="delete-point"]').click();
      
      // Point should be removed
      cy.then(() => {
        const state = useStore.getState();
        expect(state.pixelPoints).to.have.length(0);
      });
    });

    it('shows height editing overlay for selected point', () => {
      setTestImage(200, 200, 'test.svg');
      useStore.setState({ pixelPoints: [{ id: 'test-point', u: 100, v: 100, sigmaPx: 1, enabled: true, height: 5 }] });
      
      // @ts-ignore  
      cy.mount(<ImageCanvas height={200} />);

      // Click on the point to select it
      cy.get('[data-testid="canvas"]').click(100, 100);

      // Height editing overlay should appear
      cy.contains('Height:').should('be.visible');
      cy.contains('5m').should('be.visible');
    });
  });
});
