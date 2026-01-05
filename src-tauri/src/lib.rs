pub mod db;
pub mod email;
pub mod state;

#[cfg(test)]
pub mod test;

use state::AppState;
use std::path::PathBuf;
use tauri::Manager;
use keyring::Entry;

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

// Keyring token storage commands
#[derive(serde::Serialize, serde::Deserialize)]
struct OAuthTokens {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    expires_at: Option<i64>,
}

#[tauri::command]
fn store_oauth_tokens(
    user_email: String,
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    expires_at: Option<i64>,
) -> Result<(), String> {
    let tokens = OAuthTokens {
        access_token,
        refresh_token,
        id_token,
        expires_at,
    };

    let tokens_json = serde_json::to_string(&tokens)
        .map_err(|e| format!("Failed to serialize tokens: {}", e))?;

    let entry = Entry::new("meeru", &user_email)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry.set_password(&tokens_json)
        .map_err(|e| format!("Failed to store tokens in keyring: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_oauth_tokens(user_email: String) -> Result<OAuthTokens, String> {
    let entry = Entry::new("meeru", &user_email)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    let tokens_json = entry.get_password()
        .map_err(|e| format!("Failed to retrieve tokens from keyring: {}", e))?;

    let tokens: OAuthTokens = serde_json::from_str(&tokens_json)
        .map_err(|e| format!("Failed to deserialize tokens: {}", e))?;

    Ok(tokens)
}

#[tauri::command]
fn delete_oauth_tokens(user_email: String) -> Result<(), String> {
    let entry = Entry::new("meeru", &user_email)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    // Try to delete the credential. If it doesn't exist, that's fine -
    // the goal is to ensure no tokens are stored, which is already the case.
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => {
            // Entry doesn't exist, which is fine
            Ok(())
        }
        Err(e) => Err(format!("Failed to delete tokens from keyring: {}", e))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // install default crypto provider for rustls
    let _ = rustls::crypto::ring::default_provider().install_default();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_google_auth::init())
        .setup(|app| {
            //
            // Handle database migrations
            //
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
        .invoke_handler(tauri::generate_handler![
            greet,
            get_oauth_config,
            store_oauth_tokens,
            get_oauth_tokens,
            delete_oauth_tokens
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
