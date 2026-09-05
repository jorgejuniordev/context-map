import { invoke } from "@tauri-apps/api/core";
import type {
  ActiveContext,
  AppEdge,
  AppNode,
  CreateNodeRequest,
  MapInfo,
  MapSnapshot,
  Workspace,
} from "./types";

export const api = {
  listWorkspaces: () => invoke<Workspace[]>("list_workspaces"),
  createWorkspace: (name: string) =>
    invoke<Workspace>("create_workspace", { req: { name } }),
  renameWorkspace: (id: string, name: string) =>
    invoke<Workspace>("rename_workspace", { req: { id, name } }),
  deleteWorkspace: (id: string) => invoke<void>("delete_workspace", { id }),
  duplicateWorkspace: (id: string) =>
    invoke<Workspace>("duplicate_workspace", { id }),

  listMaps: (workspaceId: string) =>
    invoke<MapInfo[]>("list_maps", { workspaceId }),
  createMap: (workspaceId: string, name: string) =>
    invoke<MapInfo>("create_map", { req: { workspaceId, name } }),
  renameMap: (id: string, name: string) =>
    invoke<MapInfo>("rename_map", { req: { id, name } }),
  deleteMap: (id: string) => invoke<void>("delete_map", { id }),
  getMapSnapshot: (mapId: string) =>
    invoke<MapSnapshot>("get_map_snapshot", { mapId }),

  createNode: (req: CreateNodeRequest) =>
    invoke<AppNode>("create_node", { req }),
  updateNode: (req: {
    id: string;
    title?: string;
    content?: string;
    positionX?: number;
    positionY?: number;
    width?: number;
    height?: number;
  }) => invoke<AppNode>("update_node", { req }),
  deleteNode: (id: string) => invoke<void>("delete_node", { id }),

  createEdge: (mapId: string, sourceNodeId: string, targetNodeId: string) =>
    invoke<AppEdge>("create_edge", {
      req: { mapId, sourceNodeId, targetNodeId },
    }),
  deleteEdge: (id: string) => invoke<void>("delete_edge", { id }),
  getActiveContext: (terminalNodeId: string) =>
    invoke<ActiveContext>("get_active_context", { terminalNodeId }),

  saveAsset: (filename: string, dataBase64: string) =>
    invoke<string>("save_asset", { filename, dataBase64 }),
  readAssetDataUrl: (path: string) =>
    invoke<string>("read_asset_data_url", { path }),

  ptyStart: (terminalNodeId: string, cols: number, rows: number) =>
    invoke<string>("pty_start", {
      req: { terminalNodeId, cols, rows },
    }),
  ptyWrite: (terminalNodeId: string, data: string) =>
    invoke<void>("pty_write", { req: { terminalNodeId, data } }),
  ptyResize: (terminalNodeId: string, cols: number, rows: number) =>
    invoke<void>("pty_resize", { req: { terminalNodeId, cols, rows } }),
  ptyStop: (terminalNodeId: string) =>
    invoke<void>("pty_stop", { terminalNodeId }),
  ptyIsRunning: (terminalNodeId: string) =>
    invoke<boolean>("pty_is_running", { terminalNodeId }),
};
