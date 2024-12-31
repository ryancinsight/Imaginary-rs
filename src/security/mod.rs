use serde::Deserialize;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use anyhow::Result;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rand::Rng;

#[derive(Debug, Deserialize, Default)]
pub struct SecurityConfig {
    #[serde(default = "default_key")]
    pub key: Option<String>,
    #[serde(default = "default_salt")]
    pub salt: Option<String>,
    #[serde(default = "default_allowed_origins")]
    pub allowed_origins: Vec<String>,
}

// Default implementations
fn default_key() -> Option<String> {
    None
}

fn default_salt() -> Option<String> {
    None
}

fn default_allowed_origins() -> Vec<String> {
    vec!["*".to_string()]
}

impl SecurityConfig {
    pub fn generate_api_key(&mut self) -> String {
        let api_key: [u8; 32] = rand::thread_rng().gen();
        let api_key_str = STANDARD.encode(&api_key);
        self.key = Some(api_key_str.clone());
        api_key_str
    }

    pub fn validate_signature(&self, data: &[u8], signature: &str) -> Result<bool> {
        let key = match &self.key {
            Some(k) if !k.is_empty() => k,
            _ => return Ok(false),
        };

        let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())?;
        mac.update(data);

        let decoded_sig = STANDARD.decode(signature)?;
        Ok(mac.verify_slice(&decoded_sig).is_ok())
    }

    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.iter().any(|allowed| {
            allowed == "*" || allowed == origin
        })
    }

    pub fn generate_signature(&self, data: &[u8]) -> Result<String> {
        let key = self.key.as_ref().ok_or_else(|| anyhow::anyhow!("No security key configured"))?;
        let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())?;
        mac.update(data);
        
        let result = mac.finalize();
        Ok(STANDARD.encode(result.into_bytes()))
    }

    pub fn is_secure(&self) -> bool {
        self.key.is_some() && self.key.as_ref().map_or(false, |k| !k.is_empty())
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SecurityConfig::default();
        assert!(config.key.is_none());
        assert!(config.salt.is_none());
        assert_eq!(config.allowed_origins, vec!["*"]);
    }

    #[test]
    fn test_origin_validation() {
        let mut config = SecurityConfig::default();
        assert!(config.is_origin_allowed("http://localhost:3000"));
        
        config.allowed_origins = vec!["http://localhost:3000".to_string()];
        assert!(config.is_origin_allowed("http://localhost:3000"));
        assert!(!config.is_origin_allowed("http://other.com"));
    }

    #[test]
    fn test_signature_validation() {
        let mut config = SecurityConfig::default();
        config.key = Some("test-key".to_string());
        
        let data = b"test data";
        let signature = config.generate_signature(data).unwrap();
        assert!(config.validate_signature(data, &signature).unwrap());
    }
}