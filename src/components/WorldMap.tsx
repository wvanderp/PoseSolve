import React, { useEffect, useRef } from 'react';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
// Fix default marker icons under Vite bundling
// @ts-ignore
import markerIcon2xUrl from 'leaflet/dist/images/marker-icon-2x.png';
// @ts-ignore
import markerIconUrl from 'leaflet/dist/images/marker-icon.png';
// @ts-ignore
import markerShadowUrl from 'leaflet/dist/images/marker-shadow.png';
L.Icon.Default.mergeOptions({
  iconRetinaUrl: markerIcon2xUrl,
  iconUrl: markerIconUrl,
  shadowUrl: markerShadowUrl,
});
import { useStore, selectors } from '../state/store';

type Props = { height?: number };

// Default icon fix: Leaflet expects image urls via CSS; Vite bundles fine but ensure marker icons exist.
// In modern bundlers leaflet's default icon URLs may not resolve; keeping defaults here as CSS import often works.

export default function WorldMap({ height = 500 }: Props) {
  const mapRef = useRef<L.Map | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const worldPoints = useStore(s => s.worldPoints);
  const activeWorldId = useStore(s => s.activeWorldId);
  const setActiveWorld = useStore(s => s.setActiveWorld);
  const addWorldPoint = useStore(s => s.addWorldPoint);
  const moveWorldPoint = useStore(s => s.moveWorldPoint);
  const removeWorldPoint = useStore(s => s.removeWorldPoint);

  // Map init
  useEffect(() => {
    if (mapRef.current || !containerRef.current) return;
  // Center map on Rotterdam by default
  const map = L.map(containerRef.current, { center: [51.9225, 4.47917], zoom: 13 });
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '&copy; OpenStreetMap contributors'
    }).addTo(map);

    map.on('click', (e: L.LeafletMouseEvent) => {
      const id = addWorldPoint({ lat: e.latlng.lat, lon: e.latlng.lng });
      setActiveWorld(id);
    });

    mapRef.current = map;
  }, [addWorldPoint, setActiveWorld]);

  // Render markers (naive: recreate layer group)
  useEffect(() => {
    const map = mapRef.current;
    if (!map) return;
    const layer = L.layerGroup().addTo(map);
    const markers = worldPoints.map((wp) => {
      const marker = L.marker([wp.lat, wp.lon], { draggable: true });
      marker.addTo(layer);
      marker.on('click', () => setActiveWorld(wp.id));
      marker.on('drag', (e: any) => {
        const ll = e.latlng as L.LatLng;
        moveWorldPoint(wp.id, ll.lat, ll.lng);
      });
      marker.on('contextmenu', () => removeWorldPoint(wp.id));
      if (activeWorldId === wp.id) {
        marker.bindTooltip('Selected', { permanent: true, direction: 'top' }).openTooltip();
      }
      return marker;
    });
    return () => {
      layer.clearLayers();
      map.removeLayer(layer);
    };
  }, [worldPoints, moveWorldPoint, removeWorldPoint, setActiveWorld, activeWorldId]);

  return (
    <div>
      <div ref={containerRef} style={{ height, border: '1px solid #333', borderRadius: 8 }} />
      <p style={{ color: '#777', marginTop: 6 }}>Tip: Click to add; drag to move; right-click a marker to delete. Selecting a marker, then adding a pixel point will link them.</p>
    </div>
  );
}
