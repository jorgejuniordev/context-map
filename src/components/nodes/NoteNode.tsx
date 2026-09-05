import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { useAppStore } from "../../store";
import type { AppNode } from "../../types";
import { ResizableFrame } from "./ResizableFrame";

export type NoteFlowNode = Node<{ appNode: AppNode }, "note">;

export function NoteNode({ id, data, selected, width, height }: NodeProps<NoteFlowNode>) {
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
      minWidth={200}
      minHeight={140}
      className={`cm-node cm-note ${selected ? "selected" : ""}`}
    >
      <Handle type="source" position={Position.Right} className="cm-handle" />
      <Handle type="source" position={Position.Bottom} id="b" className="cm-handle" />
      <div className="cm-node-header">
        <input
          className="cm-node-title"
          value={n.title}
          onChange={(e) =>
            useAppStore.getState().updateNodeLocal({ ...n, title: e.target.value })
          }
          onBlur={(e) => persistNode({ id: n.id, title: e.target.value })}
        />
        <button
          className="cm-icon-btn"
          title="Excluir"
          onClick={(e) => {
            e.stopPropagation();
            void removeNode(n.id);
          }}
        >
          ×
        </button>
      </div>
      <textarea
        className="cm-note-body nodrag nowheel"
        value={n.content}
        placeholder="Escreva a nota (markdown ok)…"
        onChange={(e) =>
          useAppStore.getState().updateNodeLocal({ ...n, content: e.target.value })
        }
        onBlur={(e) => persistNode({ id: n.id, content: e.target.value })}
      />
    </ResizableFrame>
  );
}
