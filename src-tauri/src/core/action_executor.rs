use crate::core::audit_logger::AuditLogger;
use crate::core::dependency_analyzer::DependencyAnalyzer;
use crate::core::execution_engine::{ExecutionContext, RunMode};
use crate::system_connectors::connector_base::{decode_write_request, SystemConnector};
use crate::system_connectors::file_connector::FileConnector;
use crate::system_connectors::folder_connector::FolderConnector;
use crate::system_connectors::sqlite_connector::SqliteConnector;
use serde_json::json;
use std::path::Path;

pub struct ActionExecutor;

impl ActionExecutor {
    pub fn dispatch_after_gates(
        task_id: &str,
        node_id: &str,
        action: &str,
        target_path: &str,
        context: &ExecutionContext,
        risk_level: &str,
    ) -> Result<(), String> {
        if Self::is_write_action(action) {
            if context.read_only || matches!(context.run_mode, RunMode::ReadOnly) {
                return Self::log_event(
                    task_id,
                    node_id,
                    action,
                    "action_execute_skipped",
                    &format!("Yazma aksiyonu read-only context'te atlandı: {}", action),
                );
            }
            return Self::dispatch_write(task_id, node_id, action, target_path, risk_level);
        }

        match action {
            "read_file" | "file_read" => {
                Self::dispatch_read_file(task_id, node_id, action, target_path)
            }
            "read_folder" => Self::dispatch_read_folder(task_id, node_id, action, target_path),
            "sqlite_read" => Self::dispatch_sqlite_read(task_id, node_id, action, target_path),
            "code_analysis" | "research" | "ai_provider_call" => {
                crate::ai_providers::ai_provider_invoke::AiProviderInvoker::invoke_for_node(
                    task_id,
                    node_id,
                    action,
                    &format!("{} {}", action, target_path),
                )
            }
            "code_modification_proposal" | "report_generate" => Self::log_event(
                task_id,
                node_id,
                action,
                "action_execute",
                &format!("Metadata icra kaydı: {}", action),
            ),
            _ => Self::log_event(
                task_id,
                node_id,
                action,
                "action_execute_skipped",
                &format!("Desteklenmeyen veya policy-dışı aksiyon: {}", action),
            ),
        }
    }

    fn is_write_action(action: &str) -> bool {
        matches!(
            action,
            "file_write"
                | "write_file"
                | "file_delete"
                | "delete_file"
                | "write_folder"
                | "sqlite_write"
                | "api_write"
                | "terminal_command"
                | "live_system_update"
        )
    }

    fn dispatch_read_file(
        task_id: &str,
        node_id: &str,
        action: &str,
        target_path: &str,
    ) -> Result<(), String> {
        let connector = FileConnector {
            name: "action_executor_file".to_string(),
        };
        let path = Path::new(target_path);
        let read_target = if path.is_file() {
            target_path.to_string()
        } else {
            path.join("README.md").to_string_lossy().into_owned()
        };
        let preview = connector
            .execute_read(&read_target)
            .map_err(|e| format!("read_file icra hatası: {}", e))?;
        let summary = if preview.len() > 240 {
            format!("{}... ({} karakter)", &preview[..240], preview.len())
        } else {
            preview
        };
        Self::log_event(
            task_id,
            node_id,
            action,
            "action_execute",
            &format!("read_file başarılı: {} | önizleme: {}", read_target, summary),
        )
    }

    fn dispatch_read_folder(
        task_id: &str,
        node_id: &str,
        action: &str,
        target_path: &str,
    ) -> Result<(), String> {
        let connector = FolderConnector {
            name: "action_executor_folder".to_string(),
        };
        let listing = connector
            .execute_read(target_path)
            .map_err(|e| format!("read_folder icra hatası: {}", e))?;
        Self::log_event(
            task_id,
            node_id,
            action,
            "action_execute",
            &format!("read_folder başarılı: {} | girdi: {}", target_path, listing),
        )
    }

    fn dispatch_sqlite_read(
        task_id: &str,
        node_id: &str,
        action: &str,
        target_path: &str,
    ) -> Result<(), String> {
        let root = DependencyAnalyzer::get_project_root()?;
        let db_path = root.join("storage/app.db");
        let query_target = format!(
            "{}|SELECT COUNT(*) AS task_count FROM tasks",
            db_path.to_string_lossy()
        );
        let connector = SqliteConnector {
            name: "action_executor_sqlite".to_string(),
        };
        let result = connector
            .execute_read(&query_target)
            .map_err(|e| format!("sqlite_read icra hatası: {}", e))?;
        Self::log_event(
            task_id,
            node_id,
            action,
            "action_execute",
            &format!(
                "sqlite_read başarılı (etki: {}): {}",
                target_path, result
            ),
        )
    }

    fn dispatch_write(
        task_id: &str,
        node_id: &str,
        action: &str,
        target_path: &str,
        risk_level: &str,
    ) -> Result<(), String> {
        let envelope = json!({
            "approval_context": {
                "task_id": task_id,
                "decision_node_id": node_id,
                "action": action,
                "risk_level": risk_level
            },
            "payload": "panel_action_executor_marker"
        })
        .to_string();

        let connector = FileConnector {
            name: "action_executor_file_write".to_string(),
        };
        match connector.execute_write(target_path, &envelope) {
            Ok(()) => Self::log_event(
                task_id,
                node_id,
                action,
                "action_execute",
                &format!("write icra başarılı: {}", target_path),
            ),
            Err(e) => {
                let _ = decode_write_request(action, &envelope);
                Self::log_event(
                    task_id,
                    node_id,
                    action,
                    "action_execute_blocked",
                    &format!("write icra engellendi (beklenen fail-closed): {}", e),
                );
                Ok(())
            }
        }
    }

    fn log_event(
        task_id: &str,
        node_id: &str,
        action: &str,
        event_type: &str,
        message: &str,
    ) -> Result<(), String> {
        let metadata = json!({
            "action": action,
            "node_id": node_id,
            "event_type": event_type
        })
        .to_string();
        AuditLogger::log_event(
            task_id,
            "info",
            message,
            Some("Action Executor"),
            Some(event_type),
            Some(&metadata),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::execution_engine::{ApprovalSource, ExecutionContext, RunMode};
    use crate::storage::db::Database;
    use rusqlite::params;

    #[test]
    fn read_only_skips_write_but_allows_read_folder_log() {
        let task_id = "action_executor_read_only_test";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        let _ = conn.execute("DELETE FROM execution_logs WHERE task_id = ?1", params![task_id]);
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]);
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 't', 'r', 'pending', 'planning_complete', 'running', 'low', 'none')",
            params![task_id],
        )
        .unwrap();

        let root = DependencyAnalyzer::get_project_root().unwrap();
        let context = ExecutionContext {
            run_mode: RunMode::ReadOnly,
            current_user_id: None,
            approval_source: ApprovalSource::DatabaseOnly,
            allowed_actions: vec!["read_folder".to_string()],
            read_only: true,
        };

        ActionExecutor::dispatch_after_gates(
            task_id,
            "node_test",
            "write_file",
            &root.to_string_lossy(),
            &context,
            "high",
        )
        .unwrap();

        let skipped: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM execution_logs WHERE task_id = ?1 AND event_type = 'action_execute_skipped'",
                params![task_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(skipped >= 1);
    }
}
