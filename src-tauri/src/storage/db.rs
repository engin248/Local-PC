use crate::storage::migrations::get_migrations;
use rusqlite::{Connection, Error, Result};
use std::fs;
use std::path::Path;

const AI_PLATFORM_CHECK_MARKER: &str = "test_raporlama";
const AI_PLATFORM_CHECK_VALUES: &str = "'codex', 'open_agent_manager', 'antigravity', 'cursor', 'perplexity', 'verdent', 'lokal_bilgisayar_kontrol_paneli', 'asker_motoru_komuta_paneli', 'planlama_departmani', 'egitim_departmani', 'ar_ge_departmani', 'bot_agent_uretim_departmani', 'beceri_kutuphanesi', 'test_raporlama'";

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
        let storage_path = root.join("storage");
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
        Self::ensure_ai_platform_constraints(conn)?;
        Ok(())
    }

    fn table_needs_platform_rebuild(conn: &Connection, table: &str) -> Result<bool> {
        let sql: Result<String> = conn.query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table],
            |row| row.get(0),
        );
        match sql {
            Ok(sql) => Ok(!sql.contains(AI_PLATFORM_CHECK_MARKER)),
            Err(Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn ensure_ai_platform_constraints(conn: &Connection) -> Result<()> {
        if Self::table_needs_platform_rebuild(conn, "ai_task_allocations")? {
            Self::rebuild_ai_task_allocations(conn)?;
        }
        if Self::table_needs_platform_rebuild(conn, "ai_collected_reports")? {
            Self::rebuild_ai_collected_reports(conn)?;
        }
        Ok(())
    }

    fn rebuild_ai_task_allocations(conn: &Connection) -> Result<()> {
        let sql = format!(
            "ALTER TABLE ai_task_allocations RENAME TO ai_task_allocations_old;
             CREATE TABLE ai_task_allocations (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                platform_name TEXT NOT NULL CHECK(platform_name IN ({})),
                assigned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                status TEXT NOT NULL CHECK(status IN ('waiting', 'processing', 'submitted', 'failed', 'rejected')),
                payload_file_path TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES ai_tasks(id) ON DELETE CASCADE,
                UNIQUE(task_id, platform_name)
             );
             INSERT INTO ai_task_allocations (id, task_id, platform_name, assigned_at, status, payload_file_path)
                SELECT id, task_id, platform_name, assigned_at, status, payload_file_path
                FROM ai_task_allocations_old;
             DROP TABLE ai_task_allocations_old;",
            AI_PLATFORM_CHECK_VALUES
        );
        conn.execute_batch(&sql)
    }

    fn rebuild_ai_collected_reports(conn: &Connection) -> Result<()> {
        let sql = format!(
            "ALTER TABLE ai_collected_reports RENAME TO ai_collected_reports_old;
             CREATE TABLE ai_collected_reports (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                platform_name TEXT NOT NULL CHECK(platform_name IN ({})),
                submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                report_path TEXT NOT NULL,
                is_verified INTEGER DEFAULT 0 CHECK(is_verified IN (0, 1)),
                verification_error TEXT,
                FOREIGN KEY(task_id) REFERENCES ai_tasks(id) ON DELETE CASCADE,
                UNIQUE(task_id, platform_name)
             );
             INSERT INTO ai_collected_reports (id, task_id, platform_name, submitted_at, report_path, is_verified, verification_error)
                SELECT id, task_id, platform_name, submitted_at, report_path, is_verified, verification_error
                FROM ai_collected_reports_old;
             DROP TABLE ai_collected_reports_old;",
            AI_PLATFORM_CHECK_VALUES
        );
        conn.execute_batch(&sql)
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

#[cfg(test)]
mod tests {
    use super::Database;
    use rusqlite::Connection;

    #[test]
    fn expands_existing_ai_platform_constraints() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE ai_tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                risk_level TEXT NOT NULL CHECK(risk_level IN ('low', 'medium', 'high', 'critical')),
                status TEXT NOT NULL CHECK(status IN ('pending', 'approved', 'in_progress', 'completed', 'failed', 'rejected')),
                created_by TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
             );
             CREATE TABLE ai_task_allocations (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                platform_name TEXT NOT NULL CHECK(platform_name IN ('codex', 'open_agent_manager')),
                assigned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                status TEXT NOT NULL CHECK(status IN ('waiting', 'processing', 'submitted', 'failed', 'rejected')),
                payload_file_path TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES ai_tasks(id) ON DELETE CASCADE,
                UNIQUE(task_id, platform_name)
             );
             CREATE TABLE ai_collected_reports (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                platform_name TEXT NOT NULL CHECK(platform_name IN ('codex', 'open_agent_manager')),
                submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                report_path TEXT NOT NULL,
                is_verified INTEGER DEFAULT 0 CHECK(is_verified IN (0, 1)),
                verification_error TEXT,
                FOREIGN KEY(task_id) REFERENCES ai_tasks(id) ON DELETE CASCADE,
                UNIQUE(task_id, platform_name)
             );
             INSERT INTO ai_tasks (id, title, risk_level, status, created_by)
                VALUES ('task_one', 'Task One', 'low', 'pending', 'codex');
             INSERT INTO ai_task_allocations (id, task_id, platform_name, status, payload_file_path)
                VALUES ('alloc_existing', 'task_one', 'codex', 'waiting', 'ai_workflow/tasks/task.json');
             INSERT INTO ai_collected_reports (id, task_id, platform_name, report_path, is_verified)
                VALUES ('report_existing', 'task_one', 'open_agent_manager', 'ai_workflow/collected_reports/report.md', 1);",
        )
        .unwrap();

        Database::ensure_ai_platform_constraints(&conn).unwrap();

        conn.execute(
            "INSERT INTO ai_task_allocations (id, task_id, platform_name, status, payload_file_path)
             VALUES ('alloc_department', 'task_one', 'planlama_departmani', 'waiting', 'ai_workflow/tasks/department.json')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_collected_reports (id, task_id, platform_name, report_path, is_verified)
             VALUES ('report_department', 'task_one', 'test_raporlama', 'ai_workflow/collected_reports/department.md', 1)",
            [],
        )
        .unwrap();

        let existing_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ai_task_allocations WHERE id = 'alloc_existing'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(existing_count, 1);
    }
}
