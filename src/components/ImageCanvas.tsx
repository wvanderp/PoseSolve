import React, { useEffect, useRef, useState } from 'react';
import { useStore, selectors } from '../state/store';
import { fileToDataUrl, loadImage } from '../utils/image';

/**
 * ImageCanvas Component Features:
 * 
 * ✅ IMPLEMENTED & TESTED:
 * - Image upload via file input
 * - Display uploaded images with aspect ratio preservation  
 * - Add pixel points by clicking on the image
 * - Move pixel points by dragging
 * - Remove pixel points via right-click
 * - Visual representation of points with styled circles
 * - Auto-linking with active world points
 * - Image metadata display (name, dimensions)
 * - Placeholder text when no image is loaded
 * - Point selection/highlighting (visual feedback for selected points)
 * - Zoom functionality (mouse wheel zoom with reset button)
 * - Pan functionality (shift+drag to pan with reset button)
 * - Interactive UI controls for zoom/pan management
 * - Canvas interaction edge case handling
 * - Boundary checking to prevent placing/dragging points outside image bounds
 * - Comprehensive test coverage for all major features
 * 
 * ❌ POTENTIAL FUTURE ENHANCEMENTS:
 * - Keyboard shortcuts for common actions (e.g., Delete to remove selected point)
 * - Point labeling/naming functionality
 * - Multiple point selection (Ctrl+click)
 * - Undo/redo functionality 
 * - Grid overlay toggle for precise positioning
 * - Measurement tools (distance, angle calculations)
 * - Export functionality (points to JSON, image with overlays)
 * - Advanced zoom features (zoom to fit, zoom to selection)
 * - Touch gesture support for mobile devices
 * - Point snapping to grid or other points
 */

type Props = { height?: number };

export default function ImageCanvas({ height = 500 }: Props) {
    const canvasRef = useRef<HTMLCanvasElement | null>(null);
    const [imgEl, setImgEl] = useState<HTMLImageElement | null>(null);
    const [zoom, setZoom] = useState(1);
    const [panOffset, setPanOffset] = useState({ x: 0, y: 0 });
    const [isPanning, setIsPanning] = useState(false);
    const [selectedPointId, setSelectedPointId] = useState<string | null>(null);

    const image = useStore(s => s.image);
    const pixelPoints = useStore(s => s.pixelPoints);
    const setImage = useStore(s => s.setImage);
    const addPixelPoint = useStore(s => s.addPixelPoint);
    const movePixelPoint = useStore(s => s.movePixelPoint);
    const removePixelPoint = useStore(s => s.removePixelPoint);
    const linkPoints = useStore(s => s.linkPoints);
    const setActivePixel = useStore(s => s.setActivePixel);
    const activeWorldId = useStore(s => s.activeWorldId);

    // Load image element when image url changes
    useEffect(() => {
        let cancelled = false;
        if (image?.url) {
            loadImage(image.url).then((img) => {
                if (!cancelled) {
                    setImgEl(img);
                    // Update the stored image metadata with the real dimensions if they're missing or placeholder
                    if (!image.width || !image.height || image.width === 1 || image.height === 1) {
                        setImage({ url: image.url, width: img.naturalWidth, height: img.naturalHeight, name: image.name });
                    }
                }
            }).catch(() => setImgEl(null));
        } else {
            setImgEl(null);
        }
        return () => { cancelled = true; };
    }, [image?.url]);

    // Draw
    useEffect(() => {
        const c = canvasRef.current;
        if (!c) return;
        const ctx = c.getContext('2d');
        if (!ctx) return;
        ctx.clearRect(0, 0, c.width, c.height);
        if (imgEl && image) {
            // Fit image into fixed canvas height; canvas size should remain constant on zoom
            const baseScale = height / image.height;
            const canvasW = Math.round(image.width * baseScale);
            // Keep canvas dimensions fixed while zooming (crop/expand drawing instead)
            c.width = canvasW;
            c.height = height;

            ctx.save();
            // Compose transform so that: screen = zoom * (baseCoords) + panOffset
            // This keeps panOffset in screen pixels and independent of zoom factor
            ctx.setTransform(zoom, 0, 0, zoom, panOffset.x, panOffset.y);

            // Draw the image at base scale; the zoom transform scales it visually
            const drawW = image.width * baseScale;
            const drawH = image.height * baseScale;
            ctx.drawImage(imgEl, 0, 0, drawW, drawH);

            // Draw points in the same transformed space
            for (const p of pixelPoints) {
                const x = p.u * baseScale;
                const y = p.v * baseScale;
                ctx.beginPath();
                ctx.arc(x, y, 5, 0, Math.PI * 2);

                if (p.id === selectedPointId) {
                    ctx.fillStyle = '#ff6b6b';
                    ctx.strokeStyle = '#ff0000';
                    ctx.lineWidth = 3;
                } else {
                    ctx.fillStyle = '#00d1b2';
                    ctx.strokeStyle = '#013';
                    ctx.lineWidth = 2;
                }

                ctx.fill();
                ctx.stroke();
            }
            ctx.restore();
        } else {
            c.width = Math.max(400, c.parentElement?.clientWidth ?? 400);
            c.height = height;
            ctx.fillStyle = '#222';
            ctx.fillRect(0, 0, c.width, c.height);
            ctx.fillStyle = '#999';
            ctx.fillText('Upload an image to begin', 12, 20);
        }
    }, [imgEl, image, pixelPoints, height, zoom, panOffset, selectedPointId]);

    const handleUpload = async (ev: React.ChangeEvent<HTMLInputElement>) => {
        const file = ev.target.files?.[0];
        if (!file) return;
        const url = await fileToDataUrl(file);
        const img = await loadImage(url);
        setImage({ url, width: img.naturalWidth, height: img.naturalHeight, name: file.name });
    };

    // Interaction: add or drag points
    const [dragId, setDragId] = useState<string | null>(null);

    // Convert client/browser coordinates to canvas internal pixel coordinates.
    // This is necessary because the canvas DOM element may be scaled via CSS
    // (different rect.width/height vs canvas.width/height). We must operate
    // in canvas pixel space for transforms, pan, and hit-testing.
    const clientToCanvas = (clientX: number, clientY: number) => {
        const c = canvasRef.current;
        if (!c) return { x: 0, y: 0 };
        const rect = c.getBoundingClientRect();
        const scaleX = c.width / rect.width;
        const scaleY = c.height / rect.height;
        const x = (clientX - rect.left) * scaleX;
        const y = (clientY - rect.top) * scaleY;
        return { x, y };
    };

    const toImageCoords = (evt: React.MouseEvent<HTMLCanvasElement>) => {
        if (!image) return { u: 0, v: 0 };
        const c = canvasRef.current;
        if (!c) return { u: 0, v: 0 };
        const { x: cx, y: cy } = clientToCanvas(evt.clientX, evt.clientY);
        const x = (cx - panOffset.x) / zoom;
        const y = (cy - panOffset.y) / zoom;
        const baseScale = height / image.height;
        return { u: x / baseScale, v: y / baseScale };
    };

    const onCanvasDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
        if (!image) return;
        const c = canvasRef.current;
        if (!c) return;

        const { x: cx, y: cy } = clientToCanvas(e.clientX, e.clientY);
        const baseScale = height / image.height;

        // Hit test existing points in base-scaled space (canvas pixels)
        const hit = pixelPoints.find(p => {
            const px = p.u * baseScale, py = p.v * baseScale;
            const dx = px - ((cx - panOffset.x) / zoom);
            const dy = py - ((cy - panOffset.y) / zoom);
            return (dx * dx + dy * dy) <= 8 * 8;
        });

        if (hit) {
            setSelectedPointId(hit.id);
            setDragId(hit.id);
            setIsPanning(false);
            return;
        }

        // Check if holding shift for panning
        if (e.shiftKey) {
            setIsPanning(true);
            setSelectedPointId(null);
            return;
        }

        // Add new point
        const { u, v } = toImageCoords(e);

        // Check if the point is within the image boundaries
        if (u < 0 || u > image.width || v < 0 || v > image.height) {
            // Click is outside the image bounds, don't add a point
            return;
        }

        const id = addPixelPoint({ u, v, sigmaPx: 1, enabled: true });
        setSelectedPointId(id);
        setDragId(id);

        // If a world point is active, link them
        if (activeWorldId) {
            linkPoints(id, activeWorldId);
        }
    };

    const onCanvasMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
        if (!image) return;
        const c = canvasRef.current;
        if (!c) return;

        if (isPanning) {
            // movementX/Y are in client (CSS) pixels; convert to canvas pixels
            const rect = c.getBoundingClientRect();
            const scaleX = c.width / rect.width;
            const scaleY = c.height / rect.height;
            const deltaX = e.movementX * scaleX;
            const deltaY = e.movementY * scaleY;
            setPanOffset(prev => ({ x: prev.x + deltaX, y: prev.y + deltaY }));
            return;
        }

        if (dragId) {
            const { u, v } = toImageCoords(e);

            // Check if the new position is within the image boundaries
            if (u >= 0 && u <= image.width && v >= 0 && v <= image.height) {
                movePixelPoint(dragId, u, v);
            }
            // If outside bounds, don't update the point position (keep it at current position)
        }
    };

    const onCanvasUp = () => {
        setDragId(null);
        setIsPanning(false);
    };

    const onContextMenu = (e: React.MouseEvent<HTMLCanvasElement>) => {
        e.preventDefault();
        if (!image) return;
        const c = canvasRef.current;
        if (!c) return;
        const { x: cx, y: cy } = clientToCanvas(e.clientX, e.clientY);
        const baseScale = height / image.height;
        const hit = pixelPoints.find(p => {
            const px = p.u * baseScale, py = p.v * baseScale;
            const dx = px - ((cx - panOffset.x) / zoom);
            const dy = py - ((cy - panOffset.y) / zoom);
            return (dx * dx + dy * dy) <= 8 * 8;
        });
        if (hit) {
            removePixelPoint(hit.id);
            if (selectedPointId === hit.id) {
                setSelectedPointId(null);
            }
        }
    };

    const onWheel = (e: React.WheelEvent<HTMLCanvasElement>) => {
        e.preventDefault();
        const c = canvasRef.current;
        if (!c) return;

        // Determine zoom factor (in/out)
        const delta = e.deltaY > 0 ? 0.9 : 1.1;

        // Convert mouse position to canvas pixel coordinates
        const { x: mouseX, y: mouseY } = clientToCanvas(e.clientX, e.clientY);

        setZoom(prevZoom => {
            const newZoom = Math.max(0.1, Math.min(5, prevZoom * delta));
            // If zoom didn't change (clamped), do nothing to pan
            if (newZoom === prevZoom) return prevZoom;

            const scale = newZoom / prevZoom;
            // Adjust pan so that the point under the mouse stays fixed on screen
            setPanOffset(prevPan => {
                const newPanX = mouseX - scale * (mouseX - prevPan.x);
                const newPanY = mouseY - scale * (mouseY - prevPan.y);
                return { x: newPanX, y: newPanY };
            });

            return newZoom;
        });
    };

    return (
        <div>
            <div style={{ marginBottom: 8, display: 'flex', gap: 8, alignItems: 'center' }}>
                <input type="file" accept="image/*" onChange={handleUpload} data-testid="file-input" />
                {image && <span style={{ color: '#666' }}>{image.name} — {image.width}×{image.height}</span>}
                <div style={{ marginLeft: 'auto', display: 'flex', gap: 8, alignItems: 'center' }}>
                    <button
                        onClick={() => setZoom(1)}
                        data-testid="reset-zoom"
                        style={{ padding: '4px 8px' }}
                    >
                        Reset Zoom
                    </button>
                    <button
                        onClick={() => setPanOffset({ x: 0, y: 0 })}
                        data-testid="reset-pan"
                        style={{ padding: '4px 8px' }}
                    >
                        Reset Pan
                    </button>
                    <span data-testid="zoom-level" style={{ color: '#666' }}>
                        Zoom: {Math.round(zoom * 100)}%
                    </span>
                </div>
            </div>
            <div style={{ position: 'relative', display: 'inline-block', width: '100%' }}>
                <canvas
                    ref={canvasRef}
                    height={height}
                    onMouseDown={onCanvasDown}
                    onMouseMove={onCanvasMove}
                    onMouseUp={onCanvasUp}
                    onMouseLeave={onCanvasUp}
                    onContextMenu={onContextMenu}
                    onWheel={onWheel}
                    data-testid="canvas"
                    style={{ border: '1px solid #333', borderRadius: 8, maxWidth: '100%', display: 'block' }}
                />
                {/* Visible DOM placeholder so tests can find the text (canvas pixels aren't searchable) */}
                {(!imgEl || !image) && (
                    <div data-testid="placeholder" style={{ position: 'absolute', left: 12, top: 12, color: '#999', pointerEvents: 'none' }}>
                        Upload an image to begin
                    </div>
                )}
            </div>
            <p style={{ color: '#777', marginTop: 6 }}>
                Tip: Left-click to add/move; right-click a point to delete. Hold Shift and drag to pan. Mouse wheel to zoom.
                {selectedPointId && <span data-testid="selected-point"> Selected: {selectedPointId}</span>}
            </p>
        </div>
    );
}
