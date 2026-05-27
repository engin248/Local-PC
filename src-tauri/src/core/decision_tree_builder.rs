use serde::{Serialize, Deserialize};
use rusqlite::params;
use uuid::Uuid;
use crate::storage::db::Database;
use crate::core::task_decomposer::TaskBreakdown;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionNode {
    pub id: String,
    pub task_id: String,
    pub breakdown_id: String,
    pub level: i32,
    pub parent_node_id: Option<String>,
    pub required_approval: i32,
    pub gate_status: String,
    pub authorized_decider_type: String,
    pub authorized_decider_id: String,
    pub status: String,
    pub selected_option: Option<String>,
    pub reason: Option<String>,
    pub evidence_json: Option<String>,
    pub confidence: Option<f64>,
}

pub struct DecisionTreeBuilder;

impl DecisionTreeBuilder {
    pub fn build_tree(task_id: &str, breakdowns: &Vec<TaskBreakdown>) -> Result<Vec<DecisionNode>, String> {
        let mut nodes = Vec::new();
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        let matrix_path = crate::core::dependency_analyzer::DependencyAnalyzer::get_config_path("authority_matrix.json")?;
        let matrix_data = std::fs::read_to_string(&matrix_path)
            .map_err(|e| format!("Yetki matrisi (authority_matrix.json) okunamadı: {}", e))?;
        let matrix: serde_json::Value = serde_json::from_str(&matrix_data)
            .map_err(|_| "Yetki matrisi JSON formatı geçersiz!".to_string())?;

        let mut parent_id: Option<String> = None;

        for breakdown in breakdowns {
            // Determine action mapping dynamically from level_mappings in authority_matrix.json
            let action = matrix
                .get("level_mappings")
                .and_then(|mappings| {
                    let level_str = breakdown.level.to_string();
                    mappings
                        .get(&level_str)
                        .or_else(|| mappings.get("default"))
                        .and_then(|v| v.as_str())
                })
                .ok_or_else(|| {
                    format!(
                        "HATA: Görev kırılımı seviyesi için yetki aksiyonu eşleşmesi bulunamadı: {}",
                        breakdown.level
                    )
                })?;

            let mut decider_id = "user".to_string();
            let mut decider_type = "permission_manager".to_string();

            if let Some(deciders) = matrix.get(action) {
                if let Some(arr) = deciders.as_array() {
                    if !arr.is_empty() {
                        let chosen = &arr[0];
                        if let Some(s) = chosen.as_str() {
                            decider_id = s.to_string();
                            decider_type = if s == "chatgpt" || s == "gemini" {
                                "ai_provider".to_string()
                            } else {
                                "permission_manager".to_string()
                            };
                        }
                    }
                }
            }
            
            let node = DecisionNode {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                breakdown_id: breakdown.id.clone(),
                level: breakdown.level,
                parent_node_id: parent_id.clone(),
                required_approval: if breakdown.level >= 4 { 1 } else { 0 },
                gate_status: "pending".to_string(),
                authorized_decider_type: decider_type.to_string(),
                authorized_decider_id: decider_id.to_string(),
                status: "pending".to_string(),
                selected_option: None,
                reason: None,
                evidence_json: None,
                confidence: Some(0.95),
            };

            conn.execute(
                "INSERT INTO decision_nodes (id, task_id, breakdown_id, level, parent_node_id, required_approval, gate_status, authorized_decider_type, authorized_decider_id, status, selected_option, reason, evidence_json, confidence)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    node.id,
                    node.task_id,
                    node.breakdown_id,
                    node.level,
                    node.parent_node_id,
                    node.required_approval,
                    node.gate_status,
                    node.authorized_decider_type,
                    node.authorized_decider_id,
                    node.status,
                    node.selected_option,
                    node.reason,
                    node.evidence_json,
                    node.confidence
                ],
            ).map_err(|e| e.to_string())?;

            parent_id = Some(node.id.clone());
            nodes.push(node);
        }

        Ok(nodes)
    }

    pub fn get_nodes(task_id: &str) -> Result<Vec<DecisionNode>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, task_id, breakdown_id, level, parent_node_id, required_approval, gate_status, authorized_decider_type, authorized_decider_id, status, selected_option, reason, evidence_json, confidence FROM decision_nodes WHERE task_id = ?1")
            .map_err(|e| e.to_string())?;

        let rows = stmt.query_map(params![task_id], |row| {
            Ok(DecisionNode {
                id: row.get(0)?,
                task_id: row.get(1)?,
                breakdown_id: row.get(2)?,
                level: row.get(3)?,
                parent_node_id: row.get(4)?,
                required_approval: row.get(5)?,
                gate_status: row.get(6)?,
                authorized_decider_type: row.get(7)?,
                authorized_decider_id: row.get(8)?,
                status: row.get(9)?,
                selected_option: row.get(10)?,
                reason: row.get(11)?,
                evidence_json: row.get(12)?,
                confidence: row.get(13)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(row.map_err(|e| e.to_string())?);
        }

        Ok(nodes)
    }
}
