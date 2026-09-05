export type NodeType = "note" | "image" | "file" | "link" | "terminal";

export interface Workspace {
  id: string;
  name: string;
  createdAt: string;
}

export interface MapInfo {
  id: string;
  workspaceId: string;
  name: string;
  createdAt: string;
}

export interface AppNode {
  id: string;
  mapId: string;
  nodeType: NodeType;
  title: string;
  content: string;
  positionX: number;
  positionY: number;
  width: number;
  height: number;
  createdAt: string;
  updatedAt: string;
}

export interface AppEdge {
  id: string;
  mapId: string;
  sourceNodeId: string;
  targetNodeId: string;
  createdAt: string;
}

export interface MapSnapshot {
  map: MapInfo;
  nodes: AppNode[];
  edges: AppEdge[];
}

export interface ActiveContext {
  terminalNodeId: string;
  connectedNodes: AppNode[];
  contextMarkdown: string;
  contextFilePath: string;
}

export interface CreateNodeRequest {
  mapId: string;
  nodeType: NodeType;
  title: string;
  content: string;
  positionX: number;
  positionY: number;
  width?: number;
  height?: number;
}
