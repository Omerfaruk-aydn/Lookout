use crate::error::LookoutError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    pub chart_area: RegionRect,
    pub ticker_area: RegionRect,
    pub price_area: RegionRect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionRect {
    pub x_pct: f64,
    pub y_pct: f64,
    pub width_pct: f64,
    pub height_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub ticker: String,
    pub added_at: i64,
    pub auto_scan_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecord {
    pub id: String,
    pub ticker: String,
    pub created_at: i64,
    pub vision_result_json: Option<String>,
    pub technical_snapshot_json: Option<String>,
    pub sentiment_result_json: Option<String>,
    pub synthesis_report_json: String,
    pub confidence_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, LookoutError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        Ok(Database { conn })
    }

    pub fn run_migrations(&self) -> Result<(), LookoutError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS region_config (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                chart_area_json TEXT NOT NULL,
                ticker_area_json TEXT NOT NULL,
                price_area_json TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS watchlist (
                ticker TEXT PRIMARY KEY,
                added_at INTEGER NOT NULL,
                auto_scan_enabled INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS reports (
                id TEXT PRIMARY KEY,
                ticker TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                vision_result_json TEXT,
                technical_snapshot_json TEXT,
                sentiment_result_json TEXT,
                synthesis_report_json TEXT NOT NULL,
                confidence_level TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_reports_ticker ON reports(ticker);
            CREATE INDEX IF NOT EXISTS idx_reports_created_at ON reports(created_at);
            ",
        )?;
        Ok(())
    }

    pub fn save_region_config(&self, config: &RegionConfig) -> Result<(), LookoutError> {
        let now = chrono::Utc::now().timestamp();
        let chart_json = serde_json::to_string(&config.chart_area)?;
        let ticker_json = serde_json::to_string(&config.ticker_area)?;
        let price_json = serde_json::to_string(&config.price_area)?;

        self.conn.execute(
            "INSERT INTO region_config (id, chart_area_json, ticker_area_json, price_area_json, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                chart_area_json = ?1,
                ticker_area_json = ?2,
                price_area_json = ?3,
                updated_at = ?4",
            params![chart_json, ticker_json, price_json, now],
        )?;
        Ok(())
    }

    pub fn get_region_config(&self) -> Result<Option<RegionConfig>, LookoutError> {
        let mut stmt = self.conn.prepare(
            "SELECT chart_area_json, ticker_area_json, price_area_json FROM region_config WHERE id = 1",
        )?;

        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let chart_json: String = row.get(0)?;
            let ticker_json: String = row.get(1)?;
            let price_json: String = row.get(2)?;

            let chart_area: RegionRect = serde_json::from_str(&chart_json)?;
            let ticker_area: RegionRect = serde_json::from_str(&ticker_json)?;
            let price_area: RegionRect = serde_json::from_str(&price_json)?;

            Ok(Some(RegionConfig {
                chart_area,
                ticker_area,
                price_area,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn add_to_watchlist(&self, ticker: &str) -> Result<(), LookoutError> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO watchlist (ticker, added_at, auto_scan_enabled)
             VALUES (?1, ?2, 0)
             ON CONFLICT(ticker) DO NOTHING",
            params![ticker, now],
        )?;
        Ok(())
    }

    pub fn remove_from_watchlist(&self, ticker: &str) -> Result<(), LookoutError> {
        self.conn
            .execute("DELETE FROM watchlist WHERE ticker = ?1", params![ticker])?;
        Ok(())
    }

    pub fn get_watchlist(&self) -> Result<Vec<WatchlistItem>, LookoutError> {
        let mut stmt = self
            .conn
            .prepare("SELECT ticker, added_at, auto_scan_enabled FROM watchlist ORDER BY added_at DESC")?;

        let items = stmt
            .query_map([], |row| {
                Ok(WatchlistItem {
                    ticker: row.get(0)?,
                    added_at: row.get(1)?,
                    auto_scan_enabled: row.get::<_, i32>(2)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    pub fn save_report(&self, report: &ReportRecord) -> Result<(), LookoutError> {
        self.conn.execute(
            "INSERT INTO reports (id, ticker, created_at, vision_result_json, technical_snapshot_json, sentiment_result_json, synthesis_report_json, confidence_level)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                report.id,
                report.ticker,
                report.created_at,
                report.vision_result_json,
                report.technical_snapshot_json,
                report.sentiment_result_json,
                report.synthesis_report_json,
                report.confidence_level,
            ],
        )?;
        Ok(())
    }

    pub fn get_reports(&self, ticker: Option<&str>, limit: i32) -> Result<Vec<ReportRecord>, LookoutError> {
        let (sql, needs_ticker) = if let Some(_t) = ticker {
            (
                "SELECT id, ticker, created_at, vision_result_json, technical_snapshot_json, sentiment_result_json, synthesis_report_json, confidence_level FROM reports WHERE ticker = ?1 ORDER BY created_at DESC LIMIT ?2",
                true,
            )
        } else {
            (
                "SELECT id, ticker, created_at, vision_result_json, technical_snapshot_json, sentiment_result_json, synthesis_report_json, confidence_level FROM reports ORDER BY created_at DESC LIMIT ?1",
                false,
            )
        };

        let mut stmt = self.conn.prepare(sql)?;

        let items = if needs_ticker {
            stmt.query_map(params![ticker.unwrap(), limit], Self::map_report_row)?
        } else {
            stmt.query_map(params![limit], Self::map_report_row)?
        };

        let reports = items.collect::<Result<Vec<_>, _>>()?;
        Ok(reports)
    }

    pub fn get_report_by_id(&self, id: &str) -> Result<Option<ReportRecord>, LookoutError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ticker, created_at, vision_result_json, technical_snapshot_json, sentiment_result_json, synthesis_report_json, confidence_level FROM reports WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::extract_report_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn save_setting(&self, key: &str, value: &str) -> Result<(), LookoutError> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, LookoutError> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_settings(&self) -> Result<Vec<Setting>, LookoutError> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let items = stmt
            .query_map([], |row| {
                Ok(Setting {
                    key: row.get(0)?,
                    value: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    fn map_report_row(row: &rusqlite::Row) -> rusqlite::Result<ReportRecord> {
        Self::extract_report_row(row)
    }

    fn extract_report_row(row: &rusqlite::Row) -> rusqlite::Result<ReportRecord> {
        Ok(ReportRecord {
            id: row.get(0)?,
            ticker: row.get(1)?,
            created_at: row.get(2)?,
            vision_result_json: row.get(3)?,
            technical_snapshot_json: row.get(4)?,
            sentiment_result_json: row.get(5)?,
            synthesis_report_json: row.get(6)?,
            confidence_level: row.get(7)?,
        })
    }
}
