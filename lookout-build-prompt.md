# LOOKOUT — AI Engineering Build Prompt

> Bu doküman bir AI kod üretim aracına (Claude Code, Cursor, vb.) doğrudan verilmek üzere hazırlanmıştır. Placeholder, "TODO", yarım bırakılmış mantık veya generic isimlendirme kabul edilemez. Her modül production-grade, hatasız derlenen/çalışan, test edilebilir kod olarak üretilmelidir.

---

## ROL TANIMI

Sen kıdemli bir full-stack/sistem mühendisisin. Rust (Tauri), TypeScript/React, Python (sidecar süreçler) ve finansal veri sistemleri konusunda uzmansın. Aşağıdaki spesifikasyonu eksiksiz, üretim kalitesinde bir masaüstü uygulamasına dönüştüreceksin. Görevin sadece "çalışan kod" yazmak değil; hata yönetimi, performans, güvenlik ve kullanıcı deneyimi açısından savunulabilir bir mimari kurmak.

---

## PROJE KİMLİĞİ

- **Proje adı:** Lookout
- **Tek cümlelik tanım:** Webull Desktop'ın yanında çalışan, ekrandaki grafiği görsel olarak yorumlayan, bağımsız piyasa/haber verisiyle bunu doğrulayıp zenginleştiren ve kullanıcıya yapısal, gerekçeli bir piyasa durum raporu sunan bir masaüstü companion uygulaması.
- **Hedef platform:** Windows 10/11 (x64), tek kullanıcı, yerel çalışan masaüstü app.
- **Kesinlikle YAPILMAYACAKLAR (scope dışı, asla implement etme):**
  - Webull'a otomatik tıklama, emir gönderme, form doldurma, herhangi bir UI etkileşimi. Sistem SADECE okur, asla yazmaz/tıklamaz.
  - Webull'un private API'lerine reverse-engineering ile bağlanma, network trafiğini intercept etme.
  - "AL", "SAT", "şimdi gir" gibi kesin/emir niteliğinde yatırım tavsiyesi üreten metin. Her rapor "bilgilendirme amaçlıdır, yatırım tavsiyesi değildir" uyarısıyla bitmelidir — bu opsiyonel değil, zorunlu bir guard'dır ve LLM çıktısı bu ifadeyi içermiyorsa post-processing ile otomatik eklenmelidir.

---

## TEKNOLOJİ STACK'İ (KESİN, DEĞİŞTİRİLEMEZ)

| Katman | Teknoloji | Versiyon Notu |
|---|---|---|
| Shell | Tauri | v2.x, en güncel stabil |
| Backend dili | Rust | edition 2021, stabil toolchain |
| Frontend | React + TypeScript | strict mode açık, `any` kullanımı yasak |
| Stil | TailwindCSS | Tokyo Night custom theme (aşağıda token tablosu var) |
| Chart | lightweight-charts (TradingView) | npm paketi |
| Yerel DB | SQLite | `rusqlite` crate, WAL mode açık |
| Vision/LLM sidecar | Python 3.11+ | subprocess olarak Rust'tan çağrılır, stdin/stdout JSON protokolü |
| LLM Gateway | OpenRouter API | dinamik model routing |
| Piyasa verisi (MVP) | yfinance (Python) | sidecar üzerinden |
| Piyasa verisi (v2) | Alpaca Market Data API | websocket + REST |
| Haber verisi | Finnhub News API | REST, ücretsiz tier |
| Ekran yakalama | `windows` crate (Windows.Graphics.Capture API) | BitBlt fallback ile |

Bu stack dışına çıkma. Alternatif kütüphane önerisi yapma — bu kararlar zaten verildi.

---

## MİMARİ — VERİ AKIŞI (BİREBİR UYGULANACAK)

```
[Tetik: zamanlayıcı | kullanıcı butonu | sembol değişikliği]
        │
        ▼
┌───────────────────┐
│ 1. Capture Engine   │ → Webull client area screenshot (PNG, bellekte, diske yazılmaz)
└────────┬───────────┘
         ▼
┌───────────────────┐     ┌──────────────────────┐
│ 2. Vision Sidecar   │     │ 3. Data Engine         │
│ (görsel → yapısal   │     │ (paralel çalışır,      │
│  JSON yorum)        │     │  ticker'ı vision'dan    │
└────────┬───────────┘     │  alır, gerçek veri      │
         │                  │  çeker + indikatör      │
         │                  │  hesaplar)               │
         │                  └──────────┬───────────────┘
         │                             │
         │         ┌───────────────────┘
         │         ▼
         │   ┌──────────────────┐
         │   │ 4. News Sidecar    │
         │   │ (ticker için son   │
         │   │  haberleri çeker +  │
         │   │  sentiment skorlar) │
         │   └─────────┬──────────┘
         │             │
         └──────┬──────┘
                ▼
       ┌──────────────────────┐
       │ 5. Synthesis Orchestrator │
       │ (3 kaynağı birleştirir,    │
       │  tek LLM çağrısı,           │
       │  yapısal rapor üretir)      │
       └─────────┬───────────────┘
                 ▼
       ┌──────────────────────┐
       │ 6. SQLite'a kaydet     │
       │ + Frontend'e push et   │
       └──────────────────────┘
```

**Kritik kural:** Adım 2 ve 3 paralel (async, `tokio::join!`) çalışmalı, sıralı değil. Adım 5 ikisi de tamamlanmadan başlamamalı.

---

## MODÜL 1 — Capture Engine (`src-tauri/src/capture/`)

### Dosyalar
- `mod.rs`
- `window_locator.rs`
- `screenshot.rs`
- `region_config.rs`

### Gereksinimler

**`window_locator.rs`:**
- `EnumWindows` ile çalışan process'ler arasından başlık metni `"Webull"` içeren pencereyi bulan bir fonksiyon: `pub fn find_webull_window() -> Result<HWND, LookoutError>`.
- Bulunamazsa anlamlı bir hata döndürülmeli (generic "not found" değil — `WebullNotRunningError` gibi tipize edilmiş hata, frontend'de kullanıcıya "Webull açık değil, lütfen önce Webull Desktop'ı başlatın" mesajı gösterilecek).
- Pencere handle'ı cache'lenmeli ama her capture öncesi hala geçerli olup olmadığı (`IsWindow`) kontrol edilmeli — kullanıcı Webull'u kapatmış olabilir.

**`screenshot.rs`:**
- `pub async fn capture_region(hwnd: HWND, region: Rect) -> Result<Vec<u8>, LookoutError>` — PNG byte array döner, diske yazmaz.
- Birincil yöntem: `Windows.Graphics.Capture` API (modern, GPU hızlandırmalı, DWM composited pencerelerle uyumlu).
- Fallback: `BitBlt` tabanlı GDI capture (Windows Graphics Capture API başarısız olursa).
- Capture sırasında pencere minimize/kapalıysa anlamlı hata fırlat, sessizce boş görsel döndürme.

**`region_config.rs`:**
- Kullanıcının ilk kurulumda işaretlediği 3 bölgeyi (chart_area, ticker_area, price_area) `Rect { x, y, width, height }` olarak SQLite'a persist eden CRUD fonksiyonları.
- Bölgeler pencere boyutuna **oranlı** (percentage-based) saklanmalı, mutlak piksel değil — kullanıcı pencereyi yeniden boyutlandırırsa hala doğru çalışsın.

### Kabul Kriterleri
- Webull kapalıyken uygulama crash etmemeli, kullanıcıya net hata göstermeli.
- Capture işlemi 500ms altında tamamlanmalı (performans bütçesi).
- Hiçbir görsel disk'e yazılmamalı (gizlilik) — sadece bellekte base64'e çevrilip sidecar'a aktarılmalı.

---

## MODÜL 2 — Vision Sidecar (`src-tauri/sidecar-vision/`)

### Dosyalar
- `vision_client.py`
- `prompts.py`
- `schema.py` (pydantic modelleri)
- `requirements.txt`

### Protokol
Rust tarafı sidecar'ı subprocess olarak başlatır, stdin'e JSON gönderir, stdout'tan JSON okur:

**Input (stdin):**
```json
{
  "image_base64": "...",
  "request_id": "uuid",
  "model": "anthropic/claude-sonnet-4-6"
}
```

**Output (stdout):**
```json
{
  "request_id": "uuid",
  "success": true,
  "data": {
    "ticker_visible": "AAPL",
    "trend_direction": "up",
    "visible_patterns": ["ascending_triangle"],
    "support_resistance_estimate": {"support": [187.5], "resistance": [195.2]},
    "volume_observation": "Son birkaç mumda hacim artışı görünüyor",
    "indicators_visible": [{"name": "RSI", "value_estimate": "62"}],
    "confidence": 0.74,
    "notes": "Grafik küçük çözünürlükte, fiyat ekseni tam okunamadı"
  },
  "error": null
}
```

### `schema.py` Gereksinimleri
Pydantic ile **kesin** şema tanımı yap (yukarıdaki JSON'un birebir karşılığı). LLM çıktısı bu şemaya `model_validate_json` ile doğrulanmalı; doğrulama başarısız olursa **tek bir retry** yapılmalı (farklı/daha açık bir prompt ile), ikinci hata durumunda `success: false` ve hata mesajıyla dönülmeli. Asla şema dışı/serbest metin frontend'e geçmemeli.

### `prompts.py` Gereksinimleri
Sistem promptu şunu kesin olarak içermeli:
- "Sadece geçerli JSON döndür, markdown code fence kullanma, açıklama ekleme."
- "Eğer görüntüde grafik yoksa veya okunamıyorsa confidence 0.0 ver ve notes alanında nedenini belirt — tahmin uydurma."
- "Sayısal değerler (fiyat, RSI vb.) tahminidir, kesinlik iddia etme; bu değerler ayrı bir sistemde gerçek veriyle doğrulanacaktır."

### Kabul Kriterleri
- Geçersiz/bozuk görsel inputunda crash etmemeli, yapısal hata dönmeli.
- API key environment variable'dan okunmalı, asla koda hardcode edilmemeli.
- Timeout: 15 saniye üstü yanıt gelmezse `timeout_error` dönülmeli.

---

## MODÜL 3 — Data Engine (`src-tauri/src/data_engine/`)

### Dosyalar
- `mod.rs`
- `providers/yfinance_provider.rs` (Python sidecar üzerinden, çünkü yfinance Python-only)
- `providers/alpaca_provider.rs` (v2, doğrudan Rust HTTP client ile)
- `indicators.rs`
- `types.rs`

### `types.rs` — Kesin Veri Modelleri
```rust
pub struct OhlcvBar {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

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
```
Bu tipler placeholder değil — gerçek alan adları, gerçek `Option<T>` kullanımı (eksik veri durumunu açıkça modellemek için) ve gerçek serde derive'ları (`Serialize, Deserialize`) ile yazılmalı.

### `indicators.rs` Gereksinimleri
Her indikatör fonksiyonu saf, test edilebilir, yan etkisiz olmalı:
```rust
pub fn calculate_rsi(closes: &[f64], period: usize) -> Option<f64>
pub fn calculate_sma(closes: &[f64], period: usize) -> Option<f64>
pub fn calculate_macd(closes: &[f64]) -> Option<MacdResult>
pub fn detect_support_resistance(bars: &[OhlcvBar], lookback: usize) -> (Vec<f64>, Vec<f64>)
pub fn detect_volume_anomaly(bars: &[OhlcvBar], z_threshold: f64) -> Option<VolumeAnomaly>
pub fn detect_breakout(bars: &[OhlcvBar], resistance: &[f64], volume_multiplier: f64) -> Option<BreakoutSignal>
```
Her fonksiyon için **birim test** yaz (bilinen girdi/çıktı çiftleriyle — örn. RSI hesaplaması için elle doğrulanmış bir örnek seri kullan, "yaklaşık doğru görünüyor" yetmez).

Support/resistance algoritması: lokal maksimum/minimum tespiti (pivot point mantığı) + en az N bar'da (varsayılan 3) tekrar test edilmiş seviyeleri öncelikli say.

### Kabul Kriterleri
- Yetersiz veri durumunda (örn. 200 günlük veri yokken SMA200 istenmesi) panic değil, `None` dönülmeli.
- Tüm hesaplamalar `f64` hassasiyetinde, finansal yuvarlama hatalarına karşı testli.
- API rate limit aşımında exponential backoff ile retry (max 3 deneme).

---

## MODÜL 4 — News Sidecar (`src-tauri/sidecar-news/`)

### Dosyalar
- `news_client.py`
- `sentiment_analyzer.py`

### Akış
1. `fetch_news(ticker: str, hours_back: int = 48) -> List[NewsItem]` — Finnhub'dan çeker.
2. Her haber LLM'e (ucuz/hızlı model, örn. Haiku sınıfı) **batch** olarak gönderilir (tek tek değil — maliyet/hız için tüm haberler tek prompt'ta, numaralı liste halinde verilir, model her biri için ayrı JSON objesi döner).
3. Çıktı agregasyonu: ağırlıklı ortalama sentiment skoru, en yeni haberler 2x ağırlıklı.

### Veri Modeli
```python
class NewsSentimentItem(BaseModel):
    headline: str
    source: str
    published_at: datetime
    sentiment: Literal["positive", "negative", "neutral"]
    impact_score: int  # 0-10
    reasoning: str  # max 100 karakter, kısa gerekçe

class AggregatedSentiment(BaseModel):
    ticker: str
    overall_sentiment: Literal["positive", "negative", "neutral", "mixed"]
    weighted_score: float  # -1.0 ile 1.0 arası
    item_count: int
    items: List[NewsSentimentItem]
```

### Kabul Kriterleri
- Haber bulunamazsa (yeni/az bilinen ticker) `item_count: 0` ile boş ama geçerli bir `AggregatedSentiment` dönülmeli, hata fırlatılmamalı.
- Çok eski haberler (48 saatten eski) varsayılan filtre dışı tutulmalı ama kullanıcı ayarından değiştirilebilir olmalı.

---

## MODÜL 5 — Synthesis Orchestrator (`src-tauri/src/orchestrator/`)

### Dosyalar
- `mod.rs`
- `prompt_builder.rs`
- `report_validator.rs`

### Görev
3 kaynağın (`VisionResult`, `TechnicalSnapshot`, `AggregatedSentiment`) hepsini tek bir context'e derleyip nihai LLM çağrısını yapar.

### `prompt_builder.rs` — Sistem Promptu (TAM METİN, BİREBİR KULLAN)
```
Sen bir finansal analiz asistanısın. Sana 3 farklı kaynaktan veri verilecek:

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
- Sadece JSON döndür, başka metin ekleme.
```

### `report_validator.rs` Gereksinimleri
- LLM çıktısı şemaya uymuyorsa (eksik alan, yanlış tip) **tek retry**, sonra `Result::Err` ile yukarı taşı — sahte/eksik rapor frontend'e asla geçmemeli.
- **Guard fonksiyonu zorunlu:** `fn enforce_disclaimer(report: &mut Report)` — eğer LLM çıktısında "AL", "SAT", "buy", "sell" gibi emir niteliğindeki kelimeler tespit edilirse (basit keyword + regex tarama), bu rapor loglanır (debug için) VE kullanıcıya gösterilmeden önce nötrleştirici bir uyarı eklenir. Bu, LLM'in talimatı görmezden gelmesine karşı bir son savunma hattıdır.
- Her raporun sonuna otomatik, değiştirilemez şu metin eklenir: *"Bu rapor bilgilendirme amaçlıdır, yatırım tavsiyesi niteliği taşımaz."*

---

## MODÜL 6 — Frontend (`src/`)

### Tasarım Sistemi — Tokyo Night Token Tablosu
```css
:root {
  --bg-primary: #1a1b26;
  --bg-secondary: #24283b;
  --bg-tertiary: #2f334d;
  --fg-primary: #c0caf5;
  --fg-secondary: #9aa5ce;
  --fg-muted: #565f89;
  --accent-blue: #7aa2f7;
  --accent-purple: #bb9af7;
  --accent-green: #9ece6a;
  --accent-red: #f7768e;
  --accent-yellow: #e0af68;
  --accent-cyan: #7dcfff;
  --border-color: #292e42;
  --font-mono: 'JetBrains Mono', monospace;
  --font-sans: 'Inter', sans-serif;
}
```
Pozitif teknik/sentiment göstergeleri `--accent-green`, negatif `--accent-red`, nötr `--accent-yellow`, ana vurgular `--accent-blue`/`--accent-purple` kullanır.

### Bileşenler
- `SetupWizard.tsx` — İlk açılışta Webull penceresi tespit edilir, kullanıcıdan crop overlay ile 3 bölge (chart/ticker/price) işaretlemesi istenir. Tauri `invoke('save_region_config', {...})` ile Rust tarafına yazılır.
- `OverlayPanel.tsx` — Ana panel, Webull'un yanına `always-on-top` ve opsiyonel `snap-to-edge` ile yapışır (Tauri window API ile pencere pozisyonu Webull'un sağ/sol kenarına otomatik hizalanır).
- `ReportView.tsx` — Synthesis çıktısını render eder. `confidence_level` görsel olarak rozet (badge) şeklinde gösterilir (high=yeşil, medium=sarı, low=kırmızı). `conflicting_signals` varsa belirgin bir uyarı kutusu ile vurgulanır.
- `Watchlist.tsx` — Kullanıcının takip ettiği sembol listesi, her biri için son analiz zaman damgası ve özet durumu.
- `ChartView.tsx` — lightweight-charts ile kendi grafiğini de gösterir (Data Engine'in gerçek OHLCV verisinden), Webull'un grafiğine ek doğrulama referansı olarak.
- `HistoryView.tsx` — SQLite'tan geçmiş raporları sembol/tarih bazlı filtreleyerek listeler.

### State Yönetimi
Zustand kullan (Redux gereksiz ağırlık katar bu ölçekte). Store'lar: `useWatchlistStore`, `useReportStore`, `useSettingsStore`.

### Kabul Kriterleri
- Hiçbir bileşen `any` tipi kullanmamalı, tüm Tauri `invoke` çağrıları tipize edilmiş wrapper fonksiyonlardan geçmeli (`src/lib/tauri-bridge.ts`).
- Loading state'ler her zaman gösterilmeli (analiz 3-8 saniye sürebilir, kullanıcı boş ekranla bırakılmamalı — Tokyo Night temalı skeleton/spinner kullan).
- Hata durumları (Webull bulunamadı, API key eksik, rate limit) kullanıcı dostu mesajlarla, teknik stack trace olmadan gösterilmeli.

---

## MODÜL 7 — SQLite Şeması (`src-tauri/src/db.rs`)

```sql
CREATE TABLE region_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    chart_area_json TEXT NOT NULL,
    ticker_area_json TEXT NOT NULL,
    price_area_json TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE watchlist (
    ticker TEXT PRIMARY KEY,
    added_at INTEGER NOT NULL,
    auto_scan_enabled INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE reports (
    id TEXT PRIMARY KEY,  -- uuid
    ticker TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    vision_result_json TEXT,
    technical_snapshot_json TEXT,
    sentiment_result_json TEXT,
    synthesis_report_json TEXT NOT NULL,
    confidence_level TEXT NOT NULL
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX idx_reports_ticker ON reports(ticker);
CREATE INDEX idx_reports_created_at ON reports(created_at);
```

WAL mode açık olmalı (`PRAGMA journal_mode=WAL`), tüm yazma işlemleri transaction içinde.

---

## HATA YÖNETİMİ — GENEL POLİTİKA

Tek bir merkezi hata tipi tanımla (`LookoutError` enum, `thiserror` crate ile):
```rust
#[derive(thiserror::Error, Debug)]
pub enum LookoutError {
    #[error("Webull Desktop çalışmıyor")]
    WebullNotRunning,
    #[error("Ekran yakalama başarısız: {0}")]
    CaptureFailed(String),
    #[error("Vision API hatası: {0}")]
    VisionApiError(String),
    #[error("Piyasa verisi alınamadı: {0}")]
    DataProviderError(String),
    #[error("Yetersiz veri: {0}")]
    InsufficientData(String),
    #[error("LLM yanıtı şemaya uymuyor: {0}")]
    SchemaValidationError(String),
    #[error("Veritabanı hatası: {0}")]
    DatabaseError(#[from] rusqlite::Error),
}
```
Her hata frontend'e Tauri event sistemi üzerinden tipize edilmiş halde iletilmeli, asla `unwrap()`/`panic!` ile uygulama çökertilmemeli. Production kodda `unwrap()` kullanımı yasak (sadece test kodunda serbest).

---

## GÜVENLİK GEREKSİNİMLERİ

- Tüm API key'ler (OpenRouter, Finnhub, Alpaca) `.env` dosyasından veya Windows Credential Manager'dan okunmalı, asla repo'ya commit edilmemeli (`.gitignore`'da `.env` olmalı).
- Screenshot verisi diske asla yazılmamalı, sadece bellekte işlenip atılmalı.
- SQLite veritabanı dosyası kullanıcının `%APPDATA%/Lookout/` dizininde, hassas veri (API key) içermemeli.
- Sidecar process'ler (Python) sadece localhost/stdin-stdout üzerinden iletişim kurmalı, hiçbir port dışarıya açılmamalı.

---

## TEST STRATEJİSİ

- `indicators.rs` — her fonksiyon için bilinen girdi/çıktı ile birim test (minimum 5 test case, edge case'ler dahil: boş array, tek elemanlı array, yetersiz period).
- `report_validator.rs` — şema doğrulama, disclaimer enforcement, keyword guard için birim test.
- Vision/News sidecar — mock API yanıtlarıyla entegrasyon testi (gerçek API çağrısı yapmadan).
- Frontend — kritik bileşenler için Vitest + React Testing Library (en azından `ReportView` ve `SetupWizard` için).

---

## TESLİMAT FORMATI

Kod aşağıdaki sırayla, her modül tamamlandıktan sonra bir sonrakine geçilerek üretilmeli:
1. Proje iskeleti + Cargo.toml/package.json bağımlılıkları
2. SQLite şeması + db.rs
3. Capture Engine
4. Vision Sidecar
5. Data Engine + indikatörler (testleriyle birlikte)
6. News Sidecar
7. Synthesis Orchestrator
8. Frontend bileşenleri (Tokyo Night tema ile)
9. Tauri command'ları ile tüm modüllerin bağlanması
10. README.md — kurulum, .env örneği, geliştirme komutları

Her modül tamamlandığında derlenebilir/çalışabilir durumda olmalı — "sonra tamamlarız" bırakılan eksik fonksiyon kabul edilmez.
