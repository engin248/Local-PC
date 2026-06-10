//! E2E-style integration: intake → plan fields exist → execution gates module loads.

use lokal_bilgisayar_kontrol_paneli::core::ai_workflow_manager::AiWorkflowManager;
use lokal_bilgisayar_kontrol_paneli::core::asker_motoru_bridge::AskerMotoruBridge;
use lokal_bilgisayar_kontrol_paneli::storage::db::initialize_database;

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
    assert_eq!(report.module_summary.total_modules, 314);
    assert_eq!(report.module_summary.total_specialty_capabilities, 3140);
    assert_eq!(report.modules.len(), 314);
    assert!(report
        .modules
        .iter()
        .all(|module| module.specialty_capabilities.len() == 10));
    assert!(report
        .modules
        .iter()
        .any(|module| module.module_name == "314_Cross_Reality_Simulation_Engine"));
}
