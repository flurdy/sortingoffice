use std::env;

/// Centralized environment variable handling with consistent defaults
pub struct ConfigUtils;

impl ConfigUtils {
    /// Get environment variable with default value
    pub fn get_env_var(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get environment variable as optional value
    pub fn get_env_var_opt(key: &str) -> Option<String> {
        env::var(key).ok()
    }

    /// Get environment variable as integer with default
    pub fn get_env_var_int(key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(default)
    }

    /// Get environment variable as boolean (true if "true", "1", "yes")
    pub fn get_env_var_bool(key: &str, default: bool) -> bool {
        env::var(key)
            .ok()
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes"))
            .unwrap_or(default)
    }

    /// Check if running in production environment
    pub fn is_production() -> bool {
        Self::get_env_var("RUST_ENV", "") == "production"
            || Self::get_env_var("ENVIRONMENT", "") == "production"
    }

    /// Get default port (3000)
    pub fn get_default_port() -> u16 {
        Self::get_env_var_int("PORT", 3000)
    }

    /// Get default config path
    pub fn get_default_config_path() -> String {
        Self::get_env_var("CONFIG_PATH", "config/config.toml")
    }

    /// Get test database URL with fallback
    pub fn get_test_database_url() -> String {
        Self::get_env_var("TEST_DATABASE_URL", &Self::get_env_var("DATABASE_URL", ""))
    }

    /// Get smoke test configuration
    pub fn get_smoke_test_config() -> SmokeTestConfig {
        SmokeTestConfig {
            app_url: Self::get_env_var_opt("SMOKE_TEST_APP_URL"),
            headless: Self::get_env_var_bool("SMOKE_TEST_HEADLESS", true),
            timeout_seconds: Self::get_env_var_int("SMOKE_TEST_TIMEOUT", 30),
            enable_vnc: Self::get_env_var_bool("SMOKE_TEST_VNC", false),
        }
    }

    /// Get host IP for testing
    pub fn get_host_ip() -> String {
        Self::get_env_var("HOST_IP", "127.0.0.1")
    }
}

/// Smoke test configuration structure
#[derive(Debug, Clone)]
pub struct SmokeTestConfig {
    pub app_url: Option<String>,
    pub headless: bool,
    pub timeout_seconds: u16,
    pub enable_vnc: bool,
}

/// Default configuration constants
pub mod defaults {
    pub const DEFAULT_PORT: u16 = 3000;
    pub const DEFAULT_CONFIG_PATH: &str = "config/config.toml";
    pub const DEFAULT_HOST_IP: &str = "127.0.0.1";
    pub const DEFAULT_SMOKE_TIMEOUT: u16 = 30;

    // Database defaults
    pub const DEFAULT_DATABASE_URL: &str =
        "mysql://sortingoffice:password@localhost:3306/sortingoffice";

    // Pagination defaults
    pub const DEFAULT_PER_PAGE: i64 = 20;
    pub const MAX_PER_PAGE: i64 = 100;

    // Validation limits
    pub const MAX_TRANSPORT_LENGTH: usize = 255;
    pub const MAX_DOMAIN_LABEL_LENGTH: usize = 63;
    pub const MAX_DOMAIN_LENGTH: usize = 253;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_env_var_with_default() {
        // Test with non-existent env var
        assert_eq!(
            ConfigUtils::get_env_var("NON_EXISTENT", "default"),
            "default"
        );
    }

    #[test]
    fn test_get_env_var_int() {
        // Test with non-existent env var
        assert_eq!(ConfigUtils::get_env_var_int("NON_EXISTENT", 8080), 8080);
    }

    #[test]
    fn test_get_env_var_bool() {
        // Test with non-existent env var
        assert_eq!(ConfigUtils::get_env_var_bool("NON_EXISTENT", true), true);
    }

    #[test]
    fn test_is_production() {
        // Test default (should be false)
        assert!(!ConfigUtils::is_production());
    }

    #[test]
    fn test_defaults() {
        assert_eq!(defaults::DEFAULT_PORT, 3000);
        assert_eq!(defaults::DEFAULT_CONFIG_PATH, "config/config.toml");
        assert_eq!(defaults::MAX_TRANSPORT_LENGTH, 255);
    }
}
