use crate::data_engine::types::OhlcvBar;
use crate::error::LookoutError;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Serialize)]
struct YfinanceRequest {
    ticker: String,
    period: String,
    interval: String,
}

#[derive(Debug, Deserialize)]
struct YfinanceResponse {
    success: bool,
    data: Option<Vec<OhlcvBarRaw>>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OhlcvBarRaw {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: u64,
}

pub async fn fetch_ohlcv(
    sidecar_path: &str,
    ticker: &str,
    period: &str,
    interval: &str,
) -> Result<Vec<OhlcvBar>, LookoutError> {
    let script_path = format!("{}/yfinance_fetch.py", sidecar_path);

    let mut child = Command::new("python")
        .arg(&script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| LookoutError::DataProviderError(format!("Failed to spawn yfinance sidecar: {}", e)))?;

    let request = YfinanceRequest {
        ticker: ticker.to_string(),
        period: period.to_string(),
        interval: interval.to_string(),
    };

    let request_json = serde_json::to_string(&request)?;

    let stdin = child
        .stdin
        .as_mut()
        .ok_or_else(|| LookoutError::SidecarError("Failed to open stdin".to_string()))?;

    stdin
        .write_all(request_json.as_bytes())
        .await
        .map_err(|e| LookoutError::SidecarError(format!("Failed to write to stdin: {}", e)))?;
    stdin
        .write_all(b"\n")
        .await
        .map_err(|e| LookoutError::SidecarError(format!("Failed to write newline: {}", e)))?;

    drop(child.stdin.take());

    let stdout = child
        .stdout
        .as_mut()
        .ok_or_else(|| LookoutError::SidecarError("Failed to open stdout".to_string()))?;

    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| LookoutError::SidecarError(format!("Failed to read stdout: {}", e)))?;

    let response: YfinanceResponse = serde_json::from_str(&line)
        .map_err(|e| LookoutError::SchemaValidationError(format!("Invalid yfinance response: {}", e)))?;

    if !response.success {
        return Err(LookoutError::DataProviderError(
            response.error.unwrap_or_else(|| "Unknown error".to_string()),
        ));
    }

    let bars = response
        .data
        .ok_or_else(|| LookoutError::DataProviderError("No data returned".to_string()))?
        .into_iter()
        .map(|raw| OhlcvBar {
            timestamp: raw.timestamp,
            open: raw.open,
            high: raw.high,
            low: raw.low,
            close: raw.close,
            volume: raw.volume,
        })
        .collect();

    Ok(bars)
}
