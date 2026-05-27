use crate::storage::db::Database;
use rusqlite::params;

pub struct IntegrityChecker;

impl IntegrityChecker {
    pub fn check_integrity(task_id: &str) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        // 1. Dynamic Check for conflicting decisions or statements (Dynamic Integrity Check)
        let conflict_exists: bool = conn
            .query_row(
                "SELECT EXISTS (
                SELECT 1 FROM statements s1 
                JOIN statements s2 ON s1.decision_node_id = s2.decision_node_id
                WHERE s1.content LIKE '%HATA%' AND s2.content LIKE '%BAŞARI%'
             )",
                params![],
                |row| row.get(0),
            )
            .map_err(|e| format!("Bütünlük sorgusu çalıştırılamadı: {}", e))?;

        if conflict_exists {
            return Err("Bütünlük Hatası: Beyanlar arasında çelişkili kararlar veya çakışan bildirimler tespit edildi!".to_string());
        }

        // 2. Verify all decision nodes are approved/passed
        let unpassed_nodes_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM decision_nodes WHERE task_id = ?1 AND status != 'passed' AND status != 'completed'",
            params![task_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Karar düğümü bütünlük sorgusu çalıştırılamadı: {}", e))?;

        if unpassed_nodes_count > 0 {
            return Err(format!("Bütünlük Hatası: Karar düğümleri tamamlanmamış durumda! Eksik karar düğümü sayısı: {}", unpassed_nodes_count));
        }

        // 3. Verify all tests passed
        let failed_tests_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM tests WHERE task_id = ?1 AND status = 'failed'",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Test bütünlük sorgusu çalıştırılamadı: {}", e))?;

        if failed_tests_count > 0 {
            return Err(
                "Bütünlük Hatası: Başarısız testler mevcut olduğundan işlem tamamlanamaz!"
                    .to_string(),
            );
        }

        let tests_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM tests WHERE task_id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Test varlık sorgusu çalıştırılamadı: {}", e))?;
        if tests_count == 0 {
            return Err(
                "Bütünlük Hatası: Test kaydı bulunmadığından işlem tamamlanamaz!".to_string(),
            );
        }

        let pending_approval_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM approvals WHERE task_id = ?1 AND status = 'pending'",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Pending approval sorgusu çalıştırılamadı: {}", e))?;
        if pending_approval_count > 0 {
            return Err(format!(
                "Bütünlük Hatası: Bekleyen onay varken işlem tamamlanamaz. Pending onay sayısı: {}",
                pending_approval_count
            ));
        }

        let high_or_critical_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM risk_assessments
                 WHERE task_id = ?1 AND risk_level IN ('high', 'critical')",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Risk seviyesi sorgusu çalıştırılamadı: {}", e))?;
        if high_or_critical_count > 0 {
            let active_snapshot_count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM snapshots WHERE task_id = ?1 AND rollback_status = 'active'",
                    params![task_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Snapshot sorgusu çalıştırılamadı: {}", e))?;
            if active_snapshot_count == 0 {
                return Err(
                    "Bütünlük Hatası: High/Critical risk için rollback snapshot yok.".to_string(),
                );
            }
        }

        let selected_reason_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM alternatives
                 WHERE decision_node_id IN (SELECT id FROM decision_nodes WHERE task_id = ?1)
                 AND selected = 1
                 AND TRIM(COALESCE(selected_best_option_reason, selection_reason, reason, '')) != ''
                 AND TRIM(COALESCE(accepted_correct_approach_reason, '')) != ''",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Seçilen alternatif gerekçe sorgusu çalıştırılamadı: {}", e))?;
        if selected_reason_count == 0 {
            return Err("Bütünlük Hatası: selected_best_option_reason veya accepted_correct_approach_reason bulunmadan işlem tamamlanamaz.".to_string());
        }

        let principle_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM principle_evaluations
                 WHERE task_id = ?1
                 AND status = 'passed'
                 AND TRIM(accepted_correct_approach_reason) != ''
                 AND TRIM(selected_best_option_reason) != ''",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Prensip değerlendirmesi sorgusu çalıştırılamadı: {}", e))?;
        if principle_count == 0 {
            return Err(
                "Bütünlük Hatası: Prensip değerlendirmesi tamamlanmadan işlem tamamlanamaz."
                    .to_string(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::IntegrityChecker;
    use crate::storage::db::Database;
    use rusqlite::params;

    #[test]
    fn completed_status_with_no_tests_fails_integrity() {
        let task_id = "test_integrity_no_tests";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]);
        let _ = conn.execute("DELETE FROM tests WHERE task_id = ?1", params![task_id]);
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 'Test', 'Test', 'completed', 'planning_complete', 'completed', 'low', 'not_required')",
            params![task_id],
        )
        .unwrap();
        assert!(IntegrityChecker::check_integrity(task_id).is_err());
    }

    #[test]
    fn pending_approval_fails_integrity() {
        let task_id = "test_integrity_pending_approval";
        let node_id = "test_integrity_pending_approval_node";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]);
        let _ = conn.execute(
            "DELETE FROM task_breakdown WHERE task_id = ?1",
            params![task_id],
        );
        let _ = conn.execute(
            "DELETE FROM decision_nodes WHERE task_id = ?1",
            params![task_id],
        );
        let _ = conn.execute("DELETE FROM approvals WHERE task_id = ?1", params![task_id]);
        let _ = conn.execute("DELETE FROM tests WHERE task_id = ?1", params![task_id]);
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 'Test', 'Test', 'completed', 'planning_complete', 'completed', 'low', 'pending_approval')",
            params![task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO task_breakdown (id, task_id, level, topic, subtopic, criterion, subcriterion)
             VALUES ('breakdown_pending', ?1, 1, 'Konu', 'Alt', 'Kriter', 'Alt Kriter')",
            params![task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO decision_nodes (id, task_id, breakdown_id, level, required_approval, gate_status, authorized_decider_type, authorized_decider_id, status)
             VALUES (?1, ?2, 'breakdown_pending', 1, 1, 'pending', 'permission_manager', 'user', 'completed')",
            params![node_id, task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tests (id, task_id, test_name, expected_result, actual_result, status)
             VALUES ('test_pending', ?1, 'file_exists:x', 'exists', 'exists', 'passed')",
            params![task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO approvals (id, task_id, decision_node_id, action, risk_level, status)
             VALUES ('approval_pending', ?1, ?2, 'write_file', 'high', 'pending')",
            params![task_id, node_id],
        )
        .unwrap();
        assert!(IntegrityChecker::check_integrity(task_id).is_err());
    }
}
