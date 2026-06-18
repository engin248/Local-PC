use lokal_bilgisayar_kontrol_paneli::ai_providers::ai_provider_manager::AIProviderManager;
use lokal_bilgisayar_kontrol_paneli::core::env_loader::EnvLoader;
use std::fs;

#[test]
fn loads_keys_from_real_dotenv_file() {
    // Proje kökü = /workspace (package.json burada). .env oraya yazılır.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let env_path = root.join(".env");

    // Mevcut bir .env varsa dokunma (CI/komutan ortamı korunur).
    let preexisting = env_path.exists();
    if !preexisting {
        fs::write(
            &env_path,
            "GEMINI_API_KEY=e2e-gemini-secret\nCURSOR_API_KEY=e2e-cursor-secret\n",
        )
        .unwrap();
    }

    std::env::remove_var("GEMINI_API_KEY");
    std::env::remove_var("CURSOR_API_KEY");

    let report = EnvLoader::load_local_secrets();
    assert!(!report.files_read.is_empty(), "rapor: {:?}", report);

    if !preexisting {
        assert_eq!(
            std::env::var("GEMINI_API_KEY").unwrap(),
            "e2e-gemini-secret"
        );
        assert_eq!(
            std::env::var("CURSOR_API_KEY").unwrap(),
            "e2e-cursor-secret"
        );

        // Gerçek provider kodu yolu: yüklenen anahtar health-check tarafından görülmeli.
        let health = AIProviderManager::health_check_all(false).unwrap();
        let gemini = health
            .iter()
            .find(|p| p.id == "gemini")
            .expect("gemini provider config bulunamadı");
        println!(
            "GEMINI health -> enabled={} api_key_status={} status={}",
            gemini.enabled, gemini.api_key_status, gemini.status
        );
        assert!(gemini.enabled, "gemini etkin olmalı");
        assert_eq!(gemini.api_key_status, "present");

        fs::remove_file(&env_path).unwrap();
        std::env::remove_var("GEMINI_API_KEY");
        std::env::remove_var("CURSOR_API_KEY");
    }
}
