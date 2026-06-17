use crate::core::dependency_analyzer::DependencyAnalyzer;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicePersonaInfo {
    pub name: String,
    pub rank: String,
    pub role: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSynthesisConfig {
    pub lang: String,
    pub rate: f32,
    pub pitch: f32,
    pub volume: f32,
    pub prefer_female_voice: bool,
    #[serde(default)]
    pub voice_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicePersonaConfig {
    pub schema_version: u32,
    pub locale: String,
    pub persona: VoicePersonaInfo,
    pub synthesis: VoiceSynthesisConfig,
    pub templates: std::collections::HashMap<String, String>,
}

pub struct VoicePersonaRegistry;

impl VoicePersonaRegistry {
    pub fn load_config() -> Result<VoicePersonaConfig, String> {
        let path = DependencyAnalyzer::get_config_path("voice_persona.json")?;
        let data =
            fs::read_to_string(&path).map_err(|e| format!("voice_persona.json okunamadı: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("voice_persona.json geçersiz: {}", e))
    }
}
