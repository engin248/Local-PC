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

        let conflicting_impact_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM risk_assessments current_risk
                 JOIN risk_assessments other_risk
                   ON current_risk.affected_assets_json = other_risk.affected_assets_json
                  AND current_risk.task_id != other_risk.task_id
                 JOIN tasks other_task ON other_task.id = other_risk.task_id
                 WHERE current_risk.task_id = ?1
                   AND TRIM(current_risk.affected_assets_json) != ''
                   AND current_risk.risk_level IN ('high', 'critical')
                   AND other_task.status NOT IN ('completed', 'failed', 'rolled_back')",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Etki alanı çakışma sorgusu çalıştırılamadı: {}", e))?;
        if conflicting_impact_count > 0 {
            return Err(format!(
                "Bütünlük Hatası: Etki alanı çakışması tespit edildi. Çakışan aktif kayıt sayısı: {}",
                conflicting_impact_count
            ));
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
             VALUES (?1, 'Test', 'Test', 'completed', 'planning_complete', 'completed', 'low', 'policy_checked_no_user_approval_required')",
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

    fn insert_complete_low_risk_fixture(conn: &rusqlite::Connection, task_id: &str, node_id: &str) {
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]);
        let _ = conn.execute(
            "DELETE FROM task_breakdown WHERE task_id = ?1",
            params![task_id],
        );
        let _ = conn.execute(
            "DELETE FROM decision_nodes WHERE task_id = ?1",
            params![task_id],
        );
        let _ = conn.execute("DELETE FROM tests WHERE task_id = ?1", params![task_id]);
        let _ = conn.execute(
            "DELETE FROM risk_assessments WHERE task_id = ?1",
            params![task_id],
        );
        let _ = conn.execute("DELETE FROM snapshots WHERE task_id = ?1", params![task_id]);
        let _ = conn.execute("DELETE FROM approvals WHERE task_id = ?1", params![task_id]);
        let _ = conn.execute(
            "DELETE FROM principle_evaluations WHERE task_id = ?1",
            params![task_id],
        );
        let _ = conn.execute(
            "DELETE FROM alternatives WHERE decision_node_id = ?1",
            params![node_id],
        );
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 'Test', 'Test', 'completed', 'planning_complete', 'completed', 'low', 'policy_checked_no_user_approval_required')",
            params![task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO task_breakdown (id, task_id, level, topic, subtopic, criterion, subcriterion)
             VALUES (?1, ?2, 1, 'Konu', 'Alt', 'Kriter', 'Alt Kriter')",
            params![format!("{}_breakdown", task_id), task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO decision_nodes (id, task_id, breakdown_id, level, required_approval, gate_status, authorized_decider_type, authorized_decider_id, status)
             VALUES (?1, ?2, ?3, 1, 0, 'passed', 'ai', 'system', 'completed')",
            params![node_id, task_id, format!("{}_breakdown", task_id)],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tests (id, task_id, test_name, expected_result, actual_result, status)
             VALUES (?1, ?2, 'file_exists:x', 'exists', 'exists', 'passed')",
            params![format!("{}_test", task_id), task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO principle_evaluations (id, task_id, accepted_correct_approach_reason, selected_best_option_reason, status)
             VALUES (?1, ?2, 'Genel dogru yaklasim rollback test ve kullanici onayini korur.', 'Secilen secenek uygulanabilir ve test edilebilir oldugu icin uygundur.', 'passed')",
            params![format!("{}_principle", task_id), task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO alternatives (id, decision_node_id, title, description, pros_json, cons_json, accuracy_score, safety_score, dependency_score, rollback_score, maintainability_score, cost_score, time_score, user_control_score, live_impact_score, data_loss_risk_score, real_world_basis, testability_score, ethical_safety_note, selection_reason, accepted_correct_approach_reason, selected_best_option_reason, selected, reason)
             VALUES (?1, ?2, 'Secilen', 'Aciklama', '[]', '[]', 9, 9, 9, 9, 9, 9, 9, 9, 1, 1, 'Gercek operasyon dayanagi', 9, 'Etik guvenlik notu', 'Secim gerekcesi', 'Genel dogru yaklasim rollback test ve kullanici onayini korur.', 'Secilen secenek uygulanabilir ve test edilebilir oldugu icin uygundur.', 1, 'Secildi')",
            params![format!("{}_alt", task_id), node_id],
        )
        .unwrap();
    }

    #[test]
    fn incomplete_decision_node_fails_integrity() {
        let task_id = "test_integrity_incomplete_decision";
        let node_id = "test_integrity_incomplete_decision_node";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        insert_complete_low_risk_fixture(&conn, task_id, node_id);
        conn.execute(
            "UPDATE decision_nodes SET status = 'pending' WHERE id = ?1",
            params![node_id],
        )
        .unwrap();
        assert!(IntegrityChecker::check_integrity(task_id).is_err());
    }

    #[test]
    fn missing_selected_best_option_reason_fails_integrity() {
        let task_id = "test_integrity_missing_best_reason";
        let node_id = "test_integrity_missing_best_reason_node";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        insert_complete_low_risk_fixture(&conn, task_id, node_id);
        conn.execute(
            "UPDATE principle_evaluations SET selected_best_option_reason = '' WHERE task_id = ?1",
            params![task_id],
        )
        .unwrap();
        conn.execute(
            "UPDATE alternatives SET selected_best_option_reason = '' WHERE decision_node_id = ?1",
            params![node_id],
        )
        .unwrap();
        assert!(IntegrityChecker::check_integrity(task_id).is_err());
    }

    #[test]
    fn missing_accepted_correct_approach_reason_fails_integrity() {
        let task_id = "test_integrity_missing_correct_reason";
        let node_id = "test_integrity_missing_correct_reason_node";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        insert_complete_low_risk_fixture(&conn, task_id, node_id);
        conn.execute(
            "UPDATE principle_evaluations SET accepted_correct_approach_reason = '' WHERE task_id = ?1",
            params![task_id],
        )
        .unwrap();
        conn.execute(
            "UPDATE alternatives SET accepted_correct_approach_reason = '' WHERE decision_node_id = ?1",
            params![node_id],
        )
        .unwrap();
        assert!(IntegrityChecker::check_integrity(task_id).is_err());
    }

    #[test]
    fn high_risk_without_snapshot_fails_integrity() {
        let task_id = "test_integrity_high_no_snapshot";
        let node_id = "test_integrity_high_no_snapshot_node";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        insert_complete_low_risk_fixture(&conn, task_id, node_id);
        let _ = conn.execute(
            "DELETE FROM risk_assessments WHERE task_id = ?1",
            params![task_id],
        );
        let _ = conn.execute("DELETE FROM snapshots WHERE task_id = ?1", params![task_id]);
        conn.execute(
            "INSERT INTO risk_assessments (id, task_id, decision_node_id, risk_level, risk_reason, affected_assets_json, mitigation_plan)
             VALUES ('risk_high_no_snapshot', ?1, ?2, 'high', 'Risk', '[\"storage/app.db\"]', 'Mitigation')",
            params![task_id, node_id],
        )
        .unwrap();
        assert!(IntegrityChecker::check_integrity(task_id).is_err());
    }

    #[test]
    fn active_impact_area_conflict_fails_integrity() {
        let task_id = "test_integrity_impact_conflict";
        let node_id = "test_integrity_impact_conflict_node";
        let other_task_id = "test_integrity_impact_conflict_other";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        insert_complete_low_risk_fixture(&conn, task_id, node_id);
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![other_task_id]);
        let _ = conn.execute(
            "DELETE FROM task_breakdown WHERE task_id = ?1",
            params![other_task_id],
        );
        let _ = conn.execute(
            "DELETE FROM decision_nodes WHERE task_id = ?1",
            params![other_task_id],
        );
        let _ = conn.execute(
            "DELETE FROM risk_assessments WHERE task_id IN (?1, ?2)",
            params![task_id, other_task_id],
        );
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 'Other', 'Other', 'running', 'planning_complete', 'running', 'high', 'approved')",
            params![other_task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO task_breakdown (id, task_id, level, topic, subtopic, criterion, subcriterion)
             VALUES ('breakdown_conflict_other', ?1, 1, 'Konu', 'Alt', 'Kriter', 'Alt Kriter')",
            params![other_task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO decision_nodes (id, task_id, breakdown_id, level, required_approval, gate_status, authorized_decider_type, authorized_decider_id, status)
             VALUES ('other-node', ?1, 'breakdown_conflict_other', 1, 1, 'pending', 'permission_manager', 'user', 'pending')",
            params![other_task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO risk_assessments (id, task_id, decision_node_id, risk_level, risk_reason, affected_assets_json, mitigation_plan)
             VALUES ('risk_conflict_current', ?1, ?2, 'high', 'Risk', '[\"shared-file\"]', 'Mitigation')",
            params![task_id, node_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO risk_assessments (id, task_id, decision_node_id, risk_level, risk_reason, affected_assets_json, mitigation_plan)
             VALUES ('risk_conflict_other', ?1, 'other-node', 'high', 'Risk', '[\"shared-file\"]', 'Mitigation')",
            params![other_task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO snapshots (id, task_id, target_type, target_path, snapshot_path, rollback_status)
             VALUES ('snapshot_conflict_current', ?1, 'file', 'shared-file', 'snapshot', 'active')",
            params![task_id],
        )
        .unwrap();
        assert!(IntegrityChecker::check_integrity(task_id).is_err());
    }
}
