use crate::storage::db::Database;
use rusqlite::params;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationStep {
    pub order: i32,
    pub expected_action: String,
    pub description: String,
}

pub struct OperationMonitor;

impl OperationMonitor {
    pub fn initialize(task_id: &str, operation_plan: &str) -> Result<Vec<OperationStep>, String> {
        let steps = Self::parse_operation_plan(operation_plan)?;
        if steps.is_empty() {
            return Err(
                "HATA: OperationMonitor icin operasyon plani adimi bulunamadi.".to_string(),
            );
        }

        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM operation_steps WHERE task_id = ?1",
            params![task_id],
        )
        .map_err(|e| e.to_string())?;

        for step in &steps {
            conn.execute(
                "INSERT INTO operation_steps (id, task_id, step_order, expected_action, description, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'pending')",
                params![
                    Uuid::new_v4().to_string(),
                    task_id,
                    step.order,
                    step.expected_action,
                    step.description
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        Ok(steps)
    }

    pub fn parse_operation_plan(operation_plan: &str) -> Result<Vec<OperationStep>, String> {
        let normalized = operation_plan.replace(['\n', ';'], ",").replace("->", ",");
        let mut steps = Vec::new();

        for raw in normalized.split(',') {
            let description = raw.trim();
            if description.is_empty() {
                continue;
            }
            if let Some(action) = Self::infer_action(description) {
                steps.push(OperationStep {
                    order: (steps.len() + 1) as i32,
                    expected_action: action,
                    description: description.to_string(),
                });
            }
        }

        if steps.is_empty() {
            return Err("HATA: Operasyon planindan taninabilir action cikarilamadi.".to_string());
        }

        Ok(steps)
    }

    pub fn check_action(
        task_id: &str,
        decision_node_id: Option<&str>,
        actual_action: &str,
        gate_name: &str,
    ) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        let expected: Result<(String, i32), _> = conn.query_row(
            "SELECT expected_action, step_order FROM operation_steps
             WHERE task_id = ?1 AND status = 'pending'
             ORDER BY step_order ASC LIMIT 1",
            params![task_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        let (expected_action, step_order) = expected.map_err(|_| {
            "HATA: OperationMonitor beklenen sirada pending operasyon adimi bulamadi.".to_string()
        })?;

        if expected_action == "report_generate"
            && actual_action == "report_generate"
            && gate_name != "Report Gate"
        {
            Self::write_log(
                task_id,
                decision_node_id,
                Some(&expected_action),
                actual_action,
                Some(gate_name),
                "passed",
                "OperationMonitor ara karar dugumu rapor aksiyonunu dogruladi; final rapor adimi beklemede tutuldu.",
            )?;
            return Ok(());
        }

        if expected_action != actual_action {
            if expected_action == "report_generate"
                && Self::is_known_nonterminal_action(&conn, task_id, actual_action)?
            {
                Self::write_log(
                    task_id,
                    decision_node_id,
                    Some(&expected_action),
                    actual_action,
                    Some(gate_name),
                    "passed",
                    "OperationMonitor ara karar dugumu dongusunu dogruladi; final rapor adimi beklemede tutuldu.",
                )?;
                return Ok(());
            }

            let message = format!(
                "HATA: Plan disi action engellendi. Beklenen: {}, gelen: {}",
                expected_action, actual_action
            );
            Self::write_log(
                task_id,
                decision_node_id,
                Some(&expected_action),
                actual_action,
                Some(gate_name),
                "failed",
                &message,
            )?;
            return Err(message);
        }

        conn.execute(
            "UPDATE operation_steps SET status = 'completed'
             WHERE task_id = ?1 AND step_order = ?2",
            params![task_id, step_order],
        )
        .map_err(|e| e.to_string())?;

        Self::write_log(
            task_id,
            decision_node_id,
            Some(&expected_action),
            actual_action,
            Some(gate_name),
            "passed",
            "OperationMonitor action sirasi dogrulandi.",
        )?;

        Ok(())
    }

    fn is_known_nonterminal_action(
        conn: &rusqlite::Connection,
        task_id: &str,
        actual_action: &str,
    ) -> Result<bool, String> {
        if actual_action == "report_generate" {
            return Ok(false);
        }
        const DECISION_LOOP_ACTIONS: &[&str] = &[
            "read_file",
            "read_folder",
            "sqlite_read",
            "code_analysis",
            "code_modification_proposal",
            "research",
            "ai_provider_call",
            "approval_check",
            "snapshot_create",
            "test_run",
            "write_file",
            "file_write",
            "sqlite_write",
            "terminal_command",
        ];
        if DECISION_LOOP_ACTIONS.contains(&actual_action) {
            return Ok(true);
        }

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_steps
                 WHERE task_id = ?1 AND expected_action = ?2 AND expected_action != 'report_generate'",
                params![task_id, actual_action],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(count > 0)
    }

    pub fn log_gate(
        task_id: &str,
        decision_node_id: Option<&str>,
        gate_name: &str,
        status: &str,
        message: &str,
    ) -> Result<(), String> {
        Self::write_log(
            task_id,
            decision_node_id,
            None,
            "gate_checkpoint",
            Some(gate_name),
            status,
            message,
        )
    }

    pub fn ensure_all_steps_completed(task_id: &str) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let pending_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_steps WHERE task_id = ?1 AND status != 'completed'",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if pending_count > 0 {
            return Err(format!(
                "HATA: OperationMonitor tamamlanmamis operasyon adimi tespit etti: {}",
                pending_count
            ));
        }
        Ok(())
    }

    fn write_log(
        task_id: &str,
        decision_node_id: Option<&str>,
        expected_action: Option<&str>,
        actual_action: &str,
        gate_name: Option<&str>,
        status: &str,
        message: &str,
    ) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO operation_monitor_logs
             (id, task_id, decision_node_id, expected_action, actual_action, gate_name, status, message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                Uuid::new_v4().to_string(),
                task_id,
                decision_node_id,
                expected_action,
                actual_action,
                gate_name,
                status,
                message
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn infer_action(step: &str) -> Option<String> {
        let lower = step.to_lowercase();
        if let Some(action) = lower
            .split_whitespace()
            .find_map(|part| part.strip_prefix("action:"))
        {
            return Some(action.trim_matches(|c| c == ',' || c == ';').to_string());
        }

        if lower.contains("ai") || lower.contains("analiz") || lower.contains("analysis") {
            Some("code_analysis".to_string())
        } else if lower.contains("folder") || lower.contains("klasor") || lower.contains("klasör")
        {
            Some("read_folder".to_string())
        } else if lower.contains("oku") || lower.contains("read") {
            Some("read_file".to_string())
        } else if lower.contains("onay") || lower.contains("approval") {
            Some("approval_check".to_string())
        } else if lower.contains("snapshot")
            || lower.contains("yedek")
            || lower.contains("rollback")
        {
            Some("snapshot_create".to_string())
        } else if lower.contains("yaz") || lower.contains("write") {
            Some("write_file".to_string())
        } else if lower.contains("test") || lower.contains("dogrula") || lower.contains("doğrula")
        {
            Some("test_run".to_string())
        } else if lower.contains("rapor") || lower.contains("report") {
            Some("report_generate".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::Database;

    #[test]
    fn operation_plan_out_of_order_action_fails() {
        let task_id = "test_operation_monitor_out_of_order";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]);
        let _ = conn.execute(
            "DELETE FROM operation_steps WHERE task_id = ?1",
            params![task_id],
        );
        let _ = conn.execute(
            "DELETE FROM operation_monitor_logs WHERE task_id = ?1",
            params![task_id],
        );
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 'Test', 'Test', 'pending', 'planning_complete', 'not_started', 'low', 'policy_checked_no_user_approval_required')",
            params![task_id],
        )
        .unwrap();

        OperationMonitor::initialize(
            task_id,
            "action:code_analysis, action:snapshot_create, action:test_run",
        )
        .unwrap();

        let err = OperationMonitor::check_action(task_id, None, "write_file", "Authority Gate")
            .unwrap_err();
        assert!(err.contains("Plan disi action"));
    }
}
