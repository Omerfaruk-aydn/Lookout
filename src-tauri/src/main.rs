mod capture;
mod commands;
mod data_engine;
mod db;
mod error;
mod orchestrator;

use db::Database;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub cached_hwnd: Arc<Mutex<Option<isize>>>,
    pub sidecar_vision_path: String,
    pub sidecar_news_path: String,
}

fn main() {
    env_logger::init();
    dotenvy::dotenv().ok();

    let db_path = get_db_path();
    let db = Database::new(&db_path).expect("Failed to initialize database");
    db.run_migrations().expect("Failed to run migrations");

    let sidecar_vision_path = get_sidecar_path("sidecar-vision");
    let sidecar_news_path = get_sidecar_path("sidecar-news");

    let state = AppState {
        db: Arc::new(Mutex::new(db)),
        cached_hwnd: Arc::new(Mutex::new(None)),
        sidecar_vision_path,
        sidecar_news_path,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::find_webull,
            commands::capture_screen,
            commands::save_region_config,
            commands::get_region_config,
            commands::run_analysis,
            commands::add_to_watchlist,
            commands::remove_from_watchlist,
            commands::get_watchlist,
            commands::get_reports,
            commands::get_report_by_id,
            commands::get_settings,
            commands::save_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lookout");
}

fn get_db_path() -> String {
    let app_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Lookout");
    std::fs::create_dir_all(&app_dir).ok();
    app_dir.join("lookout.db").to_string_lossy().to_string()
}

fn get_sidecar_path(name: &str) -> String {
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("src-tauri")
        .join(name)
        .to_string_lossy()
        .to_string()
}
