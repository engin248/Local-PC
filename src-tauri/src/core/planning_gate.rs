use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    #[serde(default)]
    pub accepted_correct_approach_reason: String,
    #[serde(default)]
    pub selected_best_option_reason: String,
}

pub fn save_plan(task_id: &str, plan: PlanningStandardInput) -> Result<(), String> {
    let db = Database::new();
    let conn = db.get_connection().map_err(|e| e.to_string())?;

    // Strict planning and principle completeness verification
    let base_complete = !plan.task_definition.trim().is_empty()
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
    let principle_complete = PlanningGate::validate_principle_reasons(&plan).is_ok();
    let risk_complete = PlanningGate::validate_risk_requirements(&plan).is_ok();
    let is_complete = base_complete && principle_complete && risk_complete;

    let status = if is_complete {
        "planning_complete"
    } else {
        "planning_incomplete"
    };

    // Save to physical backup JSON file
    let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
    let backup_dir = root.join("storage").join("backups");
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Fiziksel plan yedek dizini oluşturulamadı: {}", e))?;
    let path_buf = backup_dir.join(format!("plan_{}.json", task_id));
    let path = path_buf.to_string_lossy().into_owned();
    let plan_json = serde_json::to_string_pretty(&plan).map_err(|e| e.to_string())?;
    std::fs::write(&path, plan_json)
        .map_err(|e| format!("Fiziksel plan yedeği kaydedilemedi: {}", e))?;

    // Update planning status in tasks
    conn.execute(
        "UPDATE tasks SET planning_status = ?1, risk_level = ?2 WHERE id = ?3",
        params![status, plan.risk_analysis, task_id],
    )
    .map_err(|e| e.to_string())?;

    if is_complete {
        PlanningGate::save_principle_evaluation(task_id, None, &plan)?;
    }

    Ok(())
}

pub struct PlanningGate;

impl PlanningGate {
    pub fn validate_planning(task_id: &str) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        let planning_status: String = conn
            .query_row(
                "SELECT planning_status FROM tasks WHERE id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|_| "Görev bulunamadı!".to_string())?;

        if planning_status != "planning_complete" {
            return Err("Planlama standardındaki 17 zorunlu alan doldurulmadan işlem başlatılamaz! Durum: planning_incomplete".to_string());
        }

        // Verify the physical plan JSON file exists and has non-empty rollback plan
        let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
        let path_buf = root
            .join("storage")
            .join("backups")
            .join(format!("plan_{}.json", task_id));
        let path = path_buf.to_string_lossy().into_owned();
        if !std::path::Path::new(&path).exists() {
            return Err(
                "HATA: Fiziksel plan yedek dosyası bulunamadı! Planlama doğrulanmamış.".to_string(),
            );
        }

        let plan_data = std::fs::read_to_string(&path)
            .map_err(|_| "HATA: Plan yedek dosyası okunamadı!".to_string())?;
        let plan: PlanningStandardInput = serde_json::from_str(&plan_data)
            .map_err(|_| "HATA: Plan yedek dosyası geçersiz JSON formatında!".to_string())?;

        if plan.rollback_plan.trim().is_empty() {
            return Err(
                "HATA: Geri alma planı (Rollback Plan) olmadan operasyon yürütülemez!".to_string(),
            );
        }
        Self::validate_principle_reasons(&plan)?;
        Self::validate_risk_requirements(&plan)?;

        Ok(())
    }

    pub fn load_plan(task_id: &str) -> Result<PlanningStandardInput, String> {
        let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
        let path_buf = root
            .join("storage")
            .join("backups")
            .join(format!("plan_{}.json", task_id));
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

    fn validate_principle_reasons(plan: &PlanningStandardInput) -> Result<(), String> {
        if plan.accepted_correct_approach_reason.trim().len() < 20 {
            return Err("HATA: accepted_correct_approach_reason eksik veya yetersiz. Genel dogru yaklasim gerekcesi zorunludur.".to_string());
        }
        if plan.selected_best_option_reason.trim().len() < 20 {
            return Err("HATA: selected_best_option_reason eksik veya yetersiz. En iyi uygulanabilir secenek gerekcesi zorunludur.".to_string());
        }
        Ok(())
    }

    fn validate_risk_requirements(plan: &PlanningStandardInput) -> Result<(), String> {
        let risk = plan.risk_analysis.to_lowercase();
        if matches!(risk.as_str(), "high" | "critical") {
            if plan.alternatives.len() < 3 {
                return Err(
                    "HATA: High/Critical risk icin en az 3 alternatif zorunludur.".to_string(),
                );
            }
            if !plan.alternatives.iter().any(|a| {
                a.to_lowercase().contains("manuel") || a.to_lowercase().contains("uygulama yapma")
            }) {
                return Err("HATA: High/Critical risk icin manuel plan veya uygulama yapmama alternatifi zorunludur.".to_string());
            }
            if plan.rollback_plan.trim().len() < 20 {
                return Err(
                    "HATA: High/Critical risk icin anlamli rollback plani zorunludur.".to_string(),
                );
            }
            if plan.test_criteria.is_empty() {
                return Err(
                    "HATA: High/Critical risk icin Test Gate kriterleri zorunludur.".to_string(),
                );
            }
            if plan.authorized_deciders.is_empty() {
                return Err(
                    "HATA: High/Critical risk icin yetkili karar noktasi zorunludur.".to_string(),
                );
            }
        }
        Ok(())
    }

    fn save_principle_evaluation(
        task_id: &str,
        decision_node_id: Option<&str>,
        plan: &PlanningStandardInput,
    ) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO principle_evaluations
             (id, task_id, decision_node_id, accepted_correct_approach_reason, selected_best_option_reason, status)
             VALUES (?1, ?2, ?3, ?4, ?5, 'passed')",
            params![
                Uuid::new_v4().to_string(),
                task_id,
                decision_node_id,
                plan.accepted_correct_approach_reason,
                plan.selected_best_option_reason
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::Database;

    fn valid_plan() -> PlanningStandardInput {
        PlanningStandardInput {
            task_definition: "Gercek gorev tanimi".to_string(),
            purpose: "Amac".to_string(),
            scope: "Kapsam".to_string(),
            topic: "Konu".to_string(),
            sub_topic: "Alt konu".to_string(),
            criterion: "Kriter".to_string(),
            sub_criterion: "Alt kriter".to_string(),
            alternatives: vec![
                "Yalnizca oku ve raporla".to_string(),
                "Uygulama yapma, manuel plan uret".to_string(),
                "Onayli ve rollback destekli uygula".to_string(),
            ],
            risk_analysis: "high".to_string(),
            impact_area: "storage/app.db".to_string(),
            technology_selection: "Rust SQLite Svelte".to_string(),
            dependency_analysis: "Lokal bagimlilik".to_string(),
            checkpoints: vec!["Plan kontrol".to_string()],
            test_criteria: vec!["file_exists:storage/app.db".to_string()],
            rollback_plan: "Degisiklik oncesi gercek snapshot alinir ve hata halinde geri yuklenir."
                .to_string(),
            operation_plan: "action:code_analysis, action:snapshot_create, action:test_run, action:report_generate"
                .to_string(),
            authorized_deciders: vec!["user".to_string()],
            accepted_correct_approach_reason:
                "Genel dogru yaklasim veri gizliligi, rollback ve kullanici onayini korur."
                    .to_string(),
            selected_best_option_reason:
                "Secilen en iyi secenek mevcut sistemle uyumlu ve test edilebilir oldugu icin uygundur."
                    .to_string(),
        }
    }

    #[test]
    fn missing_correct_approach_reason_stops_planning_gate() {
        let task_id = "test_missing_correct_approach_reason";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]);
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 'Test', 'Test', 'pending', 'planning_incomplete', 'not_started', 'high', 'pending_approval')",
            params![task_id],
        )
        .unwrap();
        let mut plan = valid_plan();
        plan.accepted_correct_approach_reason.clear();
        save_plan(task_id, plan).unwrap();
        assert!(PlanningGate::validate_planning(task_id).is_err());
    }

    #[test]
    fn missing_best_option_reason_stops_planning_gate() {
        let task_id = "test_missing_best_option_reason";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]);
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 'Test', 'Test', 'pending', 'planning_incomplete', 'not_started', 'high', 'pending_approval')",
            params![task_id],
        )
        .unwrap();
        let mut plan = valid_plan();
        plan.selected_best_option_reason.clear();
        save_plan(task_id, plan).unwrap();
        assert!(PlanningGate::validate_planning(task_id).is_err());
    }
}
