import React from 'react';
import type { SolveResponse } from '../types/solver';

interface SolveResultProps {
  result: SolveResponse;
}

export default function SolveResult({ result }: SolveResultProps) {
  const { pose, intrinsics, diagnostics } = result;

  const row = (label: string, value: string) => (
    <tr key={label}>
      <td style={{ paddingRight: 16, color: '#888', whiteSpace: 'nowrap' }}>{label}</td>
      <td style={{ fontFamily: 'monospace' }}>{value}</td>
    </tr>
  );

  return (
    <div
      data-testid="solve-result"
      style={{
        marginTop: 12,
        background: '#1a1a2e',
        color: '#e0e0e0',
        padding: 16,
        borderRadius: 8,
        fontSize: 13,
      }}
    >
      <strong style={{ fontSize: 14 }}>Solve Result</strong>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0 32px', marginTop: 10 }}>
        {/* Position */}
        <div>
          <div style={{ color: '#90caf9', marginBottom: 4, fontWeight: 600 }}>Position</div>
          <table style={{ borderCollapse: 'collapse' }}>
            <tbody>
              {row('Latitude', `${pose.lat.toFixed(7)}°`)}
              {row('Longitude', `${pose.lon.toFixed(7)}°`)}
              {row('Altitude', `${pose.alt.toFixed(1)} m`)}
            </tbody>
          </table>
        </div>

        {/* Orientation */}
        <div>
          <div style={{ color: '#90caf9', marginBottom: 4, fontWeight: 600 }}>Orientation</div>
          <table style={{ borderCollapse: 'collapse' }}>
            <tbody>
              {row('Yaw', `${pose.yawDeg.toFixed(2)}°`)}
              {row('Pitch', `${pose.pitchDeg.toFixed(2)}°`)}
              {row('Roll', `${pose.rollDeg.toFixed(2)}°`)}
            </tbody>
          </table>
        </div>

        {/* Intrinsics */}
        <div style={{ marginTop: 12 }}>
          <div style={{ color: '#90caf9', marginBottom: 4, fontWeight: 600 }}>Camera</div>
          <table style={{ borderCollapse: 'collapse' }}>
            <tbody>
              {row('Focal length', `${intrinsics.focalPx.toFixed(1)} px`)}
              {row('Principal point', `(${intrinsics.cx.toFixed(1)}, ${intrinsics.cy.toFixed(1)}) px`)}
              {intrinsics.k1 != null ? row('k1', intrinsics.k1.toFixed(5)) : null}
              {intrinsics.k2 != null ? row('k2', intrinsics.k2.toFixed(5)) : null}
            </tbody>
          </table>
        </div>

        {/* Quality */}
        <div style={{ marginTop: 12 }}>
          <div style={{ color: '#90caf9', marginBottom: 4, fontWeight: 600 }}>Quality</div>
          <table style={{ borderCollapse: 'collapse' }}>
            <tbody>
              {row('RMSE', `${diagnostics.rmsePx.toFixed(2)} px`)}
              {row('Inlier ratio', `${(diagnostics.inlierRatio * 100).toFixed(1)} %`)}
              {row('Inliers', `${diagnostics.inlierIds.length}`)}
            </tbody>
          </table>
        </div>
      </div>

      {diagnostics.warnings.length > 0 && (
        <div style={{ marginTop: 12, color: '#ffd54f' }}>
          <strong>Warnings:</strong>
          <ul style={{ margin: '4px 0 0 0', paddingLeft: 18 }}>
            {diagnostics.warnings.map((w, i) => (
              <li key={i}>{w}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
