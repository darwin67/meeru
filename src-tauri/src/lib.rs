pub mod accounts;
pub mod db;
pub mod email;

use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

use accounts::AccountManager;
use db::Database;
use email::EmailSyncService;

// Application state
pub struct AppState {
    pub db: Database,
    pub account_manager: AccountManager,
    pub sync_service: EmailSyncService,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn list_accounts(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<db::models::Account>, String> {
    let state = state.lock().await;
    state
        .account_manager
        .list_accounts()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_account(
    email: String,
    name: Option<String>,
    provider: String,
    imap_host: String,
    imap_port: u16,
    smtp_host: String,
    smtp_port: u16,
    auth_type: String,
    password: Option<String>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<db::models::Account, String> {
    let state = state.lock().await;

    let auth_type = auth_type
        .parse::<db::models::AuthType>()
        .map_err(|e| e.to_string())?;

    state
        .account_manager
        .create_account(
            email, name, provider, imap_host, imap_port, smtp_host, smtp_port, auth_type, password,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_account(
    account_id: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().await;
    state
        .account_manager
        .delete_account(&account_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_account(
    account_id: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let state = state.lock().await;
    let result = state
        .sync_service
        .sync_account(&account_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "Synced {} messages ({} new)",
        result.total_messages, result.new_messages
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Get app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Initialize database
            let db_path = app_data_dir.join("meeru.db");

            tauri::async_runtime::block_on(async move {
                let db = Database::new(db_path)
                    .await
                    .expect("Failed to initialize database");

                let account_manager = AccountManager::new(db.pool().clone());
                let sync_service = EmailSyncService::new(db.pool().clone());

                let state = Arc::new(Mutex::new(AppState {
                    db,
                    account_manager,
                    sync_service,
                }));

                app.manage(state);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            list_accounts,
            create_account,
            delete_account,
            sync_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
