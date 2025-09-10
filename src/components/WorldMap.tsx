import React, { useEffect, useRef } from "react";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
// Fix default marker icons under Vite bundling
import markerIcon2xUrl from "leaflet/dist/images/marker-icon-2x.png";
import markerIconUrl from "leaflet/dist/images/marker-icon.png";
import markerShadowUrl from "leaflet/dist/images/marker-shadow.png";
L.Icon.Default.mergeOptions({
  iconRetinaUrl: markerIcon2xUrl,
  iconUrl: markerIconUrl,
  shadowUrl: markerShadowUrl,
});
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
  const image = useStore((s) => s.image);

  // Map init
  useEffect(() => {
    if (mapRef.current || !containerRef.current) return;
    // Center map on Rotterdam by default
    const map = L.map(containerRef.current, {
      center: [51.9225, 4.47917],
      zoom: 13,
    });
    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution: "&copy; OpenStreetMap contributors",
    }).addTo(map);

    map.on("click", (e: L.LeafletMouseEvent) => {
      // Create a single unified point with world coords and optional image center coords
      const newPoint = { lat: e.latlng.lat, lon: e.latlng.lng } as any;
      if (image && image.width >= 1 && image.height >= 1) {
        newPoint.u = image.width / 2;
        newPoint.v = image.height / 2;
        newPoint.sigmaPx = 1;
        newPoint.enabled = true;
        newPoint.height = 0;
      }
      const ptId = addPoint(newPoint);
      setActivePoint(ptId);
    });

    mapRef.current = map;
  }, [addPoint, image, setActivePoint]);

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
      marker.on("contextmenu", () => removePoint(wp.id));
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
        Tip: Click to add world point and auto-link to image center; drag to
        move; right-click to delete. Selecting points shows cross-selection.
      </p>
    </div>
  );
}
