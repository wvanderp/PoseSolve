import React from "react";
import { useStore } from "../state/store";

export default function PointsTable() {
  const { points, activePointId, setActivePoint } = useStore();

  if (points.length === 0) {
    return (
      <div className="p-4 bg-gray-50 rounded-lg">
        <h3 className="text-lg font-semibold mb-2">Points List</h3>
        <p className="text-gray-500">No points created yet</p>
      </div>
    );
  }

  const formatCoordinate = (value: number | undefined, decimals = 4) => {
    return value !== undefined ? value.toFixed(decimals) : "—";
  };

  return (
    <div className="p-4 bg-gray-50 rounded-lg">
      <h3 className="text-lg font-semibold mb-4">
        Points List ({points.length})
      </h3>

      <div className="overflow-x-auto bg-white rounded border border-gray-200">
        <table className="min-w-full text-sm text-left">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-3 py-2">Point</th>
              <th className="px-3 py-2">Image (u, v)</th>
              <th className="px-3 py-2">World (lat, lon)</th>
              <th className="px-3 py-2">Height / Alt</th>
              <th className="px-3 py-2">Status</th>
            </tr>
          </thead>

          <tbody>
            {points.map((point) => {
              const isActive = point.id === activePointId;
              return (
                <tr
                  key={point.id}
                  data-cy="point-item"
                  onClick={() => setActivePoint(point.id)}
                  className={`cursor-pointer hover:bg-gray-50 ${
                    isActive ? "bg-blue-100" : ""
                  }`}
                >
                  <td className="px-3 py-2 align-top">
                    <div className="font-medium text-gray-800 text-sm">
                      Point {point.id.slice(0, 8)}...
                    </div>
                    <div className="text-xs text-gray-500">
                      {isActive && "● Active"}
                    </div>
                  </td>

                  <td
                    className="px-3 py-2 align-top text-xs"
                    data-cy="point-image"
                  >
                    {point.u !== undefined && point.v !== undefined ? (
                      <div>
                        <div>u: {formatCoordinate(point.u, 1)} px</div>
                        <div>v: {formatCoordinate(point.v, 1)} px</div>
                      </div>
                    ) : (
                      <div className="text-gray-400">Not set</div>
                    )}
                  </td>

                  <td
                    className="px-3 py-2 align-top text-xs"
                    data-cy="point-world"
                  >
                    {point.lat !== undefined && point.lon !== undefined ? (
                      <div>
                        <div>lat: {formatCoordinate(point.lat)}°</div>
                        <div>lon: {formatCoordinate(point.lon)}°</div>
                      </div>
                    ) : (
                      <div className="text-gray-400">Not set</div>
                    )}
                  </td>

                  <td className="px-3 py-2 align-top text-xs">
                    {point.height !== undefined ? (
                      <div>height: {formatCoordinate(point.height, 1)} m</div>
                    ) : point.alt !== undefined ? (
                      <div>alt: {formatCoordinate(point.alt, 1)} m</div>
                    ) : (
                      <div className="text-gray-400">—</div>
                    )}
                  </td>

                  <td className="px-3 py-2 align-top text-xs">
                    <div className="flex gap-2">
                      {point.u !== undefined && point.v !== undefined && (
                        <span className="inline-block px-2 py-1 text-xs bg-green-100 text-green-800 rounded">
                          Image
                        </span>
                      )}
                      {point.lat !== undefined && point.lon !== undefined && (
                        <span className="inline-block px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded">
                          World
                        </span>
                      )}
                      {point.u !== undefined &&
                        point.v !== undefined &&
                        point.lat !== undefined &&
                        point.lon !== undefined && (
                          <span className="inline-block px-2 py-1 text-xs bg-purple-100 text-purple-800 rounded">
                            Linked
                          </span>
                        )}
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
