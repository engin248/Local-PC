use crate::core::audit_logger::AuditLogger;
use crate::core::dependency_analyzer::DependencyAnalyzer;
use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmAllocation {
    pub platform: String,
    pub payload_path: String,
    pub status: String,
}

pub struct AiWorkflowManager;

const DEFAULT_DEPARTMENT_PLATFORMS: &[&str] = &[
    "lokal_bilgisayar_kontrol_paneli",
    "asker_motoru_komuta_paneli",
    "planlama_departmani",
    "egitim_departmani",
    "ar_ge_departmani",
    "bot_agent_uretim_departmani",
    "beceri_kutuphanesi",
    "test_raporlama",
];

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
            platforms.extend(
                DEFAULT_DEPARTMENT_PLATFORMS
                    .iter()
                    .map(|platform| platform.to_string()),
            );
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
        let platforms =
            platforms.unwrap_or_else(|| Self::parse_platforms_from_request(user_request));
        let mut seen = HashSet::new();
        let mut allocations = Vec::new();

        let root = DependencyAnalyzer::get_project_root()?;
        let workflow_root = root.join("ai_workflow");
        fs::create_dir_all(workflow_root.join("tasks")).map_err(|e| e.to_string())?;

        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let risk = Self::normalize_risk(risk_level);
        conn.execute(
            "INSERT OR IGNORE INTO tasks (id, title, user_request, status, planning_status, execution_status, current_gate, last_valid_state_id, risk_level, approval_status)
             VALUES (?1, ?2, ?3, 'pending', 'planning_incomplete', 'not_started', NULL, NULL, ?4, 'pending_approval')",
            params![panel_task_id, title, user_request, risk],
        )
        .map_err(|e| e.to_string())?;
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
            fs::write(
                &inbox_file,
                serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?,
            )
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
                payload_path: payload_file,
                status: "waiting".to_string(),
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
        let mut stmt = conn
            .prepare(
                "SELECT platform_name, payload_file_path, status FROM ai_task_allocations WHERE task_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![panel_task_id], |row| {
                Ok(SwarmAllocation {
                    platform: row.get(0)?,
                    payload_path: row.get(1)?,
                    status: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut list = Vec::new();
        for row in rows {
            list.push(row.map_err(|e| e.to_string())?);
        }
        Ok(list)
    }

    fn normalize_platform_token(token: &str) -> Option<String> {
        let key = Self::normalize_token_key(token);
        Some(
            match key.as_str() {
                "codex" => "codex",
                "oam" | "open_agent_manager" => "open_agent_manager",
                "antigravity" | "antigrav" => "antigravity",
                "cursor" => "cursor",
                "perplexity" => "perplexity",
                "verdent" => "verdent",
                "lokal"
                | "lokal_panel"
                | "lokal_kontrol_paneli"
                | "lokal_bilgisayar_kontrol_paneli" => "lokal_bilgisayar_kontrol_paneli",
                "asker" | "asker_motoru" | "asker_motoru_komuta_paneli" => {
                    "asker_motoru_komuta_paneli"
                }
                "planlama" | "planlama_departmani" => "planlama_departmani",
                "egitim" | "egitim_departmani" => "egitim_departmani",
                "arge" | "ar_ge" | "ar_ge_departmani" => "ar_ge_departmani",
                "bot_agent" | "agent_uretim" | "bot_agent_uretim_departmani" => {
                    "bot_agent_uretim_departmani"
                }
                "beceri" | "beceri_kutuphanesi" | "skills" => "beceri_kutuphanesi",
                "test" | "raporlama" | "test_raporlama" | "test_raporlama_departmani" => {
                    "test_raporlama"
                }
                _ => return None,
            }
            .to_string(),
        )
    }

    fn normalize_token_key(token: &str) -> String {
        let mut normalized = String::new();
        let mut last_was_separator = false;
        for ch in token.trim().to_lowercase().chars() {
            let mapped = match ch {
                'ç' => Some('c'),
                'ğ' => Some('g'),
                'ı' | 'i' => Some('i'),
                'ö' => Some('o'),
                'ş' => Some('s'),
                'ü' => Some('u'),
                'a'..='z' | '0'..='9' => Some(ch),
                _ => None,
            };
            if let Some(mapped) = mapped {
                normalized.push(mapped);
                last_was_separator = false;
            } else if !last_was_separator && !normalized.is_empty() {
                normalized.push('_');
                last_was_separator = true;
            }
        }
        normalized.trim_matches('_').to_string()
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
        let platforms =
            AiWorkflowManager::parse_platforms_from_request("[Kod] [Ajanlar: CODEX,CURSOR] test");
        assert_eq!(platforms, vec!["codex", "cursor"]);
    }

    #[test]
    fn parses_department_agent_tags_from_request() {
        let platforms = AiWorkflowManager::parse_platforms_from_request(
            "[Operasyon] [Ajanlar: Planlama Departmanı, AR-GE Departmanı, Test/Raporlama] test",
        );
        assert_eq!(
            platforms,
            vec!["planlama_departmani", "ar_ge_departmani", "test_raporlama"]
        );
    }

    #[test]
    fn defaults_to_one_agent_per_department() {
        let platforms = AiWorkflowManager::parse_platforms_from_request("Standart operasyon");
        assert_eq!(
            platforms,
            vec![
                "lokal_bilgisayar_kontrol_paneli",
                "asker_motoru_komuta_paneli",
                "planlama_departmani",
                "egitim_departmani",
                "ar_ge_departmani",
                "bot_agent_uretim_departmani",
                "beceri_kutuphanesi",
                "test_raporlama",
            ]
        );
    }
}
