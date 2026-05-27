use serde::{Serialize, Deserialize};
use rusqlite::params;
use uuid::Uuid;
use crate::storage::db::Database;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RiskAssessment {
    pub id: String,
    pub task_id: String,
    pub decision_node_id: String,
    pub risk_level: String,
    pub risk_reason: String,
    pub affected_assets_json: String,
    pub mitigation_plan: String,
}

pub struct RiskEngine;

impl RiskEngine {
    pub fn assess_risk(task_id: &str, node_id: &str, action_type: &str) -> Result<RiskAssessment, String> {
        let rules_path = crate::core::dependency_analyzer::DependencyAnalyzer::get_config_path("risk_rules.json")?;
        let rules_data = std::fs::read_to_string(&rules_path)
            .map_err(|e| format!("Risk kuralları (risk_rules.json) okunamadı: {}", e))?;
        let rules: serde_json::Value = serde_json::from_str(&rules_data)
            .map_err(|e| format!("Risk kuralları JSON formatı geçersiz: {}", e))?;

        // Dynamically resolve risk properties from action_mappings in risk_rules.json
        let (risk_level, reason, assets, mitigation) = if let Some(mappings) = rules.get("action_mappings") {
            let config = mappings.get(action_type)
                .ok_or_else(|| format!("Risk kurallarında eylem türü için açık yapılandırma bulunamadı: {}", action_type))?;
            
            let level = config.get("level")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("Risk kuralında level eksik: {}", action_type))?
                .to_string();
            let reason = config
                .get("reason")
                .and_then(|v| v.as_str())
                .filter(|v| !v.trim().is_empty())
                .ok_or_else(|| format!("Risk kuralında reason eksik veya boş: {}", action_type))?
                .to_string();
            
            let arr = config
                .get("assets")
                .and_then(|v| v.as_array())
                .ok_or_else(|| format!("Risk kuralında assets listesi eksik: {}", action_type))?;
            let assets = serde_json::to_string(arr)
                .map_err(|e| format!("Risk kuralı assets alanı JSON'a çevrilemedi: {}", e))?;

            let mitigation = config
                .get("mitigation")
                .and_then(|v| v.as_str())
                .filter(|v| !v.trim().is_empty())
                .ok_or_else(|| format!("Risk kuralında mitigation eksik veya boş: {}", action_type))?
                .to_string();
            
            (level, reason, assets, mitigation)
        } else {
            return Err("Risk kurallarında action_mappings bulunamadı.".to_string());
        };

        // Enforce rules from risk_rules.json dynamically
        if let Some(levels) = rules.get("levels") {
            if let Some(rules_config) = levels.get(&risk_level) {
                if let Some(double_check) = rules_config.get("requires_double_check").and_then(|v| v.as_bool()) {
                    if double_check && risk_level == "critical" {
                        crate::core::audit_logger::AuditLogger::log_event(
                            task_id,
                            "warning",
                            "KRİTİK RİSK: Çift onay ve ek güvenlik bariyerleri backend tarafından zorlanmaktadır.",
                            Some("Risk Engine"),
                            Some("double_check_enforced"),
                            None
                        )?;
                    }
                }
            }
        }

        let assessment = RiskAssessment {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            decision_node_id: node_id.to_string(),
            risk_level: risk_level.to_string(),
            risk_reason: reason.to_string(),
            affected_assets_json: assets.to_string(),
            mitigation_plan: mitigation.to_string(),
        };

        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO risk_assessments (id, task_id, decision_node_id, risk_level, risk_reason, affected_assets_json, mitigation_plan)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                assessment.id,
                assessment.task_id,
                assessment.decision_node_id,
                assessment.risk_level,
                assessment.risk_reason,
                assessment.affected_assets_json,
                assessment.mitigation_plan
            ],
        ).map_err(|e| e.to_string())?;

        // Update task risk level
        conn.execute(
            "UPDATE tasks SET risk_level = ?1 WHERE id = ?2",
            params![risk_level, task_id],
        ).map_err(|e| e.to_string())?;

        Ok(assessment)
    }

    pub fn get_assessment(node_id: &str) -> Result<Option<RiskAssessment>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, task_id, decision_node_id, risk_level, risk_reason, affected_assets_json, mitigation_plan FROM risk_assessments WHERE decision_node_id = ?1")
            .map_err(|e| e.to_string())?;

        let row = stmt.query_row(params![node_id], |row| {
            Ok(RiskAssessment {
                id: row.get(0)?,
                task_id: row.get(1)?,
                decision_node_id: row.get(2)?,
                risk_level: row.get(3)?,
                risk_reason: row.get(4)?,
                affected_assets_json: row.get(5)?,
                mitigation_plan: row.get(6)?,
            })
        });

        match row {
            Ok(assess) => Ok(Some(assess)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
}
