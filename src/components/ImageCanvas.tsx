import React, { useEffect, useRef, useState, useCallback } from "react";
import { useStore, selectors } from "../state/store";
import { fileToDataUrl, loadImage } from "../utils/image";

/**
 * ImageCanvas – Desired behaviour specification
 *
 * Purpose
 * - A single, self-contained interactive canvas that displays a user-selected image
 *   and allows the user to create, inspect, edit, move, delete and link pixel points
 *   while preserving precise mapping between screen coordinates and image (pixel)
 *   coordinates.
 *
 * High-level contract (inputs / outputs)
 * - Inputs: image metadata (url, width, height, name) from app state; user mouse/touch
 *   and keyboard interactions; active world point id from app state.
 * - Outputs: mutations to app state via provided store functions: add/move/remove
 *   pixel points, update pixel point height, link/unlink points, and select events.
 * - Error modes: invalid image, load failure, or out-of-bounds interactions must be
 *   handled gracefully (no throws; UI informs user where appropriate).
 *
 * Coordinate systems & transforms (developer guidance)
 * - Maintain three clear coordinate spaces: image pixels (u,v), base canvas-drawing
 *   coordinates (image scaled to fixed canvas height -> baseScale), and screen/canvas
 *   pixels (after zoom and pan). Transform math must be explicit and well-documented:
 *     image(u,v) -> base coords (u*baseScale, v*baseScale) -> screen coords
 *     screen = base * zoom + panOffset
 * - All hit-testing, dragging, and geometry decisions must be done in canvas pixel
 *   space (accounting for CSS scaling via getBoundingClientRect + canvas.width/height
 *   scale factors). Never rely on clientX/Y alone.
 *
 * Rendering rules
 * - Image:
 *   - Fit to a fixed canvas height (component prop `height`) while preserving aspect
 *     ratio. Canvas width is image.width * baseScale. Canvas DOM width/height must
 *     match the drawing buffer size used for transforms.
 *   - When no image is loaded, show a visible placeholder DOM element (not just
 *     pixels) so automated tests can find it.
 * - Points (pixel points):
 *   - Draw markers in screen space so their on-screen radius remains constant when
 *     zooming (i.e., draw using screen coords after applying zoom+pan but do not
 *     scale the marker size with zoom).
 *   - Selected marker style is visually distinct (stroke/fill/outline and larger
 *     stroke width). Unselected markers use a subdued style.
 *   - If a point has a numeric height, render a small text label near the marker.
 *
 * Interaction model (mouse/keyboard/touch)
 * - Add point: double-click (or double-tap) on the image area adds a new pixel
 *   point at the corresponding image pixel coordinates (u,v). If the clicked
 *   location is outside the image bounds, ignore the action.
 * - Select point: single left-click on a marker selects it. Selection must:
 *   - update local UI highlight
 *   - call store selectLinkedPoint(id, 'pixel') so the rest of the app knows the
 *     selection
 * - Drag point: click-dragging an existing point moves it. While dragging:
 *   - convert pointer to image coords and update store via movePixelPoint(id,u,v)
 *   - enforce boundary constraints (0 <= u <= image.width, 0 <= v <= image.height)
 *   - if pointer leaves bounds, stop updating position but allow cancellation and
 *     re-entry to resume drag (UX choice; must not place point outside image).
 * - Pan: click-drag on empty canvas region pans the image (no modifier key required).
 *   - Pan is in screen pixels and clamped so the image cannot be dragged fully out
 *     of view. If the image is smaller than the canvas in a dimension, it should be
 *     centered along that axis.
 * - Zoom: mouse wheel over canvas zooms in/out centered at the cursor position.
 *   - Use smooth multiplicative increments (e.g. *1.1 / *0.9), constrained to a
 *     sensible zoom range (e.g. 0.1x–10x).
 *   - When zooming, adjust panOffset so the zoom focal point (mouse) remains fixed
 *     on screen. Clamp pan after each zoom step.
 *
 * - Keyboard:
 *   - Escape cancels inline edits (height editing) and active drags.
 *   - Enter commits inline edits.
 *   - Keyboard handling should be optional and unobtrusive; if not implemented,
 *     document it as a follow-up.
 * - Touch: basic support should allow pan (single finger drag), pinch-to-zoom,
 *   and double-tap to add point. If full touch support is not implemented, the
 *   desktop interactions must degrade gracefully.
 *
 * Hit-testing & UX details
 * - Use a slightly larger hit radius for pointer interactions than visual marker
 *   radius to improve usability (e.g., marker radius 5px, hit radius 8px).
 * - Prioritize point hit-testing over pan when clicking near a marker.
 * - Pointer capture semantics: while dragging a point, continue to receive move/up
 *   events even if the cursor leaves the canvas element.
 *
 * Linking / app integration
 * - When a new pixel point is created and a world point is active, automatically
 *   call linkPoints(pixelId, activeWorldId).
 * - Selecting a pixel point must also invoke selectLinkedPoint(pixelId, 'pixel')
 *   so linked UI and maps can synchronize selection state.
 *
 * Inline editing (height overlay)
 * - When a point is selected, show a small absolute-positioned overlay near the
 *   marker that displays the point's height and allows inline editing.
 * - The overlay must be positioned in DOM pixels (screen coords) and track pan/zoom
 *   changes so it remains adjacent to the marker.
 * - Editing flow: click to edit -> input appears focused -> Enter or blur commits ->
 *   Escape cancels.
 *
 * Constraints and edge cases
 * - Never create or persist a point outside the image pixel bounds.
 * - If the image fails to load, clear any dependent UI state and display a clear
 *   placeholder and/or error state.
 * - Ensure the canvas redraw is robust to rapid changes in store state (debounce
 *   only where necessary; prefer immediate redraws for correctness in tests).
 *
 * Performance & quality
 * - Minimize full layout thrashing: use canvas drawing and keep transforms on the
 *   drawing context rather than relying on CSS transforms for image + markers.
 * - Keep per-frame work small; avoid expensive allocations inside the render loop.
 * - Provide exhaustive unit and integration tests for: hit-testing, add/move/remove
 *   semantics, zoom/pan math, overlay positioning, and linking behavior.
 *
 * Accessibility & testing hooks
 * - Provide DOM hooks (data-testid attributes) for the file input, canvas, placeholder,
 *   zoom-level text, reset buttons, delete button, and selected-point indicator so
 *   tests can assert visible state.
 * - Ensure visual feedback (selected state) has sufficient contrast.
 *
 * Acceptance criteria (happy paths)
 * - Uploading an image displays it at the correct aspect ratio and canvas height.
 * - Double-click on the image adds a point at the correct image pixel coordinates.
 * - Clicking a point selects it, highlights it, and exposes the inline height editor.
 * - Dragging a selected point moves it within image bounds and updates the store.
 * - Right-clicking a point removes it and clears selection if it was selected.
 * - Wheel zoom keeps the cursor fixed on the same image pixel and clamps zoom/pan.
 * - All above behaviors are covered by automated tests and manual UI checks.
 *
 * Notes / future improvements (documented but optional)
 * - Consider adding undo/redo stack, multi-select, snapping, and keyboard shortcuts
 *   as follow-ups. Implement touch gestures and accessibility improvements as a
 *   prioritized next step.
 */

type Props = { height?: number };

export default function ImageCanvas({ height = 520 }: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [imgEl, setImgEl] = useState<HTMLImageElement | null>(null);
  // view state: move persistent values into the store
  const zoom = useStore((s) => s.zoom);
  const panOffset = useStore((s) => s.panOffset);
  const editingHeightId = useStore((s) => s.editingHeightId);
  const setZoom = useStore((s) => s.setZoom);
  const setPanOffset = useStore((s) => s.setPanOffset);
  const setEditingHeight = useStore((s) => s.setEditingHeight);

  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [dragMode, setDragMode] = useState<"pan" | "point" | null>(null);
  const [dragPointId, setDragPointId] = useState<string | null>(null);
  // Local selectedPointId mirrors store.activePointId so UI and tests stay in sync
  const [selectedPointId, setSelectedPointId] = useState<string | null>(null);

  const [heightValue, setHeightValue] = useState("");
  const [loadError, setLoadError] = useState<string | null>(null);

  // Store state
  const image = useStore((state) => state.image);
  const points = useStore((state) => state.points);
  const activePointId = useStore((state) => state.activePointId);

  // keep local selected id in sync with store
  useEffect(() => setSelectedPointId(activePointId), [activePointId]);
  const addPoint = useStore((state) => state.addPoint);
  const updatePointImage = useStore((state) => state.updatePointImage);
  const removePoint = useStore((state) => state.removePoint);
  const updatePointHeight = useStore((state) => state.updatePointHeight);
  const selectLinkedPoint = useStore((state) => state.selectLinkedPoint);
  const setImage = useStore((state) => state.setImage);

  // Constants
  const MARKER_RADIUS = 5;
  const HIT_RADIUS = 8;
  const MIN_ZOOM = 0.1;
  const MAX_ZOOM = 50;
  const ZOOM_FACTOR = 1.1;
  const CANVAS_WIDTH = 600;
  const CANVAS_HEIGHT = height;

  // Calculate base scale and canvas dimensions
  const baseScale = image ? CANVAS_HEIGHT / image.height : 1;

  // Coordinate transformation functions
  const imageToScreen = useCallback(
    (u: number, v: number) => {
      return {
        x: u * baseScale * zoom + panOffset.x,
        y: v * baseScale * zoom + panOffset.y,
      };
    },
    [baseScale, zoom, panOffset]
  );

  const screenToImage = useCallback(
    (screenX: number, screenY: number) => {
      return {
        u: (screenX - panOffset.x) / (baseScale * zoom),
        v: (screenY - panOffset.y) / (baseScale * zoom),
      };
    },
    [baseScale, zoom, panOffset]
  );

  // Pan and zoom helpers
  const centerPan = useCallback(
    (zoomLevel: number) => {
      if (!image) return { x: 0, y: 0 };
      const canvas = canvasRef.current;
      if (!canvas) return { x: 0, y: 0 };

      const scaledWidth = image.width * baseScale * zoomLevel;
      const scaledHeight = image.height * baseScale * zoomLevel;

      return {
        x: (canvas.width - scaledWidth) / 2,
        y: (canvas.height - scaledHeight) / 2,
      };
    },
    [image, baseScale]
  );

  const clampPan = useCallback(
    (pan: { x: number; y: number }, zoomLevel: number) => {
      if (!image) return pan;
      const canvas = canvasRef.current;
      if (!canvas) return pan;

      const scaledWidth = image.width * baseScale * zoomLevel;
      const scaledHeight = image.height * baseScale * zoomLevel;

      // Allow some margin but prevent image from being dragged completely out of view
      const margin = 50;
      const minX = Math.min(
        margin - scaledWidth,
        (canvas.width - scaledWidth) / 2
      );
      const maxX = Math.max(
        canvas.width - margin,
        (canvas.width - scaledWidth) / 2
      );
      const minY = Math.min(
        margin - scaledHeight,
        (canvas.height - scaledHeight) / 2
      );
      const maxY = Math.max(
        canvas.height - margin,
        (canvas.height - scaledHeight) / 2
      );

      return {
        x: Math.max(minX, Math.min(maxX, pan.x)),
        y: Math.max(minY, Math.min(maxY, pan.y)),
      };
    },
    [image, baseScale]
  );

  // Get canvas mouse position accounting for CSS scaling
  const getCanvasMousePos = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const canvas = canvasRef.current;
      if (!canvas) return { x: 0, y: 0 };

      const rect = canvas.getBoundingClientRect();
      const scaleX = canvas.width / rect.width;
      const scaleY = canvas.height / rect.height;

      return {
        x: (e.clientX - rect.left) * scaleX,
        y: (e.clientY - rect.top) * scaleY,
      };
    },
    []
  );

  // Get canvas position from raw client coordinates (used by native wheel listener)
  const getCanvasClientPos = useCallback((clientX: number, clientY: number) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };

    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;

    return {
      x: (clientX - rect.left) * scaleX,
      y: (clientY - rect.top) * scaleY,
    };
  }, []);

  // Hit testing for image points
  const imagePoints = points.filter(
    (p) => typeof p.u === "number" && typeof p.v === "number"
  );
  const hitTestPoint = useCallback(
    (screenPos: { x: number; y: number }) => {
      for (const point of imagePoints) {
        const pointScreen = imageToScreen(point.u as number, point.v as number);
        const distance = Math.sqrt(
          Math.pow(screenPos.x - pointScreen.x, 2) +
            Math.pow(screenPos.y - pointScreen.y, 2)
        );
        if (distance <= HIT_RADIUS) {
          return point.id;
        }
      }
      return null;
    },
    [imagePoints, imageToScreen]
  );

  // File upload handler
  const handleUpload = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      try {
        setLoadError(null);
        const dataUrl = await fileToDataUrl(file);
        const img = await loadImage(dataUrl);

        // Update store with new image info
        setImage({
          url: dataUrl,
          width: img.naturalWidth,
          height: img.naturalHeight,
          name: file.name,
        });

        setImgEl(img);

        // Reset view
        setZoom(1);
        setPanOffset(centerPan(1));
        setSelectedPointId(null);
        setEditingHeight(null);
      } catch (error) {
        console.error("Failed to load image:", error);
        setLoadError("Failed to load image. Please try a different file.");
        setImgEl(null);
        setImage(null);
      }
    },
    [setImage, centerPan]
  );

  // Mouse event handlers
  const onDoubleClick = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (!image) return;

      const canvasPos = getCanvasMousePos(e);
      const imagePos = screenToImage(canvasPos.x, canvasPos.y);

      // Check if click is within image bounds
      if (
        imagePos.u < 0 ||
        imagePos.u > image.width ||
        imagePos.v < 0 ||
        imagePos.v > image.height
      ) {
        return;
      }

      // If there's an active point that has world coords but missing image coords,
      // update it instead of creating a new point (unified Point model).
      const activePt = points.find((p) => p.id === activePointId);
      if (
        activePt &&
        typeof activePt.lat === "number" &&
        typeof activePt.lon === "number" &&
        (typeof activePt.u !== "number" || typeof activePt.v !== "number")
      ) {
        updatePointImage(activePt.id, imagePos.u, imagePos.v);
        setSelectedPointId(activePt.id);
        selectLinkedPoint(activePt.id, "pixel");
      } else {
        const pointId = addPoint({ u: imagePos.u, v: imagePos.v });
        setSelectedPointId(pointId);
        selectLinkedPoint(pointId, "pixel");
      }
    },
    [
      image,
      getCanvasMousePos,
      screenToImage,
      addPoint,
      activePointId,
      updatePointImage,
      selectLinkedPoint,
      points,
    ]
  );

  const onCanvasDown = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (!image) return;

      const canvasPos = getCanvasMousePos(e);
      const hitPointId = hitTestPoint(canvasPos);

      if (e.button === 0) {
        // Left click
        if (hitPointId) {
          // Start dragging point
          setDragMode("point");
          setDragPointId(hitPointId);
          setSelectedPointId(hitPointId);
          selectLinkedPoint(hitPointId, "pixel");

          // Note: Pointer capture would require PointerEvent instead of MouseEvent
          // For now, we'll rely on onMouseLeave to handle cleanup
        } else {
          // Start panning
          setDragMode("pan");
          setSelectedPointId(null);
        }

        setIsDragging(true);
        setDragStart(canvasPos);
      }
    },
    [image, getCanvasMousePos, hitTestPoint, selectLinkedPoint]
  );

  const onCanvasMove = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (!isDragging || !image) return;

      const canvasPos = getCanvasMousePos(e);

      if (dragMode === "point" && dragPointId) {
        const imagePos = screenToImage(canvasPos.x, canvasPos.y);

        // Clamp to image bounds
        const clampedU = Math.max(0, Math.min(image.width, imagePos.u));
        const clampedV = Math.max(0, Math.min(image.height, imagePos.v));

        updatePointImage(dragPointId, clampedU, clampedV);
      } else if (dragMode === "pan") {
        const deltaX = canvasPos.x - dragStart.x;
        const deltaY = canvasPos.y - dragStart.y;

        const newPan = {
          x: panOffset.x + deltaX,
          y: panOffset.y + deltaY,
        };

        setPanOffset(clampPan(newPan, zoom));
        setDragStart(canvasPos);
      }
    },
    [
      isDragging,
      image,
      getCanvasMousePos,
      dragMode,
      dragPointId,
      screenToImage,
      updatePointImage,
      dragStart,
      panOffset,
      clampPan,
      zoom,
    ]
  );

  const onCanvasUp = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    setIsDragging(false);
    setDragMode(null);
    setDragPointId(null);
  }, []);

  // Right-click deletion removed intentionally to avoid accidental removals.
  // Previously this component removed points on contextmenu; that behavior
  // is no longer supported. The browser context menu is allowed.

  // Native wheel handler attached with passive: false so preventDefault is allowed
  const onWheelNative = useCallback(
    (e: WheelEvent) => {
      // allow preventDefault to stop page scrolling when using the canvas
      e.preventDefault();

      if (!image) return;

      const canvasPos = getCanvasClientPos(e.clientX, e.clientY);
      const zoomDelta = e.deltaY > 0 ? 1 / ZOOM_FACTOR : ZOOM_FACTOR;
      const newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoom * zoomDelta));

      if (newZoom === zoom) return;

      // Adjust pan to keep zoom focal point fixed
      const scaleFactor = newZoom / zoom;
      const newPan = {
        x: canvasPos.x - (canvasPos.x - panOffset.x) * scaleFactor,
        y: canvasPos.y - (canvasPos.y - panOffset.y) * scaleFactor,
      };

      setZoom(newZoom);
      setPanOffset(clampPan(newPan, newZoom));
    },
    [image, getCanvasClientPos, zoom, panOffset, clampPan]
  );

  // Attach native wheel listener with { passive: false } to allow preventDefault
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    canvas.addEventListener("wheel", onWheelNative, { passive: false });
    return () =>
      canvas.removeEventListener("wheel", onWheelNative as EventListener);
  }, [onWheelNative]);

  // Keyboard handler
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setEditingHeight(null);
        setIsDragging(false);
        setDragMode(null);
        setDragPointId(null);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Canvas rendering effect
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    // Set canvas size
    canvas.width = Math.round(CANVAS_WIDTH);
    canvas.height = Math.round(CANVAS_HEIGHT);

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw image if loaded
    if (imgEl && image) {
      const imageScreenPos = imageToScreen(0, 0);
      const imageScreenWidth = image.width * baseScale * zoom;
      const imageScreenHeight = image.height * baseScale * zoom;

      ctx.drawImage(
        imgEl,
        imageScreenPos.x,
        imageScreenPos.y,
        imageScreenWidth,
        imageScreenHeight
      );
    }

    // Draw points
    imagePoints.forEach((point) => {
      const screenPos = imageToScreen(point.u as number, point.v as number);
      const isSelected = point.id === selectedPointId;

      ctx.beginPath();
      ctx.arc(screenPos.x, screenPos.y, MARKER_RADIUS, 0, 2 * Math.PI);

      if (isSelected) {
        ctx.fillStyle = "#ff6b6b";
        ctx.strokeStyle = "#fff";
        ctx.lineWidth = 3;
      } else {
        // default (unselected) color expected by tests is teal (#00d1b2)
        ctx.fillStyle = "#00d1b2";
        ctx.strokeStyle = "#fff";
        ctx.lineWidth = 2;
      }

      ctx.fill();
      ctx.stroke();

      // Draw height label if point has height
      if (point.height && point.height > 0) {
        ctx.fillStyle = "#333";
        ctx.font = "12px sans-serif";
        ctx.fillText(
          `${point.height}m`,
          screenPos.x + MARKER_RADIUS + 2,
          screenPos.y - MARKER_RADIUS - 2
        );
      }
    });
  }, [
    imgEl,
    image,
    CANVAS_WIDTH,
    CANVAS_HEIGHT,
    imageToScreen,
    baseScale,
    zoom,
    imagePoints,
    selectedPointId,
  ]);

  // Load initial image effect
  useEffect(() => {
    if (image?.url && (!imgEl || imgEl.src !== image.url)) {
      loadImage(image.url)
        .then((img) => {
          setImgEl(img);
          setLoadError(null);
          // Center image on initial load
          if (zoom === 1) {
            setPanOffset(centerPan(1));
          }
        })
        .catch((error) => {
          console.error("Failed to load image:", error);
          setLoadError("Failed to load image from URL.");
          setImgEl(null);
        });
    }
  }, [image?.url, imgEl, zoom, centerPan]);

  return (
    <div>
      <div
        style={{
          marginBottom: 8,
          display: "flex",
          gap: 8,
          alignItems: "center",
        }}
      >
        <input
          type="file"
          accept="image/*"
          onChange={handleUpload}
          data-testid="file-input"
        />
        {image && (
          <span style={{ color: "#666" }}>
            {image.name} — {image.width}×{image.height}
          </span>
        )}
        {loadError && <span style={{ color: "#ff6b6b" }}>{loadError}</span>}
        <div
          style={{
            marginLeft: "auto",
            display: "flex",
            gap: 8,
            alignItems: "center",
          }}
        >
          <button
            onClick={() => {
              // reset zoom to 1 and center image
              setZoom(1);
              setPanOffset(clampPan(centerPan(1), 1));
            }}
            data-testid="reset-zoom"
            style={{ padding: "4px 8px" }}
          >
            Reset Zoom
          </button>
          <button
            onClick={() => setPanOffset(clampPan(centerPan(zoom), zoom))}
            data-testid="reset-pan"
            style={{ padding: "4px 8px" }}
          >
            Reset Pan
          </button>
          {selectedPointId && (
            <button
              onClick={() => {
                removePoint(selectedPointId);
                setSelectedPointId(null);
              }}
              data-testid="delete-point"
              style={{
                padding: "4px 8px",
                backgroundColor: "#ff6b6b",
                color: "white",
                border: "none",
                borderRadius: "4px",
              }}
            >
              Delete Point
            </button>
          )}
          <span data-testid="zoom-level" style={{ color: "#666" }}>
            Zoom: {Math.round(zoom * 100)}%
          </span>
        </div>
      </div>
      <div
        style={{
          position: "relative",
          display: "inline-block",
          width: CANVAS_WIDTH,
        }}
      >
        <canvas
          ref={canvasRef}
          width={Math.round(CANVAS_WIDTH)}
          height={Math.round(CANVAS_HEIGHT)}
          onDoubleClick={onDoubleClick}
          onMouseDown={onCanvasDown}
          onMouseMove={onCanvasMove}
          onMouseUp={onCanvasUp}
          onMouseLeave={onCanvasUp}
          data-testid="canvas"
          style={{
            border: "1px solid #333",
            borderRadius: 8,
            width: `${CANVAS_WIDTH}px`,
            height: `${CANVAS_HEIGHT}px`,
            display: "block",
          }}
        />
        {/* Height editing overlay */}
        {selectedPointId &&
          (() => {
            const selectedPoint = points.find(
              (p) =>
                p.id === selectedPointId &&
                typeof p.u === "number" &&
                typeof p.v === "number"
            );
            if (!selectedPoint || !image) return null;

            const screenPos = imageToScreen(
              selectedPoint.u as number,
              selectedPoint.v as number
            );
            const overlayX = screenPos.x + 10;
            const overlayY = screenPos.y - 10;

            return (
              <div
                style={{
                  position: "absolute",
                  left: overlayX,
                  top: overlayY,
                  background: "rgba(255, 255, 255, 0.9)",
                  padding: "4px 8px",
                  borderRadius: "4px",
                  border: "1px solid #ccc",
                  fontSize: "12px",
                  display: "flex",
                  alignItems: "center",
                  gap: "4px",
                  zIndex: 10,
                }}
              >
                <span>Height:</span>
                {editingHeightId === selectedPointId ? (
                  <input
                    type="number"
                    value={heightValue}
                    onChange={(e) => setHeightValue(e.target.value)}
                    onBlur={() => {
                      const numValue = parseFloat(heightValue) || 0;
                      updatePointHeight(selectedPointId, numValue);
                      setEditingHeight(null);
                    }}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        const numValue = parseFloat(heightValue) || 0;
                        updatePointHeight(selectedPointId, numValue);
                        setEditingHeight(null);
                      } else if (e.key === "Escape") {
                        setEditingHeight(null);
                      }
                    }}
                    autoFocus
                    style={{ width: "50px", padding: "2px" }}
                  />
                ) : (
                  <span
                    style={{ cursor: "pointer", textDecoration: "underline" }}
                    onClick={() => {
                      setEditingHeight(selectedPointId);
                      setHeightValue((selectedPoint.height || 0).toString());
                    }}
                  >
                    {selectedPoint.height || 0}m
                  </span>
                )}
              </div>
            );
          })()}
        {/* Visible DOM placeholder so tests can find the text (canvas pixels aren't searchable) */}
        {(!imgEl || !image) && (
          <div
            data-testid="placeholder"
            style={{
              position: "absolute",
              left: 12,
              top: 12,
              color: "#999",
              pointerEvents: "none",
            }}
          >
            {loadError ? "Failed to load image" : "Upload an image to begin"}
          </div>
        )}
      </div>
      <p style={{ color: "#777", marginTop: 6 }}>
        Tip: Double-click to add points; left-click to select; drag to move or
        pan. Mouse wheel to zoom.
        {selectedPointId && (
          <span data-testid="selected-point"> Selected: {selectedPointId}</span>
        )}
      </p>
    </div>
  );
}
