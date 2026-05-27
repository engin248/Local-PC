use rusqlite::params;
use uuid::Uuid;
use crate::storage::db::Database;
use std::path::Path;
use std::fs;
use sha2::{Digest, Sha256};

pub struct TestManager;

fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

impl TestManager {
    pub fn run_test(task_id: &str, test_name: &str, expected: &str, actual: &str) -> Result<(), String> {
        let mut actual_result = actual.to_string();

        if test_name.starts_with("file_exists:") {
            let file_path = test_name.trim_start_matches("file_exists:");
            if Path::new(file_path).exists() {
                actual_result = "exists".to_string();
            } else {
                actual_result = "not_found".to_string();
            }
        } else if test_name.starts_with("file_contains:") {
            let param_part = test_name.trim_start_matches("file_contains:");
            let parts: Vec<&str> = param_part.split('|').collect();
            if parts.len() >= 2 {
                let file_path = parts[0];
                let keyword = parts[1];
                if Path::new(file_path).exists() {
                    match fs::read_to_string(file_path) {
                        Ok(content) => {
                            if content.contains(keyword) {
                                actual_result = "contains".to_string();
                            } else {
                                actual_result = format!("missing_keyword: {}", keyword);
                            }
                        }
                        Err(e) => {
                            actual_result = format!("read_error: {}", e);
                        }
                    }
                } else {
                    actual_result = "not_found".to_string();
                }
            }
        } else if test_name.starts_with("file_hash_equals:") {
            let param_part = test_name.trim_start_matches("file_hash_equals:");
            let parts: Vec<&str> = param_part.split('|').collect();
            if parts.len() >= 2 {
                let file_path = parts[0];
                let expected_hash = parts[1];
                if Path::new(file_path).exists() {
                    match fs::read(file_path) {
                        Ok(data) => {
                            let hash = sha256_hash(&data);
                            if hash == expected_hash {
                                actual_result = "hash_equals".to_string();
                            } else {
                                actual_result = format!("hash_mismatch: {}", hash);
                            }
                        }
                        Err(e) => {
                            actual_result = format!("read_error: {}", e);
                        }
                    }
                } else {
                    actual_result = "not_found".to_string();
                }
            }
        } else if test_name.starts_with("file_hash_unchanged:") {
            let param_part = test_name.trim_start_matches("file_hash_unchanged:");
            let parts: Vec<&str> = param_part.split('|').collect();
            if parts.len() >= 2 {
                let file_path = parts[0];
                let expected_hash = parts[1];
                if Path::new(file_path).exists() {
                    match fs::read(file_path) {
                        Ok(data) => {
                            let hash = sha256_hash(&data);
                            if hash == expected_hash {
                                actual_result = "hash_unchanged".to_string();
                            } else {
                                actual_result = format!("hash_changed: {}", hash);
                            }
                        }
                        Err(e) => {
                            actual_result = format!("read_error: {}", e);
                        }
                    }
                } else {
                    actual_result = "not_found".to_string();
                }
            }
        } else if test_name.starts_with("sqlite_query_equals:") {
            let param_part = test_name.trim_start_matches("sqlite_query_equals:");
            let parts: Vec<&str> = param_part.split('|').collect();
            if parts.len() >= 3 {
                let db_path = parts[0];
                let query = parts[1];
                let expected_val = parts[2];
                if Path::new(db_path).exists() {
                    match rusqlite::Connection::open(db_path) {
                        Ok(conn) => {
                            let query_res: Result<String, _> = conn.query_row(query, [], |row| row.get(0));
                            match query_res {
                                Ok(val) => {
                                    if val == expected_val {
                                        actual_result = "query_equals".to_string();
                                    } else {
                                        actual_result = format!("query_mismatch: {}", val);
                                    }
                                }
                                Err(e) => {
                                    actual_result = format!("query_error: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            actual_result = format!("db_open_error: {}", e);
                        }
                    }
                } else {
                    actual_result = "not_found".to_string();
                }
            }
        } else if test_name.starts_with("approval_exists:") {
            let param_part = test_name.trim_start_matches("approval_exists:");
            let parts: Vec<&str> = param_part.split('|').collect();
            if parts.len() >= 3 {
                let t_id = parts[0];
                let n_id = parts[1];
                let act = parts[2];
                let db = Database::new();
                if let Ok(conn) = db.get_connection() {
                    let exists: bool = conn.query_row(
                        "SELECT EXISTS(
                            SELECT 1 FROM approvals
                            WHERE task_id = ?1
                            AND decision_node_id = ?2
                            AND action = ?3
                            AND status = 'approved'
                            AND approved_at IS NOT NULL
                            AND approver_id IS NOT NULL
                            AND TRIM(approver_id) != ''
                            AND approver_role IN ('admin', 'owner', 'security_officer')
                        )",
                        params![t_id, n_id, act],
                        |row| row.get(0),
                    ).map_err(|e| format!("approval_exists sorgusu başarısız: {}", e))?;
                    if exists {
                        actual_result = "approved_exists".to_string();
                    } else {
                        actual_result = "not_approved".to_string();
                    }
                } else {
                    actual_result = "db_error".to_string();
                }
            }
        } else if test_name.starts_with("snapshot_exists:") {
            let param_part = test_name.trim_start_matches("snapshot_exists:");
            let parts: Vec<&str> = param_part.split('|').collect();
            if parts.len() >= 2 {
                let t_id = parts[0];
                let n_id = parts[1];
                let db = Database::new();
                if let Ok(conn) = db.get_connection() {
                    let snapshot_path_opt: Result<String, _> = conn.query_row(
                        "SELECT snapshot_path FROM snapshots WHERE task_id = ?1 AND state_id = ?2",
                        params![t_id, n_id],
                        |row| row.get(0),
                    );
                    match snapshot_path_opt {
                        Ok(path) => {
                            if Path::new(&path).exists() {
                                actual_result = "snapshot_verified".to_string();
                            } else {
                                actual_result = "file_missing".to_string();
                            }
                        }
                        Err(_) => {
                            actual_result = "no_snapshot_record".to_string();
                        }
                    }
                } else {
                    actual_result = "db_error".to_string();
                }
            }
        } else if test_name.starts_with("rollback_restored:") {
            let param_part = test_name.trim_start_matches("rollback_restored:");
            let parts: Vec<&str> = param_part.split('|').collect();
            if parts.len() >= 2 {
                let file_path = parts[0];
                let expected_content = parts[1];
                if Path::new(file_path).exists() {
                    match fs::read_to_string(file_path) {
                        Ok(content) => {
                            if content == expected_content {
                                actual_result = "restored_equals".to_string();
                            } else {
                                actual_result = format!("restored_mismatch: {}", content);
                            }
                        }
                        Err(e) => {
                            actual_result = format!("read_error: {}", e);
                        }
                    }
                } else {
                    actual_result = "not_found".to_string();
                }
            }
        } else if test_name.starts_with("no_unapproved_write:") {
            let param_part = test_name.trim_start_matches("no_unapproved_write:");
            let parts: Vec<&str> = param_part.split('|').collect();
            if parts.len() >= 3 {
                let t_id = parts[0];
                let n_id = parts[1];
                let act = parts[2];
                let db = Database::new();
                if let Ok(conn) = db.get_connection() {
                    let unapproved_write_exists: bool = conn.query_row(
                        "SELECT EXISTS(
                            SELECT 1 FROM execution_logs
                            WHERE task_id = ?1
                            AND event_type = 'write_executed'
                            AND metadata_json LIKE '%' || ?2 || '%'
                            AND NOT EXISTS (
                                SELECT 1 FROM approvals
                                WHERE task_id = ?1
                                AND decision_node_id = ?2
                                AND action = ?3
                                AND status = 'approved'
                                AND approved_at IS NOT NULL
                                AND approver_id IS NOT NULL
                                AND TRIM(approver_id) != ''
                                AND approver_role IN ('admin', 'owner', 'security_officer')
                            )
                        )",
                        params![t_id, n_id, act],
                        |row| row.get(0),
                    ).map_err(|e| format!("no_unapproved_write sorgusu başarısız: {}", e))?;
                    actual_result = if unapproved_write_exists {
                        "unapproved_write_found".to_string()
                    } else {
                        "no_unapproved_write".to_string()
                    };
                } else {
                    actual_result = "db_error".to_string();
                }
            }
        } else if test_name.starts_with("build_command_passed:") {
            actual_result = if actual == "passed" {
                "build_passed".to_string()
            } else {
                "build_not_verified".to_string()
            };
        } else {
            return Err(format!(
                "Desteklenmeyen veya çalıştırılamayan test kriteri: {}",
                test_name
            ));
        }

        let is_passed = expected == actual_result;
        let status = if is_passed { "passed" } else { "failed" };

        let id = Uuid::new_v4().to_string();
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO tests (id, task_id, test_name, expected_result, actual_result, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, task_id, test_name, expected, actual_result, status],
        ).map_err(|e| e.to_string())?;

        if !is_passed {
            return Err(format!(
                "Test Başarısız! Test: {}, Beklenen: {}, Alınan: {}",
                test_name, expected, actual_result
            ));
        }

        Ok(())
    }
}
