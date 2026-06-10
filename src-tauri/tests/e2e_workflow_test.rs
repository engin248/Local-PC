//! E2E-style integration: intake → plan fields exist → execution gates module loads.

use lokal_bilgisayar_kontrol_paneli::core::ai_workflow_manager::AiWorkflowManager;
use lokal_bilgisayar_kontrol_paneli::core::asker_motoru_bridge::AskerMotoruBridge;
use lokal_bilgisayar_kontrol_paneli::core::dependency_analyzer::DependencyAnalyzer;
use lokal_bilgisayar_kontrol_paneli::storage::db::{initialize_database, Database};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[test]
fn e2e_swarm_allocate_and_asker_scan() {
    let _ = initialize_database();
    let platforms = AiWorkflowManager::parse_platforms_from_request(
        "[Analiz] [Ajanlar: CODEX,OAM] Test görevi",
    );
    assert!(platforms.contains(&"codex".to_string()));
    assert!(platforms.contains(&"open_agent_manager".to_string()));

    let report = AskerMotoruBridge::scan_status_files();
    assert!(!report.roots_checked.is_empty());
}

#[test]
fn e2e_default_department_swarm_allocates_eight_agents() {
    let _ = initialize_database();
    let task_id = format!("e2e_department_{}", Uuid::new_v4().simple());
    let allocations = AiWorkflowManager::allocate_task(
        &task_id,
        "Departman dağıtımı",
        "Varsayılan departman swarm görevi",
        "low",
        None,
    )
    .unwrap();

    let platforms: Vec<String> = allocations
        .iter()
        .map(|allocation| allocation.platform.clone())
        .collect();
    assert_eq!(
        platforms,
        vec![
            "lokal_bilgisayar_kontrol_paneli",
            "asker_motoru_komuta_paneli",
            "planlama_departmani",
            "egitim_departmani",
            "ar_ge_departmani",
            "bot_agent_uretim_departmani",
            "beceri_kutuphanesi",
            "test_raporlama",
        ]
    );
    for allocation in &allocations {
        let payload_path = Path::new(&allocation.payload_path);
        assert!(payload_path.exists());
        let _ = fs::remove_file(&allocation.payload_path);
        if let Some(inbox_dir) = payload_path.parent() {
            let _ = fs::remove_dir(inbox_dir);
            if let Some(platform_dir) = inbox_dir.parent() {
                let _ = fs::remove_dir(platform_dir);
            }
        }
    }
    let workflow_root = DependencyAnalyzer::get_project_root()
        .unwrap()
        .join("ai_workflow");
    let _ = fs::remove_file(
        workflow_root
            .join("tasks")
            .join(format!("{}.json", task_id)),
    );

    let db = Database::new();
    let conn = db.get_connection().unwrap();
    conn.execute("DELETE FROM ai_tasks WHERE id = ?1", [&task_id])
        .unwrap();
    conn.execute("DELETE FROM tasks WHERE id = ?1", [&task_id])
        .unwrap();
}
