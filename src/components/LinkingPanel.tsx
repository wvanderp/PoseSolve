import React from 'react';
import { useStore } from '../state/store';

export default function LinkingPanel() {
  const pixelPoints = useStore(s => s.pixelPoints);
  const worldPoints = useStore(s => s.worldPoints);
  const links = useStore(s => s.links);
  const linkPoints = useStore(s => s.linkPoints);
  const unlinkByPixel = useStore(s => s.unlinkByPixel);

  return (
    <div style={{ marginTop: 12 }}>
      <h3>Links</h3>
      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <select id="px" style={{ padding: 4 }}>
          {pixelPoints.map(p => <option key={p.id} value={p.id}>{p.id} ({p.u.toFixed(0)},{p.v.toFixed(0)})</option>)}
        </select>
        <span>↔</span>
        <select id="wp" style={{ padding: 4 }}>
          {worldPoints.map(w => <option key={w.id} value={w.id}>{w.id} ({w.lat.toFixed(5)},{w.lon.toFixed(5)})</option>)}
        </select>
        <button onClick={() => {
          const pSel = (document.getElementById('px') as HTMLSelectElement);
          const wSel = (document.getElementById('wp') as HTMLSelectElement);
          if (pSel?.value && wSel?.value) linkPoints(pSel.value, wSel.value);
        }}>Link</button>
      </div>
      <ul>
        {links.map(l => (
          <li key={l.pixelId} style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
            <code>{l.pixelId}</code> ↔ <code>{l.worldId}</code>
            <button onClick={() => unlinkByPixel(l.pixelId)}>Unlink</button>
          </li>
        ))}
      </ul>
    </div>
  );
}
