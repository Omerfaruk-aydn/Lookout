mod autonomous;
mod capture;
mod commands;
mod data_engine;
mod db;
mod error;
mod orchestrator;
mod scanner;

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
            commands::get_market_indices,
            commands::get_index_symbols,
            commands::run_batch_scan,
            commands::start_auto_scanner,
            commands::stop_auto_scanner,
            commands::is_scanner_running,
            commands::add_index_to_watchlist,
            commands::start_autonomous_mode,
            commands::stop_autonomous_mode,
            commands::is_autonomous_running,
            commands::get_autonomous_status,
            commands::analyze_ticker_for_alerts,
            commands::batch_analyze_for_signals,
            commands::get_notifications,
            commands::get_unread_count,
            commands::mark_notification_read,
            commands::mark_all_notifications_read,
            commands::clear_old_notifications,
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
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let sidecar_dir = exe_dir.join(name);
    if sidecar_dir.exists() {
        return sidecar_dir.to_string_lossy().to_string();
    }

    let fallback = exe_dir
        .join("../../..")
        .join("src-tauri")
        .join(name);
    if fallback.exists() {
        return fallback.to_string_lossy().to_string();
    }

    sidecar_dir.to_string_lossy().to_string()
}
