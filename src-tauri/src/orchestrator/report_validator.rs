use super::SynthesisReport;
use crate::error::LookoutError;
use regex::Regex;

const DISCLAIMER: &str = "Bu rapor bilgilendirme amaçlıdır, yatırım tavsiyesi niteliği taşımaz.";

const FORBIDDEN_PATTERNS: &[&str] = &[
    r"(?i)\bal\b",
    r"(?i)\bsat\b",
    r"(?i)\bbuy\b",
    r"(?i)\bsell\b",
    r"(?i)şimdi gir",
    r"(?i)şimdi çık",
    r"(?i)enter now",
    r"(?i)exit now",
    r"(?i)girmelisin",
    r"(?i)çıkmalısın",
    r"(?i)you should buy",
    r"(?i)you should sell",
];

pub fn validate_report(raw_json: &str) -> Result<SynthesisReport, LookoutError> {
    let cleaned = raw_json.trim();
    let cleaned = if cleaned.starts_with("```") {
        let lines: Vec<&str> = cleaned.lines().collect();
        if lines.len() >= 3 {
            lines[1..lines.len() - 1].join("\n")
        } else {
            cleaned.to_string()
        }
    } else {
        cleaned.to_string()
    };

    let report: SynthesisReport = serde_json::from_str(&cleaned).map_err(|e| {
        LookoutError::SchemaValidationError(format!("Failed to parse synthesis report: {}", e))
    })?;

    if report.summary.is_empty() {
        return Err(LookoutError::SchemaValidationError(
            "Report summary is empty".to_string(),
        ));
    }

    if !["high", "medium", "low"].contains(&report.confidence_level.as_str()) {
        return Err(LookoutError::SchemaValidationError(format!(
            "Invalid confidence_level: {}",
            report.confidence_level
        )));
    }

    Ok(report)
}

pub fn enforce_disclaimer(report: &mut SynthesisReport) {
    let fields_to_check = [
        &report.summary,
        &report.technical_status,
        &report.news_impact,
        &report.risk_notes,
    ];

    let mut has_forbidden = false;

    for field in &fields_to_check {
        for pattern in FORBIDDEN_PATTERNS {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(field) {
                    has_forbidden = true;
                    log::warn!(
                        "Forbidden investment advice pattern detected: '{}' in field",
                        pattern
                    );
                    break;
                }
            }
        }
        if has_forbidden {
            break;
        }
    }

    if has_forbidden {
        report.risk_notes = format!(
            "UYARI: Bu raporda yatırım tavsiyesi niteliğinde ifadeler tespit edilmiş ve nötrleştirilmiştir. {}",
            report.risk_notes
        );
    }

    if !report.summary.ends_with(DISCLAIMER) {
        report.summary = format!("{} — {}", report.summary, DISCLAIMER);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_report() {
        let json = r#"{
            "ticker": "AAPL",
            "summary": "Apple shows bullish momentum.",
            "technical_status": "RSI at 62, above SMA20.",
            "news_impact": "Positive earnings report.",
            "conflicting_signals": null,
            "risk_notes": "Market volatility expected.",
            "confidence_level": "high"
        }"#;

        let result = validate_report(json);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.ticker, "AAPL");
        assert_eq!(report.confidence_level, "high");
    }

    #[test]
    fn test_validate_invalid_confidence() {
        let json = r#"{
            "ticker": "AAPL",
            "summary": "Test summary.",
            "technical_status": "Test.",
            "news_impact": "Test.",
            "conflicting_signals": null,
            "risk_notes": "Test.",
            "confidence_level": "very_high"
        }"#;

        let result = validate_report(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_summary() {
        let json = r#"{
            "ticker": "AAPL",
            "summary": "",
            "technical_status": "Test.",
            "news_impact": "Test.",
            "conflicting_signals": null,
            "risk_notes": "Test.",
            "confidence_level": "high"
        }"#;

        let result = validate_report(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_json() {
        let json = "not valid json";
        let result = validate_report(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_enforce_disclaimer_adds_when_missing() {
        let mut report = SynthesisReport {
            ticker: "AAPL".to_string(),
            summary: "Apple shows bullish momentum.".to_string(),
            technical_status: "RSI at 62.".to_string(),
            news_impact: "Positive.".to_string(),
            conflicting_signals: None,
            risk_notes: "Volatility.".to_string(),
            confidence_level: "high".to_string(),
        };

        enforce_disclaimer(&mut report);
        assert!(report.summary.contains(DISCLAIMER));
    }

    #[test]
    fn test_enforce_disclaimer_detects_forbidden() {
        let mut report = SynthesisReport {
            ticker: "AAPL".to_string(),
            summary: "Şimdi al zamanı.".to_string(),
            technical_status: "RSI at 62.".to_string(),
            news_impact: "Positive.".to_string(),
            conflicting_signals: None,
            risk_notes: "None.".to_string(),
            confidence_level: "high".to_string(),
        };

        enforce_disclaimer(&mut report);
        assert!(report.risk_notes.contains("UYARI"));
        assert!(report.summary.contains(DISCLAIMER));
    }

    #[test]
    fn test_enforce_disclaimer_english_forbidden() {
        let mut report = SynthesisReport {
            ticker: "AAPL".to_string(),
            summary: "You should buy now.".to_string(),
            technical_status: "Strong.".to_string(),
            news_impact: "Positive.".to_string(),
            conflicting_signals: None,
            risk_notes: "None.".to_string(),
            confidence_level: "medium".to_string(),
        };

        enforce_disclaimer(&mut report);
        assert!(report.risk_notes.contains("UYARI"));
    }

    #[test]
    fn test_validate_with_markdown_fence() {
        let json = "```json\n{\"ticker\":\"AAPL\",\"summary\":\"Test.\",\"technical_status\":\"T.\",\"news_impact\":\"N.\",\"conflicting_signals\":null,\"risk_notes\":\"R.\",\"confidence_level\":\"low\"}\n```";
        let result = validate_report(json);
        assert!(result.is_ok());
    }
}
