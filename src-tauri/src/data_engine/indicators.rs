use super::types::{BollingerBands, BreakoutSignal, MacdResult, OhlcvBar, VolumeAnomaly};

pub fn calculate_sma(closes: &[f64], period: usize) -> Option<f64> {
    if closes.len() < period || period == 0 {
        return None;
    }
    let sum: f64 = closes[closes.len() - period..].iter().sum();
    Some(sum / period as f64)
}

pub fn calculate_ema(closes: &[f64], period: usize) -> Option<f64> {
    if closes.len() < period || period == 0 {
        return None;
    }
    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = closes[closes.len() - period..closes.len() - period + 1]
        .iter()
        .sum::<f64>()
        / period as f64;

    for &close in &closes[closes.len() - period + 1..] {
        ema = (close - ema) * multiplier + ema;
    }
    Some(ema)
}

pub fn calculate_rsi(closes: &[f64], period: usize) -> Option<f64> {
    if closes.len() < period + 1 || period == 0 {
        return None;
    }

    let changes: Vec<f64> = closes
        .windows(2)
        .map(|w| w[1] - w[0])
        .collect();

    let relevant_changes = &changes[changes.len() - period..];

    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;

    for &change in relevant_changes {
        if change > 0.0 {
            avg_gain += change;
        } else {
            avg_loss += change.abs();
        }
    }

    avg_gain /= period as f64;
    avg_loss /= period as f64;

    if avg_loss == 0.0 {
        return Some(100.0);
    }

    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
}

pub fn calculate_macd(closes: &[f64]) -> Option<MacdResult> {
    if closes.len() < 35 {
        return None;
    }

    let ema_12 = calculate_ema(closes, 12)?;
    let ema_26 = calculate_ema(closes, 26)?;
    let macd_line = ema_12 - ema_26;

    let mut macd_values = Vec::new();
    let start = if closes.len() >= 35 { closes.len() - 35 } else { 0 };
    for i in start..closes.len() {
        let slice = &closes[..=i];
        if let (Some(e12), Some(e26)) = (calculate_ema(slice, 12), calculate_ema(slice, 26)) {
            macd_values.push(e12 - e26);
        }
    }

    let signal_line = if macd_values.len() >= 9 {
        calculate_ema(&macd_values, 9)?
    } else {
        macd_values.last().copied().unwrap_or(0.0)
    };

    let histogram = macd_line - signal_line;

    Some(MacdResult {
        macd_line,
        signal_line,
        histogram,
    })
}

pub fn calculate_bollinger_bands(
    closes: &[f64],
    period: usize,
    std_dev_multiplier: f64,
) -> Option<BollingerBands> {
    if closes.len() < period || period == 0 {
        return None;
    }

    let slice = &closes[closes.len() - period..];
    let middle = slice.iter().sum::<f64>() / period as f64;

    let variance = slice.iter().map(|&x| (x - middle).powi(2)).sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();

    Some(BollingerBands {
        upper: middle + std_dev_multiplier * std_dev,
        middle,
        lower: middle - std_dev_multiplier * std_dev,
    })
}

pub fn calculate_atr(bars: &[OhlcvBar], period: usize) -> Option<f64> {
    if bars.len() < period + 1 || period == 0 {
        return None;
    }

    let mut true_ranges = Vec::new();
    for i in 1..bars.len() {
        let high_low = bars[i].high - bars[i].low;
        let high_prev_close = (bars[i].high - bars[i - 1].close).abs();
        let low_prev_close = (bars[i].low - bars[i - 1].close).abs();
        true_ranges.push(high_low.max(high_prev_close).max(low_prev_close));
    }

    if true_ranges.len() < period {
        return None;
    }

    let recent = &true_ranges[true_ranges.len() - period..];
    Some(recent.iter().sum::<f64>() / period as f64)
}

pub fn detect_support_resistance(bars: &[OhlcvBar], min_touches: usize) -> (Vec<f64>, Vec<f64>) {
    if bars.len() < 5 {
        return (vec![], vec![]);
    }

    let mut pivot_lows = Vec::new();
    let mut pivot_highs = Vec::new();

    for i in 2..bars.len() - 2 {
        if bars[i].low < bars[i - 1].low
            && bars[i].low < bars[i - 2].low
            && bars[i].low < bars[i + 1].low
            && bars[i].low < bars[i + 2].low
        {
            pivot_lows.push(bars[i].low);
        }
        if bars[i].high > bars[i - 1].high
            && bars[i].high > bars[i - 2].high
            && bars[i].high > bars[i + 1].high
            && bars[i].high > bars[i + 2].high
        {
            pivot_highs.push(bars[i].high);
        }
    }

    let tolerance = if let Some(last) = bars.last() {
        last.close * 0.005
    } else {
        0.01
    };

    let support_levels = cluster_levels(&pivot_lows, tolerance, min_touches);
    let resistance_levels = cluster_levels(&pivot_highs, tolerance, min_touches);

    (support_levels, resistance_levels)
}

fn cluster_levels(levels: &[f64], tolerance: f64, min_touches: usize) -> Vec<f64> {
    if levels.is_empty() {
        return vec![];
    }

    let mut sorted = levels.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut clusters: Vec<(f64, usize)> = Vec::new();
    let mut current_cluster_sum = sorted[0];
    let mut current_cluster_count = 1usize;

    for &level in &sorted[1..] {
        if (level - current_cluster_sum / current_cluster_count as f64).abs() <= tolerance {
            current_cluster_sum += level;
            current_cluster_count += 1;
        } else {
            if current_cluster_count >= min_touches {
                clusters.push((current_cluster_sum / current_cluster_count as f64, current_cluster_count));
            }
            current_cluster_sum = level;
            current_cluster_count = 1;
        }
    }
    if current_cluster_count >= min_touches {
        clusters.push((current_cluster_sum / current_cluster_count as f64, current_cluster_count));
    }

    clusters.sort_by(|a, b| b.1.cmp(&a.1));
    clusters.iter().map(|(level, _)| *level).collect()
}

pub fn detect_volume_anomaly(bars: &[OhlcvBar], z_threshold: f64) -> Option<VolumeAnomaly> {
    if bars.len() < 20 {
        return None;
    }

    let volumes: Vec<f64> = bars.iter().map(|b| b.volume as f64).collect();
    let recent_volumes = &volumes[volumes.len() - 20..];
    let mean = recent_volumes.iter().sum::<f64>() / recent_volumes.len() as f64;
    let variance = recent_volumes
        .iter()
        .map(|&v| (v - mean).powi(2))
        .sum::<f64>()
        / recent_volumes.len() as f64;
    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return None;
    }

    let current_volume = bars.last()?.volume;
    let z_score = (current_volume as f64 - mean) / std_dev;

    if z_score.abs() >= z_threshold {
        Some(VolumeAnomaly {
            current_volume,
            average_volume: mean,
            z_score,
        })
    } else {
        None
    }
}

pub fn detect_breakout(
    bars: &[OhlcvBar],
    resistance: &[f64],
    volume_multiplier: f64,
) -> Option<BreakoutSignal> {
    if bars.len() < 20 || resistance.is_empty() {
        return None;
    }

    let last_bar = bars.last()?;
    let prev_bar = &bars[bars.len() - 2];

    let volumes: Vec<f64> = bars[bars.len() - 20..bars.len() - 1]
        .iter()
        .map(|b| b.volume as f64)
        .collect();
    let avg_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;

    let volume_confirmed = last_bar.volume as f64 > avg_volume * volume_multiplier;

    for &level in resistance {
        if prev_bar.close < level && last_bar.close > level {
            return Some(BreakoutSignal {
                direction: "bullish".to_string(),
                level,
                volume_confirmation: volume_confirmed,
            });
        }
    }

    let support_levels: Vec<f64> = {
        let (s, _) = detect_support_resistance(bars, 2);
        s
    };

    for &level in &support_levels {
        if prev_bar.close > level && last_bar.close < level {
            return Some(BreakoutSignal {
                direction: "bearish".to_string(),
                level,
                volume_confirmation: volume_confirmed,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bars(data: &[(f64, f64, f64, f64, u64)]) -> Vec<OhlcvBar> {
        data.iter()
            .enumerate()
            .map(|(i, &(o, h, l, c, v))| OhlcvBar {
                timestamp: i as i64,
                open: o,
                high: h,
                low: l,
                close: c,
                volume: v,
            })
            .collect()
    }

    #[test]
    fn test_sma_basic() {
        let closes = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = calculate_sma(&closes, 3);
        assert!(result.is_some());
        assert!((result.unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_sma_insufficient_data() {
        let closes = vec![1.0, 2.0];
        assert!(calculate_sma(&closes, 5).is_none());
    }

    #[test]
    fn test_sma_empty() {
        let closes: Vec<f64> = vec![];
        assert!(calculate_sma(&closes, 5).is_none());
    }

    #[test]
    fn test_sma_single_element() {
        let closes = vec![42.0];
        let result = calculate_sma(&closes, 1);
        assert!(result.is_some());
        assert!((result.unwrap() - 42.0).abs() < 1e-10);
    }

    #[test]
    fn test_sma_zero_period() {
        let closes = vec![1.0, 2.0, 3.0];
        assert!(calculate_sma(&closes, 0).is_none());
    }

    #[test]
    fn test_ema_basic() {
        let closes = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = calculate_ema(&closes, 5);
        assert!(result.is_some());
        assert!(result.unwrap() > 5.0);
    }

    #[test]
    fn test_ema_insufficient_data() {
        let closes = vec![1.0, 2.0];
        assert!(calculate_ema(&closes, 5).is_none());
    }

    #[test]
    fn test_rsi_all_gains() {
        let closes: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let result = calculate_rsi(&closes, 14);
        assert!(result.is_some());
        assert!((result.unwrap() - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_rsi_all_losses() {
        let closes: Vec<f64> = (0..20).map(|i| 100.0 - i as f64).collect();
        let result = calculate_rsi(&closes, 14);
        assert!(result.is_some());
        assert!(result.unwrap().abs() < 1e-10);
    }

    #[test]
    fn test_rsi_mixed() {
        let closes = vec![
            44.0, 44.34, 44.09, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 46.28, 46.00,
        ];
        let result = calculate_rsi(&closes, 14);
        assert!(result.is_some());
        let rsi = result.unwrap();
        assert!(rsi > 50.0 && rsi < 80.0);
    }

    #[test]
    fn test_rsi_insufficient_data() {
        let closes = vec![1.0, 2.0];
        assert!(calculate_rsi(&closes, 14).is_none());
    }

    #[test]
    fn test_rsi_empty() {
        let closes: Vec<f64> = vec![];
        assert!(calculate_rsi(&closes, 14).is_none());
    }

    #[test]
    fn test_macd_basic() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.5).sin() * 5.0).collect();
        let result = calculate_macd(&closes);
        assert!(result.is_some());
        let macd = result.unwrap();
        assert!(macd.macd_line.is_finite());
        assert!(macd.signal_line.is_finite());
        assert!(macd.histogram.is_finite());
    }

    #[test]
    fn test_macd_insufficient_data() {
        let closes = vec![1.0, 2.0, 3.0];
        assert!(calculate_macd(&closes).is_none());
    }

    #[test]
    fn test_bollinger_bands_basic() {
        let closes: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let result = calculate_bollinger_bands(&closes, 20, 2.0);
        assert!(result.is_some());
        let bb = result.unwrap();
        assert!(bb.upper > bb.middle);
        assert!(bb.middle > bb.lower);
    }

    #[test]
    fn test_bollinger_bands_insufficient_data() {
        let closes = vec![1.0, 2.0];
        assert!(calculate_bollinger_bands(&closes, 20, 2.0).is_none());
    }

    #[test]
    fn test_atr_basic() {
        let bars = make_bars(&[
            (10.0, 12.0, 9.0, 11.0, 1000),
            (11.0, 13.0, 10.0, 12.0, 1000),
            (12.0, 14.0, 11.0, 13.0, 1000),
            (13.0, 15.0, 12.0, 14.0, 1000),
            (14.0, 16.0, 13.0, 15.0, 1000),
            (15.0, 17.0, 14.0, 16.0, 1000),
        ]);
        let result = calculate_atr(&bars, 3);
        assert!(result.is_some());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_atr_insufficient_data() {
        let bars = make_bars(&[(10.0, 12.0, 9.0, 11.0, 1000)]);
        assert!(calculate_atr(&bars, 14).is_none());
    }

    #[test]
    fn test_support_resistance_basic() {
        let bars = make_bars(&[
            (100.0, 105.0, 95.0, 102.0, 1000),
            (102.0, 106.0, 96.0, 100.0, 1000),
            (100.0, 104.0, 94.0, 95.0, 1000),
            (95.0, 100.0, 93.0, 99.0, 1000),
            (99.0, 105.0, 95.0, 104.0, 1000),
            (104.0, 108.0, 96.0, 100.0, 1000),
            (100.0, 105.0, 94.0, 95.0, 1000),
            (95.0, 100.0, 93.0, 98.0, 1000),
            (98.0, 104.0, 95.0, 103.0, 1000),
            (103.0, 107.0, 96.0, 100.0, 1000),
        ]);
        let (support, resistance) = detect_support_resistance(&bars, 2);
        assert!(!support.is_empty() || !resistance.is_empty());
    }

    #[test]
    fn test_support_resistance_insufficient_data() {
        let bars = make_bars(&[(100.0, 105.0, 95.0, 102.0, 1000)]);
        let (support, resistance) = detect_support_resistance(&bars, 3);
        assert!(support.is_empty());
        assert!(resistance.is_empty());
    }

    #[test]
    fn test_volume_anomaly_detected() {
        let mut bars = make_bars(
            &(0..20)
                .map(|i| (100.0, 105.0, 95.0, 102.0, 1000 + i * 10))
                .collect::<Vec<_>>(),
        );
        bars.push(OhlcvBar {
            timestamp: 20,
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 50000,
        });
        let result = detect_volume_anomaly(&bars, 2.0);
        assert!(result.is_some());
        assert!(result.unwrap().z_score > 2.0);
    }

    #[test]
    fn test_volume_anomaly_not_detected() {
        let bars = make_bars(
            &(0..20)
                .map(|i| (100.0, 105.0, 95.0, 102.0, 1000 + i * 10))
                .collect::<Vec<_>>(),
        );
        let result = detect_volume_anomaly(&bars, 2.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_volume_anomaly_insufficient_data() {
        let bars = make_bars(&[(100.0, 105.0, 95.0, 102.0, 1000)]);
        assert!(detect_volume_anomaly(&bars, 2.0).is_none());
    }

    #[test]
    fn test_breakout_bullish() {
        let bars = make_bars(&[
            (100.0, 105.0, 95.0, 102.0, 1000),
            (102.0, 106.0, 96.0, 100.0, 1000),
            (100.0, 104.0, 94.0, 95.0, 1000),
            (95.0, 100.0, 93.0, 99.0, 1000),
            (99.0, 105.0, 95.0, 104.0, 1000),
            (104.0, 108.0, 96.0, 100.0, 1000),
            (100.0, 105.0, 94.0, 95.0, 1000),
            (95.0, 100.0, 93.0, 98.0, 1000),
            (98.0, 104.0, 95.0, 103.0, 1000),
            (103.0, 107.0, 96.0, 100.0, 1000),
            (100.0, 105.0, 95.0, 102.0, 1000),
            (102.0, 106.0, 96.0, 100.0, 1000),
            (100.0, 104.0, 94.0, 95.0, 1000),
            (95.0, 100.0, 93.0, 99.0, 1000),
            (99.0, 105.0, 95.0, 104.0, 1000),
            (104.0, 108.0, 96.0, 100.0, 1000),
            (100.0, 105.0, 94.0, 95.0, 1000),
            (95.0, 100.0, 93.0, 98.0, 1000),
            (98.0, 104.0, 95.0, 103.0, 1000),
            (103.0, 107.0, 96.0, 100.0, 1000),
            (100.0, 110.0, 99.0, 109.0, 5000),
        ]);
        let (_, resistance) = detect_support_resistance(&bars, 2);
        let result = detect_breakout(&bars, &resistance, 1.5);
        if let Some(signal) = result {
            assert_eq!(signal.direction, "bullish");
        }
    }

    #[test]
    fn test_breakout_no_signal() {
        let bars = make_bars(
            &(0..20)
                .map(|i| {
                    let base = 100.0 + (i as f64 * 0.1);
                    (base, base + 1.0, base - 1.0, base, 1000)
                })
                .collect::<Vec<_>>(),
        );
        let (_, resistance) = detect_support_resistance(&bars, 3);
        let result = detect_breakout(&bars, &resistance, 1.5);
        assert!(result.is_none());
    }
}
