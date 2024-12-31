use serde::Deserialize;
use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use rand::{Rng, distributions::Alphanumeric};
type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize, Default, Clone)]
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
    Some("default_key".to_string())
}

fn default_salt() -> Option<String> {
    Some("default_salt".to_string())
}

fn default_allowed_origins() -> Vec<String> {
    vec!["*".to_string(), "http://localhost".to_string()]
}

impl SecurityConfig {
    fn prepare_key(&self) -> Vec<u8> {
        let key = self.key.clone().unwrap_or_else(|| "default_key".to_string());
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hasher.finalize().to_vec()
    }

    pub fn generate_api_key(&mut self) -> String {
        if self.key.is_none() || self.key.as_ref().unwrap().is_empty() {
            let generated_key: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();
            self.key = Some(generated_key);
        }
        self.key.clone().unwrap()
    }

    pub fn set_api_key(&mut self, key: String) {
        self.key = Some(key);
    }

    pub fn validate_signature(&self, data: &[u8], signature: &str) -> Result<bool> {
        let key = self.prepare_key();
        let mut mac = HmacSha256::new_from_slice(&key)?;
        mac.update(data);

        let sig_bytes = hex::decode(signature)?;
        Ok(mac.verify_slice(&sig_bytes).is_ok())
    }

    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.contains(&origin.to_string()) || self.allowed_origins.contains(&"*".to_string())
    }

    pub fn generate_signature(&self, data: &[u8]) -> Result<String> {
        let key = self.prepare_key();
        let mut mac = HmacSha256::new_from_slice(&key)?;
        mac.update(data);
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    pub fn is_secure(&self) -> bool {
        self.key.is_some() && self.salt.is_some()
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
        assert_eq!(config.allowed_origins, vec!["*", "http://localhost"]);
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