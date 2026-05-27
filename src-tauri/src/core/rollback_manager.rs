use serde_json::json;
use rusqlite::params;
use uuid::Uuid;
use std::fs;
use std::path::Path;
use crate::storage::db::Database;
use sha2::{Digest, Sha256};

pub struct RollbackManager;

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

impl RollbackManager {
    pub fn create_snapshot(task_id: &str, target_type: &str, target_path: &str) -> Result<String, String> {
        Self::create_snapshot_with_context(task_id, None, target_type, target_path, None)
    }

    pub fn create_snapshot_with_context(
        task_id: &str,
        decision_node_id: Option<&str>,
        target_type: &str,
        target_path: &str,
        operation_id: Option<&str>,
    ) -> Result<String, String> {
        let snapshot_id = Uuid::new_v4().to_string();
        let state_id = Uuid::new_v4().to_string();
        let operation_id = operation_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
        let snapshots_dir = root.join("storage").join("snapshots");
        
        // Ensure snapshots directory exists
        fs::create_dir_all(&snapshots_dir)
            .map_err(|e| format!("HATA: Snapshots dizini olusturulamadi: {}", e))?;

        let snapshot_file = snapshots_dir.join(format!("{}.bak", snapshot_id));
        let snapshot_path = snapshot_file.to_string_lossy().into_owned();

        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let hash_before = Self::hash_target(target_type, target_path)?;

        // 1. PHYSICAL MULTI-TYPE SNAPSHOT COPY
        if target_type == "file" || target_type == "sqlite" {
            if Path::new(target_path).exists() {
                fs::copy(target_path, &snapshot_path)
                    .map_err(|e| format!("HATA: Fiziksel dosya/db yedekleme basarisiz oldu: {}", e))?;
            } else {
                return Err(format!("HATA: Yedeklenecek hedef dosya mevcut degil: {} (Fail-Closed)", target_path));
            }
        } else if target_type == "folder" {
            if Path::new(target_path).exists() {
                copy_dir_all(target_path, &snapshot_path)
                    .map_err(|e| format!("HATA: Fiziksel klasor yedekleme basarisiz oldu: {}", e))?;
            } else {
                return Err(format!("HATA: Yedeklenecek hedef klasor mevcut degil: {} (Fail-Closed)", target_path));
            }
        } else {
            return Err(format!("HATA: Desteklenmeyen yedekleme hedef turu: {} (Fail-Closed)", target_type));
        }

        // Register snapshot in DB
        conn.execute(
            "INSERT INTO snapshots (id, task_id, target_type, target_path, snapshot_path, hash_before, hash_after, state_id, rollback_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                snapshot_id,
                task_id,
                target_type,
                target_path,
                snapshot_path,
                hash_before,
                "",
                state_id,
                "active"
            ],
        ).map_err(|e| e.to_string())?;

        // Write state history for rollback point
        let state_json = json!({
            "task_id": task_id,
            "target_path": target_path,
            "target_type": target_type,
            "snapshot_path": snapshot_path,
            "hash_before": hash_before,
            "decision_node_id": decision_node_id,
            "operation_id": operation_id,
            "state_id": state_id,
            "created_at": chrono::Utc::now().to_rfc3339()
        }).to_string();

        conn.execute(
            "INSERT INTO state_history (id, task_id, state_name, state_json, is_valid)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                state_id,
                task_id,
                "Pre-execution Snapshot State",
                state_json,
                1
            ],
        ).map_err(|e| e.to_string())?;

        // Save last valid state to task
        conn.execute(
            "UPDATE tasks SET last_valid_state_id = ?1 WHERE id = ?2",
            params![state_id, task_id],
        ).map_err(|e| e.to_string())?;

        Ok(snapshot_path)
    }

    fn hash_target(target_type: &str, target_path: &str) -> Result<String, String> {
        match target_type {
            "file" | "sqlite" => Self::hash_file(target_path),
            "folder" => Self::hash_folder(target_path),
            _ => Err(format!("HATA: Hash hesaplanamayan hedef türü: {}", target_type)),
        }
    }

    fn hash_file(path: &str) -> Result<String, String> {
        let bytes = fs::read(path)
            .map_err(|e| format!("HATA: Hash için hedef okunamadı ({}): {}", path, e))?;
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn hash_folder(path: &str) -> Result<String, String> {
        let mut entries = Vec::new();
        Self::collect_folder_entries(Path::new(path), Path::new(path), &mut entries)?;
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let mut hasher = Sha256::new();
        for (relative_path, file_hash) in entries {
            hasher.update(relative_path.as_bytes());
            hasher.update(file_hash.as_bytes());
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn collect_folder_entries(
        root: &Path,
        current: &Path,
        entries: &mut Vec<(String, String)>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(current)
            .map_err(|e| format!("HATA: Klasör hash için okunamadı: {}", e))?
        {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                Self::collect_folder_entries(root, &path, entries)?;
            } else {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace("\\", "/");
                let hash = Self::hash_file(&path.to_string_lossy())?;
                entries.push((relative, hash));
            }
        }
        Ok(())
    }

    pub fn trigger_rollback(task_id: &str) -> Result<String, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        // Retrieve last snapshot
        let mut stmt = conn.prepare(
            "SELECT id, target_path, snapshot_path, target_type FROM snapshots 
             WHERE task_id = ?1 AND rollback_status = 'active' 
             ORDER BY created_at DESC LIMIT 1"
        ).map_err(|e| e.to_string())?;

        let snapshot_row = stmt.query_row(params![task_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        });

        if let Ok((snapshot_id, target_path, snapshot_path, target_type)) = snapshot_row {
            // 2. PHYSICAL RESTORE BY TYPE
            if (target_type == "file" || target_type == "sqlite") && Path::new(&snapshot_path).exists() {
                // Ensure parent directory of target path exists
                if let Some(parent) = Path::new(&target_path).parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                if let Err(e) = fs::copy(&snapshot_path, &target_path) {
                    let message = format!("HATA: Fiziksel dosya/db geri yukleme basarisiz oldu: {}", e);
                    Self::mark_rollback_failure(task_id, &message)?;
                    return Err(message);
                }
            } else if target_type == "folder" && Path::new(&snapshot_path).exists() {
                if Path::new(&target_path).exists() {
                    if let Err(e) = fs::remove_dir_all(&target_path) {
                        let message = format!("HATA: Eski hedef klasor temizlenemedi: {}", e);
                        Self::mark_rollback_failure(task_id, &message)?;
                        return Err(message);
                    }
                }
                if let Err(e) = copy_dir_all(&snapshot_path, &target_path) {
                    let message = format!("HATA: Fiziksel klasor geri yukleme basarisiz oldu: {}", e);
                    Self::mark_rollback_failure(task_id, &message)?;
                    return Err(message);
                }
            } else {
                let message = format!("HATA: Snapshot yedek dosyalari fiziksel olarak mevcut degil: {}", snapshot_path);
                Self::mark_rollback_failure(task_id, &message)?;
                return Err(message);
            }

            conn.execute(
                "UPDATE snapshots SET rollback_status = 'rolled_back', hash_after = ?1 WHERE id = ?2",
                params![Self::hash_target(&target_type, &target_path).unwrap_or_default(), snapshot_id],
            ).map_err(|e| e.to_string())?;

            return Ok(format!(
                "Geri Yukleme Basarili! Sistem son fiziksel yedekten (Snapshot ID: {}) basariyla donduruldu.",
                snapshot_id
            ));
        }

        let message = "Geri alinabilecek gecerli bir sistem snapshot yedegi bulunamadi!".to_string();
        Self::mark_rollback_failure(task_id, &message)?;
        Err(message)
    }

    fn mark_rollback_failure(task_id: &str, reason: &str) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE tasks
             SET status = 'failed',
                 execution_status = 'failed',
                 risk_level = 'critical',
                 current_gate = 'Rollback Failure'
             WHERE id = ?1",
            params![task_id],
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE snapshots
             SET rollback_status = 'failed'
             WHERE task_id = ?1 AND rollback_status = 'active'",
            params![task_id],
        ).map_err(|e| e.to_string())?;
        crate::core::audit_logger::AuditLogger::log_event(
            task_id,
            "error",
            reason,
            Some("Rollback Gate"),
            Some("rollback_failed"),
            None,
        )?;
        Ok(())
    }
}

pub fn rollback_task(task_id: &str) -> Result<bool, String> {
    match RollbackManager::trigger_rollback(task_id) {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}
