use serde::Deserialize;
use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use std::env;
use std::process::Command;
use std::fmt;
use std::ops::Deref;

type HmacSha256 = Hmac<Sha256>;

/// Represents a secret API key, ensuring it's handled with care.
#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct ApiKey(String);

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted api key>")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted api key>")
    }
}

impl AsRef<str> for ApiKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for ApiKey {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for ApiKey {
    fn from(s: String) -> Self {
        ApiKey(s)
    }
}

/// Represents a secret API salt, ensuring it's handled with care.
#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct ApiSalt(String);

impl fmt::Debug for ApiSalt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted api salt>")
    }
}

impl fmt::Display for ApiSalt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted api salt>")
    }
}

impl AsRef<str> for ApiSalt {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for ApiSalt {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for ApiSalt {
    fn from(s: String) -> Self {
        ApiSalt(s)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    #[serde(default = "default_key")]
    key: Option<ApiKey>,
    #[serde(default = "default_salt")]
    salt: Option<ApiSalt>,
    #[serde(default = "default_allowed_origins")]
    allowed_origins: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            key: default_key(),
            salt: default_salt(),
            allowed_origins: default_allowed_origins(),
        }
    }
}

/// Returns None for key by default; must be set explicitly for security.
fn default_key() -> Option<ApiKey> {
    None
}

/// Returns None for salt by default; must be set explicitly for security.
fn default_salt() -> Option<ApiSalt> {
    None
}

/// Only allow localhost by default; wildcard is not secure.
fn default_allowed_origins() -> Vec<String> {
    vec!["http://localhost".to_string()]
}

impl SecurityConfig {
    /// Get the API key (if set)
    pub fn key(&self) -> Option<&ApiKey> {
        self.key.as_ref()
    }
    /// Set the API key
    pub fn set_key(&mut self, key: ApiKey) {
        self.key = Some(key);
    }
    /// Get the salt (if set)
    pub fn salt(&self) -> Option<&ApiSalt> {
        self.salt.as_ref()
    }
    /// Set the salt
    pub fn set_salt(&mut self, salt: ApiSalt) {
        self.salt = Some(salt);
    }
    /// Get allowed origins
    pub fn allowed_origins(&self) -> &[String] {
        &self.allowed_origins
    }
    /// Set allowed origins
    pub fn set_allowed_origins(&mut self, origins: Vec<String>) {
        self.allowed_origins = origins;
    }

    /// Prepares the key for HMAC operations by SHA256 hashing it.
    /// Uses the configured key, or a default placeholder if none is set.
    /// Note: Using a default placeholder key is insecure and only for non-localhost fallback.
    fn prepare_key(&self) -> Vec<u8> {
        let key_string = self.key.as_ref().map(|k| k.0.clone()).unwrap_or_else(|| "default_key_placeholder_insecure".to_string());
        let mut hasher = Sha256::new();
        hasher.update(key_string.as_bytes());
        hasher.finalize().to_vec()
    }

    /// Generates a new random API key if one is not already set or is empty.
    /// Returns a clone of the (potentially newly generated) key.
    pub fn generate_api_key(&mut self) -> ApiKey {
        if self.key.as_ref().map_or(true, |k| k.0.is_empty()) {
            let generated_key_string: String = thread_rng()
                .sample_iter(Alphanumeric)
                .take(32) // Ensure generated key is long enough
                .map(char::from)
                .collect();
            self.key = Some(ApiKey(generated_key_string));
        }
        self.key.as_ref().unwrap().clone() // Should be Some now
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

    /// Checks if both key and salt are set (basic check, not length/content).
    pub fn is_secure(&self) -> bool {
        self.key.is_some() && self.salt.is_some()
    }

    /// Validate that the security config is safe for production use.
    /// - key and salt must be set and at least 32 chars
    /// - allowed_origins must not contain "*"
    pub fn validate_secure(&self) -> Result<(), String> {
        match &self.key {
            Some(k) if k.0.len() >= 32 => {},
            _ => return Err("Security key must be set and at least 32 characters long".to_string()),
        }
        match &self.salt {
            Some(s) if s.0.len() >= 32 => {},
            _ => return Err("Security salt must be set and at least 32 characters long".to_string()),
        }
        if self.allowed_origins.iter().any(|o| o == "*") {
            return Err("Wildcard '*' in allowed_origins is not allowed in production".to_string());
        }
        Ok(())
    }
}

/// Generate a system-unique secret (SHA256 of username, hostname, and OS info)
/// Returns a String, to be wrapped in ApiKey or ApiSalt by the caller.
pub(crate) fn generate_local_machine_secret() -> String {
    let username = env::var("USERNAME").or_else(|_| env::var("USER")).unwrap_or_default();
    let hostname = get_hostname().unwrap_or_default();
    let os_info = get_os_info().unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    hasher.update(hostname.as_bytes());
    hasher.update(os_info.as_bytes());
    hex::encode(hasher.finalize())
}

fn get_hostname() -> Option<String> {
    // Try std::env, then fallback to hostname command
    env::var("COMPUTERNAME").ok()
        .or_else(|| env::var("HOSTNAME").ok())
        .or_else(|| {
            Command::new("hostname").output().ok().and_then(|o| String::from_utf8(o.stdout).ok())
        })
        .map(|s| s.trim().to_string())
}

fn get_os_info() -> Option<String> {
    #[cfg(target_os = "windows")]
    { Some("windows".to_string()) }
    #[cfg(target_os = "linux")]
    { fs::read_to_string("/etc/os-release").ok().or(Some("linux".to_string())) }
    #[cfg(target_os = "macos")]
    { Some("macos".to_string()) }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    { Some("unknown".to_string()) }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SecurityConfig::default();
        assert!(config.key().is_none());
        assert!(config.salt().is_none());
        assert_eq!(config.allowed_origins(), &["http://localhost"]);
    }

    #[test]
    fn test_origin_validation() {
        let mut config = SecurityConfig::default();
        config.set_allowed_origins(vec!["http://localhost:3000".to_string()]);
        assert!(config.is_origin_allowed("http://localhost:3000"));
        assert!(!config.is_origin_allowed("http://other.com"));
    }

    #[test]
    fn test_signature_validation() {
        let mut config = SecurityConfig::default();
        config.set_key(ApiKey("a_secure_key_that_is_long_enough_1234567890".to_string()));
        config.set_salt(ApiSalt("a_secure_salt_that_is_long_enough_1234567890".to_string()));
        let data = b"test data";
        let signature = config.generate_signature(data).unwrap();
        assert!(config.validate_signature(data, &signature).unwrap());
    }

    #[test]
    fn test_validate_secure() {
        let mut config = SecurityConfig::default();
        // Should fail: missing key/salt
        assert!(config.validate_secure().is_err());
        config.set_key(ApiKey("short".to_string()));
        config.set_salt(ApiSalt("short".to_string()));
        assert!(config.validate_secure().is_err());
        config.set_key(ApiKey("a_secure_key_that_is_long_enough_1234567890".to_string()));
        config.set_salt(ApiSalt("a_secure_salt_that_is_long_enough_1234567890".to_string()));
        assert!(config.validate_secure().is_ok());
        config.set_allowed_origins(vec!["*".to_string()]);
        assert!(config.validate_secure().is_err());
    }

    #[test]
    fn test_api_key_debug_display() {
        let key = ApiKey("secret_key_value".to_string());
        assert_eq!(format!("{:?}", key), "<redacted api key>");
        assert_eq!(format!("{}", key), "<redacted api key>");
    }
}