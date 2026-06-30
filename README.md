# Lookout

A Webull Desktop companion application that visually interprets on-screen charts, cross-references with real market data and news sentiment, and delivers structured market analysis reports.

## Tech Stack

- **Shell:** Tauri v2.x
- **Backend:** Rust (edition 2021)
- **Frontend:** React + TypeScript (strict mode)
- **Styling:** TailwindCSS (Tokyo Night theme)
- **Charts:** lightweight-charts (TradingView)
- **Database:** SQLite (rusqlite, WAL mode)
- **Vision/LLM Sidecar:** Python 3.11+ (stdin/stdout JSON protocol)
- **LLM Gateway:** OpenRouter API
- **Market Data:** yfinance (Python sidecar)
- **News:** Finnhub News API
- **Web Search:** DuckDuckGo + LLM summarization (Python sidecar)
- **Screen Capture:** Windows Graphics.Capture API (BitBlt fallback)

## Prerequisites

- Windows 10/11 (x64)
- [Node.js](https://nodejs.org/) v18+
- [Rust](https://rustup.rs/) stable toolchain
- [Python](https://www.python.org/) 3.11+
- Webull Desktop installed and running

## Setup

1. **Clone and install dependencies:**

```bash
npm install
```

2. **Install Python dependencies:**

```bash
pip install -r src-tauri/sidecar-vision/requirements.txt
pip install -r src-tauri/sidecar-news/requirements.txt
pip install -r src-tauri/sidecar-web/requirements.txt
```

3. **Configure environment variables:**

Copy `.env.example` to `.env` and fill in your API keys:

```bash
cp .env.example .env
```

Required keys:
- `OPENROUTER_API_KEY` — for LLM vision and synthesis calls
- `FINNHUB_API_KEY` — for news data (free tier available at finnhub.io)
- `ALPACA_API_KEY` / `ALPACA_SECRET_KEY` — for v2 market data (optional for MVP)

4. **Run in development mode:**

```bash
npm run tauri dev
```

5. **Build for production:**

```bash
npm run tauri build
```

## Project Structure

```
Lookout/
├── src/                          # React frontend
│   ├── components/               # UI components
│   │   ├── SetupWizard.tsx       # First-run region configuration
│   │   ├── OverlayPanel.tsx      # Main always-on-top panel
│   │   ├── ReportView.tsx        # Synthesis report display
│   │   ├── Watchlist.tsx         # Ticker watchlist management
│   │   ├── ScannerView.tsx       # Batch scan + auto-scan controls
│   │   ├── ChartView.tsx         # lightweight-charts reference chart
│   │   └── HistoryView.tsx       # Past reports browser
│   ├── stores/                   # Zustand state management
│   ├── lib/
│   │   └── tauri-bridge.ts       # Typed Tauri invoke wrappers
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── capture/              # Screen capture engine
│   │   ├── data_engine/          # Market data + technical indicators
│   │   ├── scanner/              # Batch scan + auto-scan scheduler
│   │   ├── orchestrator/         # Synthesis report generation
│   │   ├── commands.rs           # Tauri command handlers
│   │   ├── db.rs                 # SQLite database layer
│   │   ├── error.rs              # Centralized error types
│   │   └── main.rs
│   ├── sidecar-vision/           # Python vision sidecar
│   ├── sidecar-news/             # Python news/sentiment sidecar
│   └── sidecar-web/              # Python web search sidecar
├── package.json
├── tsconfig.json
├── tailwind.config.js
└── vite.config.ts
```

## Development Commands

| Command | Description |
|---|---|
| `npm run dev` | Start Vite dev server (frontend only) |
| `npm run tauri dev` | Full-stack dev mode (Rust + React) |
| `npm run build` | Build frontend |
| `npm run tauri build` | Build production app |
| `npm run typecheck` | TypeScript type checking |
| `npm run lint` | ESLint |
| `cargo test` | Run Rust unit tests (in `src-tauri/`) |

## Architecture

The application follows a 6-step pipeline:

1. **Capture Engine** — Takes a screenshot of the Webull chart area using Windows APIs
2. **Vision Sidecar** — Sends screenshot to LLM for visual chart analysis (parallel with steps 3-4)
3. **Data Engine** — Fetches real OHLCV data and computes technical indicators (parallel with steps 2, 4)
4. **News Sidecar** — Fetches recent news and runs batch sentiment analysis (parallel with steps 2-3)
5. **Web Search Sidecar** — Searches DuckDuckGo for recent market events, summarizes with LLM (parallel with steps 2-4)
6. **Synthesis Orchestrator** — Combines all 4 data sources into a single LLM call for structured report
7. **Storage + Display** — Saves to SQLite and pushes to frontend

## Security

- All API keys are read from `.env` or environment variables, never hardcoded
- Screenshots are processed in memory only, never written to disk
- Python sidecars communicate only via stdin/stdout (no network ports)
- SQLite database stored in `%LOCALAPPDATA%/Lookout/` without sensitive data

## Disclaimer

This application is for informational and educational purposes only. It does not constitute investment advice. Always do your own research before making any investment decisions.
