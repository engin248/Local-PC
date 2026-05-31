use crate::core::dependency_analyzer::DependencyAnalyzer;
use crate::storage::db::Database;
use rusqlite::params;
use std::fs;
use std::path::Path;

const MAX_DB_BYTES: u64 = 50 * 1024 * 1024;
const LOG_RETENTION_DAYS_ESTIMATE: i64 = 5000;

pub struct LogRotation;

impl LogRotation {
    pub fn run_if_needed() -> Result<Option<String>, String> {
        let root = DependencyAnalyzer::get_project_root()?;
        let db_path = root.join("storage/app.db");
        if !db_path.exists() {
            return Ok(None);
        }

        let size = fs::metadata(&db_path)
            .map_err(|e| e.to_string())?
            .len();
        if size < MAX_DB_BYTES {
            return Ok(None);
        }

        let archive_dir = root.join("storage/archive");
        fs::create_dir_all(&archive_dir).map_err(|e| e.to_string())?;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let archive_file = archive_dir.join(format!("execution_logs_{}.sql", stamp));

        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let deleted: i64 = conn
            .execute(
                "DELETE FROM execution_logs WHERE id IN (
                    SELECT id FROM execution_logs
                    ORDER BY timestamp ASC
                    LIMIT ?1
                )",
                params![LOG_RETENTION_DAYS_ESTIMATE],
            )
            .map_err(|e| e.to_string())? as i64;

        conn.execute("VACUUM", []).ok();

        let summary = format!(
            "Log rotasyon: {} bayt -> arşiv notu {}; {} log silindi",
            size,
            archive_file.display(),
            deleted
        );
        fs::write(
            archive_dir.join(format!("rotation_note_{}.txt", stamp)),
            &summary,
        )
        .map_err(|e| e.to_string())?;

        Ok(Some(summary))
    }

    pub fn db_size_bytes() -> Result<u64, String> {
        let root = DependencyAnalyzer::get_project_root()?;
        let db_path = root.join("storage/app.db");
        if !Path::new(&db_path).exists() {
            return Ok(0);
        }
        Ok(fs::metadata(&db_path).map_err(|e| e.to_string())?.len())
    }
}
