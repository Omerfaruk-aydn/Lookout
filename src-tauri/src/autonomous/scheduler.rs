use crate::autonomous::alerts::{self, Alert};
use crate::autonomous::notifications::NotificationStore;
use crate::data_engine::types::OhlcvBar;
use crate::db::Database;
use crate::scanner::engine::run_single_analysis;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

static AUTONOMOUS_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousStatus {
    pub running: bool,
    pub last_scan_at: Option<i64>,
    pub total_scans: i64,
    pub total_alerts: i64,
    pub tickers_monitored: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketActivity {
    pub ticker: String,
    pub activity_score: f64,
    pub price_change_pct: f64,
    pub volume_change_pct: f64,
    pub last_close: f64,
    pub signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DailySummary {
    pub date: String,
    pub tickers_scanned: usize,
    pub alerts_generated: usize,
    pub critical_alerts: usize,
    pub top_movers: Vec<MarketActivity>,
    pub market_sentiment: String,
}

fn get_notifications_conn(db_path: &str) -> Result<Connection, String> {
    Connection::open(db_path).map_err(|e| e.to_string())
}

pub async fn start_autonomous_mode(
    db: Arc<Mutex<Database>>,
    db_path: String,
    sidecar_vision_path: String,
    sidecar_news_path: String,
    interval_seconds: u64,
    use_web_search: bool,
) -> Result<(), String> {
    if AUTONOMOUS_RUNNING.swap(true, Ordering::SeqCst) {
        return Err("Autonomous mode already running".to_string());
    }

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_seconds));
        let mut scan_count: i64 = 0;
        let mut alert_count: i64 = 0;

        loop {
            ticker.tick().await;

            if !AUTONOMOUS_RUNNING.load(Ordering::SeqCst) {
                break;
            }

            let watchlist = {
                match db.lock().await.get_watchlist() {
                    Ok(items) => items.into_iter().map(|i| i.ticker).collect::<Vec<_>>(),
                    Err(_) => vec![],
                }
            };

            if watchlist.is_empty() {
                continue;
            }

            let hwnd = crate::capture::find_webull_window().ok();
            let region = {
                let db_guard = db.lock().await;
                db_guard.get_region_config().ok().flatten()
            };

            let mut all_alerts: Vec<Alert> = Vec::new();
            let mut activities: Vec<MarketActivity> = Vec::new();

            for ticker in &watchlist {
                let analysis = run_single_analysis(
                    db.clone(),
                    &sidecar_vision_path,
                    &sidecar_news_path,
                    ticker,
                    hwnd,
                    region.as_ref(),
                    use_web_search,
                )
                .await;

                match analysis {
                    Ok(result) => {
                        scan_count += 1;

                        if let Some(ref snapshot) = result.technical_snapshot {
                            let bars = load_bars_from_snapshot(snapshot);
                            let alert_list = alerts::evaluate_alerts(snapshot, &bars);
                            all_alerts.extend(alert_list);

                            let activity_score = alerts::compute_activity_score(&bars);
                            let (price_change, vol_change, last_close) = compute_changes(&bars);
                            let signals = extract_signals(snapshot);

                            activities.push(MarketActivity {
                                ticker: ticker.clone(),
                                activity_score,
                                price_change_pct: price_change,
                                volume_change_pct: vol_change,
                                last_close,
                                signals,
                            });
                        }
                    }
                    Err(_) => {}
                }

                tokio::time::sleep(Duration::from_secs(2)).await;
            }

            if !all_alerts.is_empty() {
                alert_count += all_alerts.len() as i64;
                if let Ok(conn) = get_notifications_conn(&db_path) {
                    let store = NotificationStore::new(conn);
                    let _ = store.push_alerts(&all_alerts);
                }
            }

            let _ = scan_count;
            let _ = alert_count;
            let _ = activities;
        }
    });

    Ok(())
}

pub fn stop_autonomous_mode() {
    AUTONOMOUS_RUNNING.store(false, Ordering::SeqCst);
}

pub fn is_autonomous_running() -> bool {
    AUTONOMOUS_RUNNING.load(Ordering::SeqCst)
}

fn load_bars_from_snapshot(_snapshot: &crate::data_engine::types::TechnicalSnapshot) -> Vec<OhlcvBar> {
    Vec::new()
}

fn compute_changes(bars: &[OhlcvBar]) -> (f64, f64, f64) {
    if bars.len() < 2 {
        return (0.0, 0.0, 0.0);
    }

    let last = &bars[bars.len() - 1];
    let prev = &bars[bars.len() - 2];

    let price_change = if prev.close > 0.0 {
        ((last.close - prev.close) / prev.close) * 100.0
    } else {
        0.0
    };

    let vol_change = if prev.volume > 0 {
        ((last.volume as f64 - prev.volume as f64) / prev.volume as f64) * 100.0
    } else {
        0.0
    };

    (price_change, vol_change, last.close)
}

fn extract_signals(snapshot: &crate::data_engine::types::TechnicalSnapshot) -> Vec<String> {
    let mut signals = Vec::new();

    if let Some(rsi) = snapshot.rsi_14 {
        if rsi <= 30.0 {
            signals.push("RSI Oversold".to_string());
        } else if rsi >= 70.0 {
            signals.push("RSI Overbought".to_string());
        }
    }

    if let Some(ref macd) = snapshot.macd {
        if macd.histogram > 0.0 {
            signals.push("MACD Bullish".to_string());
        } else {
            signals.push("MACD Bearish".to_string());
        }
    }

    if let Some(ref breakout) = snapshot.breakout_signal {
        signals.push(format!("{} Breakout", breakout.direction));
    }

    signals
}
