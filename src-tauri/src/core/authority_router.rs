use std::fs;
use serde_json::Value;
use crate::core::decision_tree_builder::DecisionNode;

pub struct AuthorityRouter;

impl AuthorityRouter {
    pub fn route_and_validate(node: &DecisionNode) -> Result<(), String> {
        let matrix_path = crate::core::dependency_analyzer::DependencyAnalyzer::get_config_path("authority_matrix.json")?;
        let matrix_data = fs::read_to_string(&matrix_path)
            .map_err(|e| format!("Yetki matrisi (authority_matrix.json) okunamadı: {}", e))?;

        let matrix: Value = serde_json::from_str(&matrix_data)
            .map_err(|_| "Yetki matrisi JSON formatı geçersiz!".to_string())?;

        // Determine action mapping dynamically from level_mappings in authority_matrix.json
        let action = matrix
            .get("level_mappings")
            .and_then(|mappings| {
                let level_str = node.level.to_string();
                mappings
                    .get(&level_str)
                    .or_else(|| mappings.get("default"))
                    .and_then(|v| v.as_str())
            })
            .ok_or_else(|| {
                format!(
                    "HATA: Karar düğümü seviyesi için yetki aksiyonu eşleşmesi bulunamadı: {}",
                    node.level
                )
            })?;

        if let Some(deciders) = matrix.get(action) {
            if let Some(arr) = deciders.as_array() {
                let decider_id = &node.authorized_decider_id;
                let exists = arr.iter().any(|d| d.as_str() == Some(decider_id) || d.as_str() == Some("user"));
                
                if exists {
                    return Ok(());
                }
            }
        }

        Err(format!(
            "HATA: Karar düğümü (ID: {}) için yetkili karar noktası (Decider: {}) atanamadı!",
            node.id, node.authorized_decider_id
        ))
    }
}
