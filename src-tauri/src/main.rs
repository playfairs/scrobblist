#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod lastfm;
mod state;

use database::Database;
use state::AppState;

fn default_database_path() -> String {
    #[cfg(target_os = "windows")]
    {
        let app_data = std::env::var("APPDATA").unwrap_or_else(|_| "./.scrobblist".to_string());
        format!(
            "{}/Scrobblist/scrobblist.db",
            app_data.trim_end_matches('\\')
        )
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "./.scrobblist".to_string());
        format!(
            "{}/Library/Application Support/Scrobblist/scrobblist.db",
            home
        )
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "./.scrobblist".to_string());
        format!("{}/.local/share/scrobblist/scrobblist.db", home)
    }
}

#[tokio::main]
async fn main() {
    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let root_env = project_root.join(".env");

    if root_env.exists() {
        dotenv::from_filename(root_env).ok();
    } else {
        dotenv::dotenv().ok();
    }

    let db_path = default_database_path();
    let db = Database::new(&db_path)
        .await
        .expect("Failed to initialize database");
    let state = AppState::new(db);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::auth::get_auth_url,
            commands::auth::start_lastfm_auth,
            commands::auth::complete_auth,
            commands::auth::handle_auth_callback,
            commands::auth::get_session,
            commands::user::get_profile,
            commands::user::get_recent_tracks,
            commands::user::get_top_items,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
