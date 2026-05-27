use crate::ai_providers::provider_base::AIProvider;

pub struct CustomProvider {
    pub name: String,
    pub model: String,
}

impl AIProvider for CustomProvider {
    fn query(&self, _prompt: &str) -> Result<String, String> {
        Err("Custom Provider (örn: Verdent) sadece Canlı modda etkindir. İlk Sprint'te devre dışıdır.".to_string())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_model(&self) -> &str {
        &self.model
    }
}
