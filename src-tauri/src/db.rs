use crate::models::*;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| e.to_string())?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS workspaces (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS maps (
                    id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS nodes (
                    id TEXT PRIMARY KEY,
                    map_id TEXT NOT NULL,
                    node_type TEXT NOT NULL,
                    title TEXT NOT NULL,
                    content TEXT NOT NULL DEFAULT '',
                    position_x REAL NOT NULL DEFAULT 0,
                    position_y REAL NOT NULL DEFAULT 0,
                    width REAL NOT NULL DEFAULT 240,
                    height REAL NOT NULL DEFAULT 160,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (map_id) REFERENCES maps(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS edges (
                    id TEXT PRIMARY KEY,
                    map_id TEXT NOT NULL,
                    source_node_id TEXT NOT NULL,
                    target_node_id TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    FOREIGN KEY (map_id) REFERENCES maps(id) ON DELETE CASCADE,
                    FOREIGN KEY (source_node_id) REFERENCES nodes(id) ON DELETE CASCADE,
                    FOREIGN KEY (target_node_id) REFERENCES nodes(id) ON DELETE CASCADE,
                    UNIQUE(source_node_id, target_node_id)
                );
                "#,
            )
            .map_err(|e| e.to_string())
    }

    // --- Workspaces ---

    pub fn list_workspaces(&self) -> Result<Vec<Workspace>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, created_at FROM workspaces ORDER BY created_at ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Workspace {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn create_workspace(&self, name: &str) -> Result<Workspace, String> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO workspaces (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![id, name, created_at],
            )
            .map_err(|e| e.to_string())?;

        // Default map
        let map_id = Uuid::new_v4().to_string();
        self.conn
            .execute(
                "INSERT INTO maps (id, workspace_id, name, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![map_id, id, "Mapa principal", created_at],
            )
            .map_err(|e| e.to_string())?;

        Ok(Workspace {
            id,
            name: name.to_string(),
            created_at,
        })
    }

    pub fn rename_workspace(&self, id: &str, name: &str) -> Result<Workspace, String> {
        self.conn
            .execute(
                "UPDATE workspaces SET name = ?1 WHERE id = ?2",
                params![name, id],
            )
            .map_err(|e| e.to_string())?;
        self.get_workspace(id)?
            .ok_or_else(|| "Workspace not found".into())
    }

    pub fn delete_workspace(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM workspaces WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn duplicate_workspace(&self, id: &str) -> Result<Workspace, String> {
        let original = self
            .get_workspace(id)?
            .ok_or_else(|| "Workspace not found".to_string())?;
        let new_ws = self.create_workspace(&format!("{} (cópia)", original.name))?;

        // Remove the auto-created default map, then copy everything
        let maps = self.list_maps(&new_ws.id)?;
        for m in maps {
            self.conn
                .execute("DELETE FROM maps WHERE id = ?1", params![m.id])
                .map_err(|e| e.to_string())?;
        }

        let source_maps = self.list_maps(id)?;
        for map in source_maps {
            let new_map_id = Uuid::new_v4().to_string();
            let created_at = Utc::now().to_rfc3339();
            self.conn
                .execute(
                    "INSERT INTO maps (id, workspace_id, name, created_at) VALUES (?1, ?2, ?3, ?4)",
                    params![new_map_id, new_ws.id, map.name, created_at],
                )
                .map_err(|e| e.to_string())?;

            let nodes = self.list_nodes(&map.id)?;
            let mut id_map = std::collections::HashMap::new();
            for node in &nodes {
                let new_node_id = Uuid::new_v4().to_string();
                id_map.insert(node.id.clone(), new_node_id.clone());
                self.conn
                    .execute(
                        r#"INSERT INTO nodes
                        (id, map_id, node_type, title, content, position_x, position_y, width, height, created_at, updated_at)
                        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)"#,
                        params![
                            new_node_id,
                            new_map_id,
                            node.node_type,
                            node.title,
                            node.content,
                            node.position_x,
                            node.position_y,
                            node.width,
                            node.height,
                            created_at,
                            created_at
                        ],
                    )
                    .map_err(|e| e.to_string())?;
            }

            let edges = self.list_edges(&map.id)?;
            for edge in edges {
                let Some(src) = id_map.get(&edge.source_node_id) else {
                    continue;
                };
                let Some(tgt) = id_map.get(&edge.target_node_id) else {
                    continue;
                };
                let new_edge_id = Uuid::new_v4().to_string();
                self.conn
                    .execute(
                        "INSERT INTO edges (id, map_id, source_node_id, target_node_id, created_at) VALUES (?1,?2,?3,?4,?5)",
                        params![new_edge_id, new_map_id, src, tgt, created_at],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }

        Ok(new_ws)
    }

    fn get_workspace(&self, id: &str) -> Result<Option<Workspace>, String> {
        self.conn
            .query_row(
                "SELECT id, name, created_at FROM workspaces WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Workspace {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    // --- Maps ---

    pub fn list_maps(&self, workspace_id: &str) -> Result<Vec<Map>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, workspace_id, name, created_at FROM maps WHERE workspace_id = ?1 ORDER BY created_at ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![workspace_id], |row| {
                Ok(Map {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn create_map(&self, workspace_id: &str, name: &str) -> Result<Map, String> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO maps (id, workspace_id, name, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, workspace_id, name, created_at],
            )
            .map_err(|e| e.to_string())?;
        Ok(Map {
            id,
            workspace_id: workspace_id.to_string(),
            name: name.to_string(),
            created_at,
        })
    }

    pub fn rename_map(&self, id: &str, name: &str) -> Result<Map, String> {
        self.conn
            .execute("UPDATE maps SET name = ?1 WHERE id = ?2", params![name, id])
            .map_err(|e| e.to_string())?;
        self.get_map(id)?
            .ok_or_else(|| "Map not found".into())
    }

    pub fn delete_map(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM maps WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_map(&self, id: &str) -> Result<Option<Map>, String> {
        self.conn
            .query_row(
                "SELECT id, workspace_id, name, created_at FROM maps WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Map {
                        id: row.get(0)?,
                        workspace_id: row.get(1)?,
                        name: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn get_map_snapshot(&self, map_id: &str) -> Result<MapSnapshot, String> {
        let map = self
            .get_map(map_id)?
            .ok_or_else(|| "Map not found".to_string())?;
        Ok(MapSnapshot {
            nodes: self.list_nodes(map_id)?,
            edges: self.list_edges(map_id)?,
            map,
        })
    }

    // --- Nodes ---

    pub fn list_nodes(&self, map_id: &str) -> Result<Vec<Node>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"SELECT id, map_id, node_type, title, content, position_x, position_y, width, height, created_at, updated_at
                   FROM nodes WHERE map_id = ?1"#,
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![map_id], |row| {
                Ok(Node {
                    id: row.get(0)?,
                    map_id: row.get(1)?,
                    node_type: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    position_x: row.get(5)?,
                    position_y: row.get(6)?,
                    width: row.get(7)?,
                    height: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn get_node(&self, id: &str) -> Result<Option<Node>, String> {
        self.conn
            .query_row(
                r#"SELECT id, map_id, node_type, title, content, position_x, position_y, width, height, created_at, updated_at
                   FROM nodes WHERE id = ?1"#,
                params![id],
                |row| {
                    Ok(Node {
                        id: row.get(0)?,
                        map_id: row.get(1)?,
                        node_type: row.get(2)?,
                        title: row.get(3)?,
                        content: row.get(4)?,
                        position_x: row.get(5)?,
                        position_y: row.get(6)?,
                        width: row.get(7)?,
                        height: row.get(8)?,
                        created_at: row.get(9)?,
                        updated_at: row.get(10)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn create_node(&self, req: CreateNodeRequest) -> Result<Node, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let (default_w, default_h) = match req.node_type.as_str() {
            "terminal" => (520.0, 320.0),
            "note" => (280.0, 200.0),
            "image" => (280.0, 220.0),
            _ => (240.0, 140.0),
        };
        let width = req.width.unwrap_or(default_w);
        let height = req.height.unwrap_or(default_h);

        self.conn
            .execute(
                r#"INSERT INTO nodes
                (id, map_id, node_type, title, content, position_x, position_y, width, height, created_at, updated_at)
                VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)"#,
                params![
                    id,
                    req.map_id,
                    req.node_type,
                    req.title,
                    req.content,
                    req.position_x,
                    req.position_y,
                    width,
                    height,
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_node(&id)?
            .ok_or_else(|| "Failed to load created node".into())
    }

    pub fn update_node(&self, req: UpdateNodeRequest) -> Result<Node, String> {
        let mut node = self
            .get_node(&req.id)?
            .ok_or_else(|| "Node not found".to_string())?;

        if let Some(title) = req.title {
            node.title = title;
        }
        if let Some(content) = req.content {
            node.content = content;
        }
        if let Some(x) = req.position_x {
            node.position_x = x;
        }
        if let Some(y) = req.position_y {
            node.position_y = y;
        }
        if let Some(w) = req.width {
            node.width = w;
        }
        if let Some(h) = req.height {
            node.height = h;
        }
        node.updated_at = Utc::now().to_rfc3339();

        self.conn
            .execute(
                r#"UPDATE nodes SET title=?1, content=?2, position_x=?3, position_y=?4, width=?5, height=?6, updated_at=?7
                   WHERE id=?8"#,
                params![
                    node.title,
                    node.content,
                    node.position_x,
                    node.position_y,
                    node.width,
                    node.height,
                    node.updated_at,
                    node.id
                ],
            )
            .map_err(|e| e.to_string())?;

        Ok(node)
    }

    pub fn delete_node(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM nodes WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // --- Edges ---

    pub fn list_edges(&self, map_id: &str) -> Result<Vec<Edge>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, map_id, source_node_id, target_node_id, created_at FROM edges WHERE map_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![map_id], |row| {
                Ok(Edge {
                    id: row.get(0)?,
                    map_id: row.get(1)?,
                    source_node_id: row.get(2)?,
                    target_node_id: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn create_edge(&self, req: CreateEdgeRequest) -> Result<Edge, String> {
        let source = self
            .get_node(&req.source_node_id)?
            .ok_or_else(|| "Source node not found".to_string())?;
        let target = self
            .get_node(&req.target_node_id)?
            .ok_or_else(|| "Target node not found".to_string())?;

        if target.node_type != "terminal" {
            return Err("Edges must connect content nodes to a terminal".into());
        }
        if source.node_type == "terminal" {
            return Err("Cannot connect a terminal as source".into());
        }
        if source.map_id != target.map_id || source.map_id != req.map_id {
            return Err("Nodes must belong to the same map".into());
        }

        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO edges (id, map_id, source_node_id, target_node_id, created_at) VALUES (?1,?2,?3,?4,?5)",
                params![id, req.map_id, req.source_node_id, req.target_node_id, created_at],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE") {
                    "Connection already exists".to_string()
                } else {
                    e.to_string()
                }
            })?;

        Ok(Edge {
            id,
            map_id: req.map_id,
            source_node_id: req.source_node_id,
            target_node_id: req.target_node_id,
            created_at,
        })
    }

    pub fn delete_edge(&self, id: &str) -> Result<Option<Edge>, String> {
        let edge: Option<Edge> = self
            .conn
            .query_row(
                "SELECT id, map_id, source_node_id, target_node_id, created_at FROM edges WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Edge {
                        id: row.get(0)?,
                        map_id: row.get(1)?,
                        source_node_id: row.get(2)?,
                        target_node_id: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;

        if edge.is_some() {
            self.conn
                .execute("DELETE FROM edges WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())?;
        }
        Ok(edge)
    }

    pub fn edges_for_terminal(&self, terminal_node_id: &str) -> Result<Vec<Edge>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, map_id, source_node_id, target_node_id, created_at FROM edges WHERE target_node_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![terminal_node_id], |row| {
                Ok(Edge {
                    id: row.get(0)?,
                    map_id: row.get(1)?,
                    source_node_id: row.get(2)?,
                    target_node_id: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn terminals_affected_by_node(&self, node_id: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT target_node_id FROM edges WHERE source_node_id = ?1 OR target_node_id = ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![node_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
}

pub fn app_data_dir() -> Result<PathBuf, String> {
    let base = dirs::data_dir().ok_or_else(|| "Could not resolve data directory".to_string())?;
    let dir = base.join("context-map");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn db_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("context-map.db"))
}

pub fn contexts_dir() -> Result<PathBuf, String> {
    let dir = app_data_dir()?.join("contexts");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn assets_dir() -> Result<PathBuf, String> {
    let dir = app_data_dir()?.join("assets");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}
