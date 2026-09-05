mod commands;
mod context;
mod db;
mod models;
mod pty;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new().expect("failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::list_workspaces,
            commands::create_workspace,
            commands::rename_workspace,
            commands::delete_workspace,
            commands::duplicate_workspace,
            commands::list_maps,
            commands::create_map,
            commands::rename_map,
            commands::delete_map,
            commands::get_map_snapshot,
            commands::create_node,
            commands::update_node,
            commands::delete_node,
            commands::create_edge,
            commands::delete_edge,
            commands::get_active_context,
            commands::save_asset,
            commands::read_asset_data_url,
            commands::pty_start,
            commands::pty_write,
            commands::pty_resize,
            commands::pty_stop,
            commands::pty_is_running,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
