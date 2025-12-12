pub mod db;
pub mod email;
pub mod state;

#[cfg(test)]
pub mod test;

use state::AppState;
use std::path::PathBuf;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Google OAuth2 configuration
#[tauri::command]
fn get_oauth_config() -> Result<serde_json::Value, String> {
    // In production, load from secure config
    // For development, return the config values
    let config = serde_json::json!({
        "client_id": "913235410408-c6kuf21o8401g7d52rcos51j2708v6dm.apps.googleusercontent.com",
        "client_secret": "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",
        "redirect_uri": "http://localhost"
    });
    Ok(config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // install default crypto provider for rustls
    let _ = rustls::crypto::ring::default_provider().install_default();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_google_auth::init())
        .setup(|app| {
            let app_data_dir = if cfg!(debug_assertions) {
                // Use target/debug/db for development
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/db")
            } else {
                // Get app data directory
                app.path().app_data_dir()?
            };

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
        .invoke_handler(tauri::generate_handler![greet, get_oauth_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
