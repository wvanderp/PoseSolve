import React, { useEffect, useRef } from "react";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
// Fix default marker icons under Vite bundling using import.meta.url so Vite
// resolves the asset paths correctly at runtime.
const iconRetinaUrl = new URL(
  "../node_modules/leaflet/dist/images/marker-icon-2x.png",
  import.meta.url
).href;
const iconUrl = new URL(
  "../node_modules/leaflet/dist/images/marker-icon.png",
  import.meta.url
).href;
const shadowUrl = new URL(
  "../node_modules/leaflet/dist/images/marker-shadow.png",
  import.meta.url
).href;

const DefaultIcon = L.icon({
  iconRetinaUrl,
  iconUrl,
  shadowUrl,
  iconSize: [25, 41],
  iconAnchor: [12, 41],
  popupAnchor: [1, -34],
  tooltipAnchor: [16, -28],
  shadowSize: [41, 41],
});

// Apply as the default marker icon so code creating markers without an
// explicit icon gets the correct asset URLs.
L.Marker.prototype.options.icon = DefaultIcon;
import { useStore, selectors } from "../state/store";

type Props = { height?: number };

// Default icon fix: Leaflet expects image urls via CSS; Vite bundles fine but ensure marker icons exist.
// In modern bundlers leaflet's default icon URLs may not resolve; keeping defaults here as CSS import often works.

export default function WorldMap({ height = 500 }: Props) {
  const mapRef = useRef<L.Map | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const points = useStore((s) => s.points);
  const worldPoints = points.filter(
    (p) => typeof p.lat === "number" && typeof p.lon === "number"
  );
  const activePointId = useStore((s) => s.activePointId);
  const setActivePoint = useStore((s) => s.setActivePoint);
  const selectLinkedPoint = useStore((s) => s.selectLinkedPoint);
  const addPoint = useStore((s) => s.addPoint);
  const updatePointWorld = useStore((s) => s.updatePointWorld);
  const removePoint = useStore((s) => s.removePoint);
  const setMapCenter = useStore((s) => s.setMapCenter);
  const image = useStore((s) => s.image);

  // Map init
  useEffect(() => {
    if (mapRef.current || !containerRef.current) return;
    // Center map on Rotterdam by default
    const map = L.map(containerRef.current, {
      center: [51.9225, 4.47917],
      zoom: 13,
      // disable Leaflet's default double-click zoom so we can use dblclick for adding points
      doubleClickZoom: false,
    });
    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution: "&copy; OpenStreetMap contributors",
    }).addTo(map);

    // Use double-click to add or attach world points to the active pixel point
    map.on("dblclick", (e: L.LeafletMouseEvent) => {
      const lat = e.latlng.lat;
      const lon = e.latlng.lng;

      // If there's an active point that has image coords but missing world coords,
      // update that point instead of creating a new one (mirror ImageCanvas behaviour).
      const activePt = points.find((p) => p.id === activePointId);
      if (
        activePt &&
        typeof activePt.u === "number" &&
        typeof activePt.v === "number" &&
        (typeof activePt.lat !== "number" || typeof activePt.lon !== "number")
      ) {
        updatePointWorld(activePt.id, lat, lon);
        setActivePoint(activePt.id);
        selectLinkedPoint(activePt.id, "world");
        return;
      }

      // Otherwise create a new world point. If an image is loaded, seed its image
      // coords to the image centre for convenience (existing behaviour).
      const newPoint: any = { lat, lon };
      if (image && image.width >= 1 && image.height >= 1) {
        newPoint.u = image.width / 2;
        newPoint.v = image.height / 2;
        newPoint.sigmaPx = 1;
        newPoint.enabled = true;
        newPoint.height = 0;
      }
      const ptId = addPoint(newPoint);
      setActivePoint(ptId);
      selectLinkedPoint(ptId, "world");
    });

    mapRef.current = map;
    // store initial map center
    setMapCenter(map.getCenter().lat, map.getCenter().lng);
    // update center on moveend
    map.on("moveend", () => {
      const c = map.getCenter();
      setMapCenter(c.lat, c.lng);
    });
  }, [
    addPoint,
    image,
    setActivePoint,
    points,
    activePointId,
    updatePointWorld,
    selectLinkedPoint,
    setMapCenter,
  ]);

  // Render markers (naive: recreate layer group)
  useEffect(() => {
    const map = mapRef.current;
    if (!map) return;
    const layer = L.layerGroup().addTo(map);
    const markers = worldPoints.map((wp) => {
      const marker = L.marker([wp.lat as number, wp.lon as number], {
        draggable: true,
      });
      marker.addTo(layer);
      marker.on("click", () => {
        setActivePoint(wp.id);
        selectLinkedPoint(wp.id, "world");
      });
      marker.on("drag", (e: any) => {
        const ll = e.latlng as L.LatLng;
        updatePointWorld(wp.id, ll.lat, ll.lng);
      });
      if (activePointId === wp.id) {
        marker
          .bindTooltip("Selected", { permanent: true, direction: "top" })
          .openTooltip();
      }
      return marker;
    });
    return () => {
      layer.clearLayers();
      map.removeLayer(layer);
    };
  }, [
    worldPoints,
    updatePointWorld,
    removePoint,
    setActivePoint,
    selectLinkedPoint,
    activePointId,
  ]);

  return (
    <div>
      <div
        ref={containerRef}
        style={{ height, border: "1px solid #333", borderRadius: 8 }}
      />
      <p style={{ color: "#777", marginTop: 6 }}>
        Tip: Double-click to add world point (or attach to active image point);
        left-click to select; drag to move.
      </p>
    </div>
  );
}
