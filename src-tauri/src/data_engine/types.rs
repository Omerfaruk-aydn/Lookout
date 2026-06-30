use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvBar {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacdResult {
    pub macd_line: f64,
    pub signal_line: f64,
    pub histogram: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerBands {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeAnomaly {
    pub current_volume: u64,
    pub average_volume: f64,
    pub z_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakoutSignal {
    pub direction: String,
    pub level: f64,
    pub volume_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalSnapshot {
    pub ticker: String,
    pub sma_20: Option<f64>,
    pub sma_50: Option<f64>,
    pub sma_200: Option<f64>,
    pub ema_12: Option<f64>,
    pub ema_26: Option<f64>,
    pub rsi_14: Option<f64>,
    pub macd: Option<MacdResult>,
    pub bollinger: Option<BollingerBands>,
    pub atr_14: Option<f64>,
    pub volume_anomaly: Option<VolumeAnomaly>,
    pub support_levels: Vec<f64>,
    pub resistance_levels: Vec<f64>,
    pub breakout_signal: Option<BreakoutSignal>,
    pub computed_at: i64,
}

impl TechnicalSnapshot {
    pub fn compute(ticker: &str, bars: &[OhlcvBar]) -> Self {
        let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();

        let sma_20 = super::indicators::calculate_sma(&closes, 20);
        let sma_50 = super::indicators::calculate_sma(&closes, 50);
        let sma_200 = super::indicators::calculate_sma(&closes, 200);
        let ema_12 = super::indicators::calculate_ema(&closes, 12);
        let ema_26 = super::indicators::calculate_ema(&closes, 26);
        let rsi_14 = super::indicators::calculate_rsi(&closes, 14);
        let macd = super::indicators::calculate_macd(&closes);
        let bollinger = super::indicators::calculate_bollinger_bands(&closes, 20, 2.0);
        let atr_14 = super::indicators::calculate_atr(bars, 14);
        let volume_anomaly = super::indicators::detect_volume_anomaly(bars, 2.0);
        let (support_levels, resistance_levels) =
            super::indicators::detect_support_resistance(bars, 3);
        let breakout_signal =
            super::indicators::detect_breakout(bars, &resistance_levels, 1.5);

        TechnicalSnapshot {
            ticker: ticker.to_string(),
            sma_20,
            sma_50,
            sma_200,
            ema_12,
            ema_26,
            rsi_14,
            macd,
            bollinger,
            atr_14,
            volume_anomaly,
            support_levels,
            resistance_levels,
            breakout_signal,
            computed_at: chrono::Utc::now().timestamp(),
        }
    }
}
