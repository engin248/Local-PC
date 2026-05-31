use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct SupabaseSyncStatus {
    pub enabled: bool,
    pub last_result: String,
    pub pushed_rows: usize,
}

pub struct SupabaseSync;

impl SupabaseSync {
    pub fn is_configured() -> bool {
        let url = std::env::var("SUPABASE_URL").unwrap_or_default();
        let key = std::env::var("SUPABASE_SERVICE_KEY")
            .or_else(|_| std::env::var("SUPABASE_ANON_KEY"))
            .unwrap_or_default();
        !url.trim().is_empty() && !key.trim().is_empty()
    }

    pub fn sync_recent_tasks(limit: usize) -> Result<SupabaseSyncStatus, String> {
        if !Self::is_configured() {
            return Ok(SupabaseSyncStatus {
                enabled: false,
                last_result: "SUPABASE_URL veya anahtar tanımlı değil; sync atlandı.".to_string(),
                pushed_rows: 0,
            });
        }

        let url = std::env::var("SUPABASE_URL").map_err(|e| e.to_string())?;
        let key = std::env::var("SUPABASE_SERVICE_KEY")
            .or_else(|_| std::env::var("SUPABASE_ANON_KEY"))
            .map_err(|e| e.to_string())?;
        let endpoint = format!(
            "{}/rest/v1/panel_tasks",
            url.trim_end_matches('/')
        );

        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, title, status, planning_status, execution_status, risk_level
                 FROM tasks ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "status": row.get::<_, String>(2)?,
                    "planning_status": row.get::<_, String>(3)?,
                    "execution_status": row.get::<_, String>(4)?,
                    "risk_level": row.get::<_, String>(5)?,
                }))
            })
            .map_err(|e| e.to_string())?;

        let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build();
        let mut pushed = 0usize;
        for row in rows {
            let payload = row.map_err(|e| e.to_string())?;
            let response = agent
                .post(&endpoint)
                .set("apikey", &key)
                .set("Authorization", &format!("Bearer {}", key))
                .set("Content-Type", "application/json")
                .set("Prefer", "resolution=merge-duplicates")
                .send_json(payload)
                .map_err(|e| format!("Supabase push hatası: {}", e))?;
            if response.status() < 400 {
                pushed += 1;
            }
        }

        Ok(SupabaseSyncStatus {
            enabled: true,
            last_result: format!("{} görev Supabase'e gönderildi", pushed),
            pushed_rows: pushed,
        })
    }
}
