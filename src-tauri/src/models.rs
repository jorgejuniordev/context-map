use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Map {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Note,
    Image,
    File,
    Link,
    Terminal,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Note => "note",
            NodeType::Image => "image",
            NodeType::File => "file",
            NodeType::Link => "link",
            NodeType::Terminal => "terminal",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "note" => Some(NodeType::Note),
            "image" => Some(NodeType::Image),
            "file" => Some(NodeType::File),
            "link" => Some(NodeType::Link),
            "terminal" => Some(NodeType::Terminal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub map_id: String,
    pub node_type: String,
    pub title: String,
    pub content: String,
    pub position_x: f64,
    pub position_y: f64,
    pub width: f64,
    pub height: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: String,
    pub map_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameRequest {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMapRequest {
    pub workspace_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeRequest {
    pub map_id: String,
    pub node_type: String,
    pub title: String,
    pub content: String,
    pub position_x: f64,
    pub position_y: f64,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNodeRequest {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEdgeRequest {
    pub map_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapSnapshot {
    pub map: Map,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveContext {
    pub terminal_node_id: String,
    pub connected_nodes: Vec<Node>,
    pub context_markdown: String,
    pub context_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyStartRequest {
    pub terminal_node_id: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyWriteRequest {
    pub terminal_node_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyResizeRequest {
    pub terminal_node_id: String,
    pub cols: u16,
    pub rows: u16,
}
