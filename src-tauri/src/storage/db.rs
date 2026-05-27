use rusqlite::{Connection, Error, Result};
use std::fs;
use std::path::Path;
use crate::storage::migrations::get_migrations;

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
    db.run_migrations()
}
