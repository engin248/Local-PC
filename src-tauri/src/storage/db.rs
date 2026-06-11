use crate::storage::migrations::get_migrations;
use rusqlite::{Connection, Error, Result};
use std::fs;
use std::path::Path;

pub struct Database {
    db_path: String,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|e| {
            eprintln!("KRITIK HATA: Veritabani baslatilamadi: {}", e);
            std::process::exit(1);
        })
    }

    pub fn try_new() -> std::result::Result<Self, String> {
        let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()
            .map_err(|e| format!("KRITIK HATA: Proje kok dizini bulunamadi: {}", e))?;
        let storage_path = root.join(Self::storage_dir_name());
        let storage_dir = storage_path.to_string_lossy().into_owned();

        // Create storage and subdirectories
        let subdirs = vec!["", "/logs", "/reports", "/backups", "/snapshots"];
        for sub in subdirs {
            let path = format!("{}{}", storage_dir, sub);
            if !Path::new(&path).exists() {
                fs::create_dir_all(&path)
                    .map_err(|e| format!("KRITIK HATA: Depolama dizini olusturulamadi: {}", e))?;
            }
        }

        let db_path = format!("{}/app.db", storage_dir);
        let db = Database { db_path };
        db.run_migrations().map_err(|e| e.to_string())?;
        Ok(db)
    }

    fn storage_dir_name() -> String {
        #[cfg(test)]
        {
            let thread_id = format!("{:?}", std::thread::current().id())
                .replace("ThreadId(", "")
                .replace(')', "");
            return std::env::temp_dir()
                .join("lokal_panel_test_storage")
                .join(format!("{}-{}", std::process::id(), thread_id))
                .to_string_lossy()
                .into_owned();
        }
        #[cfg(not(test))]
        {
            "storage".to_string()
        }
    }

    pub fn get_connection(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute("PRAGMA foreign_keys = ON;", [])?;
        let _ = conn.execute("PRAGMA journal_mode = WAL;", []);
        let _ = conn.execute("PRAGMA busy_timeout = 5000;", []);
        Self::ensure_optional_columns(&conn)?;

        Ok(conn)
    }

    fn add_column_if_missing(conn: &Connection, table: &str, definition: &str) -> Result<()> {
        let sql = format!("ALTER TABLE {} ADD COLUMN {}", table, definition);
        match conn.execute(&sql, []) {
            Ok(_) => Ok(()),
            Err(Error::SqliteFailure(err, Some(message)))
                if err.extended_code == 1
                    && (message.contains("duplicate column name")
                        || message.contains("no such table")) =>
            {
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn ensure_extended_platform_tables(conn: &Connection) -> Result<()> {
        let needs_upgrade: bool = conn
            .prepare(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'ai_task_allocations'",
            )
            .ok()
            .and_then(|mut stmt| {
                stmt.query_row([], |row| row.get::<_, String>(0)).ok()
            })
            .map(|sql| !sql.contains("burhan_command"))
            .unwrap_or(false);

        if !needs_upgrade {
            return Ok(());
        }

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS ai_task_allocations_v2 (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                platform_name TEXT NOT NULL,
                assigned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                status TEXT NOT NULL CHECK(status IN ('waiting', 'processing', 'submitted', 'failed', 'rejected')),
                payload_file_path TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES ai_tasks(id) ON DELETE CASCADE,
                UNIQUE(task_id, platform_name)
            );
            INSERT OR IGNORE INTO ai_task_allocations_v2
                SELECT id, task_id, platform_name, assigned_at, status, payload_file_path
                FROM ai_task_allocations;
            DROP TABLE IF EXISTS ai_task_allocations;
            ALTER TABLE ai_task_allocations_v2 RENAME TO ai_task_allocations;

            CREATE TABLE IF NOT EXISTS ai_collected_reports_v2 (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                platform_name TEXT NOT NULL,
                submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                report_path TEXT NOT NULL,
                is_verified INTEGER DEFAULT 0 CHECK(is_verified IN (0, 1)),
                verification_error TEXT,
                FOREIGN KEY(task_id) REFERENCES ai_tasks(id) ON DELETE CASCADE,
                UNIQUE(task_id, platform_name)
            );
            INSERT OR IGNORE INTO ai_collected_reports_v2
                SELECT id, task_id, platform_name, submitted_at, report_path, is_verified, verification_error
                FROM ai_collected_reports;
            DROP TABLE IF EXISTS ai_collected_reports;
            ALTER TABLE ai_collected_reports_v2 RENAME TO ai_collected_reports;
            "#,
        )?;
        Ok(())
    }

    fn ensure_optional_columns(conn: &Connection) -> Result<()> {
        Self::add_column_if_missing(conn, "approvals", "approver_id TEXT")?;
        Self::add_column_if_missing(conn, "approvals", "approver_role TEXT")?;
        Self::add_column_if_missing(conn, "approvals", "approval_source TEXT")?;
        Self::add_column_if_missing(conn, "alternatives", "real_world_basis TEXT")?;
        Self::add_column_if_missing(conn, "alternatives", "testability_score INTEGER DEFAULT 0")?;
        Self::add_column_if_missing(conn, "alternatives", "ethical_safety_note TEXT")?;
        Self::add_column_if_missing(conn, "alternatives", "selection_reason TEXT")?;
        Self::add_column_if_missing(
            conn,
            "alternatives",
            "accepted_correct_approach_reason TEXT",
        )?;
        Self::add_column_if_missing(conn, "alternatives", "selected_best_option_reason TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "operation_type TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "technology TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "impact_area TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "control_point TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "control_criteria TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "test_plan TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "rollback_plan TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "executor_role TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "correctness_guard_role TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "controller_role TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "independent_verifier_role TEXT")?;
        Self::add_column_if_missing(conn, "operation_steps", "final_approver_role TEXT")?;
        Self::add_column_if_missing(conn, "system_alarm_events", "alarm_code TEXT")?;
        Self::ensure_extended_platform_tables(conn)?;
        Ok(())
    }

    pub fn run_migrations(&self) -> Result<()> {
        let mut conn = self.get_connection()?;
        let tx = conn.transaction()?;

        for migration in get_migrations() {
            tx.execute(migration, [])?;
        }

        tx.commit()?;
        Self::ensure_optional_columns(&conn)?;
        Ok(())
    }
}

pub fn init_db() -> Result<Connection> {
    let db = Database::new();
    db.get_connection()
}

pub fn initialize_database() -> Result<()> {
    let db = Database::new();
    db.run_migrations()?;
    let _ = crate::storage::log_rotation::LogRotation::run_if_needed();
    Ok(())
}
