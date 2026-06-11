pub mod ai_providers;
pub mod core;
pub mod storage;
pub mod system_connectors;

use crate::ai_providers::ai_provider_manager::AIProviderManager;
use crate::ai_providers::provider_base::AIProviderHealth;
use crate::core::approval_manager::submit_approval;
use crate::core::audit_logger::AuditLogger;
use crate::core::execution_engine::{execute_task_pipeline, ExecutionResult};
use crate::core::planning_gate::{save_plan, PlanningStandardInput};
use crate::core::rollback_manager::rollback_task;
use crate::core::system_validator::{SystemValidationIssue, SystemValidator};
use crate::core::task_intake::{create_task, Task, TaskIntakeRequest};
use crate::storage::db::init_db;
use crate::system_connectors::connector_base::SystemConnectorHealth;
use crate::system_connectors::system_connector_manager::SystemConnectorManager;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use std::panic::PanicHookInfo;
use std::sync::OnceLock;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[derive(Clone, Serialize, Deserialize)]
struct CriticalErrorEvent {
    command: String,
    source: String,
    message: String,
    correlation_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct OperationAuditRecord {
    id: String,
    actor: String,
    action: String,
    target_type: Option<String>,
    target_id: Option<String>,
    status: String,
    details: Option<String>,
    metadata_json: Option<String>,
    error_message: Option<String>,
    correlation_id: Option<String>,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
struct NewOperationAudit {
    actor: String,
    action: String,
    target_type: Option<String>,
    target_id: Option<String>,
    status: String,
    details: Option<String>,
    metadata_json: Option<String>,
    error_message: Option<String>,
    correlation_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RuntimeAlarmInput {
    source: String,
    message: String,
    timestamp: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AlarmCard {
    id: String,
    title: String,
    source_kind: String,
    health: String,
    runtime_only: bool,
    source_path: Option<String>,
    last_checked: String,
    error: Option<String>,
    details: String,
}

fn emit_critical_error(app: &AppHandle, command: &str, source: &str, message: &str, correlation_id: Option<String>) {
    let payload = CriticalErrorEvent {
        command: command.to_string(),
        source: source.to_string(),
        message: message.to_string(),
        correlation_id,
    };

    if let Err(err) = app.emit("critical-error", payload) {
        eprintln!(
            "Kritik hata olayı yayınlanamadı [command={command}]: {err}"
        );
    }
}

fn panic_to_message(info: &PanicHookInfo<'_>) -> String {
    let location = info
        .location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| "konum bilgisi yok".to_string());

    if let Some(payload) = info.payload().downcast_ref::<&str>() {
        format!("Rust panic [{location}]: {payload}")
    } else if let Some(payload) = info.payload().downcast_ref::<String>() {
        format!("Rust panic [{location}]: {payload}")
    } else {
        format!("Rust panic [{location}]: bilinmeyen hata")
    }
}

fn install_rust_panic_listener() {
    std::panic::set_hook(Box::new(|info| {
        let message = panic_to_message(info);
        if let Some(app) = APP_HANDLE.get() {
            emit_critical_error(app, "rust_panic", "Rust panic", &message, None);
        }
        eprintln!("{message}");
    }));
}

fn emit_if_error<T>(app: &AppHandle, command: &str, result: Result<T, String>) -> Result<T, String> {
    result.map_err(|error| {
        emit_critical_error(app, command, command, &error, None);
        error
    })
}

// UI'dan tetiklenecek Tauri komutları

#[tauri::command]
fn create_task_cmd(app: AppHandle, title: String, user_request: String) -> Result<Task, String> {
    emit_if_error(
        &app,
        "create_task_cmd",
        create_task(TaskIntakeRequest {
            title,
            user_request,
        }),
    )
}

#[tauri::command]
fn save_plan_cmd(app: AppHandle, task_id: String, plan: PlanningStandardInput) -> Result<(), String> {
    emit_if_error(&app, "save_plan_cmd", save_plan(&task_id, plan))
}

#[tauri::command]
fn execute_task_cmd(app: AppHandle, task_id: String) -> Result<ExecutionResult, String> {
    emit_if_error(&app, "execute_task_cmd", execute_task_pipeline(&task_id))
}

#[tauri::command]
fn submit_approval_cmd(
    app: AppHandle,
    approval_id: String,
    approve: bool,
    user_note: Option<String>,
    approver_id: Option<String>,
    approver_role: Option<String>,
) -> Result<(), String> {
    emit_if_error(
        &app,
        "submit_approval_cmd",
        submit_approval(
            &approval_id,
            approve,
            user_note.as_deref(),
            approver_id.as_deref(),
            approver_role.as_deref(),
            Some("ui"),
        ),
    )
}

#[tauri::command]
fn rollback_task_cmd(app: AppHandle, task_id: String) -> Result<bool, String> {
    emit_if_error(&app, "rollback_task_cmd", rollback_task(&task_id))
}

#[tauri::command]
fn append_operation_audit_cmd(app: AppHandle, input: NewOperationAudit) -> Result<(), String> {
    emit_if_error(
        &app,
        "append_operation_audit_cmd",
        AuditLogger::log_operation_event(
            &input.actor,
            &input.action,
            &input.status,
            input.target_type.as_deref(),
            input.target_id.as_deref(),
            input.details.as_deref(),
            input.metadata_json.as_deref(),
            input.error_message.as_deref(),
            input.correlation_id.as_deref(),
        ),
    )
}

#[tauri::command]
fn get_operation_audit_logs_cmd(
    app: AppHandle,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<OperationAuditRecord>, String> {
    emit_if_error(
        &app,
        "get_operation_audit_logs_cmd",
        (|| -> Result<Vec<OperationAuditRecord>, String> {
            let conn = init_db().map_err(|e| e.to_string())?;
            let limit = limit.unwrap_or(40);
            let offset = offset.unwrap_or(0);

            let mut stmt = conn
                .prepare(
                    "SELECT
                        id,
                        actor,
                        action,
                        target_type,
                        target_id,
                        status,
                        details,
                        metadata_json,
                        error_message,
                        correlation_id,
                        created_at
                    FROM operation_audit_events
                    ORDER BY created_at DESC
                    LIMIT ?1
                    OFFSET ?2",
                )
                .map_err(|e| e.to_string())?;

            let rows = stmt
                .query_map(rusqlite::params![limit, offset], |row| {
                    Ok(OperationAuditRecord {
                        id: row.get(0)?,
                        actor: row.get(1)?,
                        action: row.get(2)?,
                        target_type: row.get(3)?,
                        target_id: row.get(4)?,
                        status: row.get(5)?,
                        details: row.get(6)?,
                        metadata_json: row.get(7)?,
                        error_message: row.get(8)?,
                        correlation_id: row.get(9)?,
                        created_at: row.get(10)?,
                    })
                })
                .map_err(|e| e.to_string())?;

            let mut list = Vec::new();
            for item in rows {
                list.push(item.map_err(|e| e.to_string())?);
            }
            Ok(list)
        })(),
    )
}

#[tauri::command]
fn get_system_health_cmd(app: AppHandle) -> Result<Vec<SystemValidationIssue>, String> {
    emit_if_error(&app, "get_system_health_cmd", SystemValidator::validate())
}

#[tauri::command]
fn get_ai_provider_health_cmd(app: AppHandle, write_audit: Option<bool>) -> Result<Vec<AIProviderHealth>, String> {
    emit_if_error(
        &app,
        "get_ai_provider_health_cmd",
        AIProviderManager::health_check_all(write_audit.unwrap_or(false)),
    )
}

#[tauri::command]
fn get_system_connector_health_cmd(
    app: AppHandle,
    write_audit: Option<bool>,
) -> Result<Vec<SystemConnectorHealth>, String> {
    emit_if_error(
        &app,
        "get_system_connector_health_cmd",
        SystemConnectorManager::health_check_all(write_audit.unwrap_or(false)),
    )
}

// Veritabanı sorgulama yardımcı komutları

#[tauri::command]
fn get_tasks_cmd(app: AppHandle) -> Result<Vec<Task>, String> {
    emit_if_error(
        &app,
        "get_tasks_cmd",
        (|| -> Result<Vec<Task>, String> {
            let conn = init_db().map_err(|e| e.to_string())?;
            let mut stmt = conn
                .prepare("SELECT CAST(id AS TEXT), title, user_request, status, planning_status, execution_status, current_gate, last_valid_state_id, risk_level, approval_status, created_at FROM tasks ORDER BY created_at DESC")
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
        })(),
    )
}

#[tauri::command]
fn get_task_breakdowns_cmd(
    app: AppHandle,
    task_id: String,
) -> Result<Vec<crate::core::task_decomposer::TaskBreakdown>, String> {
    emit_if_error(
        &app,
        "get_task_breakdowns_cmd",
        crate::core::task_decomposer::TaskDecomposer::get_breakdowns(&task_id),
    )
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
    app: AppHandle,
    task_id: String,
) -> Result<Vec<crate::core::ai_workflow_manager::SwarmAllocation>, String> {
    emit_if_error(
        &app,
        "get_swarm_allocations_cmd",
        crate::core::ai_workflow_manager::AiWorkflowManager::list_allocations(&task_id),
    )
}

#[tauri::command]
fn get_asker_motoru_status_cmd(
    app: AppHandle,
) -> Result<crate::core::asker_motoru_bridge::AskerMotoruBridgeReport, String> {
    emit_if_error(
        &app,
        "get_asker_motoru_status_cmd",
        Ok(crate::core::asker_motoru_bridge::AskerMotoruBridge::scan_status_files()),
    )
}

#[tauri::command]
fn submit_command_sentence_cmd(
    app: AppHandle,
    sentence: String,
    operator_id: Option<String>,
) -> Result<crate::core::command_orchestrator::CommandSentenceResult, String> {
    emit_if_error(
        &app,
        "submit_command_sentence_cmd",
        crate::core::command_orchestrator::CommandOrchestrator::submit_sentence(
            &app,
            &sentence,
            operator_id.as_deref(),
        ),
    )
}

#[tauri::command]
fn get_live_command_feed_cmd(
    app: AppHandle,
    limit: Option<usize>,
) -> Result<Vec<crate::core::command_orchestrator::CommandFeedItem>, String> {
    emit_if_error(
        &app,
        "get_live_command_feed_cmd",
        crate::core::command_orchestrator::CommandOrchestrator::get_feed(limit.unwrap_or(50)),
    )
}

#[tauri::command]
fn get_alarm_codes_cmd(
    app: AppHandle,
) -> Result<Vec<crate::core::alarm_registry::AlarmCodeDefinition>, String> {
    emit_if_error(
        &app,
        "get_alarm_codes_cmd",
        crate::core::alarm_registry::AlarmRegistry::list_codes(),
    )
}

#[tauri::command]
fn raise_alarm_code_cmd(
    app: AppHandle,
    code: String,
    source: String,
    message: String,
) -> Result<crate::core::alarm_registry::AlarmEventRecord, String> {
    emit_if_error(
        &app,
        "raise_alarm_code_cmd",
        (|| -> Result<crate::core::alarm_registry::AlarmEventRecord, String> {
            let event = crate::core::alarm_registry::AlarmRegistry::raise_code(
                &code,
                &source,
                &message,
                "manual",
            )?;
            crate::core::live_event_bus::LiveEventBus::alarm_code(
                &app,
                &code,
                &message,
                event.speak_text.clone(),
            );
            Ok(event)
        })(),
    )
}

#[tauri::command]
fn resolve_alarm_code_cmd(app: AppHandle, alarm_id: String) -> Result<bool, String> {
    emit_if_error(
        &app,
        "resolve_alarm_code_cmd",
        crate::core::alarm_registry::AlarmRegistry::resolve_code(&alarm_id),
    )
}

#[tauri::command]
fn scan_algorithm_health_cmd(
    app: AppHandle,
    task_id: Option<String>,
) -> Result<crate::core::alarm_registry::HealthScanResult, String> {
    emit_if_error(
        &app,
        "scan_algorithm_health_cmd",
        (|| -> Result<crate::core::alarm_registry::HealthScanResult, String> {
            let scan = crate::core::alarm_registry::AlarmRegistry::scan_algorithm_health(
                task_id.as_deref(),
            )?;
            for event in &scan.events {
                if let Some(code) = &event.alarm_code {
                    crate::core::live_event_bus::LiveEventBus::alarm_code(
                        &app,
                        code,
                        &event.message,
                        event.speak_text.clone(),
                    );
                }
            }
            Ok(scan)
        })(),
    )
}

#[tauri::command]
fn get_active_alarm_events_cmd(
    app: AppHandle,
    limit: Option<usize>,
) -> Result<Vec<crate::core::alarm_registry::AlarmEventRecord>, String> {
    emit_if_error(
        &app,
        "get_active_alarm_events_cmd",
        crate::core::alarm_registry::AlarmRegistry::list_active_events(limit.unwrap_or(20)),
    )
}

#[tauri::command]
fn get_asker_motoru_live_status_cmd(
    app: AppHandle,
) -> Result<crate::core::asker_motoru_live_bridge::LiveStatusSnapshot, String> {
    emit_if_error(
        &app,
        "get_asker_motoru_live_status_cmd",
        Ok(crate::core::asker_motoru_live_bridge::AskerMotoruLiveBridge::fetch_live_status()),
    )
}

#[tauri::command]
fn post_asker_motoru_command_cmd(app: AppHandle, sentence: String) -> Result<String, String> {
    emit_if_error(
        &app,
        "post_asker_motoru_command_cmd",
        crate::core::asker_motoru_live_bridge::AskerMotoruLiveBridge::post_command(&sentence),
    )
}

#[tauri::command]
fn get_pinokio_health_cmd(app: AppHandle) -> Result<(String, Option<String>), String> {
    emit_if_error(
        &app,
        "get_pinokio_health_cmd",
        Ok(crate::system_connectors::pinokio_connector::PinokioConnector::health_check()),
    )
}

#[tauri::command]
fn get_pinokio_app_status_cmd(
    app: AppHandle,
    app_id: String,
) -> Result<crate::system_connectors::pinokio_connector::PinokioAppStatus, String> {
    emit_if_error(
        &app,
        "get_pinokio_app_status_cmd",
        Ok(crate::system_connectors::pinokio_connector::PinokioConnector::status(&app_id)),
    )
}

#[tauri::command]
fn run_pinokio_app_cmd(app: AppHandle, app_id: String) -> Result<String, String> {
    emit_if_error(
        &app,
        "run_pinokio_app_cmd",
        crate::system_connectors::pinokio_connector::PinokioConnector::run_app(&app_id),
    )
}

#[tauri::command]
fn get_alarm_cards_cmd(
    app: AppHandle,
    runtime_alarms: Option<Vec<RuntimeAlarmInput>>,
) -> Result<Vec<AlarmCard>, String> {
    emit_if_error(
        &app,
        "get_alarm_cards_cmd",
        (|| -> Result<Vec<AlarmCard>, String> {
            let now = chrono::Utc::now().to_rfc3339();
            let mut cards = Vec::new();

            for (index, alarm) in runtime_alarms.unwrap_or_default().into_iter().enumerate() {
                cards.push(AlarmCard {
                    id: format!("runtime-{}", index),
                    title: alarm.source,
                    source_kind: "unavailable".to_string(),
                    health: "runtime_only".to_string(),
                    runtime_only: true,
                    source_path: None,
                    last_checked: alarm.timestamp.unwrap_or_else(|| now.clone()),
                    error: Some("runtime only".to_string()),
                    details: alarm.message,
                });
            }

            match crate::core::asker_motoru_bridge::AskerMotoruBridge::read_alarm_status_file() {
                Some((path, content)) => cards.push(AlarmCard {
                    id: "asker-motoru-json".to_string(),
                    title: "SISTEM_ALARM_DURUMU.json".to_string(),
                    source_kind: "json".to_string(),
                    health: "available".to_string(),
                    runtime_only: false,
                    source_path: Some(path),
                    last_checked: now.clone(),
                    error: None,
                    details: content.chars().take(500).collect(),
                }),
                None => cards.push(AlarmCard {
                    id: "asker-motoru-json".to_string(),
                    title: "SISTEM_ALARM_DURUMU.json".to_string(),
                    source_kind: "unavailable".to_string(),
                    health: "unavailable".to_string(),
                    runtime_only: false,
                    source_path: None,
                    last_checked: now.clone(),
                    error: Some("SISTEM_ALARM_DURUMU.json bağlı değil.".to_string()),
                    details: "bağlı değil".to_string(),
                }),
            }

            let conn = init_db().map_err(|e| e.to_string())?;
            let persistent_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM system_alarm_events WHERE resolved_at IS NULL",
                    [],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            let last_persistent: Option<(String, String, String)> = conn
                .query_row(
                    "SELECT source, message, persisted_at FROM system_alarm_events WHERE resolved_at IS NULL ORDER BY persisted_at DESC LIMIT 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            cards.push(AlarmCard {
                id: "sqlite-persistent-alarms".to_string(),
                title: "SQLite kalıcı alarm kaydı".to_string(),
                source_kind: "sqlite".to_string(),
                health: if persistent_count > 0 { "available" } else { "available_empty" }.to_string(),
                runtime_only: false,
                source_path: Some("storage/app.db:system_alarm_events".to_string()),
                last_checked: now,
                error: None,
                details: match last_persistent {
                    Some((source, message, persisted_at)) => format!(
                        "{} aktif kalıcı alarm. Son: {} / {} / {}",
                        persistent_count, source, message, persisted_at
                    ),
                    None => "Aktif kalıcı alarm yok.".to_string(),
                },
            });

            Ok(cards)
        })(),
    )
}

#[tauri::command]
fn sync_supabase_cmd(
    app: AppHandle,
    limit: Option<usize>,
) -> Result<crate::storage::supabase_sync::SupabaseSyncStatus, String> {
    emit_if_error(
        &app,
        "sync_supabase_cmd",
        crate::storage::supabase_sync::SupabaseSync::sync_recent_tasks(limit.unwrap_or(20)),
    )
}

#[tauri::command]
fn get_db_size_cmd(app: AppHandle) -> Result<u64, String> {
    emit_if_error(
        &app,
        "get_db_size_cmd",
        crate::storage::log_rotation::LogRotation::db_size_bytes(),
    )
}

#[tauri::command]
fn get_operation_packages_cmd(app: AppHandle, task_id: String) -> Result<Vec<OperationPackageUi>, String> {
    emit_if_error(
        &app,
        "get_operation_packages_cmd",
        (|| -> Result<Vec<OperationPackageUi>, String> {
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
        })(),
    )
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
fn get_task_logs_cmd(app: AppHandle, task_id: String) -> Result<Vec<LogItem>, String> {
    emit_if_error(
        &app,
        "get_task_logs_cmd",
        (|| -> Result<Vec<LogItem>, String> {
            let conn = init_db().map_err(|e| e.to_string())?;
            let mut stmt = conn
                .prepare("SELECT CAST(id AS TEXT), timestamp, level, message, gate_name FROM execution_logs WHERE task_id = ?1 ORDER BY timestamp ASC")
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
        })(),
    )
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
fn get_decisions_cmd(app: AppHandle, task_id: String) -> Result<Vec<DecisionUiNode>, String> {
    emit_if_error(
        &app,
        "get_decisions_cmd",
        (|| -> Result<Vec<DecisionUiNode>, String> {
            let conn = init_db().map_err(|e| e.to_string())?;
            let mut stmt = conn
                .prepare("SELECT CAST(id AS TEXT), authorized_decider_id, status, selected_option, reason, level, required_approval FROM decision_nodes WHERE task_id = ?1 ORDER BY level ASC")
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
        })(),
    )
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
fn get_alternatives_cmd(app: AppHandle, task_id: String) -> Result<Vec<AlternativeUi>, String> {
    emit_if_error(
        &app,
        "get_alternatives_cmd",
        (|| -> Result<Vec<AlternativeUi>, String> {
            let conn = init_db().map_err(|e| e.to_string())?;
            let mut stmt = conn
                .prepare(
                    "SELECT CAST(a.id AS TEXT), CAST(a.decision_node_id AS TEXT), a.title, a.description, a.accuracy_score, a.safety_score, a.dependency_score, a.selected, a.reason 
         FROM alternatives a
         INNER JOIN decision_nodes d ON a.decision_node_id = d.id
         WHERE d.task_id = ?1",
                )
                .map_err(|e| e.to_string())?;

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
        })(),
    )
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
fn get_approvals_cmd(app: AppHandle, task_id: String) -> Result<Vec<ApprovalUi>, String> {
    emit_if_error(
        &app,
        "get_approvals_cmd",
        (|| -> Result<Vec<ApprovalUi>, String> {
            let conn = init_db().map_err(|e| e.to_string())?;
            let mut stmt = conn
                .prepare("SELECT CAST(id AS TEXT), action, risk_level, status, approver_id, approver_role, approval_source FROM approvals WHERE task_id = ?1")
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
        })(),
    )
}

#[derive(Serialize, Deserialize)]
struct CheckpointUi {
    id: String,
    checkpoint_type: String,
    status: String,
    result: String,
}

#[tauri::command]
fn get_checkpoints_cmd(app: AppHandle, task_id: String) -> Result<Vec<CheckpointUi>, String> {
    emit_if_error(
        &app,
        "get_checkpoints_cmd",
        (|| -> Result<Vec<CheckpointUi>, String> {
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
        })(),
    )
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
fn get_tests_cmd(app: AppHandle, task_id: String) -> Result<Vec<TestUi>, String> {
    emit_if_error(
        &app,
        "get_tests_cmd",
        (|| -> Result<Vec<TestUi>, String> {
            let conn = init_db().map_err(|e| e.to_string())?;
            let mut stmt = conn
                .prepare("SELECT CAST(id AS TEXT), test_name, expected_result, actual_result, status FROM tests WHERE task_id = ?1")
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
        })(),
    )
}

#[derive(Serialize, Deserialize)]
struct ReportUi {
    id: String,
    report_type: String,
    content: String,
}

#[tauri::command]
fn get_reports_cmd(app: AppHandle, task_id: String) -> Result<Vec<ReportUi>, String> {
    emit_if_error(
        &app,
        "get_reports_cmd",
        (|| -> Result<Vec<ReportUi>, String> {
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
        })(),
    )
}

#[derive(Serialize, Deserialize)]
pub struct SkillSummary {
    pub total_count: i64,
    pub python_count: i64,
    pub javascript_count: i64,
}

#[derive(Serialize, Deserialize)]
pub struct SkillItemUi {
    pub skill_id: String,
    pub name: String,
    pub language: String,
    pub category: String,
    pub status: String,
    pub created_at: String,
    pub description: String,
}

#[tauri::command]
fn get_skill_library_summary_cmd(app: AppHandle) -> Result<SkillSummary, String> {
    emit_if_error(
        &app,
        "get_skill_library_summary_cmd",
        (|| -> Result<SkillSummary, String> {
            let db_path = "C:\\Users\\Esisya\\Desktop\\Lokal Kütüphane\\database\\skill_library.sqlite";
            let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;
            
            let total_count: i64 = conn.query_row("SELECT COUNT(*) FROM skills", [], |row| row.get(0)).unwrap_or(0);
            let python_count: i64 = conn.query_row("SELECT COUNT(*) FROM skills WHERE language = 'python'", [], |row| row.get(0)).unwrap_or(0);
            let javascript_count: i64 = conn.query_row("SELECT COUNT(*) FROM skills WHERE language = 'javascript'", [], |row| row.get(0)).unwrap_or(0);
            
            Ok(SkillSummary {
                total_count,
                python_count,
                javascript_count,
            })
        })(),
    )
}

#[tauri::command]
fn search_skill_library_cmd(app: AppHandle, query: String, category: Option<String>) -> Result<Vec<SkillItemUi>, String> {
    emit_if_error(
        &app,
        "search_skill_library_cmd",
        (|| -> Result<Vec<SkillItemUi>, String> {
            let db_path = "C:\\Users\\Esisya\\Desktop\\Lokal Kütüphane\\database\\skill_library.sqlite";
            let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;
            
            let mut sql = "SELECT skill_id, name, language, category, status, created_at, description FROM skills WHERE 1=1".to_string();
            let mut params: Vec<String> = Vec::new();
            
            if !query.is_empty() {
                sql.push_str(" AND (name LIKE ?1 OR skill_id LIKE ?1 OR description LIKE ?1)");
                params.push(format!("%{}%", query));
            }
            
            if let Some(cat) = &category {
                if !cat.is_empty() {
                    let idx = params.len() + 1;
                    sql.push_str(&format!(" AND category = ?{}", idx));
                    params.push(cat.clone());
                }
            }
            
            sql.push_str(" ORDER BY created_at DESC LIMIT 100");
            
            let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
            let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
                Ok(SkillItemUi {
                    skill_id: row.get(0)?,
                    name: row.get(1)?,
                    language: row.get(2)?,
                    category: row.get(3)?,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                    description: row.get(6)?,
                })
            }).map_err(|e| e.to_string())?;
            
            let mut list = Vec::new();
            for item in rows {
                list.push(item.map_err(|e| e.to_string())?);
            }
            Ok(list)
        })(),
    )
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
        .setup(|app| {
            let _ = APP_HANDLE.set(app.handle().clone());
            install_rust_panic_listener();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_task_cmd,
            save_plan_cmd,
            execute_task_cmd,
            submit_approval_cmd,
            rollback_task_cmd,
            append_operation_audit_cmd,
            get_operation_audit_logs_cmd,
            get_system_health_cmd,
            get_ai_provider_health_cmd,
            get_system_connector_health_cmd,
            get_alarm_cards_cmd,
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
            submit_command_sentence_cmd,
            get_live_command_feed_cmd,
            get_alarm_codes_cmd,
            raise_alarm_code_cmd,
            resolve_alarm_code_cmd,
            scan_algorithm_health_cmd,
            get_active_alarm_events_cmd,
            get_asker_motoru_live_status_cmd,
            post_asker_motoru_command_cmd,
            get_pinokio_health_cmd,
            get_pinokio_app_status_cmd,
            run_pinokio_app_cmd,
            sync_supabase_cmd,
            get_db_size_cmd,
            get_skill_library_summary_cmd,
            search_skill_library_cmd
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| eprintln!("Tauri uygulamasi calistirilamadi: {}", e));
}
