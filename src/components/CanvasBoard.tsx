import {
  Background,
  Controls,
  MiniMap,
  ReactFlow,
  useEdgesState,
  useNodesState,
  type Connection,
  type Edge,
  type Node,
  type OnNodeDrag,
  type ReactFlowInstance,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { useCallback, useEffect, useMemo, useRef } from "react";
import { useAppStore } from "../store";
import type { NodeType } from "../types";
import { Toolbar } from "./Toolbar";
import { FileNode } from "./nodes/FileNode";
import { ImageNode } from "./nodes/ImageNode";
import { LinkNode } from "./nodes/LinkNode";
import { NoteNode } from "./nodes/NoteNode";
import { TerminalNode } from "./nodes/TerminalNode";

const nodeTypes = {
  note: NoteNode,
  image: ImageNode,
  file: FileNode,
  link: LinkNode,
  terminal: TerminalNode,
};

function toFlowNodes(nodes: ReturnType<typeof useAppStore.getState>["nodes"]): Node[] {
  return nodes.map((n) => ({
    id: n.id,
    type: n.nodeType,
    position: { x: n.positionX, y: n.positionY },
    style: { width: n.width, height: n.height },
    data: { appNode: n },
  }));
}

function toFlowEdges(edges: ReturnType<typeof useAppStore.getState>["edges"]): Edge[] {
  return edges.map((e) => ({
    id: e.id,
    source: e.sourceNodeId,
    target: e.targetNodeId,
    animated: true,
    className: "cm-context-edge",
  }));
}

export function CanvasBoard() {
  const storeNodes = useAppStore((s) => s.nodes);
  const storeEdges = useAppStore((s) => s.edges);
  const addNode = useAppStore((s) => s.addNode);
  const persistNode = useAppStore((s) => s.persistNode);
  const removeNode = useAppStore((s) => s.removeNode);
  const connect = useAppStore((s) => s.connect);
  const disconnect = useAppStore((s) => s.disconnect);
  const selectTerminal = useAppStore((s) => s.selectTerminal);
  const activeMapId = useAppStore((s) => s.activeMapId);

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const rf = useRef<ReactFlowInstance | null>(null);

  useEffect(() => {
    setNodes(toFlowNodes(storeNodes));
    setEdges(toFlowEdges(storeEdges));
  }, [storeNodes, storeEdges, setNodes, setEdges, activeMapId]);

  const onConnect = useCallback(
    async (connection: Connection) => {
      if (!connection.source || !connection.target) return;
      try {
        await connect(connection.source, connection.target);
      } catch (e) {
        useAppStore.getState().setError(String(e));
      }
    },
    [connect],
  );

  const onNodeDragStop: OnNodeDrag = useCallback(
    (_evt, node) => {
      void persistNode({
        id: node.id,
        positionX: node.position.x,
        positionY: node.position.y,
      });
    },
    [persistNode],
  );

  const handleAdd = useCallback(
    async (type: NodeType) => {
      const center = rf.current
        ? rf.current.screenToFlowPosition({
            x: window.innerWidth / 2,
            y: window.innerHeight / 2,
          })
        : { x: 120 + Math.random() * 80, y: 120 + Math.random() * 80 };
      await addNode(type, center);
    },
    [addNode],
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if (e.key === "n") void handleAdd("note");
      if (e.key === "t") void handleAdd("terminal");
      if (e.key === "i") void handleAdd("image");
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [handleAdd]);

  const proOptions = useMemo(() => ({ hideAttribution: true }), []);

  if (!activeMapId) {
    return <div className="cm-empty">Crie um mapa para começar.</div>;
  }

  return (
    <div className="cm-canvas-wrap">
      <Toolbar onAdd={(t) => void handleAdd(t)} />
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={(c) => void onConnect(c)}
        onNodeDragStop={onNodeDragStop}
        onNodesDelete={(deleted) => {
          for (const n of deleted) void removeNode(n.id);
        }}
        onEdgesDelete={(eds) => {
          for (const e of eds) void disconnect(e.id);
        }}
        onNodeClick={(_e, node) => {
          if (node.type === "terminal") void selectTerminal(node.id);
        }}
        nodeTypes={nodeTypes}
        fitView
        minZoom={0.2}
        maxZoom={2}
        proOptions={proOptions}
        onInit={(instance) => {
          rf.current = instance;
        }}
        deleteKeyCode={["Backspace", "Delete"]}
      >
        <Background gap={24} size={1} />
        <Controls />
        <MiniMap pannable zoomable />
      </ReactFlow>
    </div>
  );
}
