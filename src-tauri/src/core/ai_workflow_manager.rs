use crate::core::audit_logger::AuditLogger;
use crate::core::dependency_analyzer::DependencyAnalyzer;
use crate::storage::db::Database;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmAllocation {
    pub platform: String,
    pub payload_path: String,
    pub status: String,
    pub raw_status: String,
    pub source_kind: String,
    pub task_status: Option<String>,
    pub inbox_path: String,
    pub inbox_exists: bool,
    pub outbox_path: String,
    pub outbox_exists: bool,
    pub worker_status: String,
    pub worker_heartbeat_path: Option<String>,
    pub report_returned: bool,
    pub last_report_at: Option<String>,
}

pub struct AiWorkflowManager;

impl AiWorkflowManager {
    pub fn parse_platforms_from_request(user_request: &str) -> Vec<String> {
        let mut platforms = Vec::new();
        if let Some(start) = user_request.find("[Ajanlar:") {
            if let Some(rel_end) = user_request[start..].find(']') {
                let block = &user_request[start..start + rel_end];
                let inner = block
                    .split_once(':')
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or("");
                for token in inner.split(',') {
                    let normalized = Self::normalize_platform_token(token.trim());
                    if let Some(p) = normalized {
                        platforms.push(p);
                    }
                }
            }
        }
        if platforms.is_empty() {
            platforms.push("codex".to_string());
            platforms.push("open_agent_manager".to_string());
        }
        platforms
    }

    pub fn allocate_task(
        panel_task_id: &str,
        title: &str,
        user_request: &str,
        risk_level: &str,
        platforms: Option<Vec<String>>,
    ) -> Result<Vec<SwarmAllocation>, String> {
        let platforms = platforms.unwrap_or_else(|| Self::parse_platforms_from_request(user_request));
        let mut seen = HashSet::new();
        let mut allocations = Vec::new();

        let root = DependencyAnalyzer::get_project_root()?;
        let workflow_root = root.join("ai_workflow");
        fs::create_dir_all(workflow_root.join("tasks")).map_err(|e| e.to_string())?;

        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let risk = Self::normalize_risk(risk_level);
        conn.execute(
            "INSERT OR REPLACE INTO ai_tasks (id, title, description, risk_level, status, created_by)
             VALUES (?1, ?2, ?3, ?4, 'pending', 'panel_intake')",
            params![panel_task_id, title, user_request, risk],
        )
        .map_err(|e| e.to_string())?;

        let task_payload_path = workflow_root
            .join("tasks")
            .join(format!("{}.json", panel_task_id));
        let payload = json!({
            "panel_task_id": panel_task_id,
            "title": title,
            "user_request": user_request,
            "risk_level": risk,
            "platforms": platforms
        });
        fs::write(
            &task_payload_path,
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

        for platform in platforms {
            if !seen.insert(platform.clone()) {
                continue;
            }
            let inbox = workflow_root
                .join("platforms")
                .join(&platform)
                .join("inbox");
            fs::create_dir_all(&inbox).map_err(|e| e.to_string())?;
            let inbox_file = inbox.join(format!("{}.json", panel_task_id));
            fs::write(&inbox_file, serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;

            let alloc_id = Uuid::new_v4().to_string();
            let payload_file = inbox_file.to_string_lossy().into_owned();
            conn.execute(
                "INSERT OR REPLACE INTO ai_task_allocations (id, task_id, platform_name, status, payload_file_path)
                 VALUES (?1, ?2, ?3, 'waiting', ?4)",
                params![alloc_id, panel_task_id, platform, payload_file],
            )
            .map_err(|e| e.to_string())?;

            allocations.push(SwarmAllocation {
                platform,
                payload_path: payload_file.clone(),
                status: "waiting".to_string(),
                raw_status: "waiting".to_string(),
                source_kind: "sqlite".to_string(),
                task_status: Some("pending".to_string()),
                inbox_path: payload_file.clone(),
                inbox_exists: true,
                outbox_path: String::new(),
                outbox_exists: false,
                worker_status: "heartbeat_missing".to_string(),
                worker_heartbeat_path: None,
                report_returned: false,
                last_report_at: None,
            });
        }

        AuditLogger::log_event(
            panel_task_id,
            "info",
            &format!("Swarm tahsis: {} platform", allocations.len()),
            Some("Intake Gate"),
            Some("swarm_allocate"),
            Some(&serde_json::to_string(&allocations).unwrap_or_default()),
        )?;

        Ok(allocations)
    }

    pub fn list_allocations(panel_task_id: &str) -> Result<Vec<SwarmAllocation>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let workflow_root = DependencyAnalyzer::get_project_root()?.join("ai_workflow");
        let task_status = conn
            .query_row(
                "SELECT status FROM ai_tasks WHERE id = ?1",
                params![panel_task_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT platform_name, payload_file_path, status FROM ai_task_allocations WHERE task_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![panel_task_id], |row| {
                let platform: String = row.get(0)?;
                let payload_path: String = row.get(1)?;
                let raw_status: String = row.get(2)?;
                Ok((platform, payload_path, raw_status))
            })
            .map_err(|e| e.to_string())?;
        let mut list = Vec::new();
        for row in rows {
            let (platform, payload_path, raw_status) = row.map_err(|e| e.to_string())?;
            let report = Self::report_status(&conn, panel_task_id, &platform)?;
            let inbox_path = PathBuf::from(&payload_path);
            let outbox_path = workflow_root
                .join("platforms")
                .join(&platform)
                .join("outbox")
                .join(format!("{}.json", panel_task_id));
            let heartbeat_path = workflow_root
                .join("platforms")
                .join(&platform)
                .join("heartbeat.json");
            let inbox_exists = inbox_path.exists();
            let outbox_exists = outbox_path.exists();
            let worker_status = if heartbeat_path.exists() {
                "heartbeat_present".to_string()
            } else {
                "heartbeat_missing".to_string()
            };
            let report_returned = report.is_some() || outbox_exists;
            let status = Self::normalize_allocation_status(
                &raw_status,
                task_status.as_deref(),
                report_returned,
            );
            list.push(SwarmAllocation {
                platform,
                payload_path,
                status,
                raw_status,
                source_kind: "sqlite".to_string(),
                task_status: task_status.clone(),
                inbox_path: inbox_path.display().to_string(),
                inbox_exists,
                outbox_path: outbox_path.display().to_string(),
                outbox_exists,
                worker_status,
                worker_heartbeat_path: heartbeat_path
                    .exists()
                    .then(|| heartbeat_path.display().to_string()),
                report_returned,
                last_report_at: report,
            });
        }
        Ok(list)
    }

    fn report_status(
        conn: &rusqlite::Connection,
        panel_task_id: &str,
        platform: &str,
    ) -> Result<Option<String>, String> {
        conn.query_row(
            "SELECT submitted_at FROM ai_collected_reports WHERE task_id = ?1 AND platform_name = ?2 ORDER BY submitted_at DESC LIMIT 1",
            params![panel_task_id, platform],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())
    }

    fn normalize_allocation_status(
        raw_status: &str,
        task_status: Option<&str>,
        report_returned: bool,
    ) -> String {
        if report_returned {
            return "report_returned".to_string();
        }
        match raw_status {
            "waiting" => "assigned".to_string(),
            "processing" => "running".to_string(),
            "submitted" => "completed".to_string(),
            "failed" | "rejected" => "failed".to_string(),
            other if matches!(task_status, Some("completed")) && other != "failed" => {
                "completed".to_string()
            }
            other => other.to_string(),
        }
    }

    fn normalize_platform_token(token: &str) -> Option<String> {
        let key = token.to_ascii_lowercase();
        Some(
            match key.as_str() {
                "codex" => "codex",
                "oam" | "open_agent_manager" => "open_agent_manager",
                "antigravity" | "antigrav" => "antigravity",
                "cursor" => "cursor",
                "perplexity" => "perplexity",
                "verdent" => "verdent",
                "burhan" | "burhan_command" | "albay" | "albay_burhan" => "burhan_command",
                "egitim" | "education" | "education_office" => "education_office",
                _ => return None,
            }
            .to_string(),
        )
    }

    fn normalize_risk(risk: &str) -> String {
        match risk.to_ascii_lowercase().as_str() {
            "critical" => "critical".to_string(),
            "high" => "high".to_string(),
            "medium" => "medium".to_string(),
            _ => "low".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_agent_tags_from_request() {
        let platforms = AiWorkflowManager::parse_platforms_from_request(
            "[Kod] [Ajanlar: CODEX,CURSOR] test",
        );
        assert_eq!(platforms, vec!["codex", "cursor"]);
    }

    #[test]
    fn normalizes_swarm_statuses_for_panel_cards() {
        assert_eq!(
            AiWorkflowManager::normalize_allocation_status("waiting", Some("pending"), false),
            "assigned"
        );
        assert_eq!(
            AiWorkflowManager::normalize_allocation_status("processing", Some("in_progress"), false),
            "running"
        );
        assert_eq!(
            AiWorkflowManager::normalize_allocation_status("submitted", Some("completed"), false),
            "completed"
        );
        assert_eq!(
            AiWorkflowManager::normalize_allocation_status("waiting", Some("pending"), true),
            "report_returned"
        );
        assert_eq!(
            AiWorkflowManager::normalize_allocation_status("failed", Some("pending"), false),
            "failed"
        );
    }
}
