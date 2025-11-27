pub mod db;
pub mod email;
pub mod state;

use state::AppState;
use tauri::Manager;

// // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Get app data directory
            let app_data_dir = app.path().app_data_dir()?;

            // Ensure the directory exists
            std::fs::create_dir_all(&app_data_dir)?;

            // Initialize AppState asynchronously
            let state =
                tauri::async_runtime::block_on(async { AppState::new(app_data_dir).await })?;

            // Run database migrations
            tauri::async_runtime::block_on(async { state.db.migrate().await })?;

            // Manage the state in Tauri
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
