pub mod prompt_builder;
pub mod report_validator;

use crate::data_engine::TechnicalSnapshot;
use crate::error::LookoutError;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionResult {
    pub ticker_visible: Option<String>,
    pub trend_direction: Option<String>,
    pub visible_patterns: Vec<String>,
    pub support_resistance_estimate: SupportResistanceEstimate,
    pub volume_observation: Option<String>,
    pub indicators_visible: Vec<IndicatorObservation>,
    pub confidence: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportResistanceEstimate {
    pub support: Vec<f64>,
    pub resistance: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorObservation {
    pub name: String,
    pub value_estimate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    pub ticker: String,
    pub overall_sentiment: String,
    pub weighted_score: f64,
    pub item_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisReport {
    pub ticker: String,
    pub summary: String,
    pub technical_status: String,
    pub news_impact: String,
    pub conflicting_signals: Option<String>,
    pub risk_notes: String,
    pub confidence_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullAnalysisResult {
    pub ticker: String,
    pub vision_result: Option<VisionResult>,
    pub technical_snapshot: Option<TechnicalSnapshot>,
    pub sentiment_result: Option<SentimentResult>,
    pub synthesis_report: SynthesisReport,
    pub report_id: String,
    pub created_at: i64,
}

pub async fn call_vision_sidecar(
    sidecar_path: &str,
    image_base64: &str,
    model: &str,
) -> Result<VisionResult, LookoutError> {
    let script_path = format!("{}/vision_client.py", sidecar_path);
    let request_id = uuid::Uuid::new_v4().to_string();

    let request = serde_json::json!({
        "image_base64": image_base64,
        "request_id": request_id,
        "model": model,
    });

    let response = run_sidecar(&script_path, &request.to_string()).await?;
    let parsed: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| LookoutError::SchemaValidationError(format!("Vision response: {}", e)))?;

    if parsed["success"].as_bool().unwrap_or(false) {
        let data = parsed["data"].clone();
        let vision: VisionResult = serde_json::from_value(data)
            .map_err(|e| LookoutError::SchemaValidationError(format!("Vision data: {}", e)))?;
        Ok(vision)
    } else {
        let error = parsed["error"]
            .as_str()
            .unwrap_or("Unknown vision error")
            .to_string();
        Err(LookoutError::VisionApiError(error))
    }
}

pub async fn call_news_sidecar(
    sidecar_path: &str,
    ticker: &str,
    hours_back: i32,
    model: &str,
) -> Result<SentimentResult, LookoutError> {
    let script_path = format!("{}/news_client.py", sidecar_path);
    let request_id = uuid::Uuid::new_v4().to_string();

    let request = serde_json::json!({
        "ticker": ticker,
        "hours_back": hours_back,
        "request_id": request_id,
        "model": model,
    });

    let response = run_sidecar(&script_path, &request.to_string()).await?;
    let parsed: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| LookoutError::SchemaValidationError(format!("News response: {}", e)))?;

    if parsed["success"].as_bool().unwrap_or(false) {
        let data = parsed["data"].clone();
        let sentiment: SentimentResult = serde_json::from_value(data)
            .map_err(|e| LookoutError::SchemaValidationError(format!("Sentiment data: {}", e)))?;
        Ok(sentiment)
    } else {
        let error = parsed["error"]
            .as_str()
            .unwrap_or("Unknown news error")
            .to_string();
        Err(LookoutError::DataProviderError(error))
    }
}

pub async fn synthesize_report(
    vision: Option<&VisionResult>,
    technical: Option<&TechnicalSnapshot>,
    sentiment: Option<&SentimentResult>,
    ticker: &str,
) -> Result<SynthesisReport, LookoutError> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .map_err(|_| LookoutError::ConfigError("OPENROUTER_API_KEY not set".to_string()))?;

    let prompt = prompt_builder::build_synthesis_prompt(vision, technical, sentiment);

    let client = reqwest::Client::new();
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "anthropic/claude-sonnet-4-6",
            "messages": [
                {"role": "system", "content": prompt_builder::SYSTEM_PROMPT},
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 2048,
            "temperature": 0.3
        }))
        .send()
        .await
        .map_err(|e| LookoutError::VisionApiError(format!("Synthesis API call failed: {}", e)))?;

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| LookoutError::VisionApiError(format!("Failed to parse synthesis response: {}", e)))?;

    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| LookoutError::SchemaValidationError("No content in synthesis response".to_string()))?;

    let mut report = report_validator::validate_report(content)?;
    report_validator::enforce_disclaimer(&mut report);

    if report.ticker.is_empty() {
        report.ticker = ticker.to_string();
    }

    Ok(report)
}

async fn run_sidecar(script_path: &str, input: &str) -> Result<String, LookoutError> {
    let mut child = Command::new("python")
        .arg(script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| LookoutError::SidecarError(format!("Failed to spawn sidecar: {}", e)))?;

    let stdin = child
        .stdin
        .as_mut()
        .ok_or_else(|| LookoutError::SidecarError("Failed to open stdin".to_string()))?;

    stdin
        .write_all(input.as_bytes())
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

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        reader.read_line(&mut line),
    )
    .await;

    match result {
        Ok(Ok(_)) => Ok(line.trim().to_string()),
        Ok(Err(e)) => Err(LookoutError::SidecarError(format!(
            "Failed to read stdout: {}",
            e
        ))),
        Err(_) => Err(LookoutError::TimeoutError(
            "Sidecar response timed out (30s)".to_string(),
        )),
    }
}
