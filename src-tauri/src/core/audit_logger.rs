use rusqlite::params;
use crate::storage::db::Database;

pub struct AuditLogger;

impl AuditLogger {
    pub fn log_event(task_id: &str, level: &str, message: &str, gate_name: Option<&str>, event_type: Option<&str>, metadata_json: Option<&str>) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO execution_logs (task_id, level, message, gate_name, event_type, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                task_id,
                level,
                message,
                gate_name,
                event_type,
                metadata_json
            ],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }
}
