use crate::core::ai_workflow_manager::AiWorkflowManager;
use crate::core::alarm_registry::AlarmRegistry;
use crate::core::live_event_bus::LiveEventBus;
use crate::core::task_intake::{create_task, Task, TaskIntakeRequest};
use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSentenceRequest {
    pub sentence: String,
    pub operator_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSentenceResult {
    pub task: Task,
    pub platforms: Vec<String>,
    pub feed_id: String,
    pub alarm_scan: Vec<String>,
    pub burhan_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFeedItem {
    pub id: String,
    pub event_type: String,
    pub source: String,
    pub message: String,
    pub task_id: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: String,
}

pub struct CommandOrchestrator;

impl CommandOrchestrator {
    pub fn submit_sentence(
        app: &AppHandle,
        sentence: &str,
        operator_id: Option<&str>,
    ) -> Result<CommandSentenceResult, String> {
        let trimmed = sentence.trim();
        if trimmed.is_empty() {
            return Err("Komut cümlesi boş olamaz.".to_string());
        }

        let title = if trimmed.len() > 80 {
            format!("{}...", &trimmed[..77])
        } else {
            trimmed.to_string()
        };
        let user_request = format!(
            "[KomutMerkezi:{}] [Ajanlar:CODEX,OAM,BURHAN,EGITIM] {}",
            operator_id.unwrap_or("kurucu"),
            trimmed
        );

        let task = create_task(TaskIntakeRequest {
            title,
            user_request: user_request.clone(),
        })?;

        let platforms = AiWorkflowManager::parse_platforms_from_request(&user_request);
        let feed_id = Self::append_feed(
            "command-submitted",
            "command_orchestrator",
            trimmed,
            Some(&task.id),
            Some(serde_json::json!({ "platforms": platforms }).to_string()),
        )?;

        let burhan_message = format!(
            "Albay Burhan emri aldı. Görev {} platforma dağıtıldı.",
            platforms.len()
        );
        Self::append_feed(
            "burhan-dispatch",
            "burhan_command",
            &burhan_message,
            Some(&task.id),
            Some(serde_json::json!({ "platforms": platforms }).to_string()),
        )?;

        LiveEventBus::command_submitted(app, &task.id, trimmed, &platforms);
        LiveEventBus::burhan_dispatch(app, &task.id, &burhan_message, &platforms);

        for platform in &platforms {
            LiveEventBus::agent_status(app, &task.id, platform, "assigned");
        }

        let scan = AlarmRegistry::scan_algorithm_health(Some(&task.id))?;
        for event in &scan.events {
            LiveEventBus::alarm_code(
                app,
                event.alarm_code.as_deref().unwrap_or("000"),
                &event.message,
                event.speak_text.clone(),
            );
        }

        Ok(CommandSentenceResult {
            task,
            platforms,
            feed_id,
            alarm_scan: scan.triggered_codes,
            burhan_message,
        })
    }

    pub fn get_feed(limit: usize) -> Result<Vec<CommandFeedItem>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, event_type, source, message, task_id, metadata_json, created_at
                 FROM command_feed
                 ORDER BY created_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(CommandFeedItem {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    source: row.get(2)?,
                    message: row.get(3)?,
                    task_id: row.get(4)?,
                    metadata_json: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut list = Vec::new();
        for row in rows {
            list.push(row.map_err(|e| e.to_string())?);
        }
        Ok(list)
    }

    pub fn append_feed(
        event_type: &str,
        source: &str,
        message: &str,
        task_id: Option<&str>,
        metadata_json: Option<String>,
    ) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO command_feed (id, event_type, source, message, task_id, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, event_type, source, message, task_id, metadata_json],
        )
        .map_err(|e| e.to_string())?;
        Ok(id)
    }
}
