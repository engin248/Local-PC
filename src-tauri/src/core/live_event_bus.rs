use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEventPayload {
    pub event_type: String,
    pub source: String,
    pub message: String,
    pub task_id: Option<String>,
    pub metadata_json: Option<String>,
    pub timestamp: String,
}

pub struct LiveEventBus;

impl LiveEventBus {
    pub fn emit(app: &AppHandle, payload: LiveEventPayload) {
        if let Err(err) = app.emit(payload.event_type.as_str(), payload.clone()) {
            eprintln!(
                "Canlı olay yayınlanamadı [type={}]: {err}",
                payload.event_type
            );
        }
        if payload.event_type != "critical-error" {
            let _ = app.emit("live-feed", payload);
        }
    }

    pub fn command_submitted(
        app: &AppHandle,
        task_id: &str,
        sentence: &str,
        platforms: &[String],
    ) {
        Self::emit(
            app,
            LiveEventPayload {
                event_type: "command-submitted".to_string(),
                source: "command_orchestrator".to_string(),
                message: sentence.to_string(),
                task_id: Some(task_id.to_string()),
                metadata_json: Some(serde_json::json!({ "platforms": platforms }).to_string()),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        );
    }

    pub fn burhan_dispatch(app: &AppHandle, task_id: &str, message: &str, platforms: &[String]) {
        Self::emit(
            app,
            LiveEventPayload {
                event_type: "burhan-dispatch".to_string(),
                source: "burhan_command".to_string(),
                message: message.to_string(),
                task_id: Some(task_id.to_string()),
                metadata_json: Some(serde_json::json!({ "platforms": platforms }).to_string()),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        );
    }

    pub fn agent_status(app: &AppHandle, task_id: &str, platform: &str, status: &str) {
        Self::emit(
            app,
            LiveEventPayload {
                event_type: "agent-status".to_string(),
                source: platform.to_string(),
                message: status.to_string(),
                task_id: Some(task_id.to_string()),
                metadata_json: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        );
    }

    pub fn report_returned(app: &AppHandle, task_id: &str, platform: &str, summary: &str) {
        Self::emit(
            app,
            LiveEventPayload {
                event_type: "report-returned".to_string(),
                source: platform.to_string(),
                message: summary.to_string(),
                task_id: Some(task_id.to_string()),
                metadata_json: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        );
    }

    pub fn alarm_code(app: &AppHandle, code: &str, message: &str, speak_text: Option<String>) {
        Self::emit(
            app,
            LiveEventPayload {
                event_type: "alarm-code".to_string(),
                source: format!("alarm_{code}"),
                message: message.to_string(),
                task_id: None,
                metadata_json: speak_text.map(|text| {
                    serde_json::json!({ "code": code, "speak_text": text }).to_string()
                }),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        );
    }
}
