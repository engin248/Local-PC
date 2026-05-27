pub mod storage;
pub mod core;
pub mod ai_providers;
pub mod system_connectors;

use serde::{Deserialize, Serialize};
use crate::core::task_intake::{create_task, TaskIntakeRequest, Task};
use crate::core::planning_gate::{save_plan, PlanningStandardInput};
use crate::core::execution_engine::{execute_task_pipeline, ExecutionResult};
use crate::core::approval_manager::submit_approval;
use crate::core::rollback_manager::rollback_task;
use crate::storage::db::init_db;

// UI'dan tetiklenecek Tauri komutları

#[tauri::command]
fn create_task_cmd(title: String, user_request: String) -> Result<Task, String> {
    create_task(TaskIntakeRequest { title, user_request })
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

// Veritabanı sorgulama yardımcı komutları

#[tauri::command]
fn get_tasks_cmd() -> Result<Vec<Task>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, title, user_request, status, planning_status, execution_status, current_gate, last_valid_state_id, risk_level, approval_status, created_at FROM tasks ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
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
    }).map_err(|e| e.to_string())?;

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
    let mut stmt = conn.prepare("SELECT id, timestamp, level, message, gate_name FROM execution_logs WHERE task_id = ?1 ORDER BY timestamp ASC")
        .map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map(rusqlite::params![task_id], |row| {
        Ok(LogItem {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            level: row.get(2)?,
            message: row.get(3)?,
            gate_name: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

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
    let mut stmt = conn.prepare("SELECT id, authorized_decider_id, status, selected_option, reason, level, required_approval FROM decision_nodes WHERE task_id = ?1 ORDER BY level ASC")
        .map_err(|e| e.to_string())?;
        
    let rows = stmt.query_map(rusqlite::params![task_id], |row| {
        Ok(DecisionUiNode {
            id: row.get(0)?,
            authorized_decider_id: row.get(1)?,
            status: row.get(2)?,
            selected_option: row.get(3)?,
            reason: row.get(4)?,
            level: row.get(5)?,
            required_approval: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

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
        "SELECT a.id, a.decision_node_id, a.title, a.description, a.accuracy_score, a.safety_score, a.dependency_score, a.selected, a.reason 
         FROM alternatives a
         INNER JOIN decision_nodes d ON a.decision_node_id = d.id
         WHERE d.task_id = ?1"
    ).map_err(|e| e.to_string())?;
        
    let rows = stmt.query_map(rusqlite::params![task_id], |row| {
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
    }).map_err(|e| e.to_string())?;

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
    let mut stmt = conn.prepare("SELECT id, action, risk_level, status, approver_id, approver_role, approval_source FROM approvals WHERE task_id = ?1")
        .map_err(|e| e.to_string())?;
        
    let rows = stmt.query_map(rusqlite::params![task_id], |row| {
        Ok(ApprovalUi {
            id: row.get(0)?,
            action: row.get(1)?,
            risk_level: row.get(2)?,
            status: row.get(3)?,
            approver_id: row.get(4)?,
            approver_role: row.get(5)?,
            approval_source: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

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
    let mut stmt = conn.prepare("SELECT id, checkpoint_type, status, result FROM checkpoints WHERE task_id = ?1")
        .map_err(|e| e.to_string())?;
        
    let rows = stmt.query_map(rusqlite::params![task_id], |row| {
        Ok(CheckpointUi {
            id: row.get(0)?,
            checkpoint_type: row.get(1)?,
            status: row.get(2)?,
            result: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

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
    let mut stmt = conn.prepare("SELECT id, test_name, expected_result, actual_result, status FROM tests WHERE task_id = ?1")
        .map_err(|e| e.to_string())?;
        
    let rows = stmt.query_map(rusqlite::params![task_id], |row| {
        Ok(TestUi {
            id: row.get(0)?,
            test_name: row.get(1)?,
            expected_result: row.get(2)?,
            actual_result: row.get(3)?,
            status: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

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
    let mut stmt = conn.prepare("SELECT id, report_type, content FROM reports WHERE task_id = ?1")
        .map_err(|e| e.to_string())?;
        
    let rows = stmt.query_map(rusqlite::params![task_id], |row| {
        Ok(ReportUi {
            id: row.get(0)?,
            report_type: row.get(1)?,
            content: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;

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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            create_task_cmd,
            save_plan_cmd,
            execute_task_cmd,
            submit_approval_cmd,
            rollback_task_cmd,
            get_tasks_cmd,
            get_task_logs_cmd,
            get_decisions_cmd,
            get_alternatives_cmd,
            get_approvals_cmd,
            get_checkpoints_cmd,
            get_tests_cmd,
            get_reports_cmd
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| eprintln!("Tauri uygulamasi calistirilamadi: {}", e));
}
