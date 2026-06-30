use crate::db::Database;
use crate::scanner::engine::run_batch_scan;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

static SCANNER_RUNNING: AtomicBool = AtomicBool::new(false);

pub async fn start_auto_scanner(
    db: Arc<Mutex<Database>>,
    sidecar_vision_path: String,
    sidecar_news_path: String,
    interval_minutes: u64,
    use_web_search: bool,
) -> Result<(), String> {
    if SCANNER_RUNNING.swap(true, Ordering::SeqCst) {
        return Err("Scanner already running".to_string());
    }

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_minutes * 60));

        loop {
            ticker.tick().await;

            if !SCANNER_RUNNING.load(Ordering::SeqCst) {
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

            let _ = run_batch_scan(
                db.clone(),
                sidecar_vision_path.clone(),
                sidecar_news_path.clone(),
                watchlist,
                use_web_search,
            )
            .await;
        }
    });

    Ok(())
}

pub fn stop_auto_scanner() {
    SCANNER_RUNNING.store(false, Ordering::SeqCst);
}

pub fn is_scanner_running() -> bool {
    SCANNER_RUNNING.load(Ordering::SeqCst)
}
