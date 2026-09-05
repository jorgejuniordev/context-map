import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { useEffect, useState } from "react";
import { api } from "../../api";
import { useAppStore } from "../../store";
import type { AppNode } from "../../types";
import { ResizableFrame } from "./ResizableFrame";

export type ImageFlowNode = Node<{ appNode: AppNode }, "image">;

export function ImageNode({ id, data, selected, width, height }: NodeProps<ImageFlowNode>) {
  const persistNode = useAppStore((s) => s.persistNode);
  const removeNode = useAppStore((s) => s.removeNode);
  const n = data.appNode;
  const [src, setSrc] = useState<string>("");
  const [fullscreen, setFullscreen] = useState(false);
  const w = width ?? n.width;
  const h = height ?? n.height;

  useEffect(() => {
    let cancelled = false;
    if (!n.content) {
      setSrc("");
      return;
    }
    api
      .readAssetDataUrl(n.content)
      .then((url) => {
        if (!cancelled) setSrc(url);
      })
      .catch(() => setSrc(""));
    return () => {
      cancelled = true;
    };
  }, [n.content]);

  const onFile = async (file: File) => {
    const reader = new FileReader();
    reader.onload = async () => {
      const dataUrl = String(reader.result);
      const path = await api.saveAsset(file.name, dataUrl);
      await persistNode({ id: n.id, content: path, title: file.name });
    };
    reader.readAsDataURL(file);
  };

  return (
    <ResizableFrame
      nodeId={id}
      selected={!!selected}
      width={w}
      height={h}
      minWidth={180}
      minHeight={140}
      className={`cm-node cm-image ${selected ? "selected" : ""}`}
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
          onClick={(e) => {
            e.stopPropagation();
            void removeNode(n.id);
          }}
        >
          ×
        </button>
      </div>
      {src ? (
        <img
          className="cm-image-preview nodrag"
          src={src}
          alt={n.title}
          onDoubleClick={() => setFullscreen(true)}
        />
      ) : (
        <label className="cm-dropzone nodrag">
          Arraste ou clique para enviar
          <input
            type="file"
            accept="image/*"
            hidden
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f) void onFile(f);
            }}
          />
        </label>
      )}
      {fullscreen && src && (
        <div className="cm-lightbox" onClick={() => setFullscreen(false)}>
          <img src={src} alt={n.title} />
        </div>
      )}
    </ResizableFrame>
  );
}
