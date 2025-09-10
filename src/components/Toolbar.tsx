import React from "react";

type Counts = {
  image: number;
  world: number;
  linked: number;
};

interface ToolbarProps {
  activePointId?: string | null;
  onDelete: () => void;
  onSolve: () => void;
  status: string;
  counts: Counts;
}

export default function Toolbar({
  activePointId,
  onDelete,
  onSolve,
  status,
  counts,
}: ToolbarProps) {
  return (
    <>
      <div
        style={{
          display: "flex",
          gap: 8,
          alignItems: "center",
          marginBottom: 12,
        }}
      >
        <button
          onClick={onDelete}
          data-testid="delete-point"
          style={{
            padding: "6px 10px",
            backgroundColor: activePointId ? "#ff6b6b" : "#eee",
            color: activePointId ? "white" : "#666",
            border: "none",
            borderRadius: 6,
          }}
          disabled={!activePointId}
          title={activePointId ? "Delete selected point" : "No point selected"}
        >
          Delete Point
        </button>
      </div>

      <div
        style={{
          marginTop: 12,
          display: "flex",
          gap: 12,
          alignItems: "center",
          flexWrap: "wrap",
        }}
      >
        <button
          onClick={onSolve}
          style={{
            padding: "8px 12px",
            border: "1px solid #ccc",
            borderRadius: 6,
          }}
        >
          Solve
        </button>
        <span>Status: {status}</span>
        <span>
          | Points: {counts.image} px, {counts.world} world, linked:{" "}
          {counts.linked}
        </span>
      </div>
    </>
  );
}
