use serde::{Deserialize, Serialize};
use rusqlite::params;
use crate::storage::db::Database;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlanningStandardInput {
    pub task_definition: String,
    pub purpose: String,
    pub scope: String,
    pub topic: String,
    pub sub_topic: String,
    pub criterion: String,
    pub sub_criterion: String,
    pub alternatives: Vec<String>,
    pub risk_analysis: String,
    pub impact_area: String,
    pub technology_selection: String,
    pub dependency_analysis: String,
    pub checkpoints: Vec<String>,
    pub test_criteria: Vec<String>,
    pub rollback_plan: String,
    pub operation_plan: String,
    pub authorized_deciders: Vec<String>,
}

pub fn save_plan(task_id: &str, plan: PlanningStandardInput) -> Result<(), String> {
    let db = Database::new();
    let conn = db.get_connection().map_err(|e| e.to_string())?;

    // Strict 17-field completeness verification
    let is_complete = !plan.task_definition.trim().is_empty()
        && !plan.purpose.trim().is_empty()
        && !plan.scope.trim().is_empty()
        && !plan.topic.trim().is_empty()
        && !plan.sub_topic.trim().is_empty()
        && !plan.criterion.trim().is_empty()
        && !plan.sub_criterion.trim().is_empty()
        && !plan.alternatives.is_empty()
        && !plan.alternatives.iter().any(|a| a.trim().is_empty())
        && !plan.risk_analysis.trim().is_empty()
        && !plan.impact_area.trim().is_empty()
        && !plan.technology_selection.trim().is_empty()
        && !plan.dependency_analysis.trim().is_empty()
        && !plan.checkpoints.is_empty()
        && !plan.checkpoints.iter().any(|c| c.trim().is_empty())
        && !plan.test_criteria.is_empty()
        && !plan.test_criteria.iter().any(|t| t.trim().is_empty())
        && !plan.rollback_plan.trim().is_empty()
        && !plan.operation_plan.trim().is_empty()
        && !plan.authorized_deciders.is_empty()
        && !plan.authorized_deciders.iter().any(|d| d.trim().is_empty());

    let status = if is_complete {
        "planning_complete"
    } else {
        "planning_incomplete"
    };

    // Save to physical backup JSON file
    let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
    let path_buf = root.join("storage").join("backups").join(format!("plan_{}.json", task_id));
    let path = path_buf.to_string_lossy().into_owned();
    let plan_json = serde_json::to_string_pretty(&plan).map_err(|e| e.to_string())?;
    std::fs::write(&path, plan_json).map_err(|e| format!("Fiziksel plan yedeği kaydedilemedi: {}", e))?;

    // Update planning status in tasks
    conn.execute(
        "UPDATE tasks SET planning_status = ?1, risk_level = ?2 WHERE id = ?3",
        params![status, plan.risk_analysis, task_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

pub struct PlanningGate;

impl PlanningGate {
    pub fn validate_planning(task_id: &str) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        let planning_status: String = conn.query_row(
            "SELECT planning_status FROM tasks WHERE id = ?1",
            params![task_id],
            |row| row.get(0),
        ).map_err(|_| "Görev bulunamadı!".to_string())?;

        if planning_status != "planning_complete" {
            return Err("Planlama standardındaki 17 zorunlu alan doldurulmadan işlem başlatılamaz! Durum: planning_incomplete".to_string());
        }

        // Verify the physical plan JSON file exists and has non-empty rollback plan
        let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
        let path_buf = root.join("storage").join("backups").join(format!("plan_{}.json", task_id));
        let path = path_buf.to_string_lossy().into_owned();
        if !std::path::Path::new(&path).exists() {
            return Err("HATA: Fiziksel plan yedek dosyası bulunamadı! Planlama doğrulanmamış.".to_string());
        }

        let plan_data = std::fs::read_to_string(&path)
            .map_err(|_| "HATA: Plan yedek dosyası okunamadı!".to_string())?;
        let plan: PlanningStandardInput = serde_json::from_str(&plan_data)
            .map_err(|_| "HATA: Plan yedek dosyası geçersiz JSON formatında!".to_string())?;

        if plan.rollback_plan.trim().is_empty() {
            return Err("HATA: Geri alma planı (Rollback Plan) olmadan operasyon yürütülemez!".to_string());
        }

        Ok(())
    }

    pub fn load_plan(task_id: &str) -> Result<PlanningStandardInput, String> {
        let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
        let path_buf = root.join("storage").join("backups").join(format!("plan_{}.json", task_id));
        let path = path_buf.to_string_lossy().into_owned();
        if !std::path::Path::new(&path).exists() {
            return Err("HATA: Plan yedek dosyası bulunamadı!".to_string());
        }
        let plan_data = std::fs::read_to_string(&path)
            .map_err(|e| format!("HATA: Plan yedek dosyası okunamadı: {}", e))?;
        let plan: PlanningStandardInput = serde_json::from_str(&plan_data)
            .map_err(|e| format!("HATA: Plan yedek dosyası geçersiz JSON formatında: {}", e))?;
        Ok(plan)
    }
}

