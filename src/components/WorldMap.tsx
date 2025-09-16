/**
 * WorldMap Component User Behavior Documentation
 *
 * This component provides an interactive Leaflet map for managing world coordinate points.
 * The map integrates with the global state store for point management and cross-selection
 * with image points.
 *
 * User Interactions:
 * 1. **Double-click on map**: Adds a new marker at the clicked location
 *    - Creates both world coordinates (lat/lon) and image coordinates
 *    - Sets the new point as active
 *    - Updates the store with the new point
 *    - This action replaces the default Leaflet zoom-in behavior on double-click
 *
 * 2. **Left-click on marker**: Selects the marker and makes its point active
 *    - Updates activePointId in the store
 *    - Enables cross-selection with corresponding image points
 *
 * 3. **Drag marker**: Moves the marker to a new location
 *    - dragging starts on mousedown on the marker and then moving the mouse
 *    - Updates the marker's lat/lon coordinates in the store
 *    - Automatically selects the dragged marker
 *    - Provides visual feedback during drag operation
 *
 * 4. **Map movement**: Panning or zooming
 *   - Most of the default Leaflet interactions are preserved
 *   - If the map is dragged then the map and marker positions update accordingly
 *        - this is default Leaflet behavior
 *        - if the marker is grabbed then the map will not move
 *    - zooming in/out with mouse wheel or controls is preserved
 *
 * 5. **Visual feedback**:
 *    - Selected markers have different styling (selected-marker-icon class)
 *    - Unselected markers use standard styling (marker-icon class)
 *    - Tip text explains available interactions
 *
 * Integration:
 * - Syncs with store for point data, active selection, and map center
 * - Supports auto-linking between image and world coordinates
 * - Updates map center in store when map is moved
 */

import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import "leaflet/dist/leaflet.css";
import L from "leaflet";
import {
  MapContainer,
  TileLayer,
  Marker,
  useMapEvents,
  useMap,
} from "react-leaflet";
import { useStore } from "../state/store";

interface Props {
  height?: number;
}

// Component to handle map events (double-click to add points and moveend to update center)
function MapEventHandler() {
  const { addPoint, setActivePoint, setMapCenter, mapCenter } = useStore();

  const map = useMapEvents({
    dblclick: (e) => {
      const { lat, lng } = e.latlng;
      const pointId = addPoint({ lat, lon: lng });
      setActivePoint(pointId);
    },
  });

  return null;
}

// Draggable marker component for world points
function WorldPointMarker({ point }: { point: any }) {
  const { activePointId, setActivePoint, updatePointWorld } = useStore();
  const [dragging, setDragging] = useState(false);
  const markerRef = useRef<L.Marker>(null);
  const map = useMap();

  const isActive = point.id === activePointId;

  // Create custom icon with different styles for active/inactive state
  const customIcon = useMemo(() => {
    const className = isActive ? "selected-marker-icon" : "marker-icon";
    return L.divIcon({
      className: className,
      html: "<div></div>",
      iconSize: [20, 20],
      iconAnchor: [10, 10],
      popupAnchor: [0, -10],
    });
  }, [isActive]);

  const eventHandlers = useMemo(
    () => ({
      click: () => {
        setActivePoint(point.id);
      },
      mousedown: () => {
        setActivePoint(point.id);
      },
      dragstart: (e: any) => {
        setActivePoint(point.id);
        setDragging(true);
        // Disable map dragging while dragging marker
        map.dragging.disable();
      },
      drag: (e: any) => {
        // Keep marker selected during drag and update coordinates in real-time
        setActivePoint(point.id);
        const newLatLng = e.target.getLatLng();
        updatePointWorld(point.id, newLatLng.lat, newLatLng.lng);
      },
      dragend: (e: any) => {
        const newLatLng = e.target.getLatLng();
        updatePointWorld(point.id, newLatLng.lat, newLatLng.lng);
        setDragging(false);
        // Re-enable map dragging after marker drag ends
        map.dragging.enable();
      },
    }),
    [point.id, setActivePoint, updatePointWorld, map]
  );

  if (typeof point.lat !== "number" || typeof point.lon !== "number") {
    return null;
  }

  return (
    <Marker
      position={[point.lat, point.lon]}
      draggable={true}
      eventHandlers={eventHandlers}
      ref={markerRef}
      icon={customIcon}
      key={`${point.id}-${point.lat}-${point.lon}`}
    />
  );
}

export default function WorldMap({ height = 500 }: Props) {
  const { points, mapCenter, setMapCenter, selectors } = useStore();

  // Get markers from the store
  const worldPoints = selectors.getWorldPoints({ points });

  // Default center (Rotterdam area as suggested by tests)
  const defaultCenter: [number, number] = [51.9225, 4.47917];
  const center = mapCenter
    ? ([mapCenter.lat, mapCenter.lon] as [number, number])
    : defaultCenter;

  // Set initial map center in store if not set
  useEffect(() => {
    if (!mapCenter) {
      setMapCenter(center[0], center[1]);
    }
  }, [mapCenter, setMapCenter, center]);

  return (
    <div className="flex flex-col h-full">
      <MapContainer
        center={center}
        zoom={13}
        style={{ height: `${height}px`, width: "800px" }}
        className="w-full"
        doubleClickZoom={false}
        id="map"
      >
        <TileLayer
          attribution="© OpenStreetMap contributors"
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />

        <MapEventHandler />

        {worldPoints.map((point) => (
          <WorldPointMarker key={point.id} point={point} />
        ))}
      </MapContainer>

      <div className="p-2 text-xs text-gray-600 bg-gray-50 border-t">
        <p>
          Tip: Double-click to add world point (or attach to active image
          point); left-click to select
        </p>
      </div>
    </div>
  );
}
