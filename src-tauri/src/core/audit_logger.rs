use crate::storage::db::Database;
use rusqlite::params;
use uuid::Uuid;

pub struct AuditLogger;

impl AuditLogger {
    pub fn log_event(
        task_id: &str,
        level: &str,
        message: &str,
        gate_name: Option<&str>,
        event_type: Option<&str>,
        metadata_json: Option<&str>,
    ) -> Result<(), String> {
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

    pub fn log_operation_event(
        actor: &str,
        action: &str,
        status: &str,
        target_type: Option<&str>,
        target_id: Option<&str>,
        details: Option<&str>,
        metadata_json: Option<&str>,
        error_message: Option<&str>,
        correlation_id: Option<&str>,
    ) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO operation_audit_events (
                id,
                actor,
                action,
                target_type,
                target_id,
                status,
                details,
                metadata_json,
                error_message,
                correlation_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                actor,
                action,
                target_type,
                target_id,
                status,
                details,
                metadata_json,
                error_message,
                correlation_id
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}
