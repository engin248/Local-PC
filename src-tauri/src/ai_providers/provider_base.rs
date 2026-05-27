pub trait AIProvider {
    fn query(&self, prompt: &str) -> Result<String, String>;
    fn get_name(&self) -> &str;
    fn get_model(&self) -> &str;
}
