use super::{SentimentResult, VisionResult};
use crate::data_engine::TechnicalSnapshot;

pub const SYSTEM_PROMPT: &str = r#"Sen bir finansal analiz asistanısın. Sana 3 farklı kaynaktan veri verilecek:

1. GÖRSEL ANALİZ (düşük güvenilirlik, ekrandan okunan yorum — sadece genel yön/pattern fikri için kullan, sayısal değerlerine güvenme)
2. TEKNİK VERİ (yüksek güvenilirlik, gerçek hesaplanmış indikatörler — kesin sayılar buradan gelir)
3. HABER SENTIMENT (son 48 saatteki haberlerin toplu duygu analizi)

GÖREV: Bu üç kaynağı birleştirip kullanıcıya YAPISAL BİR DURUM RAPORU sun. Aşağıdaki formatı KESİNLİKLE takip et:

{
  "ticker": "...",
  "summary": "1-2 cümlelik genel görünüm",
  "technical_status": "somut sayılarla teknik durum açıklaması",
  "news_impact": "haber akışının olası etkisi",
  "conflicting_signals": "varsa çelişen sinyaller (örn: teknik pozitif ama haber negatif), yoksa null",
  "risk_notes": "dikkat edilmesi gereken riskler",
  "confidence_level": "high|medium|low — kaynaklar arası tutarlılığa göre belirle"
}

KESİN KURALLAR:
- ASLA "al", "sat", "şimdi gir/çık" gibi emir niteliğinde ifade kullanma.
- ASLA görsel analizdeki sayısal tahminleri (fiyat, RSI vb.) teknik veri ile çelişiyorsa görseli tercih etme — teknik veri her zaman önceliklidir.
- Eğer 3 kaynaktan biri eksik/başarısızsa, bunu confidence_level'a yansıt ve hangi kaynağın eksik olduğunu belirt.
- Sadece JSON döndür, başka metin ekleme."#;

pub fn build_synthesis_prompt(
    vision: Option<&VisionResult>,
    technical: Option<&TechnicalSnapshot>,
    sentiment: Option<&SentimentResult>,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("VERİLER:\n\n");

    if let Some(v) = vision {
        prompt.push_str(&format!(
            "1. GÖRSEL ANALİZ (güvenilirlik: {:.0}%):\n",
            v.confidence * 100.0
        ));
        if let Some(ref ticker) = v.ticker_visible {
            prompt.push_str(&format!("  - Ticker: {}\n", ticker));
        }
        if let Some(ref trend) = v.trend_direction {
            prompt.push_str(&format!("  - Trend: {}\n", trend));
        }
        if !v.visible_patterns.is_empty() {
            prompt.push_str(&format!("  - Patterns: {:?}\n", v.visible_patterns));
        }
        if !v.support_resistance_estimate.support.is_empty() {
            prompt.push_str(&format!(
                "  - Support (tahmin): {:?}\n",
                v.support_resistance_estimate.support
            ));
        }
        if !v.support_resistance_estimate.resistance.is_empty() {
            prompt.push_str(&format!(
                "  - Resistance (tahmin): {:?}\n",
                v.support_resistance_estimate.resistance
            ));
        }
        if let Some(ref vol) = v.volume_observation {
            prompt.push_str(&format!("  - Volume: {}\n", vol));
        }
        if !v.indicators_visible.is_empty() {
            prompt.push_str("  - Visible indicators:\n");
            for ind in &v.indicators_visible {
                prompt.push_str(&format!("    * {}: {}\n", ind.name, ind.value_estimate));
            }
        }
        if let Some(ref notes) = v.notes {
            prompt.push_str(&format!("  - Notes: {}\n", notes));
        }
    } else {
        prompt.push_str("1. GÖRSEL ANALİZ: VERİ YOK (capture başarısız veya atlandı)\n");
    }

    prompt.push('\n');

    if let Some(t) = technical {
        prompt.push_str("2. TEKNİK VERİ (hesaplanmış):\n");
        if let Some(sma) = t.sma_20 {
            prompt.push_str(&format!("  - SMA20: {:.2}\n", sma));
        }
        if let Some(sma) = t.sma_50 {
            prompt.push_str(&format!("  - SMA50: {:.2}\n", sma));
        }
        if let Some(sma) = t.sma_200 {
            prompt.push_str(&format!("  - SMA200: {:.2}\n", sma));
        }
        if let Some(ema) = t.ema_12 {
            prompt.push_str(&format!("  - EMA12: {:.2}\n", ema));
        }
        if let Some(ema) = t.ema_26 {
            prompt.push_str(&format!("  - EMA26: {:.2}\n", ema));
        }
        if let Some(rsi) = t.rsi_14 {
            prompt.push_str(&format!("  - RSI14: {:.2}\n", rsi));
        }
        if let Some(ref macd) = t.macd {
            prompt.push_str(&format!(
                "  - MACD: line={:.4}, signal={:.4}, histogram={:.4}\n",
                macd.macd_line, macd.signal_line, macd.histogram
            ));
        }
        if let Some(ref bb) = t.bollinger {
            prompt.push_str(&format!(
                "  - Bollinger: upper={:.2}, middle={:.2}, lower={:.2}\n",
                bb.upper, bb.middle, bb.lower
            ));
        }
        if let Some(atr) = t.atr_14 {
            prompt.push_str(&format!("  - ATR14: {:.2}\n", atr));
        }
        if !t.support_levels.is_empty() {
            prompt.push_str(&format!("  - Support levels: {:?}\n", t.support_levels));
        }
        if !t.resistance_levels.is_empty() {
            prompt.push_str(&format!(
                "  - Resistance levels: {:?}\n",
                t.resistance_levels
            ));
        }
        if let Some(ref breakout) = t.breakout_signal {
            prompt.push_str(&format!(
                "  - Breakout: {} at {:.2} (volume confirmed: {})\n",
                breakout.direction, breakout.level, breakout.volume_confirmation
            ));
        }
    } else {
        prompt.push_str("2. TEKNİK VERİ: VERİ YOK (yetersiz veri veya hata)\n");
    }

    prompt.push('\n');

    if let Some(s) = sentiment {
        prompt.push_str("3. HABER SENTIMENT:\n");
        prompt.push_str(&format!("  - Overall: {}\n", s.overall_sentiment));
        prompt.push_str(&format!("  - Weighted score: {:.4}\n", s.weighted_score));
        prompt.push_str(&format!("  - Item count: {}\n", s.item_count));
    } else {
        prompt.push_str("3. HABER SENTIMENT: VERİ YOK (haber bulunamadı veya hata)\n");
    }

    prompt
}
