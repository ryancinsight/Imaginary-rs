use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct SecurityConfig {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub salt: Option<String>,
    #[serde(default = "default_allowed_origins")]
    pub allowed_origins: Vec<String>,
}

fn default_allowed_origins() -> Vec<String> {
    vec!["*".to_string()]
}

// Add any security-related functions here
pub fn validate_key(key: &str) -> bool {
    !key.is_empty() && key.len() >= 32
}