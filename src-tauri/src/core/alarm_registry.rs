use crate::core::asker_motoru_bridge::AskerMotoruBridge;
use crate::core::dependency_analyzer::DependencyAnalyzer;
use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmCodeDefinition {
    pub code: String,
    pub title: String,
    pub severity: String,
    pub description: String,
    pub auto_speak: bool,
    pub speak_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmCodesConfig {
    pub schema_version: u32,
    pub locale: String,
    pub codes: Vec<AlarmCodeDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmEventRecord {
    pub id: String,
    pub alarm_code: Option<String>,
    pub source: String,
    pub message: String,
    pub severity: String,
    pub source_kind: String,
    pub persisted_at: String,
    pub resolved_at: Option<String>,
    pub auto_speak: bool,
    pub speak_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScanResult {
    pub triggered_codes: Vec<String>,
    pub events: Vec<AlarmEventRecord>,
}

pub struct AlarmRegistry;

impl AlarmRegistry {
    pub fn load_config() -> Result<AlarmCodesConfig, String> {
        let path = DependencyAnalyzer::get_config_path("alarm_codes.json")?;
        let data = fs::read_to_string(&path).map_err(|e| format!("alarm_codes.json okunamadı: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("alarm_codes.json geçersiz: {}", e))
    }

    pub fn list_codes() -> Result<Vec<AlarmCodeDefinition>, String> {
        Ok(Self::load_config()?.codes)
    }

    pub fn find_code(code: &str) -> Result<Option<AlarmCodeDefinition>, String> {
        let config = Self::load_config()?;
        Ok(config
            .codes
            .into_iter()
            .find(|item| item.code == code))
    }

    pub fn raise_code(
        code: &str,
        source: &str,
        message: &str,
        source_kind: &str,
    ) -> Result<AlarmEventRecord, String> {
        let definition = Self::find_code(code)?
            .ok_or_else(|| format!("Tanımsız alarm kodu: {}", code))?;
        let speak_text = if definition.auto_speak {
            Some(
                definition
                    .speak_template
                    .replace("{title}", &definition.title)
                    .replace("{message}", message),
            )
        } else {
            None
        };
        let id = Uuid::new_v4().to_string();
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO system_alarm_events (id, source, message, severity, source_kind, alarm_code)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                source,
                message,
                definition.severity,
                source_kind,
                code
            ],
        )
        .map_err(|e| e.to_string())?;
        let persisted_at: String = conn
            .query_row(
                "SELECT persisted_at FROM system_alarm_events WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(AlarmEventRecord {
            id,
            alarm_code: Some(code.to_string()),
            source: source.to_string(),
            message: message.to_string(),
            severity: definition.severity,
            source_kind: source_kind.to_string(),
            persisted_at,
            resolved_at: None,
            auto_speak: definition.auto_speak,
            speak_text,
        })
    }

    pub fn resolve_code(alarm_id: &str) -> Result<bool, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let updated = conn
            .execute(
                "UPDATE system_alarm_events SET resolved_at = CURRENT_TIMESTAMP WHERE id = ?1 AND resolved_at IS NULL",
                params![alarm_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(updated > 0)
    }

    pub fn list_active_events(limit: usize) -> Result<Vec<AlarmEventRecord>, String> {
        let config = Self::load_config()?;
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, alarm_code, source, message, severity, source_kind, persisted_at, resolved_at
                 FROM system_alarm_events
                 WHERE resolved_at IS NULL
                 ORDER BY persisted_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut list = Vec::new();
        for row in rows {
            let (
                id,
                alarm_code,
                source,
                message,
                severity,
                source_kind,
                persisted_at,
                resolved_at,
            ) = row.map_err(|e| e.to_string())?;
            let definition = alarm_code
                .as_deref()
                .and_then(|code| config.codes.iter().find(|item| item.code == code));
            let auto_speak = definition.map(|item| item.auto_speak).unwrap_or(false);
            let speak_text = definition.and_then(|item| {
                if item.auto_speak {
                    Some(
                        item.speak_template
                            .replace("{title}", &item.title)
                            .replace("{message}", &message),
                    )
                } else {
                    None
                }
            });
            list.push(AlarmEventRecord {
                id,
                alarm_code,
                source,
                message,
                severity,
                source_kind,
                persisted_at,
                resolved_at,
                auto_speak,
                speak_text,
            });
        }
        Ok(list)
    }

    pub fn scan_algorithm_health(task_id: Option<&str>) -> Result<HealthScanResult, String> {
        let mut triggered = Vec::new();
        let mut events = Vec::new();

        if let Some(task_id) = task_id {
            let allocations =
                crate::core::ai_workflow_manager::AiWorkflowManager::list_allocations(task_id)?;
            let missing = allocations
                .iter()
                .filter(|row| row.worker_status == "heartbeat_missing")
                .count();
            if !allocations.is_empty() && missing == allocations.len() {
                let message = format!(
                    "Görev {} için tüm platform heartbeat kayıpları tespit edildi.",
                    task_id
                );
                if let Ok(event) = Self::raise_code("011", "algorithm_monitor", &message, "swarm") {
                    triggered.push("011".to_string());
                    events.push(event);
                }
            } else if missing > 0 {
                let message = format!(
                    "Görev {} için {} platform heartbeat eksik.",
                    task_id, missing
                );
                if let Ok(event) = Self::raise_code("012", "swarm_monitor", &message, "swarm") {
                    triggered.push("012".to_string());
                    events.push(event);
                }
            }
        }

        if let Some((_, content)) = AskerMotoruBridge::read_alarm_status_file() {
            let lowered = content.to_ascii_lowercase();
            if !lowered.contains("healthy") && !lowered.contains("pass") {
                let message = "Asker Motoru alarm dosyası sağlıklı durumda değil.".to_string();
                if let Ok(event) =
                    Self::raise_code("013", "asker_motoru_bridge", &message, "json")
                {
                    triggered.push("013".to_string());
                    events.push(event);
                }
            }
        }

        Ok(HealthScanResult {
            triggered_codes: triggered,
            events,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::AlarmRegistry;

    #[test]
    fn alarm_codes_config_loads() {
        let codes = AlarmRegistry::list_codes().expect("alarm codes");
        assert!(codes.iter().any(|code| code.code == "011"));
    }
}
