pub mod ai_providers;
pub mod core;
pub mod storage;
pub mod system_connectors;

use crate::ai_providers::ai_provider_manager::AIProviderManager;
use crate::ai_providers::provider_base::AIProviderHealth;
use crate::core::approval_manager::submit_approval;
use crate::core::execution_engine::{execute_task_pipeline, ExecutionResult};
use crate::core::planning_gate::{save_plan, PlanningStandardInput};
use crate::core::rollback_manager::rollback_task;
use crate::core::system_validator::{SystemValidationIssue, SystemValidator};
use crate::core::task_intake::{create_task, Task, TaskIntakeRequest};
use crate::storage::db::init_db;
use crate::system_connectors::connector_base::SystemConnectorHealth;
use crate::system_connectors::system_connector_manager::SystemConnectorManager;
use serde::{Deserialize, Serialize};

// UI'dan tetiklenecek Tauri komutları

#[tauri::command]
fn create_task_cmd(title: String, user_request: String) -> Result<Task, String> {
    create_task(TaskIntakeRequest {
        title,
        user_request,
    })
}

#[tauri::command]
fn save_plan_cmd(task_id: String, plan: PlanningStandardInput) -> Result<(), String> {
    save_plan(&task_id, plan)
}

#[tauri::command]
fn execute_task_cmd(task_id: String) -> Result<ExecutionResult, String> {
    execute_task_pipeline(&task_id)
}

#[tauri::command]
fn submit_approval_cmd(
    approval_id: String,
    approve: bool,
    user_note: Option<String>,
    approver_id: Option<String>,
    approver_role: Option<String>,
) -> Result<(), String> {
    submit_approval(
        &approval_id,
        approve,
        user_note.as_deref(),
        approver_id.as_deref(),
        approver_role.as_deref(),
        Some("ui"),
    )
}

#[tauri::command]
fn rollback_task_cmd(task_id: String) -> Result<bool, String> {
    rollback_task(&task_id)
}

#[tauri::command]
fn get_system_health_cmd() -> Result<Vec<SystemValidationIssue>, String> {
    SystemValidator::validate()
}

#[tauri::command]
fn get_ai_provider_health_cmd(write_audit: Option<bool>) -> Result<Vec<AIProviderHealth>, String> {
    AIProviderManager::health_check_all(write_audit.unwrap_or(false))
}

#[tauri::command]
fn get_system_connector_health_cmd(
    write_audit: Option<bool>,
) -> Result<Vec<SystemConnectorHealth>, String> {
    SystemConnectorManager::health_check_all(write_audit.unwrap_or(false))
}

// Veritabanı sorgulama yardımcı komutları

#[tauri::command]
fn get_tasks_cmd() -> Result<Vec<Task>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT CAST(id AS TEXT), title, user_request, status, planning_status, execution_status, current_gate, last_valid_state_id, risk_level, approval_status, created_at FROM tasks ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                user_request: row.get(2)?,
                status: row.get(3)?,
                planning_status: row.get(4)?,
                execution_status: row.get(5)?,
                current_gate: row.get(6)?,
                last_valid_state_id: row.get(7)?,
                risk_level: row.get(8)?,
                approval_status: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[tauri::command]
fn get_task_breakdowns_cmd(task_id: String) -> Result<Vec<crate::core::task_decomposer::TaskBreakdown>, String> {
    crate::core::task_decomposer::TaskDecomposer::get_breakdowns(&task_id)
}

#[derive(Serialize, Deserialize)]
struct OperationPackageUi {
    id: String,
    package_order: i32,
    package_type: String,
    subject: String,
    sub_topic: String,
    criterion: String,
    sub_criterion: String,
    accepted_truth: String,
    selected_best_alternative: String,
    operation_sequence: String,
    technology: String,
    impact_area: String,
    control_point: String,
    control_criteria: String,
    test_plan: String,
    rollback_plan: String,
    executor_role: String,
    correctness_guard_role: String,
    controller_role: String,
    independent_verifier_role: String,
    final_approver_role: String,
    status: String,
}

#[tauri::command]
fn get_swarm_allocations_cmd(
    task_id: String,
) -> Result<Vec<crate::core::ai_workflow_manager::SwarmAllocation>, String> {
    crate::core::ai_workflow_manager::AiWorkflowManager::list_allocations(&task_id)
}

#[tauri::command]
fn get_asker_motoru_status_cmd() -> Result<crate::core::asker_motoru_bridge::AskerMotoruBridgeReport, String> {
    Ok(crate::core::asker_motoru_bridge::AskerMotoruBridge::scan_status_files())
}

#[tauri::command]
fn sync_supabase_cmd(limit: Option<usize>) -> Result<crate::storage::supabase_sync::SupabaseSyncStatus, String> {
    crate::storage::supabase_sync::SupabaseSync::sync_recent_tasks(limit.unwrap_or(20))
}

#[tauri::command]
fn get_db_size_cmd() -> Result<u64, String> {
    crate::storage::log_rotation::LogRotation::db_size_bytes()
}

#[tauri::command]
fn get_operation_packages_cmd(task_id: String) -> Result<Vec<OperationPackageUi>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT CAST(id AS TEXT), package_order, package_type, subject, sub_topic, criterion, sub_criterion,
                accepted_truth, selected_best_alternative, operation_sequence, technology, impact_area,
                control_point, control_criteria, test_plan, rollback_plan, executor_role,
                correctness_guard_role, controller_role, independent_verifier_role, final_approver_role, status
         FROM operation_packages
         WHERE task_id = ?1
         ORDER BY package_order ASC",
    )
    .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![task_id], |row| {
            Ok(OperationPackageUi {
                id: row.get(0)?,
                package_order: row.get(1)?,
                package_type: row.get(2)?,
                subject: row.get(3)?,
                sub_topic: row.get(4)?,
                criterion: row.get(5)?,
                sub_criterion: row.get(6)?,
                accepted_truth: row.get(7)?,
                selected_best_alternative: row.get(8)?,
                operation_sequence: row.get(9)?,
                technology: row.get(10)?,
                impact_area: row.get(11)?,
                control_point: row.get(12)?,
                control_criteria: row.get(13)?,
                test_plan: row.get(14)?,
                rollback_plan: row.get(15)?,
                executor_role: row.get(16)?,
                correctness_guard_role: row.get(17)?,
                controller_role: row.get(18)?,
                independent_verifier_role: row.get(19)?,
                final_approver_role: row.get(20)?,
                status: row.get(21)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[derive(Serialize, Deserialize)]
struct LogItem {
    id: String,
    timestamp: String,
    level: String,
    message: String,
    gate_name: Option<String>,
}

#[tauri::command]
fn get_task_logs_cmd(task_id: String) -> Result<Vec<LogItem>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT CAST(id AS TEXT), timestamp, level, message, gate_name FROM execution_logs WHERE task_id = ?1 ORDER BY timestamp ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![task_id], |row| {
            Ok(LogItem {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                level: row.get(2)?,
                message: row.get(3)?,
                gate_name: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[derive(Serialize, Deserialize)]
struct DecisionUiNode {
    id: String,
    authorized_decider_id: String,
    status: String,
    selected_option: Option<String>,
    reason: Option<String>,
    level: i32,
    required_approval: Option<String>,
}

#[tauri::command]
fn get_decisions_cmd(task_id: String) -> Result<Vec<DecisionUiNode>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT CAST(id AS TEXT), authorized_decider_id, status, selected_option, reason, level, required_approval FROM decision_nodes WHERE task_id = ?1 ORDER BY level ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![task_id], |row| {
            let req_app: Option<i32> = row.get(6)?;
            Ok(DecisionUiNode {
                id: row.get(0)?,
                authorized_decider_id: row.get(1)?,
                status: row.get(2)?,
                selected_option: row.get(3)?,
                reason: row.get(4)?,
                level: row.get(5)?,
                required_approval: req_app.map(|v| v.to_string()),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[derive(Serialize, Deserialize)]
struct AlternativeUi {
    id: String,
    decision_node_id: String,
    title: String,
    description: String,
    accuracy_score: i32,
    safety_score: i32,
    dependency_score: i32,
    selected: i32,
    reason: Option<String>,
}

#[tauri::command]
fn get_alternatives_cmd(task_id: String) -> Result<Vec<AlternativeUi>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT CAST(a.id AS TEXT), CAST(a.decision_node_id AS TEXT), a.title, a.description, a.accuracy_score, a.safety_score, a.dependency_score, a.selected, a.reason 
         FROM alternatives a
         INNER JOIN decision_nodes d ON a.decision_node_id = d.id
         WHERE d.task_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![task_id], |row| {
            Ok(AlternativeUi {
                id: row.get(0)?,
                decision_node_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                accuracy_score: row.get(4)?,
                safety_score: row.get(5)?,
                dependency_score: row.get(6)?,
                selected: row.get(7)?,
                reason: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[derive(Serialize, Deserialize)]
struct ApprovalUi {
    id: String,
    action: String,
    risk_level: String,
    status: String,
    approver_id: Option<String>,
    approver_role: Option<String>,
    approval_source: Option<String>,
}

#[tauri::command]
fn get_approvals_cmd(task_id: String) -> Result<Vec<ApprovalUi>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT CAST(id AS TEXT), action, risk_level, status, approver_id, approver_role, approval_source FROM approvals WHERE task_id = ?1")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![task_id], |row| {
            Ok(ApprovalUi {
                id: row.get(0)?,
                action: row.get(1)?,
                risk_level: row.get(2)?,
                status: row.get(3)?,
                approver_id: row.get(4)?,
                approver_role: row.get(5)?,
                approval_source: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[derive(Serialize, Deserialize)]
struct CheckpointUi {
    id: String,
    checkpoint_type: String,
    status: String,
    result: String,
}

#[tauri::command]
fn get_checkpoints_cmd(task_id: String) -> Result<Vec<CheckpointUi>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT CAST(id AS TEXT), checkpoint_type, status, result FROM checkpoints WHERE task_id = ?1")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![task_id], |row| {
            Ok(CheckpointUi {
                id: row.get(0)?,
                checkpoint_type: row.get(1)?,
                status: row.get(2)?,
                result: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[derive(Serialize, Deserialize)]
struct TestUi {
    id: String,
    test_name: String,
    expected_result: String,
    actual_result: String,
    status: String,
}

#[tauri::command]
fn get_tests_cmd(task_id: String) -> Result<Vec<TestUi>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT CAST(id AS TEXT), test_name, expected_result, actual_result, status FROM tests WHERE task_id = ?1")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![task_id], |row| {
            Ok(TestUi {
                id: row.get(0)?,
                test_name: row.get(1)?,
                expected_result: row.get(2)?,
                actual_result: row.get(3)?,
                status: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[derive(Serialize, Deserialize)]
struct ReportUi {
    id: String,
    report_type: String,
    content: String,
}

#[tauri::command]
fn get_reports_cmd(task_id: String) -> Result<Vec<ReportUi>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT CAST(id AS TEXT), report_type, content FROM reports WHERE task_id = ?1")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![task_id], |row| {
            Ok(ReportUi {
                id: row.get(0)?,
                report_type: row.get(1)?,
                content: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = crate::storage::db::initialize_database() {
        eprintln!("Veritabani baslatilamadi: {}", e);
        return;
    }
    if let Err(e) = SystemValidator::validate_or_fail() {
        eprintln!("{}", e);
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            create_task_cmd,
            save_plan_cmd,
            execute_task_cmd,
            submit_approval_cmd,
            rollback_task_cmd,
            get_system_health_cmd,
            get_ai_provider_health_cmd,
            get_system_connector_health_cmd,
            get_tasks_cmd,
            get_task_logs_cmd,
            get_decisions_cmd,
            get_alternatives_cmd,
            get_approvals_cmd,
            get_checkpoints_cmd,
            get_tests_cmd,
            get_reports_cmd,
            get_task_breakdowns_cmd,
            get_operation_packages_cmd,
            get_swarm_allocations_cmd,
            get_asker_motoru_status_cmd,
            sync_supabase_cmd,
            get_db_size_cmd
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| eprintln!("Tauri uygulamasi calistirilamadi: {}", e));
}
