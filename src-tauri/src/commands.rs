use crate::context::ContextManager;
use crate::db::{assets_dir, db_path, Database};
use crate::models::*;
use crate::pty::SharedPty;
use base64::{engine::general_purpose::STANDARD, Engine};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Database>,
    pub pty: SharedPty,
}

impl AppState {
    pub fn new() -> Result<Self, String> {
        let db = Database::open(&db_path()?)?;
        Ok(Self {
            db: Mutex::new(db),
            pty: Arc::new(crate::pty::PtyManager::new()),
        })
    }
}

// --- Workspaces ---

#[tauri::command]
pub fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<Workspace>, String> {
    state.db.lock().list_workspaces()
}

#[tauri::command]
pub fn create_workspace(
    state: State<'_, AppState>,
    req: CreateWorkspaceRequest,
) -> Result<Workspace, String> {
    state.db.lock().create_workspace(&req.name)
}

#[tauri::command]
pub fn rename_workspace(
    state: State<'_, AppState>,
    req: RenameRequest,
) -> Result<Workspace, String> {
    state.db.lock().rename_workspace(&req.id, &req.name)
}

#[tauri::command]
pub fn delete_workspace(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.lock().delete_workspace(&id)
}

#[tauri::command]
pub fn duplicate_workspace(state: State<'_, AppState>, id: String) -> Result<Workspace, String> {
    state.db.lock().duplicate_workspace(&id)
}

// --- Maps ---

#[tauri::command]
pub fn list_maps(state: State<'_, AppState>, workspace_id: String) -> Result<Vec<Map>, String> {
    state.db.lock().list_maps(&workspace_id)
}

#[tauri::command]
pub fn create_map(state: State<'_, AppState>, req: CreateMapRequest) -> Result<Map, String> {
    state.db.lock().create_map(&req.workspace_id, &req.name)
}

#[tauri::command]
pub fn rename_map(state: State<'_, AppState>, req: RenameRequest) -> Result<Map, String> {
    state.db.lock().rename_map(&req.id, &req.name)
}

#[tauri::command]
pub fn delete_map(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.lock().delete_map(&id)
}

#[tauri::command]
pub fn get_map_snapshot(state: State<'_, AppState>, map_id: String) -> Result<MapSnapshot, String> {
    state.db.lock().get_map_snapshot(&map_id)
}

// --- Nodes ---

#[tauri::command]
pub fn create_node(state: State<'_, AppState>, req: CreateNodeRequest) -> Result<Node, String> {
    state.db.lock().create_node(req)
}

#[tauri::command]
pub fn update_node(state: State<'_, AppState>, req: UpdateNodeRequest) -> Result<Node, String> {
    let db = state.db.lock();
    let node = db.update_node(req)?;
    let terminals = db.terminals_affected_by_node(&node.id)?;
    drop(db);

    // Refresh context files for any terminal that depends on this node
    let db = state.db.lock();
    let _ = ContextManager::rebuild_for_terminals(&db, &terminals);
    Ok(node)
}

#[tauri::command]
pub fn delete_node(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock();
    let terminals = db.terminals_affected_by_node(&id)?;
    let was_terminal = db
        .get_node(&id)?
        .map(|n| n.node_type == "terminal")
        .unwrap_or(false);
    db.delete_node(&id)?;
    drop(db);

    if was_terminal {
        let _ = state.pty.stop(&id);
    } else {
        let db = state.db.lock();
        let _ = ContextManager::rebuild_for_terminals(&db, &terminals);
    }
    Ok(())
}

// --- Edges / Context ---

#[tauri::command]
pub fn create_edge(state: State<'_, AppState>, req: CreateEdgeRequest) -> Result<Edge, String> {
    let db = state.db.lock();
    let edge = db.create_edge(req)?;
    let _ = ContextManager::rebuild_for_terminal(&db, &edge.target_node_id)?;
    Ok(edge)
}

#[tauri::command]
pub fn delete_edge(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock();
    if let Some(edge) = db.delete_edge(&id)? {
        let _ = ContextManager::rebuild_for_terminal(&db, &edge.target_node_id)?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_active_context(
    state: State<'_, AppState>,
    terminal_node_id: String,
) -> Result<ActiveContext, String> {
    let db = state.db.lock();
    ContextManager::rebuild_for_terminal(&db, &terminal_node_id)
}

// --- Assets ---

#[tauri::command]
pub fn save_asset(
    state: State<'_, AppState>,
    filename: String,
    data_base64: String,
) -> Result<String, String> {
    let _ = state;
    let dir = assets_dir()?;
    let safe_name = filename
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let unique = format!("{}_{}", uuid::Uuid::new_v4(), safe_name);
    let path = dir.join(&unique);
    let bytes = STANDARD
        .decode(data_base64.split(',').last().unwrap_or(&data_base64))
        .map_err(|e| e.to_string())?;
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn read_asset_data_url(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let mime = if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    };
    Ok(format!(
        "data:{mime};base64,{}",
        STANDARD.encode(bytes)
    ))
}

// --- PTY ---

#[tauri::command]
pub fn pty_start(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    req: PtyStartRequest,
) -> Result<String, String> {
    let context = {
        let db = state.db.lock();
        ContextManager::rebuild_for_terminal(&db, &req.terminal_node_id)?
    };

    let session = ContextManager::session_dir(&req.terminal_node_id)?;
    let rcfile = session.join(".cm_bashrc");

    state.pty.start(
        app,
        req.terminal_node_id.clone(),
        req.cols,
        req.rows,
        Some(session.to_string_lossy().to_string()),
        Some(context.context_file_path.clone()),
        Some(rcfile.to_string_lossy().to_string()),
    )?;

    Ok(context.context_file_path)
}

#[tauri::command]
pub fn pty_write(state: State<'_, AppState>, req: PtyWriteRequest) -> Result<(), String> {
    state.pty.write(&req.terminal_node_id, &req.data)
}

#[tauri::command]
pub fn pty_resize(state: State<'_, AppState>, req: PtyResizeRequest) -> Result<(), String> {
    state.pty.resize(&req.terminal_node_id, req.cols, req.rows)
}

#[tauri::command]
pub fn pty_stop(state: State<'_, AppState>, terminal_node_id: String) -> Result<(), String> {
    state.pty.stop(&terminal_node_id)
}

#[tauri::command]
pub fn pty_is_running(state: State<'_, AppState>, terminal_node_id: String) -> Result<bool, String> {
    Ok(state.pty.is_running(&terminal_node_id))
}
