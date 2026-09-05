import { create } from "zustand";
import { api } from "./api";
import type {
  ActiveContext,
  AppEdge,
  AppNode,
  MapInfo,
  NodeType,
  Workspace,
} from "./types";

interface AppStore {
  workspaces: Workspace[];
  maps: MapInfo[];
  nodes: AppNode[];
  edges: AppEdge[];
  activeWorkspaceId: string | null;
  activeMapId: string | null;
  selectedTerminalId: string | null;
  activeContext: ActiveContext | null;
  theme: "dark" | "light";
  recentDisconnected: { sourceId: string; targetId: string; title: string }[];
  loading: boolean;
  error: string | null;

  bootstrap: () => Promise<void>;
  selectWorkspace: (id: string) => Promise<void>;
  selectMap: (id: string) => Promise<void>;
  createWorkspace: (name: string) => Promise<void>;
  renameWorkspace: (id: string, name: string) => Promise<void>;
  deleteWorkspace: (id: string) => Promise<void>;
  duplicateWorkspace: (id: string) => Promise<void>;
  createMap: (name: string) => Promise<void>;
  renameMap: (id: string, name: string) => Promise<void>;
  deleteMap: (id: string) => Promise<void>;
  addNode: (
    nodeType: NodeType,
    position: { x: number; y: number },
    extras?: { title?: string; content?: string },
  ) => Promise<AppNode | null>;
  updateNodeLocal: (node: AppNode) => void;
  persistNode: (patch: {
    id: string;
    title?: string;
    content?: string;
    positionX?: number;
    positionY?: number;
    width?: number;
    height?: number;
  }) => Promise<void>;
  removeNode: (id: string) => Promise<void>;
  connect: (sourceId: string, targetId: string) => Promise<void>;
  disconnect: (edgeId: string) => Promise<void>;
  selectTerminal: (id: string | null) => Promise<void>;
  refreshContext: (terminalId: string) => Promise<void>;
  reconnectRecent: (sourceId: string, targetId: string) => Promise<void>;
  toggleTheme: () => void;
  setError: (msg: string | null) => void;
}

const defaults: Record<NodeType, { title: string; content: string }> = {
  note: { title: "Nova nota", content: "" },
  image: { title: "Imagem", content: "" },
  file: { title: "Arquivo", content: "" },
  link: { title: "Link", content: "https://" },
  terminal: { title: "Terminal", content: "" },
};

export const useAppStore = create<AppStore>((set, get) => ({
  workspaces: [],
  maps: [],
  nodes: [],
  edges: [],
  activeWorkspaceId: null,
  activeMapId: null,
  selectedTerminalId: null,
  activeContext: null,
  theme: (localStorage.getItem("cm-theme") as "dark" | "light") || "dark",
  recentDisconnected: [],
  loading: false,
  error: null,

  setError: (msg) => set({ error: msg }),

  toggleTheme: () => {
    const theme = get().theme === "dark" ? "light" : "dark";
    localStorage.setItem("cm-theme", theme);
    document.documentElement.dataset.theme = theme;
    set({ theme });
  },

  bootstrap: async () => {
    set({ loading: true, error: null });
    document.documentElement.dataset.theme = get().theme;
    try {
      let workspaces = await api.listWorkspaces();
      if (workspaces.length === 0) {
        const ws = await api.createWorkspace("Meu Workspace");
        workspaces = [ws];
      }
      set({ workspaces });
      await get().selectWorkspace(workspaces[0].id);
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  selectWorkspace: async (id) => {
    const maps = await api.listMaps(id);
    set({
      activeWorkspaceId: id,
      maps,
      activeMapId: null,
      nodes: [],
      edges: [],
      selectedTerminalId: null,
      activeContext: null,
    });
    if (maps.length > 0) {
      await get().selectMap(maps[0].id);
    }
  },

  selectMap: async (id) => {
    const snap = await api.getMapSnapshot(id);
    set({
      activeMapId: id,
      nodes: snap.nodes,
      edges: snap.edges,
      selectedTerminalId: null,
      activeContext: null,
    });
  },

  createWorkspace: async (name) => {
    const ws = await api.createWorkspace(name);
    const workspaces = await api.listWorkspaces();
    set({ workspaces });
    await get().selectWorkspace(ws.id);
  },

  renameWorkspace: async (id, name) => {
    await api.renameWorkspace(id, name);
    set({ workspaces: await api.listWorkspaces() });
  },

  deleteWorkspace: async (id) => {
    await api.deleteWorkspace(id);
    let workspaces = await api.listWorkspaces();
    if (workspaces.length === 0) {
      const ws = await api.createWorkspace("Meu Workspace");
      workspaces = [ws];
    }
    set({ workspaces });
    await get().selectWorkspace(workspaces[0].id);
  },

  duplicateWorkspace: async (id) => {
    const ws = await api.duplicateWorkspace(id);
    set({ workspaces: await api.listWorkspaces() });
    await get().selectWorkspace(ws.id);
  },

  createMap: async (name) => {
    const wsId = get().activeWorkspaceId;
    if (!wsId) return;
    const map = await api.createMap(wsId, name);
    set({ maps: await api.listMaps(wsId) });
    await get().selectMap(map.id);
  },

  renameMap: async (id, name) => {
    await api.renameMap(id, name);
    const wsId = get().activeWorkspaceId;
    if (wsId) set({ maps: await api.listMaps(wsId) });
  },

  deleteMap: async (id) => {
    await api.deleteMap(id);
    const wsId = get().activeWorkspaceId;
    if (!wsId) return;
    const maps = await api.listMaps(wsId);
    set({ maps });
    if (maps.length > 0) await get().selectMap(maps[0].id);
    else set({ activeMapId: null, nodes: [], edges: [] });
  },

  addNode: async (nodeType, position, extras) => {
    const mapId = get().activeMapId;
    if (!mapId) return null;
    const d = defaults[nodeType];
    const node = await api.createNode({
      mapId,
      nodeType,
      title: extras?.title ?? d.title,
      content: extras?.content ?? d.content,
      positionX: position.x,
      positionY: position.y,
    });
    set({ nodes: [...get().nodes, node] });
    return node;
  },

  updateNodeLocal: (node) => {
    set({
      nodes: get().nodes.map((n) => (n.id === node.id ? node : n)),
    });
  },

  persistNode: async (patch) => {
    const current = get().nodes.find((n) => n.id === patch.id);
    if (!current) return; // node already deleted — ignore stale drag/resize
    if (current) {
      get().updateNodeLocal({
        ...current,
        title: patch.title ?? current.title,
        content: patch.content ?? current.content,
        positionX: patch.positionX ?? current.positionX,
        positionY: patch.positionY ?? current.positionY,
        width: patch.width ?? current.width,
        height: patch.height ?? current.height,
      });
    }
    try {
      const node = await api.updateNode(patch);
      // Only apply if node still exists (wasn't deleted mid-flight)
      if (get().nodes.some((n) => n.id === node.id)) {
        get().updateNodeLocal(node);
      }
      const tid = get().selectedTerminalId;
      if (tid) await get().refreshContext(tid);
    } catch (e) {
      get().setError(String(e));
    }
  },

  removeNode: async (id) => {
    const prevNodes = get().nodes;
    const prevEdges = get().edges;
    const prevTerminal = get().selectedTerminalId;
    const prevContext = get().activeContext;

    // Optimistic: remove from UI/store immediately so a drag can't resurrect it
    set({
      nodes: prevNodes.filter((n) => n.id !== id),
      edges: prevEdges.filter(
        (e) => e.sourceNodeId !== id && e.targetNodeId !== id,
      ),
      selectedTerminalId: prevTerminal === id ? null : prevTerminal,
      activeContext: prevTerminal === id ? null : prevContext,
    });

    try {
      await api.deleteNode(id);
    } catch (e) {
      set({
        nodes: prevNodes,
        edges: prevEdges,
        selectedTerminalId: prevTerminal,
        activeContext: prevContext,
        error: String(e),
      });
    }
  },

  connect: async (sourceId, targetId) => {
    const mapId = get().activeMapId;
    if (!mapId) return;
    const edge = await api.createEdge(mapId, sourceId, targetId);
    set({ edges: [...get().edges, edge] });
    if (get().selectedTerminalId === targetId) {
      await get().refreshContext(targetId);
    }
  },

  disconnect: async (edgeId) => {
    const edge = get().edges.find((e) => e.id === edgeId);
    if (!edge) return;
    const source = get().nodes.find((n) => n.id === edge.sourceNodeId);
    await api.deleteEdge(edgeId);
    set({
      edges: get().edges.filter((e) => e.id !== edgeId),
      recentDisconnected: [
        {
          sourceId: edge.sourceNodeId,
          targetId: edge.targetNodeId,
          title: source?.title ?? "Nó",
        },
        ...get().recentDisconnected.filter(
          (r) =>
            !(r.sourceId === edge.sourceNodeId && r.targetId === edge.targetNodeId),
        ),
      ].slice(0, 8),
    });
    if (get().selectedTerminalId === edge.targetNodeId) {
      await get().refreshContext(edge.targetNodeId);
    }
  },

  selectTerminal: async (id) => {
    set({ selectedTerminalId: id });
    if (id) await get().refreshContext(id);
    else set({ activeContext: null });
  },

  refreshContext: async (terminalId) => {
    const ctx = await api.getActiveContext(terminalId);
    set({ activeContext: ctx });
  },

  reconnectRecent: async (sourceId, targetId) => {
    await get().connect(sourceId, targetId);
    set({
      recentDisconnected: get().recentDisconnected.filter(
        (r) => !(r.sourceId === sourceId && r.targetId === targetId),
      ),
    });
  },
}));
