use crate::data_engine::types::{OhlcvBar, TechnicalSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    RsiOversold,
    RsiOverbought,
    MacdBullishCross,
    MacdBearishCross,
    BreakoutBullish,
    BreakoutBearish,
    VolumeSpike,
    BollingerSqueeze,
    BollingerBreakoutUp,
    BollingerBreakoutDown,
    SmGoldenCross,
    SmDeathCross,
    PriceAboveSma200,
    PriceBelowSma200,
    HighVolatility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub ticker: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub value: Option<f64>,
    pub threshold: Option<f64>,
    pub created_at: i64,
    pub acknowledged: bool,
}

impl Alert {
    fn new(
        ticker: &str,
        alert_type: AlertType,
        severity: AlertSeverity,
        title: &str,
        message: &str,
        value: Option<f64>,
        threshold: Option<f64>,
    ) -> Self {
        Alert {
            id: uuid::Uuid::new_v4().to_string(),
            ticker: ticker.to_string(),
            alert_type,
            severity,
            title: title.to_string(),
            message: message.to_string(),
            value,
            threshold,
            created_at: chrono::Utc::now().timestamp(),
            acknowledged: false,
        }
    }
}

pub fn evaluate_alerts(snapshot: &TechnicalSnapshot, bars: &[OhlcvBar]) -> Vec<Alert> {
    let mut alerts = Vec::new();

    if let Some(rsi) = snapshot.rsi_14 {
        if rsi <= 30.0 {
            alerts.push(Alert::new(
                &snapshot.ticker,
                AlertType::RsiOversold,
                AlertSeverity::High,
                "RSI Oversold",
                &format!("RSI(14) = {:.1} — potential buy signal (oversold territory)", rsi),
                Some(rsi),
                Some(30.0),
            ));
        } else if rsi >= 70.0 {
            alerts.push(Alert::new(
                &snapshot.ticker,
                AlertType::RsiOverbought,
                AlertSeverity::High,
                "RSI Overbought",
                &format!("RSI(14) = {:.1} — potential sell signal (overbought territory)", rsi),
                Some(rsi),
                Some(70.0),
            ));
        }
    }

    if let Some(ref macd) = snapshot.macd {
        if macd.histogram > 0.0 && macd.macd_line > macd.signal_line {
            alerts.push(Alert::new(
                &snapshot.ticker,
                AlertType::MacdBullishCross,
                AlertSeverity::Medium,
                "MACD Bullish Signal",
                &format!(
                    "MACD line ({:.4}) above signal line ({:.4}), histogram = {:.4}",
                    macd.macd_line, macd.signal_line, macd.histogram
                ),
                Some(macd.histogram),
                None,
            ));
        } else if macd.histogram < 0.0 && macd.macd_line < macd.signal_line {
            alerts.push(Alert::new(
                &snapshot.ticker,
                AlertType::MacdBearishCross,
                AlertSeverity::Medium,
                "MACD Bearish Signal",
                &format!(
                    "MACD line ({:.4}) below signal line ({:.4}), histogram = {:.4}",
                    macd.macd_line, macd.signal_line, macd.histogram
                ),
                Some(macd.histogram),
                None,
            ));
        }
    }

    if let Some(ref breakout) = snapshot.breakout_signal {
        if breakout.direction == "bullish" {
            alerts.push(Alert::new(
                &snapshot.ticker,
                AlertType::BreakoutBullish,
                AlertSeverity::Critical,
                "Bullish Breakout Detected",
                &format!(
                    "Price broke above resistance at {:.2} (volume confirmed: {})",
                    breakout.level, breakout.volume_confirmation
                ),
                Some(breakout.level),
                None,
            ));
        } else if breakout.direction == "bearish" {
            alerts.push(Alert::new(
                &snapshot.ticker,
                AlertType::BreakoutBearish,
                AlertSeverity::Critical,
                "Bearish Breakdown Detected",
                &format!(
                    "Price broke below support at {:.2} (volume confirmed: {})",
                    breakout.level, breakout.volume_confirmation
                ),
                Some(breakout.level),
                None,
            ));
        }
    }

    if let Some(ref vol) = snapshot.volume_anomaly {
        if vol.z_score >= 3.0 {
            alerts.push(Alert::new(
                &snapshot.ticker,
                AlertType::VolumeSpike,
                AlertSeverity::High,
                "Volume Spike Detected",
                &format!(
                    "Current volume {} vs avg {:.0} (z-score: {:.2})",
                    vol.current_volume, vol.average_volume, vol.z_score
                ),
                Some(vol.z_score),
                Some(3.0),
            ));
        }
    }

    if let Some(ref bb) = snapshot.bollinger {
        if let Some(last_bar) = bars.last() {
            let band_width = (bb.upper - bb.lower) / bb.middle;
            if band_width < 0.05 {
                alerts.push(Alert::new(
                    &snapshot.ticker,
                    AlertType::BollingerSqueeze,
                    AlertSeverity::Medium,
                    "Bollinger Band Squeeze",
                    &format!("Band width = {:.4} — volatility contraction, breakout imminent", band_width),
                    Some(band_width),
                    Some(0.05),
                ));
            }

            if last_bar.close > bb.upper {
                alerts.push(Alert::new(
                    &snapshot.ticker,
                    AlertType::BollingerBreakoutUp,
                    AlertSeverity::Medium,
                    "Price Above Upper Bollinger Band",
                    &format!("Close {:.2} > Upper band {:.2}", last_bar.close, bb.upper),
                    Some(last_bar.close),
                    Some(bb.upper),
                ));
            } else if last_bar.close < bb.lower {
                alerts.push(Alert::new(
                    &snapshot.ticker,
                    AlertType::BollingerBreakoutDown,
                    AlertSeverity::Medium,
                    "Price Below Lower Bollinger Band",
                    &format!("Close {:.2} < Lower band {:.2}", last_bar.close, bb.lower),
                    Some(last_bar.close),
                    Some(bb.lower),
                ));
            }
        }
    }

    if let (Some(sma20), Some(sma50)) = (snapshot.sma_20, snapshot.sma_50) {
        if sma20 > sma50 {
            alerts.push(Alert::new(
                &snapshot.ticker,
                AlertType::SmGoldenCross,
                AlertSeverity::Low,
                "SMA20 Above SMA50",
                &format!("SMA20 ({:.2}) > SMA50 ({:.2}) — short-term uptrend", sma20, sma50),
                Some(sma20),
                Some(sma50),
            ));
        } else {
            alerts.push(Alert::new(
                &snapshot.ticker,
                AlertType::SmDeathCross,
                AlertSeverity::Low,
                "SMA20 Below SMA50",
                &format!("SMA20 ({:.2}) < SMA50 ({:.2}) — short-term downtrend", sma20, sma50),
                Some(sma20),
                Some(sma50),
            ));
        }
    }

    if let Some(sma200) = snapshot.sma_200 {
        if let Some(last_bar) = bars.last() {
            if last_bar.close > sma200 {
                alerts.push(Alert::new(
                    &snapshot.ticker,
                    AlertType::PriceAboveSma200,
                    AlertSeverity::Low,
                    "Price Above SMA200",
                    &format!("Close {:.2} > SMA200 ({:.2}) — long-term uptrend", last_bar.close, sma200),
                    Some(last_bar.close),
                    Some(sma200),
                ));
            } else {
                alerts.push(Alert::new(
                    &snapshot.ticker,
                    AlertType::PriceBelowSma200,
                    AlertSeverity::Low,
                    "Price Below SMA200",
                    &format!("Close {:.2} < SMA200 ({:.2}) — long-term downtrend", last_bar.close, sma200),
                    Some(last_bar.close),
                    Some(sma200),
                ));
            }
        }
    }

    if let Some(atr) = snapshot.atr_14 {
        if let Some(last_bar) = bars.last() {
            let volatility_pct = atr / last_bar.close * 100.0;
            if volatility_pct > 5.0 {
                alerts.push(Alert::new(
                    &snapshot.ticker,
                    AlertType::HighVolatility,
                    AlertSeverity::High,
                    "High Volatility",
                    &format!("ATR/Price = {:.1}% — elevated volatility", volatility_pct),
                    Some(volatility_pct),
                    Some(5.0),
                ));
            }
        }
    }

    alerts
}

pub fn compute_activity_score(bars: &[OhlcvBar]) -> f64 {
    if bars.len() < 20 {
        return 0.0;
    }

    let recent = &bars[bars.len() - 5..];
    let earlier = &bars[bars.len() - 20..bars.len() - 5];

    let recent_avg_vol: f64 = recent.iter().map(|b| b.volume as f64).sum::<f64>() / 5.0;
    let earlier_avg_vol: f64 = earlier.iter().map(|b| b.volume as f64).sum::<f64>() / 15.0;

    let vol_ratio = if earlier_avg_vol > 0.0 {
        recent_avg_vol / earlier_avg_vol
    } else {
        1.0
    };

    let recent_price_change: f64 =
        (recent.last().unwrap().close - recent.first().unwrap().open).abs() / recent.first().unwrap().open;

    let earlier_price_change: f64 = earlier
        .iter()
        .map(|b| (b.high - b.low) / b.close)
        .sum::<f64>()
        / 15.0;

    let price_change_ratio = if earlier_price_change > 0.0 {
        recent_price_change / earlier_price_change
    } else {
        1.0
    };

    let score = (vol_ratio * 0.4 + price_change_ratio * 0.4 + (recent_avg_vol / earlier_avg_vol).min(5.0) * 0.2) * 25.0;
    score.min(100.0)
}

#[allow(dead_code)]
pub fn rank_by_activity(tickers: &[(String, f64)]) -> Vec<(String, f64)> {
    let mut sorted = tickers.to_vec();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_engine::types::{BreakoutSignal, MacdResult, VolumeAnomaly};

    fn make_snapshot(rsi: Option<f64>, macd: Option<MacdResult>) -> TechnicalSnapshot {
        TechnicalSnapshot {
            ticker: "TEST".to_string(),
            sma_20: Some(100.0),
            sma_50: Some(98.0),
            sma_200: Some(95.0),
            ema_12: Some(101.0),
            ema_26: Some(99.0),
            rsi_14: rsi,
            macd,
            bollinger: None,
            atr_14: None,
            volume_anomaly: None,
            support_levels: vec![],
            resistance_levels: vec![],
            breakout_signal: None,
            computed_at: 0,
        }
    }

    #[test]
    fn test_rsi_oversold() {
        let snapshot = make_snapshot(Some(25.0), None);
        let alerts = evaluate_alerts(&snapshot, &[]);
        assert!(alerts.iter().any(|a| a.alert_type == AlertType::RsiOversold));
    }

    #[test]
    fn test_rsi_overbought() {
        let snapshot = make_snapshot(Some(75.0), None);
        let alerts = evaluate_alerts(&snapshot, &[]);
        assert!(alerts.iter().any(|a| a.alert_type == AlertType::RsiOverbought));
    }

    #[test]
    fn test_macd_bullish() {
        let macd = MacdResult { macd_line: 0.5, signal_line: 0.3, histogram: 0.2 };
        let snapshot = make_snapshot(None, Some(macd));
        let alerts = evaluate_alerts(&snapshot, &[]);
        assert!(alerts.iter().any(|a| a.alert_type == AlertType::MacdBullishCross));
    }

    #[test]
    fn test_volume_spike() {
        let mut snapshot = make_snapshot(None, None);
        snapshot.volume_anomaly = Some(VolumeAnomaly {
            current_volume: 100000,
            average_volume: 10000.0,
            z_score: 3.5,
        });
        let alerts = evaluate_alerts(&snapshot, &[]);
        assert!(alerts.iter().any(|a| a.alert_type == AlertType::VolumeSpike));
    }

    #[test]
    fn test_breakout_bullish() {
        let mut snapshot = make_snapshot(None, None);
        snapshot.breakout_signal = Some(BreakoutSignal {
            direction: "bullish".to_string(),
            level: 150.0,
            volume_confirmation: true,
        });
        let alerts = evaluate_alerts(&snapshot, &[]);
        assert!(alerts.iter().any(|a| a.alert_type == AlertType::BreakoutBullish));
    }

    #[test]
    fn test_activity_score() {
        let bars: Vec<OhlcvBar> = (0..30)
            .map(|i| OhlcvBar {
                timestamp: i,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0 + i as f64 * 0.1,
                volume: 1000 + i as u64 * 100,
            })
            .collect();
        let score = compute_activity_score(&bars);
        assert!(score >= 0.0 && score <= 100.0);
    }
}
