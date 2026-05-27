use crate::storage::db::Database;
use rusqlite::params;
use uuid::Uuid;

pub struct CheckpointManager;

impl CheckpointManager {
    pub fn verify_checkpoint(
        task_id: &str,
        node_id: Option<&str>,
        checkpoint_type: &str,
        is_valid: bool,
    ) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        // Real control check: check if there are any error logs for this task
        let error_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM execution_logs WHERE task_id = ?1 AND level = 'error'",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Kontrol noktası hata logu sorgulanamadı: {}", e))?;

        let valid = is_valid && (error_count == 0);
        let status = if valid { "passed" } else { "failed" };
        let result_msg = if valid {
            format!(
                "Kontrol noktası ({}) başarıyla doğrulandı. Hata oranı: 0%",
                checkpoint_type
            )
        } else {
            format!("HATA: Kontrol noktası ({}) doğrulanamadı! Sistemde {} adet hata logu tespit edildi.", checkpoint_type, error_count)
        };

        let id = Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO checkpoints (id, task_id, decision_node_id, checkpoint_type, status, result)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, task_id, node_id, checkpoint_type, status, result_msg],
        ).map_err(|e| e.to_string())?;

        if !valid {
            return Err(result_msg);
        }

        Ok(())
    }
}
