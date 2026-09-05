import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { useAppStore } from "../../store";
import type { AppNode } from "../../types";
import { ResizableFrame } from "./ResizableFrame";

export type LinkFlowNode = Node<{ appNode: AppNode }, "link">;

export function LinkNode({ id, data, selected, width, height }: NodeProps<LinkFlowNode>) {
  const persistNode = useAppStore((s) => s.persistNode);
  const removeNode = useAppStore((s) => s.removeNode);
  const n = data.appNode;
  const w = width ?? n.width;
  const h = height ?? n.height;

  return (
    <ResizableFrame
      nodeId={id}
      selected={!!selected}
      width={w}
      height={h}
      minWidth={180}
      minHeight={90}
      className={`cm-node cm-link ${selected ? "selected" : ""}`}
    >
      <Handle type="source" position={Position.Right} className="cm-handle" />
      <Handle type="source" position={Position.Bottom} id="b" className="cm-handle" />
      <div className="cm-node-header">
        <span className="cm-badge">link</span>
        <button
          className="cm-icon-btn"
          onClick={(e) => {
            e.stopPropagation();
            void removeNode(n.id);
          }}
        >
          ×
        </button>
      </div>
      <input
        className="cm-node-title"
        value={n.title}
        onChange={(e) =>
          useAppStore.getState().updateNodeLocal({ ...n, title: e.target.value })
        }
        onBlur={(e) => persistNode({ id: n.id, title: e.target.value })}
      />
      <input
        className="cm-path-input nodrag"
        value={n.content}
        placeholder="https://"
        onChange={(e) =>
          useAppStore.getState().updateNodeLocal({ ...n, content: e.target.value })
        }
        onBlur={(e) => persistNode({ id: n.id, content: e.target.value })}
      />
    </ResizableFrame>
  );
}
