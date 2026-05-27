use crate::ai_providers::provider_base::AIProvider;

pub struct OpenAICompatibleProvider {
    pub name: String,
    pub base_url: String,
    pub model: String,
}

impl AIProvider for OpenAICompatibleProvider {
    fn query(&self, _prompt: &str) -> Result<String, String> {
        Err("OpenAI Compatible Provider sadece Canlı modda etkindir. İlk Sprint'te devre dışıdır.".to_string())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_model(&self) -> &str {
        &self.model
    }
}
