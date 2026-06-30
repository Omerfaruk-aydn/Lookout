use crate::data_engine::types::OhlcvBar;
use crate::error::LookoutError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AlpacaBarResponse {
    bars: Option<Vec<AlpacaBar>>,
}

#[derive(Debug, Deserialize)]
struct AlpacaBar {
    t: String,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: u64,
}

pub async fn fetch_ohlcv_alpaca(
    ticker: &str,
    timeframe: &str,
    limit: u32,
) -> Result<Vec<OhlcvBar>, LookoutError> {
    let api_key = std::env::var("ALPACA_API_KEY")
        .map_err(|_| LookoutError::ConfigError("ALPACA_API_KEY not set".to_string()))?;
    let api_secret = std::env::var("ALPACA_API_SECRET")
        .map_err(|_| LookoutError::ConfigError("ALPACA_API_SECRET not set".to_string()))?;

    let url = format!(
        "https://data.alpaca.markets/v2/stocks/{}/bars?timeframe={}&limit={}",
        ticker, timeframe, limit
    );

    let client = reqwest::Client::new();

    let mut retries = 0;
    let max_retries = 3;

    loop {
        let response = client
            .get(&url)
            .header("APCA-API-KEY-ID", &api_key)
            .header("APCA-API-SECRET-KEY", &api_secret)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    retries += 1;
                    if retries >= max_retries {
                        return Err(LookoutError::DataProviderError(
                            "Rate limit exceeded after 3 retries".to_string(),
                        ));
                    }
                    let delay = std::time::Duration::from_millis(1000 * 2u64.pow(retries));
                    tokio::time::sleep(delay).await;
                    continue;
                }

                let data: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| LookoutError::DataProviderError(format!("Failed to parse response: {}", e)))?;

                let bars_data = data
                    .get("bars")
                    .and_then(|b| b.as_array())
                    .ok_or_else(|| LookoutError::DataProviderError("No bars in response".to_string()))?;

                let mut bars = Vec::new();
                for bar in bars_data {
                    let timestamp = bar
                        .get("t")
                        .and_then(|t| t.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.timestamp())
                        .unwrap_or(0);

                    bars.push(OhlcvBar {
                        timestamp,
                        open: bar.get("o").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        high: bar.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        low: bar.get("l").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        close: bar.get("c").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        volume: bar.get("v").and_then(|v| v.as_u64()).unwrap_or(0),
                    });
                }

                return Ok(bars);
            }
            Err(e) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(LookoutError::DataProviderError(format!(
                        "Request failed after 3 retries: {}",
                        e
                    )));
                }
                let delay = std::time::Duration::from_millis(1000 * 2u64.pow(retries));
                tokio::time::sleep(delay).await;
            }
        }
    }
}
