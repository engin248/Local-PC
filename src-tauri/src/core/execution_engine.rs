use crate::core::action_executor::ActionExecutor;
use crate::core::alternative_analyzer::AlternativeAnalyzer;
use crate::core::approval_manager::ApprovalManager;
use crate::core::audit_logger::AuditLogger;
use crate::core::authority_router::AuthorityRouter;
use crate::core::checkpoint_manager::CheckpointManager;
use crate::core::decision_tree_builder::DecisionTreeBuilder;
use crate::core::integrity_checker::IntegrityChecker;
use crate::core::operation_monitor::OperationMonitor;
use crate::core::planning_gate::PlanningGate;
use crate::core::report_manager::ReportManager;
use crate::core::risk_engine::RiskEngine;
use crate::core::rollback_manager::RollbackManager;
use crate::core::statement_collector::StatementCollector;
use crate::core::task_decomposer::TaskDecomposer;
use crate::core::task_intake::Task;
use crate::core::test_manager::TestManager;
use crate::storage::db::Database;
use rusqlite::params;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum RunMode {
    ReadOnly,
    ApprovedExecution,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ApprovalSource {
    DatabaseOnly,
    PolicyValidated,
    UiUser,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ExecutionContext {
    pub run_mode: RunMode,
    pub current_user_id: Option<String>,
    pub approval_source: ApprovalSource,
    pub allowed_actions: Vec<String>,
    pub read_only: bool,
}

impl ExecutionContext {
    pub fn read_only_pipeline() -> Self {
        Self {
            run_mode: RunMode::ReadOnly,
            current_user_id: None,
            approval_source: ApprovalSource::DatabaseOnly,
            allowed_actions: vec![
                "read_file".to_string(),
                "read_folder".to_string(),
                "sqlite_read".to_string(),
                "code_analysis".to_string(),
                "code_modification_proposal".to_string(),
                "research".to_string(),
                "ai_provider_call".to_string(),
                "report_generate".to_string(),
            ],
            read_only: true,
        }
    }

    /// Onay kapıları geçildikten sonra yazma/icra aksiyonlarını açar (URETIM-01).
    pub fn upgrade_to_approved_execution(&mut self) {
        if matches!(self.run_mode, RunMode::ApprovedExecution) && !self.read_only {
            return;
        }
        self.run_mode = RunMode::ApprovedExecution;
        self.read_only = false;
        for action in [
            "file_write",
            "write_file",
            "file_delete",
            "delete_file",
            "write_folder",
            "sqlite_write",
            "api_write",
            "terminal_command",
            "live_system_update",
            "snapshot_create",
            "test_run",
        ] {
            if !self.allowed_actions.iter().any(|existing| existing == action) {
                self.allowed_actions.push(action.to_string());
            }
        }
    }
}

pub fn execute_task_pipeline(task_id: &str) -> Result<ExecutionResult, String> {
    // 1. Fetch task
    let task = crate::core::task_intake::TaskIntake::get_task(task_id)?;

    // 2. Varsayılan read-only; onay kapıları geçilince ApprovedExecution'a yükselir (URETIM-01)
    let mut context = ExecutionContext::read_only_pipeline();

    // 3. Execute workflow
    match ExecutionEngine::execute_workflow(task, &mut context) {
        Ok(report) => {
            let db = Database::new();
            let conn = db.get_connection().map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE tasks SET status = 'completed', execution_status = 'completed' WHERE id = ?1",
                params![task_id],
            ).map_err(|e| e.to_string())?;

            Ok(ExecutionResult {
                success: true,
                message: format!("İşlem başarıyla tamamlandı!\nNihai Rapor:\n{}", report),
            })
        }
        Err(e) => {
            let db = Database::new();
            let conn = db.get_connection().map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE tasks SET status = 'failed', execution_status = 'failed' WHERE id = ?1",
                params![task_id],
            )
            .map_err(|e| e.to_string())?;

            Ok(ExecutionResult {
                success: false,
                message: format!("İşlem başarısız oldu: {}", e),
            })
        }
    }
}

pub struct ExecutionEngine;

impl ExecutionEngine {
    pub fn execute_workflow(mut task: Task, context: &mut ExecutionContext) -> Result<String, String> {
        let task_id = &task.id;
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        // --- GATE 1: PLANNING GATE ---
        task.current_gate = Some("Planning Gate".to_string());
        AuditLogger::log_event(
            task_id,
            "info",
            "Gate 1/8: Planning Gate başlatılıyor...",
            Some("Planning Gate"),
            Some("gate_start"),
            None,
        )?;
        if let Err(e) = PlanningGate::validate_planning(task_id) {
            AuditLogger::log_event(
                task_id,
                "error",
                &format!("Planning Gate BAŞARISIZ: {}", e),
                Some("Planning Gate"),
                Some("gate_fail"),
                None,
            )?;
            return Err(e);
        }
        CheckpointManager::verify_checkpoint(task_id, None, "Planning Verification", true)?;
        AuditLogger::log_event(
            task_id,
            "info",
            "Planning Gate BAŞARIYLA GEÇİLDİ.",
            Some("Planning Gate"),
            Some("gate_success"),
            None,
        )?;

        // Load the actual physical plan standard inputs
        let plan = PlanningGate::load_plan(task_id)?;
        OperationMonitor::initialize(task_id, &plan.operation_plan)?;
        OperationMonitor::log_gate(
            task_id,
            None,
            "Planning Gate",
            "passed",
            "Operasyon plani parse edildi ve monitor adimlari olusturuldu.",
        )?;

        // Check if decision nodes already exist in the database for this task
        let mut stmt = conn.prepare(
            "SELECT id, task_id, breakdown_id, level, parent_node_id, required_approval, gate_status, authorized_decider_type, authorized_decider_id, status, selected_option, reason, evidence_json, confidence 
             FROM decision_nodes WHERE task_id = ?1 ORDER BY level ASC"
        ).map_err(|e| e.to_string())?;

        let existing_nodes: Vec<crate::core::decision_tree_builder::DecisionNode> = stmt
            .query_map(params![task_id], |row| {
                Ok(crate::core::decision_tree_builder::DecisionNode {
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
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let nodes = if !existing_nodes.is_empty() {
            existing_nodes
        } else {
            // Decompose task and build decision tree
            let breakdowns = TaskDecomposer::decompose_task(task_id, &task.user_request)?;
            DecisionTreeBuilder::build_tree(task_id, &breakdowns)?
        };

        if nodes.is_empty() {
            return Err("Görev ağacında işlenecek hiçbir karar düğümü bulunamadı!".to_string());
        }

        // Load authority matrix for dynamic mapping
        let matrix_path = crate::core::dependency_analyzer::DependencyAnalyzer::get_config_path(
            "authority_matrix.json",
        )?;
        let matrix_data = std::fs::read_to_string(&matrix_path)
            .map_err(|e| format!("Yetki matrisi (authority_matrix.json) okunamadı: {}", e))?;
        let matrix: serde_json::Value = serde_json::from_str(&matrix_data)
            .map_err(|_| "Yetki matrisi JSON formatı geçersiz!".to_string())?;

        // Loop over and process all decision nodes sequentially
        for node in &nodes {
            let node_id = &node.id;

            AuditLogger::log_event(
                task_id,
                "info",
                &format!(
                    "Düğüm {} işleniyor (Grup: {})...",
                    node_id, node.breakdown_id
                ),
                None,
                Some("node_start"),
                None,
            )?;

            // Resolve dynamic action mapped from the node level
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

            OperationMonitor::check_action(task_id, Some(node_id), action, "Authority Gate")?;

            // Enforce that the context allows this action
            if !context.allowed_actions.contains(&action.to_string())
                && matches!(
                    context.run_mode,
                    RunMode::ApprovedExecution | RunMode::ReadOnly
                )
            {
                let err = format!(
                    "HATA: Yürütme bağlamı bu yetkilendirme aksiyonuna izin vermiyor: {}",
                    action
                );
                AuditLogger::log_event(
                    task_id,
                    "error",
                    &err,
                    Some("Authority Gate"),
                    Some("gate_fail"),
                    None,
                )?;
                return Err(err);
            }

            let write_like_action = matches!(
                action,
                "file_write"
                    | "write_file"
                    | "file_delete"
                    | "delete_file"
                    | "write_folder"
                    | "sqlite_write"
                    | "api_write"
                    | "terminal_command"
                    | "live_system_update"
            );
            if context.read_only && write_like_action {
                AuditLogger::log_event(
                    task_id,
                    "info",
                    &format!(
                        "Yazma aksiyonu onay kapıları tamamlanana kadar ertelendi: {}",
                        action
                    ),
                    Some("Authority Gate"),
                    Some("write_deferred_until_approval"),
                    None,
                )?;
            }

            // --- GATE 2: AUTHORITY GATE ---
            task.current_gate = Some("Authority Gate".to_string());
            AuditLogger::log_event(
                task_id,
                "info",
                &format!(
                    "Gate 2/8: Authority Gate başlatılıyor (Düğüm: {})...",
                    node_id
                ),
                Some("Authority Gate"),
                Some("gate_start"),
                None,
            )?;
            if let Err(e) = AuthorityRouter::route_and_validate(node) {
                AuditLogger::log_event(
                    task_id,
                    "error",
                    &format!("Authority Gate BAŞARISIZ: {}", e),
                    Some("Authority Gate"),
                    Some("gate_fail"),
                    None,
                )?;
                return Err(e);
            }
            CheckpointManager::verify_checkpoint(
                task_id,
                Some(node_id),
                "Authority Verification",
                true,
            )?;
            AuditLogger::log_event(
                task_id,
                "info",
                "Authority Gate BAŞARIYLA GEÇİLDİ.",
                Some("Authority Gate"),
                Some("gate_success"),
                None,
            )?;

            // --- GATE 3: STATEMENT GATE ---
            task.current_gate = Some("Statement Gate".to_string());
            AuditLogger::log_event(
                task_id,
                "info",
                &format!(
                    "Gate 3/8: Statement Gate başlatılıyor (Düğüm: {})...",
                    node_id
                ),
                Some("Statement Gate"),
                Some("gate_start"),
                None,
            )?;
            StatementCollector::collect_statement(
                node_id,
                "user_instruction",
                "current_user_request",
                &task.user_request,
                None,
            )?;
            if matches!(
                action,
                "research" | "ai_provider_call" | "code_analysis" | "code_modification_proposal"
            ) {
                match crate::ai_providers::ai_provider_invoke::AiProviderInvoker::invoke_for_node(
                    task_id,
                    node_id,
                    action,
                    &task.user_request,
                ) {
                    Ok(()) => {}
                    Err(err) => {
                        AuditLogger::log_event(
                            task_id,
                            "warning",
                            &format!("Statement Gate AI provider atlandı: {}", err),
                            Some("Statement Gate"),
                            Some("ai_statement_skipped"),
                            None,
                        )?;
                    }
                }
            }
            CheckpointManager::verify_checkpoint(
                task_id,
                Some(node_id),
                "Statement Verification",
                true,
            )?;
            AuditLogger::log_event(
                task_id,
                "info",
                "Statement Gate BAŞARIYLA GEÇİLDİ.",
                Some("Statement Gate"),
                Some("gate_success"),
                None,
            )?;

            // --- GATE 4: ALTERNATIVE GATE ---
            task.current_gate = Some("Alternative Gate".to_string());
            AuditLogger::log_event(
                task_id,
                "info",
                &format!(
                    "Gate 4/8: Alternative Gate başlatılıyor (Düğüm: {})...",
                    node_id
                ),
                Some("Alternative Gate"),
                Some("gate_start"),
                None,
            )?;
            let alts = AlternativeAnalyzer::generate_alternatives(node_id)?;
            if alts.len() < 3 {
                let err =
                    "HATA: Kritik karar için en az 3 alternatif alternatif analiz edilmelidir!"
                        .to_string();
                AuditLogger::log_event(
                    task_id,
                    "error",
                    &format!("Alternative Gate BAŞARISIZ: {}", err),
                    Some("Alternative Gate"),
                    Some("gate_fail"),
                    None,
                )?;
                return Err(err);
            }
            CheckpointManager::verify_checkpoint(
                task_id,
                Some(node_id),
                "Alternative Verification",
                true,
            )?;
            AuditLogger::log_event(
                task_id,
                "info",
                "Alternative Gate BAŞARIYLA GEÇİLDİ.",
                Some("Alternative Gate"),
                Some("gate_success"),
                None,
            )?;

            // --- GATE 5: RISK GATE ---
            task.current_gate = Some("Risk Gate".to_string());
            AuditLogger::log_event(
                task_id,
                "info",
                &format!("Gate 5/8: Risk Gate başlatılıyor (Düğüm: {})...", node_id),
                Some("Risk Gate"),
                Some("gate_start"),
                None,
            )?;
            let risk = RiskEngine::assess_risk(task_id, node_id, action)?;
            CheckpointManager::verify_checkpoint(
                task_id,
                Some(node_id),
                "Risk Verification",
                true,
            )?;
            AuditLogger::log_event(
                task_id,
                "info",
                &format!(
                    "Risk Gate BAŞARIYLA GEÇİLDİ. Belirlenen Risk: {}",
                    risk.risk_level
                ),
                Some("Risk Gate"),
                Some("gate_success"),
                None,
            )?;

            // --- GATE 6: APPROVAL GATE ---
            task.current_gate = Some("Approval Gate".to_string());
            AuditLogger::log_event(
                task_id,
                "info",
                &format!(
                    "Gate 6/8: Approval Gate başlatılıyor (Düğüm: {})...",
                    node_id
                ),
                Some("Approval Gate"),
                Some("gate_start"),
                None,
            )?;

            // Parse risk_rules.json to check if approval or double check is required
            let rules_path = crate::core::dependency_analyzer::DependencyAnalyzer::get_config_path(
                "risk_rules.json",
            )?;
            let rules_data = std::fs::read_to_string(&rules_path)
                .map_err(|e| format!("Risk kuralları okunamadı ({}): {}", rules_path, e))?;
            let rules: serde_json::Value = serde_json::from_str(&rules_data)
                .map_err(|e| format!("Risk kuralları JSON formatı geçersiz: {}", e))?;
            let rules_config = rules
                .get("levels")
                .and_then(|levels| levels.get(&risk.risk_level))
                .ok_or_else(|| {
                    format!("Risk seviyesi için kural bulunamadı: {}", risk.risk_level)
                })?;
            let requires_approval = rules_config
                .get("requires_approval")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| {
                    format!(
                        "Risk seviyesi için requires_approval alanı eksik veya geçersiz: {}",
                        risk.risk_level
                    )
                })?;
            let requires_double_check = rules_config
                .get("requires_double_check")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| {
                    format!(
                        "Risk seviyesi için requires_double_check alanı eksik veya geçersiz: {}",
                        risk.risk_level
                    )
                })?;

            if requires_approval {
                OperationMonitor::check_action(
                    task_id,
                    Some(node_id),
                    "approval_check",
                    "Approval Gate",
                )?;
                let required_approval_count = if requires_double_check { 2 } else { 1 };
                ApprovalManager::ensure_pending_approval_requests(
                    task_id,
                    Some(node_id),
                    action,
                    &risk.risk_level,
                    required_approval_count,
                )?;

                // Triple-Lock Security Model for High/Critical Risks
                if requires_double_check {
                    AuditLogger::log_event(task_id, "warning", "[GÜVENLİK KİLİDİ] Çift onay ve Üçlü Kilit Güvenlik modeli doğrulaması başlatıldı...", Some("Approval Gate"), Some("double_check_pending"), None)?;

                    // 1. Lock: System / Policy Check
                    if !std::path::Path::new(&rules_path).exists() {
                        let err = "HATA: Üçlü Kilit Modeli başarısız - Sistem policy dosyası (risk_rules.json) mevcut değil!".to_string();
                        AuditLogger::log_event(
                            task_id,
                            "error",
                            &format!("Approval Gate BAŞARISIZ: {}", err),
                            Some("Approval Gate"),
                            Some("policy_check_failed"),
                            None,
                        )?;
                        return Err(err);
                    }

                    // 2. Lock: User / Admin Approval Check (Ensure at least 2 distinct user/admin approval signatures exist in DB)
                    let approved_count: i32 = conn
                        .query_row(
                            "SELECT COUNT(DISTINCT approver_id) FROM approvals 
                         WHERE task_id = ?1
                         AND decision_node_id = ?2
                         AND action = ?3
                         AND risk_level IN ('high', 'critical')
                         AND status = 'approved'
                         AND approver_id IS NOT NULL
                         AND TRIM(approver_id) != ''",
                            params![task_id, node_id, action],
                            |row| row.get(0),
                        )
                        .map_err(|e| format!("Onay sayısı sorgulanamadı: {}", e))?;

                    let authorized_approved_count: i32 = conn
                        .query_row(
                            "SELECT COUNT(DISTINCT approver_id) FROM approvals 
                         WHERE task_id = ?1
                         AND decision_node_id = ?2
                         AND action = ?3
                         AND risk_level IN ('high', 'critical')
                         AND status = 'approved'
                         AND approver_id IS NOT NULL
                         AND TRIM(approver_id) != ''
                         AND approver_role IN ('admin', 'owner', 'security_officer')",
                            params![task_id, node_id, action],
                            |row| row.get(0),
                        )
                        .map_err(|e| format!("Yetkili onay sayısı sorgulanamadı: {}", e))?;

                    if approved_count < 2 || authorized_approved_count < 2 {
                        let err = format!(
                            "HATA: Üçlü Kilit - Kullanıcı/Admin onay kuralı ihlal edildi! En az 2 ayrı yetkili onay imzası gereklidir! Mevcut onay sayısı: {}, yetkili onay sayısı: {}",
                            approved_count,
                            authorized_approved_count
                        );
                        AuditLogger::log_event(
                            task_id,
                            "error",
                            &format!("Approval Gate BAŞARISIZ: {}", err),
                            Some("Approval Gate"),
                            Some("user_check_failed"),
                            None,
                        )?;
                        return Err(err);
                    }

                    // 3. Lock: Rollback / Test Gate Validation
                    let planning_status: String = conn
                        .query_row(
                            "SELECT planning_status FROM tasks WHERE id = ?1",
                            params![task_id],
                            |row| row.get(0),
                        )
                        .map_err(|e| format!("Planlama durumu sorgulanamadı: {}", e))?;

                    if planning_status != "planning_complete" {
                        let err = "HATA: Üçlü Kilit - Geri alma ve planlama doğrulaması başarısız!"
                            .to_string();
                        AuditLogger::log_event(
                            task_id,
                            "error",
                            &format!("Approval Gate BAŞARISIZ: {}", err),
                            Some("Approval Gate"),
                            Some("rollback_check_failed"),
                            None,
                        )?;
                        return Err(err);
                    }

                    AuditLogger::log_event(
                        task_id,
                        "info",
                        "Üçlü Kilit Güvenlik Modeli başarıyla doğrulandı.",
                        Some("Approval Gate"),
                        Some("double_check_passed"),
                        None,
                    )?;
                } else {
                    let approval_id: String = conn
                        .query_row(
                            "SELECT id FROM approvals
                         WHERE task_id = ?1
                         AND decision_node_id = ?2
                         AND action = ?3
                         AND risk_level = ?4
                         ORDER BY requested_at ASC
                         LIMIT 1",
                            params![task_id, node_id, action, risk.risk_level],
                            |row| row.get(0),
                        )
                        .map_err(|e| e.to_string())?;

                    let approved = ApprovalManager::check_approval_status(&approval_id)?;
                    if !approved {
                        let err = "HATA: Onay kaydı veritabanında aktif olarak 'approved' işaretlenmediğinden işlem durduruldu!".to_string();
                        AuditLogger::log_event(
                            task_id,
                            "error",
                            &format!("Approval Gate BAŞARISIZ: {}", err),
                            Some("Approval Gate"),
                            Some("gate_fail"),
                            None,
                        )?;
                        return Err(err);
                    }
                }
            } else {
                AuditLogger::log_event(
                    task_id,
                    "info",
                    "Düşük riskli işlem: Onay gerekmiyor.",
                    Some("Approval Gate"),
                    Some("policy_no_approval_required"),
                    None,
                )?;
            }

            CheckpointManager::verify_checkpoint(
                task_id,
                Some(node_id),
                "Approval Verification",
                true,
            )?;
            AuditLogger::log_event(
                task_id,
                "info",
                "Approval Gate BAŞARIYLA GEÇİLDİ.",
                Some("Approval Gate"),
                Some("gate_success"),
                None,
            )?;

            if write_like_action && requires_approval {
                context.upgrade_to_approved_execution();
                AuditLogger::log_event(
                    task_id,
                    "info",
                    "Onay tamamlandı: yürütme modu ApprovedExecution olarak açıldı.",
                    Some("Approval Gate"),
                    Some("run_mode_upgraded"),
                    None,
                )?;
            }

            // --- GATE 7: ROLLBACK GATE ---
            task.current_gate = Some("Rollback Gate".to_string());
            AuditLogger::log_event(
                task_id,
                "info",
                &format!(
                    "Gate 7/8: Rollback Gate başlatılıyor (Düğüm: {})...",
                    node_id
                ),
                Some("Rollback Gate"),
                Some("gate_start"),
                None,
            )?;

            let target_path = Self::resolve_target_path(&plan.impact_area)?;
            OperationMonitor::check_action(
                task_id,
                Some(node_id),
                "snapshot_create",
                "Rollback Gate",
            )?;
            let target_type =
                if target_path.ends_with(".db") || target_path.contains("storage/app.db") {
                    "sqlite"
                } else if target_path.contains("/") || target_path.contains("\\") {
                    if std::path::Path::new(&target_path).is_dir() {
                        "folder"
                    } else if std::path::Path::new(&target_path).extension().is_some() {
                        "file"
                    } else {
                        "folder"
                    }
                } else {
                    "file"
                };

            let snapshot_path = RollbackManager::create_snapshot_with_context(
                task_id,
                Some(node_id),
                target_type,
                &target_path,
                Some(&format!("{}:{}", task_id, node_id)),
            )?;
            AuditLogger::log_event(
                task_id,
                "info",
                &format!("Snapshot yedek başarıyla oluşturuldu: {}", snapshot_path),
                Some("Rollback Gate"),
                Some("snapshot_created"),
                None,
            )?;
            CheckpointManager::verify_checkpoint(
                task_id,
                Some(node_id),
                "Rollback Verification",
                true,
            )?;
            AuditLogger::log_event(
                task_id,
                "info",
                "Rollback Gate BAŞARIYLA GEÇİLDİ.",
                Some("Rollback Gate"),
                Some("gate_success"),
                None,
            )?;

            // --- GATE 8: TEST GATE ---
            task.current_gate = Some("Test Gate".to_string());
            AuditLogger::log_event(
                task_id,
                "info",
                &format!("Gate 8/8: Test Gate başlatılıyor (Düğüm: {})...", node_id),
                Some("Test Gate"),
                Some("gate_start"),
                None,
            )?;

            if plan.test_criteria.is_empty() {
                let err = "HATA: Test Gate için gerçek test kriteri bulunamadı.".to_string();
                AuditLogger::log_event(
                    task_id,
                    "error",
                    &err,
                    Some("Test Gate"),
                    Some("gate_fail"),
                    None,
                )?;
                return Err(err);
            }

            OperationMonitor::check_action(task_id, Some(node_id), "test_run", "Test Gate")?;
            for criterion in &plan.test_criteria {
                let test_cmd = Self::normalize_test_criterion(criterion, &target_path);
                let expected = Self::expected_result_for_test(&test_cmd)?;
                if let Err(e) = TestManager::run_test(task_id, &test_cmd, expected, "pending") {
                    AuditLogger::log_event(
                        task_id,
                        "error",
                        &format!("Test Gate BAŞARISIZ: {}", e),
                        Some("Test Gate"),
                        Some("gate_fail"),
                        None,
                    )?;
                    RollbackManager::trigger_rollback(task_id)?;
                    return Err(e);
                }
            }
            CheckpointManager::verify_checkpoint(
                task_id,
                Some(node_id),
                "Test Verification",
                true,
            )?;
            AuditLogger::log_event(
                task_id,
                "info",
                "Test Gate BAŞARIYLA GEÇİLDİ.",
                Some("Test Gate"),
                Some("gate_success"),
                None,
            )?;

            ActionExecutor::dispatch_after_gates(
                task_id,
                node_id,
                action,
                &target_path,
                context,
                &risk.risk_level,
            )?;

            // Update node status to completed in DB
            conn.execute(
                "UPDATE decision_nodes SET status = 'completed' WHERE id = ?1",
                params![node_id],
            )
            .map_err(|e| e.to_string())?;
        }

        // --- INTEGRITY CHECK & FINAL REPORT ---
        IntegrityChecker::check_integrity(task_id)?;
        OperationMonitor::check_action(task_id, None, "report_generate", "Report Gate")?;
        OperationMonitor::ensure_all_steps_completed(task_id)?;
        let report_content = ReportManager::generate_final_report(task_id)?;

        AuditLogger::log_event(
            task_id,
            "info",
            "TÜM KAPILAR BAŞARIYLA GEÇİLDİ. OPERASYON TAMAMLANDI.",
            None,
            Some("task_finished"),
            None,
        )?;

        Ok(report_content)
    }

    fn normalize_test_criterion(criterion: &str, target_path: &str) -> String {
        for prefix in [
            "file_exists:",
            "file_contains:",
            "file_hash_equals:",
            "file_hash_unchanged:",
            "rollback_restored:",
        ] {
            if let Some(rest) = criterion.strip_prefix(prefix) {
                let mut parts = rest.splitn(2, '|');
                if let Some(raw_path) = parts.next() {
                    if let Ok(resolved_path) = Self::resolve_test_path(raw_path, target_path) {
                        if let Some(remainder) = parts.next() {
                            return format!("{}{}|{}", prefix, resolved_path, remainder);
                        }
                        return format!("{}{}", prefix, resolved_path);
                    }
                }
            }
        }

        if let Some(file_name) = std::path::Path::new(target_path)
            .file_name()
            .and_then(|v| v.to_str())
        {
            let file_exists_prefix = "file_exists:";
            if let Some(raw_path) = criterion.strip_prefix(file_exists_prefix) {
                if raw_path == file_name {
                    return format!("{}{}", file_exists_prefix, target_path);
                }
            }
        }

        criterion.to_string()
    }

    fn resolve_test_path(raw_path: &str, target_path: &str) -> Result<String, String> {
        if raw_path == target_path {
            return Ok(target_path.to_string());
        }

        let path = std::path::Path::new(raw_path);
        if path.is_absolute() {
            return Ok(raw_path.to_string());
        }

        if let Some(target_file_name) = std::path::Path::new(target_path)
            .file_name()
            .and_then(|v| v.to_str())
        {
            if raw_path == target_file_name {
                return Ok(target_path.to_string());
            }
        }

        let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
        Ok(root.join(raw_path).to_string_lossy().into_owned())
    }

    fn resolve_target_path(raw_target: &str) -> Result<String, String> {
        let target = raw_target.trim();
        if target.is_empty() {
            return Err("HATA: Rollback/Test hedef yolu boş olamaz.".to_string());
        }

        let path = std::path::Path::new(target);
        if path.is_absolute() {
            return Ok(target.to_string());
        }

        let root = crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root()?;
        Ok(root.join(target).to_string_lossy().into_owned())
    }

    fn expected_result_for_test(test_cmd: &str) -> Result<&'static str, String> {
        if test_cmd.starts_with("file_exists:") {
            Ok("exists")
        } else if test_cmd.starts_with("file_contains:") {
            Ok("contains")
        } else if test_cmd.starts_with("file_hash_equals:") {
            Ok("hash_equals")
        } else if test_cmd.starts_with("file_hash_unchanged:") {
            Ok("hash_unchanged")
        } else if test_cmd.starts_with("sqlite_query_equals:") {
            Ok("query_equals")
        } else if test_cmd.starts_with("approval_exists:") {
            Ok("approved_exists")
        } else if test_cmd.starts_with("snapshot_exists:") {
            Ok("snapshot_verified")
        } else if test_cmd.starts_with("rollback_restored:") {
            Ok("restored_equals")
        } else if test_cmd.starts_with("no_unapproved_write:") {
            Ok("no_unapproved_write")
        } else if test_cmd.starts_with("build_command_passed:") {
            Ok("build_passed")
        } else {
            Err(format!(
                "HATA: Desteklenmeyen veya gerçek çalıştırılamayan test kriteri: {}",
                test_cmd
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::approval_manager::ApprovalManager;
    use crate::core::dependency_analyzer::DependencyAnalyzer;
    use crate::core::planning_gate::{save_plan, PlanningStandardInput};
    use crate::storage::db::Database;

    #[test]
    fn read_only_context_upgrades_to_approved_execution() {
        let mut ctx = ExecutionContext::read_only_pipeline();
        assert!(matches!(ctx.run_mode, RunMode::ReadOnly));
        assert!(ctx.read_only);
        ctx.upgrade_to_approved_execution();
        assert!(matches!(ctx.run_mode, RunMode::ApprovedExecution));
        assert!(!ctx.read_only);
        assert!(ctx.allowed_actions.iter().any(|a| a == "write_file"));
    }

    #[test]
    fn test_triple_lock_execution_workflow() {
        let task_id = "test_triple_lock_task";
        let db = Database::new();
        let conn = db.get_connection().unwrap();

        // 1. Insert Task into database
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]);
        let _ = conn.execute("DELETE FROM approvals WHERE task_id = ?1", params![task_id]);
        let _ = conn.execute(
            "DELETE FROM dependency_assessments WHERE task_id = ?1",
            params![task_id],
        );
        let _ = conn.execute(
            "DELETE FROM task_breakdown WHERE task_id = ?1",
            params![task_id],
        );
        let _ = conn.execute(
            "DELETE FROM decision_nodes WHERE task_id = ?1",
            params![task_id],
        );

        // Establish real file for physical test gate
        let root =
            crate::core::dependency_analyzer::DependencyAnalyzer::get_project_root().unwrap();
        let test_log_path = root
            .join("storage")
            .join("logs")
            .join("integration_test_target.txt");
        let _ = std::fs::create_dir_all(test_log_path.parent().unwrap());
        let _ = std::fs::write(
            &test_log_path,
            b"Initial contents for rollback snapshot integration test",
        );
        let test_log_str = test_log_path.to_string_lossy().into_owned();

        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 'Integration Test Task', 'Write changes to integration test target.', 'pending', 'planning_incomplete', 'not_started', 'high', 'pending_approval')",
            params![task_id]
        ).unwrap();

        // 2. Save complete planning standard inputs to satisfy the planning gate
        let mut plan = PlanningStandardInput {
            task_definition: "Integration Test Task Definition".to_string(),
            purpose: "To test the full 8-gate Triple-Lock pipeline".to_string(),
            scope: "Lokal ve Güvenli Test".to_string(),
            topic: "Dosya Sistemi Operasyonu".to_string(),
            sub_topic: "Dosya/Klasör Modifikasyonu".to_string(),
            criterion: "Fiziksel Veri Kriteri".to_string(),
            sub_criterion: "Dosya Değişiklik Kontrolü".to_string(),
            alternatives: vec![
                "Yalnizca oku ve raporla".to_string(),
                "Uygulama yapma, manuel plan uret".to_string(),
                "Onayli ve rollback destekli uygula".to_string(),
                "Onaysiz ve rollback'siz dogrudan uygulama - elenen alternatif".to_string(),
            ],
            risk_analysis: "high".to_string(),
            impact_area: test_log_str.clone(),
            technology_selection: "Tauri, Rust, SQLite".to_string(),
            dependency_analysis: "Local file system dependencies analyzed".to_string(),
            checkpoints: vec!["Checkpoint 1".to_string(), "Checkpoint 2".to_string()],
            test_criteria: vec!["file_exists:integration_test_target.txt".to_string()],
            rollback_plan: "Restore integration test target from snapshot backup".to_string(),
            operation_plan: String::new(),
            authorized_deciders: vec!["Admin".to_string(), "Security Officer".to_string()],
            accepted_correct_approach_reason:
                "Genel dogru yaklasim kullanici onayi, rollback, test ve veri guvenligini korur."
                    .to_string(),
            selected_best_option_reason:
                "Secilen en iyi secenek mevcut test hedefiyle uyumlu, geri alinabilir ve dogrulanabilir."
                    .to_string(),
            operation_sequence: vec![
                "Cozumleme yap".to_string(),
                "Kabul edilmis dogruyu sec".to_string(),
                "En iyi uygulanabilir alternatifi sec".to_string(),
                "Uygulama paketini alt birime ver".to_string(),
                "Kontrol et".to_string(),
                "Bagimsiz dogrula".to_string(),
                "Son onay ver".to_string(),
            ],
            control_criteria: vec!["Plan var".to_string(), "Rollback var".to_string()],
            executor_role: "executor".to_string(),
            correctness_guard_role: "correctness_guard".to_string(),
            controller_role: "controller".to_string(),
            independent_verifier_role: "independent_verifier".to_string(),
            final_approver_role: "final_approver".to_string(),
            per_part_alternative_policy:
                "Her atomik parca icin real hayattaki tum makul alternatifler ayni kriterlerle degerlendirilir."
                    .to_string(),
        };

        // 3. Decompose task and build tree to generate nodes
        let breakdowns = crate::core::task_decomposer::TaskDecomposer::decompose_task(
            task_id,
            "Write changes to integration test target.",
        )
        .unwrap();
        let nodes = crate::core::decision_tree_builder::DecisionTreeBuilder::build_tree(
            task_id,
            &breakdowns,
        )
        .unwrap();
        assert!(!nodes.is_empty(), "Decision tree nodes should be generated");

        // Load authority matrix for dynamic mapping in tests
        let matrix_path = crate::core::dependency_analyzer::DependencyAnalyzer::get_config_path(
            "authority_matrix.json",
        )
        .unwrap();
        let matrix_data = std::fs::read_to_string(&matrix_path).unwrap();
        let matrix: serde_json::Value = serde_json::from_str(&matrix_data).unwrap();
        let risk_rules_path = DependencyAnalyzer::get_config_path("risk_rules.json").unwrap();
        let risk_rules_data = std::fs::read_to_string(&risk_rules_path).unwrap();
        let risk_rules: serde_json::Value = serde_json::from_str(&risk_rules_data).unwrap();
        let mut inferred_plan_actions: Vec<String> = Vec::new();

        // 4. Create and approve 2 distinct approvals to satisfy the Lock 2 check (approved_count >= 2) per node
        for node in &nodes {
            let node_id = &node.id;

            // Resolve action dynamically
            let action = if let Some(mappings) = matrix.get("level_mappings") {
                let level_str = node.level.to_string();
                mappings
                    .get(&level_str)
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        mappings
                            .get("default")
                            .and_then(|v| v.as_str())
                            .unwrap_or("terminal_command")
                    })
            } else {
                "terminal_command"
            };
            inferred_plan_actions.push(format!("action:{}", action));
            let requires_approval = if let Some(risk_level) = risk_rules
                .get("action_mappings")
                .and_then(|action_mappings| action_mappings.get(action))
                .and_then(|action_cfg| action_cfg.get("level"))
                .and_then(|level| level.as_str())
            {
                risk_rules
                    .get("levels")
                    .and_then(|levels| levels.get(risk_level))
                    .and_then(|level_cfg| level_cfg.get("requires_approval"))
                    .and_then(|required| required.as_bool())
                    .unwrap_or(false)
            } else {
                false
            };
            if requires_approval {
                inferred_plan_actions.push("action:approval_check".to_string());
            }
            inferred_plan_actions.push("action:snapshot_create".to_string());
            inferred_plan_actions.push("action:test_run".to_string());

            // Insert first approval by user_1
            let approval1_id = ApprovalManager::request_approval(
                task_id,
                Some(node_id),
                action,
                "high",
            )
            .unwrap();
            conn.execute(
                "UPDATE approvals SET approver_id = 'user_1', approver_role = 'admin', approval_source = 'database', approved_at = CURRENT_TIMESTAMP, status = 'approved' WHERE id = ?1",
                params![approval1_id]
            ).unwrap();

            // Insert second approval by user_2
            let approval2_id = ApprovalManager::request_approval(
                task_id,
                Some(node_id),
                action,
                "high",
            )
            .unwrap();
            conn.execute(
                "UPDATE approvals SET approver_id = 'user_2', approver_role = 'security_officer', approval_source = 'database', approved_at = CURRENT_TIMESTAMP, status = 'approved' WHERE id = ?1",
                params![approval2_id]
            ).unwrap();
        }
        inferred_plan_actions.push("action:report_generate".to_string());
        plan.operation_plan = inferred_plan_actions.join(", ");
        save_plan(task_id, plan).unwrap();

        // 5. Fetch task from db
        let task = crate::core::task_intake::TaskIntake::get_task(task_id).unwrap();

        // 6. Establish strict test context
        let mut context = ExecutionContext {
            run_mode: RunMode::ApprovedExecution,
            current_user_id: Some("test_runner".to_string()),
            approval_source: ApprovalSource::DatabaseOnly,
            allowed_actions: vec![
                "file_write".to_string(),
                "write_file".to_string(),
                "sqlite_write".to_string(),
                "snapshot_create".to_string(),
                "terminal_command".to_string(),
                "code_modification_proposal".to_string(),
                "code_analysis".to_string(),
                "research".to_string(),
                "ai_provider_call".to_string(),
                "test_run".to_string(),
            ],
            read_only: false,
        };

        // 7. Run the entire workflow!
        let result = ExecutionEngine::execute_workflow(task, &mut context);
        assert!(
            result.is_ok(),
            "Workflow execution should succeed with all 8 gates passing: {:?}",
            result.err()
        );

        // Clean up
        let _ = std::fs::remove_file(test_log_path);
        let plan_file = root
            .join("storage")
            .join("backups")
            .join(format!("plan_{}.json", task_id));
        let _ = std::fs::remove_file(plan_file);
    }
}
