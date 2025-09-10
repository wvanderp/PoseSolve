# PoseSolve - User Guide for New Features

## New Interaction Model

### Image Canvas
- **Double-click** to add new pixel points
- **Left-click** on existing points to select them
- **Left-drag** to pan the image or move selected points
- **Mouse wheel** to zoom (up to 10x magnification)
- **Delete button** appears when a point is selected
- **Height editing** overlay allows setting elevation values

### World Map
- **Single-click** on the map to add a world point
- **Automatic linking** creates a pixel point at the image center and links them
- **Cross-selection** between map and image points
- **Drag** markers to reposition world points
- **Right-click** markers to delete them

### Point Management
- Points now have **height values** that can be edited via the overlay
- **Cross-selection** shows linked points across both components
- **1-to-1 linking** constraint ensures clean correspondences
- **Visual feedback** with different colors for selected/unselected points

## Workflow

1. **Load an image** (or use the default Rotterdam skyline)
2. **Click on the map** where you know a world location - this automatically:
   - Creates a world point at that lat/lon
   - Creates a pixel point at the image center
   - Links them together
3. **Move the pixel point** to the correct location on the image by dragging
4. **Edit the height** if needed by clicking on the height value in the overlay
5. **Repeat** for additional correspondence points
6. **Cross-select** by clicking points to see their links
7. **Run the solver** when you have enough correspondences

## Testing

The application includes comprehensive tests:

- **Unit tests** for store functionality (`store.test.ts`)
- **Component tests** for UI interactions (`*.cy.tsx` files)
- **Integration tests** for cross-component behavior

Run tests with:
```bash
npm run ct:run  # Cypress component tests
```

## Constraints and Improvements

- **Pan constraints** prevent going past image edges
- **Zoom constraints** allow 0.1x to 10x magnification
- **Boundary checking** prevents placing points outside the image
- **Double-click detection** with movement tolerance prevents accidental points
- **Height persistence** maintains elevation values during point operations