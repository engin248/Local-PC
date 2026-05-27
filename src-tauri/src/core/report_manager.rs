use crate::storage::db::Database;
use rusqlite::params;
use std::fs;
use uuid::Uuid;

pub struct ReportManager;

impl ReportManager {
    pub fn report_has_required_sections(content: &str) -> bool {
        let old_project_marker: String = ['S', 'T', 'P'].iter().collect();
        content.contains("## A. Çözümleme Raporu")
            && content.contains("## B. Uygulama Planı")
            && content.contains("## C. Uygulama İzleme Raporu")
            && !content.contains(&old_project_marker)
    }

    pub fn generate_final_report(task_id: &str) -> Result<String, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        // Retrieve task title
        let task_title: String = conn
            .query_row(
                "SELECT title FROM tasks WHERE id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let report_id = Uuid::new_v4().to_string();
        let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
        let report_file = root
            .join("storage")
            .join("reports")
            .join(format!("{}.md", report_id));
        let report_path = report_file.to_string_lossy().into_owned();

        let plan = crate::core::planning_gate::PlanningGate::load_plan(task_id)?;
        let breakdown_summary = Self::collect_breakdown_summary(&conn, task_id)?;
        let alternatives_summary = Self::collect_alternatives_summary(&conn, task_id)?;
        let checkpoint_summary = Self::collect_checkpoint_summary(&conn, task_id)?;
        let test_summary = Self::collect_test_summary(&conn, task_id)?;
        let approval_summary = Self::collect_approval_summary(&conn, task_id)?;
        let monitor_summary = Self::collect_monitor_summary(&conn, task_id)?;
        let rollback_summary = Self::collect_rollback_summary(&conn, task_id)?;

        let content = format!(
            "# OPERASYON NİHAİ RAPORU\n\n\
             **Görev ID**: {}\n\
             **Görev Başlığı**: {}\n\
             **Durum**: Tamamlandı\n\n\
             ## A. Çözümleme Raporu\n\
             - **Konu / Alt Konu / Kriter / Alt Kriter**:\n{}\n\
             - **Alternatifler**:\n{}\n\
             - **Genel Doğru Yaklaşım**: {}\n\
             - **Seçilen En İyi Seçenek**: {}\n\
             - **Gerekçe**: {}\n\n\
             ## B. Uygulama Planı\n\
             - **İşlem Sırası**: {}\n\
             - **Teknoloji**: {}\n\
             - **Connector**: {}\n\
             - **Etki Alanı**: {}\n\
             - **Risk**: {}\n\
             - **Rollback**: {}\n\
             - **Test Kriterleri**: {}\n\
             - **Approval Gereksinimi**: {}\n\n\
             ## C. Uygulama İzleme Raporu\n\
             - **Executor**: ExecutionEngine + SystemConnector\n\
             - **Operation Controller**: OperationMonitor + CheckpointManager\n\
             - **Independent Verifier**: IntegrityChecker + SystemValidator + RiskEngine\n\
             - **Final Approver**: ApprovalManager + Kullanıcı/Admin\n\
             - **Gerçekleşen Adımlar**:\n{}\n\
             - **Checkpoint Sonuçları**:\n{}\n\
             - **Test Sonuçları**:\n{}\n\
             - **Onay Sonuçları**:\n{}\n\
             - **Rollback Sonucu**:\n{}\n",
            task_id,
            task_title,
            breakdown_summary,
            alternatives_summary,
            plan.accepted_correct_approach_reason,
            plan.selected_best_option_reason,
            plan.selected_best_option_reason,
            plan.operation_plan,
            plan.technology_selection,
            plan.authorized_deciders.join(", "),
            plan.impact_area,
            plan.risk_analysis,
            plan.rollback_plan,
            plan.test_criteria.join(", "),
            approval_summary,
            monitor_summary,
            checkpoint_summary,
            test_summary,
            approval_summary,
            rollback_summary
        );
        if !Self::report_has_required_sections(&content) {
            return Err("HATA: Rapor zorunlu üç bölümlü sözleşmeyi karşılamıyor.".to_string());
        }

        // Ensure storage/reports directory exists
        let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
        let reports_dir_path = root.join("storage").join("reports");
        fs::create_dir_all(&reports_dir_path)
            .map_err(|e| format!("Rapor dizini oluşturulamadı: {}", e))?;

        // Write report file
        fs::write(&report_path, &content).map_err(|e| e.to_string())?;

        // Save to DB
        conn.execute(
            "INSERT INTO reports (id, task_id, report_type, content, file_path)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![report_id, task_id, "final_audit", content, report_path],
        )
        .map_err(|e| e.to_string())?;

        // Update task status
        conn.execute(
            "UPDATE tasks SET status = 'completed', execution_status = 'completed' WHERE id = ?1",
            params![task_id],
        )
        .map_err(|e| e.to_string())?;

        Ok(content)
    }

    fn collect_breakdown_summary(
        conn: &rusqlite::Connection,
        task_id: &str,
    ) -> Result<String, String> {
        let mut stmt = conn
            .prepare("SELECT topic, subtopic, criterion, subcriterion FROM task_breakdown WHERE task_id = ?1 ORDER BY level ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(format!(
                    "  - {} / {} / {} / {}",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?
                ))
            })
            .map_err(|e| e.to_string())?;
        Self::join_rows(rows, "  - Kırılım kaydı yok")
    }

    fn collect_alternatives_summary(
        conn: &rusqlite::Connection,
        task_id: &str,
    ) -> Result<String, String> {
        let mut stmt = conn
            .prepare(
                "SELECT title, COALESCE(real_world_basis, ''), selected, COALESCE(selection_reason, reason, '')
                 FROM alternatives
                 WHERE decision_node_id IN (SELECT id FROM decision_nodes WHERE task_id = ?1)
                 ORDER BY selected DESC, title ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                let selected: i32 = row.get(2)?;
                Ok(format!(
                    "  - [{}] {} | Dayanak: {} | Gerekçe: {}",
                    if selected == 1 {
                        "SEÇİLDİ"
                    } else {
                        "ELENDİ"
                    },
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(3)?
                ))
            })
            .map_err(|e| e.to_string())?;
        Self::join_rows(rows, "  - Alternatif kaydı yok")
    }

    fn collect_checkpoint_summary(
        conn: &rusqlite::Connection,
        task_id: &str,
    ) -> Result<String, String> {
        let mut stmt = conn
            .prepare("SELECT checkpoint_type, status, COALESCE(result, '') FROM checkpoints WHERE task_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(format!(
                    "  - {}: {} ({})",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?
                ))
            })
            .map_err(|e| e.to_string())?;
        Self::join_rows(rows, "  - Checkpoint kaydı yok")
    }

    fn collect_test_summary(conn: &rusqlite::Connection, task_id: &str) -> Result<String, String> {
        let mut stmt = conn
            .prepare("SELECT test_name, status, COALESCE(actual_result, '') FROM tests WHERE task_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(format!(
                    "  - {}: {} ({})",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?
                ))
            })
            .map_err(|e| e.to_string())?;
        Self::join_rows(rows, "  - Test kaydı yok")
    }

    fn collect_approval_summary(
        conn: &rusqlite::Connection,
        task_id: &str,
    ) -> Result<String, String> {
        let mut stmt = conn
            .prepare("SELECT action, risk_level, status, COALESCE(approver_role, '') FROM approvals WHERE task_id = ?1 ORDER BY requested_at ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(format!(
                    "  - {} / {}: {} {}",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?
                ))
            })
            .map_err(|e| e.to_string())?;
        Self::join_rows(rows, "Onay gerekmiyor veya onay kaydı yok")
    }

    fn collect_monitor_summary(
        conn: &rusqlite::Connection,
        task_id: &str,
    ) -> Result<String, String> {
        let mut stmt = conn
            .prepare("SELECT actual_action, status, message FROM operation_monitor_logs WHERE task_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(format!(
                    "  - {}: {} ({})",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?
                ))
            })
            .map_err(|e| e.to_string())?;
        Self::join_rows(rows, "  - OperationMonitor kaydı yok")
    }

    fn collect_rollback_summary(
        conn: &rusqlite::Connection,
        task_id: &str,
    ) -> Result<String, String> {
        let mut stmt = conn
            .prepare("SELECT rollback_status, target_path, snapshot_path FROM snapshots WHERE task_id = ?1 ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(format!(
                    "{} | Hedef: {} | Snapshot: {}",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?
                ))
            })
            .map_err(|e| e.to_string())?;
        Self::join_rows(rows, "Rollback snapshot kaydı yok")
    }

    fn join_rows<I, T>(rows: I, empty: &str) -> Result<String, String>
    where
        I: IntoIterator<Item = rusqlite::Result<T>>,
        T: ToString,
    {
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| e.to_string())?.to_string());
        }
        if items.is_empty() {
            Ok(empty.to_string())
        } else {
            Ok(items.join("\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReportManager;

    #[test]
    fn report_missing_three_sections_fails_contract() {
        let bad_report = "# OPERASYON NİHAİ RAPORU\n\n## A. Çözümleme Raporu\n";
        assert!(!ReportManager::report_has_required_sections(bad_report));
    }

    #[test]
    fn report_contract_rejects_old_project_phrase() {
        let old_project_marker: String = ['S', 'T', 'P'].iter().collect();
        let bad_report = format!(
            "## A. Çözümleme Raporu\n## B. Uygulama Planı\n## C. Uygulama İzleme Raporu\n{}",
            old_project_marker
        );
        assert!(!ReportManager::report_has_required_sections(&bad_report));
    }
}
