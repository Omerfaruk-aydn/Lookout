use crate::autonomous::alerts::Alert;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub alert_id: String,
    pub ticker: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub read: bool,
    pub created_at: i64,
}

pub struct NotificationStore {
    conn: Connection,
}

impl NotificationStore {
    pub fn new(conn: Connection) -> Self {
        NotificationStore { conn }
    }

    #[allow(dead_code)]
    pub fn run_migrations(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS notifications (
                id TEXT PRIMARY KEY,
                alert_id TEXT NOT NULL,
                ticker TEXT NOT NULL,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                severity TEXT NOT NULL,
                read INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read);
            CREATE INDEX IF NOT EXISTS idx_notifications_created ON notifications(created_at);
            ",
        )?;
        Ok(())
    }

    pub fn push_alert(&self, alert: &Alert) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO notifications (id, alert_id, ticker, title, message, severity, read, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
            params![
                uuid::Uuid::new_v4().to_string(),
                alert.id,
                alert.ticker,
                alert.title,
                alert.message,
                format!("{:?}", alert.severity),
                alert.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn push_alerts(&self, alerts: &[Alert]) -> Result<(), rusqlite::Error> {
        for alert in alerts {
            self.push_alert(alert)?;
        }
        Ok(())
    }

    pub fn get_notifications(
        &self,
        limit: i32,
        unread_only: bool,
    ) -> Result<Vec<Notification>, rusqlite::Error> {
        let sql = if unread_only {
            "SELECT id, alert_id, ticker, title, message, severity, read, created_at
             FROM notifications WHERE read = 0 ORDER BY created_at DESC LIMIT ?1"
        } else {
            "SELECT id, alert_id, ticker, title, message, severity, read, created_at
             FROM notifications ORDER BY created_at DESC LIMIT ?1"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let items = stmt
            .query_map(params![limit], |row| {
                Ok(Notification {
                    id: row.get(0)?,
                    alert_id: row.get(1)?,
                    ticker: row.get(2)?,
                    title: row.get(3)?,
                    message: row.get(4)?,
                    severity: row.get(5)?,
                    read: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    pub fn get_unread_count(&self) -> Result<i64, rusqlite::Error> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM notifications WHERE read = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn mark_read(&self, id: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("UPDATE notifications SET read = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn mark_all_read(&self) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("UPDATE notifications SET read = 1 WHERE read = 0", [])?;
        Ok(())
    }

    pub fn clear_old(&self, days: i64) -> Result<usize, rusqlite::Error> {
        let cutoff = chrono::Utc::now().timestamp() - (days * 86400);
        let affected = self.conn.execute(
            "DELETE FROM notifications WHERE created_at < ?1",
            params![cutoff],
        )?;
        Ok(affected)
    }
}
