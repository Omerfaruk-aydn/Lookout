use crate::capture::{self, screenshot::Rect};
use crate::data_engine::TechnicalSnapshot;
use crate::db::{RegionConfig, RegionRect, ReportRecord, Setting, WatchlistItem};
use crate::error::LookoutError;
use crate::orchestrator::{self, FullAnalysisResult, SentimentResult, VisionResult, WebSearchResult};
use crate::scanner::{self, indices, scheduler};
use crate::AppState;
use base64::Engine;
use tauri::State;

#[tauri::command]
pub async fn find_webull(_state: State<'_, AppState>) -> Result<isize, String> {
    capture::find_webull_window().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn capture_screen(
    _state: State<'_, AppState>,
    hwnd: isize,
    region: Rect,
) -> Result<String, String> {
    capture::validate_hwnd(hwnd).map_err(|e| e.to_string())?;
    let png_data = capture::capture_region(hwnd, region)
        .await
        .map_err(|e| e.to_string())?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
    Ok(b64)
}

#[tauri::command]
pub async fn save_region_config(
    state: State<'_, AppState>,
    chart_area: RegionRect,
    ticker_area: RegionRect,
    price_area: RegionRect,
) -> Result<(), String> {
    let db = state.db.lock().await;
    capture::save_region_config(&db, chart_area, ticker_area, price_area).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_region_config(state: State<'_, AppState>) -> Result<Option<RegionConfig>, String> {
    let db = state.db.lock().await;
    capture::get_region_config(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_analysis(
    state: State<'_, AppState>,
    ticker: String,
    image_base64: Option<String>,
    use_web_search: Option<bool>,
) -> Result<FullAnalysisResult, String> {
    let sidecar_vision_path = state.sidecar_vision_path.clone();
    let sidecar_news_path = state.sidecar_news_path.clone();
    let use_web_search = use_web_search.unwrap_or(true);

    let vision_task: tokio::task::JoinHandle<Result<VisionResult, LookoutError>> =
        if let Some(ref img) = image_base64 {
            let img_clone = img.clone();
            let path = sidecar_vision_path.clone();
            tokio::spawn(async move {
                orchestrator::call_vision_sidecar(&path, &img_clone, "xiaomi/mimo-v2.5").await
            })
        } else {
            tokio::spawn(async { Err(LookoutError::CaptureFailed("No image provided".to_string())) })
        };

    let news_ticker = ticker.clone();
    let news_path = sidecar_news_path.clone();
    let news_task = tokio::spawn(async move {
        orchestrator::call_news_sidecar(&news_path, &news_ticker, 48, "xiaomi/mimo-v2.5").await
    });

    let data_ticker = ticker.clone();
    let data_path = sidecar_vision_path.clone();
    let data_task = tokio::spawn(async move {
        let bars = crate::data_engine::providers::yfinance_provider::fetch_ohlcv(
            &data_path,
            &data_ticker,
            "1y",
            "1d",
        )
        .await?;
        Ok::<TechnicalSnapshot, crate::error::LookoutError>(TechnicalSnapshot::compute(&data_ticker, &bars))
    });

    let web_ticker = ticker.clone();
    let web_path = sidecar_vision_path.clone();
    let web_task: tokio::task::JoinHandle<Result<WebSearchResult, LookoutError>> = if use_web_search {
        tokio::spawn(async move {
            orchestrator::call_web_search_sidecar(&web_path, &web_ticker, "xiaomi/mimo-v2.5").await
        })
    } else {
        tokio::spawn(async { Err(LookoutError::DataProviderError("Web search disabled".to_string())) })
    };

    let (vision_result, news_result, data_result, web_result) =
        tokio::join!(vision_task, news_task, data_task, web_task);

    let vision: Option<VisionResult> = vision_result.ok().and_then(|r| r.ok());
    let sentiment: Option<SentimentResult> = news_result.ok().and_then(|r| r.ok());
    let technical: Option<TechnicalSnapshot> = data_result.ok().and_then(|r: Result<TechnicalSnapshot, crate::error::LookoutError>| r.ok());
    let web_search: Option<WebSearchResult> = web_result.ok().and_then(|r| r.ok());

    let report = orchestrator::synthesize_report(
        vision.as_ref(),
        technical.as_ref(),
        sentiment.as_ref(),
        web_search.as_ref(),
        &ticker,
    )
    .await
    .map_err(|e| e.to_string())?;

    let report_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().timestamp();

    let record = ReportRecord {
        id: report_id.clone(),
        ticker: ticker.clone(),
        created_at,
        vision_result_json: vision.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()),
        technical_snapshot_json: technical.as_ref().map(|t| serde_json::to_string(t).unwrap_or_default()),
        sentiment_result_json: sentiment.as_ref().map(|s| serde_json::to_string(s).unwrap_or_default()),
        web_search_result_json: web_search.as_ref().map(|w| serde_json::to_string(w).unwrap_or_default()),
        synthesis_report_json: serde_json::to_string(&report).unwrap_or_default(),
        confidence_level: report.confidence_level.clone(),
    };

    {
        let db = state.db.lock().await;
        db.save_report(&record).map_err(|e| e.to_string())?;
    }

    Ok(FullAnalysisResult {
        ticker,
        vision_result: vision,
        technical_snapshot: technical,
        sentiment_result: sentiment,
        web_search_result: web_search,
        synthesis_report: report,
        report_id,
        created_at,
    })
}

#[tauri::command]
pub async fn add_to_watchlist(state: State<'_, AppState>, ticker: String) -> Result<(), String> {
    let db = state.db.lock().await;
    db.add_to_watchlist(&ticker).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_from_watchlist(
    state: State<'_, AppState>,
    ticker: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.remove_from_watchlist(&ticker).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_watchlist(state: State<'_, AppState>) -> Result<Vec<WatchlistItem>, String> {
    let db = state.db.lock().await;
    db.get_watchlist().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_reports(
    state: State<'_, AppState>,
    ticker: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<ReportRecord>, String> {
    let db = state.db.lock().await;
    db.get_reports(ticker.as_deref(), limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_report_by_id(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<ReportRecord>, String> {
    let db = state.db.lock().await;
    db.get_report_by_id(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Vec<Setting>, String> {
    let db = state.db.lock().await;
    db.get_all_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.save_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_market_indices(_state: State<'_, AppState>) -> Result<Vec<scanner::engine::MarketIndexInfo>, String> {
    Ok(indices::list_indices()
        .into_iter()
        .map(|(k, v)| scanner::engine::MarketIndexInfo {
            key: k.to_string(),
            label: v.to_string(),
        })
        .collect())
}

#[tauri::command]
pub async fn get_index_symbols(
    _state: State<'_, AppState>,
    index: String,
) -> Result<Vec<String>, String> {
    indices::get_index_symbols(&index).ok_or_else(|| "Unknown index".to_string())
}

#[tauri::command]
pub async fn run_batch_scan(
    state: State<'_, AppState>,
    tickers: Vec<String>,
    use_web_search: Option<bool>,
) -> Result<Vec<scanner::engine::ScanResult>, String> {
    let results = scanner::engine::run_batch_scan(
        state.db.clone(),
        state.sidecar_vision_path.clone(),
        state.sidecar_news_path.clone(),
        tickers,
        use_web_search.unwrap_or(true),
    )
    .await;
    Ok(results)
}

#[tauri::command]
pub async fn start_auto_scanner(
    state: State<'_, AppState>,
    interval_minutes: u64,
    use_web_search: Option<bool>,
) -> Result<(), String> {
    scheduler::start_auto_scanner(
        state.db.clone(),
        state.sidecar_vision_path.clone(),
        state.sidecar_news_path.clone(),
        interval_minutes,
        use_web_search.unwrap_or(true),
    )
    .await
}

#[tauri::command]
pub async fn stop_auto_scanner(_state: State<'_, AppState>) -> Result<(), String> {
    scheduler::stop_auto_scanner();
    Ok(())
}

#[tauri::command]
pub async fn is_scanner_running(_state: State<'_, AppState>) -> Result<bool, String> {
    Ok(scheduler::is_scanner_running())
}

#[tauri::command]
pub async fn add_index_to_watchlist(
    state: State<'_, AppState>,
    index: String,
) -> Result<usize, String> {
    let symbols = indices::get_index_symbols(&index).ok_or_else(|| "Unknown index".to_string())?;
    let count = symbols.len();
    {
        let db = state.db.lock().await;
        for ticker in symbols {
            db.add_to_watchlist(&ticker).map_err(|e| e.to_string())?;
        }
    }
    Ok(count)
}
