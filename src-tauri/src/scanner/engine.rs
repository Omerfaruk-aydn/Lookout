use crate::capture;
use crate::capture::screenshot::Rect;
use crate::data_engine::TechnicalSnapshot;
use crate::db::Database;
use crate::error::LookoutError;
use crate::orchestrator::{self, SentimentResult, VisionResult, WebSearchResult};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ScanJob {
    pub ticker: String,
    pub capture_image: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub ticker: String,
    pub success: bool,
    pub report_id: Option<String>,
    pub error: Option<String>,
}

pub async fn run_single_analysis(
    db: Arc<Mutex<Database>>,
    sidecar_vision_path: &str,
    sidecar_news_path: &str,
    ticker: &str,
    hwnd: Option<isize>,
    region: Option<&crate::db::RegionConfig>,
    use_web_search: bool,
) -> Result<orchestrator::FullAnalysisResult, LookoutError> {
    let vision_task: tokio::task::JoinHandle<Result<VisionResult, LookoutError>> =
        if let (Some(hwnd), Some(region)) = (hwnd, region) {
            let hwnd = hwnd;
            let rect = compute_chart_rect(region);
            let path = sidecar_vision_path.to_string();

            tokio::spawn(async move {
                let png = capture::capture_region(hwnd, rect).await?;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
                orchestrator::call_vision_sidecar(&path, &b64, "xiaomi/mimo-v2.5").await
            })
        } else {
            tokio::spawn(async { Err(LookoutError::CaptureFailed("No image provided".to_string())) })
        };

    let news_ticker = ticker.to_string();
    let news_path = sidecar_news_path.to_string();
    let news_task = tokio::spawn(async move {
        orchestrator::call_news_sidecar(&news_path, &news_ticker, 48, "xiaomi/mimo-v2.5").await
    });

    let data_ticker = ticker.to_string();
    let data_path = sidecar_vision_path.to_string();
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

    let web_ticker = ticker.to_string();
    let web_path = sidecar_vision_path.to_string();
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
        ticker,
    )
    .await?;

    let report_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().timestamp();

    let record = crate::db::ReportRecord {
        id: report_id.clone(),
        ticker: ticker.to_string(),
        created_at,
        vision_result_json: vision.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()),
        technical_snapshot_json: technical.as_ref().map(|t| serde_json::to_string(t).unwrap_or_default()),
        sentiment_result_json: sentiment.as_ref().map(|s| serde_json::to_string(s).unwrap_or_default()),
        web_search_result_json: web_search.as_ref().map(|w| serde_json::to_string(w).unwrap_or_default()),
        synthesis_report_json: serde_json::to_string(&report).unwrap_or_default(),
        confidence_level: report.confidence_level.clone(),
    };

    {
        let db = db.lock().await;
        db.save_report(&record)?;
    }

    Ok(orchestrator::FullAnalysisResult {
        ticker: ticker.to_string(),
        vision_result: vision,
        technical_snapshot: technical,
        sentiment_result: sentiment,
        web_search_result: web_search,
        synthesis_report: report,
        report_id,
        created_at,
    })
}

pub async fn run_batch_scan(
    db: Arc<Mutex<Database>>,
    sidecar_vision_path: String,
    sidecar_news_path: String,
    tickers: Vec<String>,
    use_web_search: bool,
) -> Vec<ScanResult> {
    let hwnd = capture::find_webull_window().ok();
    let region = {
        let db = db.lock().await;
        db.get_region_config().ok().flatten()
    };

    let mut results = Vec::new();

    for ticker in tickers {
        let result = run_single_analysis(
            db.clone(),
            &sidecar_vision_path,
            &sidecar_news_path,
            &ticker,
            hwnd,
            region.as_ref(),
            use_web_search,
        )
        .await;

        match result {
            Ok(analysis) => results.push(ScanResult {
                ticker: analysis.ticker.clone(),
                success: true,
                report_id: Some(analysis.report_id),
                error: None,
            }),
            Err(e) => results.push(ScanResult {
                ticker,
                success: false,
                report_id: None,
                error: Some(e.to_string()),
            }),
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    results
}

fn compute_chart_rect(region: &crate::db::RegionConfig) -> Rect {
    Rect {
        x: (region.chart_area.x_pct * 1920.0) as i32,
        y: (region.chart_area.y_pct * 1080.0) as i32,
        width: (region.chart_area.width_pct * 1920.0) as i32,
        height: (region.chart_area.height_pct * 1080.0) as i32,
    }
}
