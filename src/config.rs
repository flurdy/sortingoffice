use crate::models::RequiredAliasConfig;
use envsubst::substitute;
use serde::{Deserialize, Serialize};

use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub enum AdminRole {
    #[serde(rename = "read-only")]
    ReadOnly,
    #[serde(rename = "edit")]
    #[default]
    Edit,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AdminCredentials {
    pub username: String,
    pub password_hash: String,
    #[serde(default)]
    pub role: AdminRole,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub id: String,
    pub label: String,
    pub url: String,
    #[serde(default)]
    pub features: DatabaseFeatures,
    #[serde(default)]
    pub field_map: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub connection_pool: ConnectionPoolConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DatabaseFeatures {
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub no_new_users: bool,
    #[serde(default)]
    pub no_new_domains: bool,
    #[serde(default)]
    pub no_password_updates: bool,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub no_seeding: bool,
    #[serde(default)]
    pub no_migrations: bool,
    #[serde(default)]
    pub no_relays: bool,
    #[serde(default)]
    pub no_relocated: bool,
    #[serde(default)]
    pub no_clients: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GlobalFeatures {
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub no_new_users: bool,
    #[serde(default)]
    pub no_new_domains: bool,
    #[serde(default)]
    pub no_password_updates: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContactInfo {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub contact_form: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub required_aliases: Vec<String>,
    pub common_aliases: Vec<String>,
    #[serde(default)]
    pub admins: Vec<AdminCredentials>,
    #[serde(default)]
    pub admin: Option<AdminCredentials>,
    #[serde(default)]
    pub databases: Vec<DatabaseConfig>,
    #[serde(default)]
    pub global_features: GlobalFeatures,
    #[serde(default)]
    pub contact: Option<ContactInfo>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        // Migrate old config: if admins is empty and admin is present, push admin into admins
        if config.admins.is_empty() {
            if let Some(admin) = config.admin.take() {
                config.admins.push(admin);
            }
        }
        Ok(config)
    }

    /// Load configuration from the default config file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Check if CONFIG_PATH environment variable is set
        if let Ok(config_path) = std::env::var("CONFIG_PATH") {
            if Path::new(&config_path).exists() {
                return Self::from_file(&config_path);
            }
        }

        let config_paths = [
            // "config/required_aliases.toml",
            "config/config.toml",
            // "/etc/sortingoffice/required_aliases.toml",
            "/etc/sortingoffice/config.toml",
            "./config.toml",
        ];

        for path in &config_paths {
            if Path::new(path).exists() {
                return Self::from_file(path);
            }
        }

        // Return default config if no file found
        Ok(Config {
            required_aliases: vec![
                "postmaster".to_string(),
                "abuse".to_string(),
                "hostmaster".to_string(),
            ],
            common_aliases: vec![
                "webmaster".to_string(),
                "admin".to_string(),
                "support".to_string(),
                "info".to_string(),
                "noreply".to_string(),
                "no-reply".to_string(),
                "security".to_string(),
                "help".to_string(),
                "contact".to_string(),
                "sales".to_string(),
                "marketing".to_string(),
                "hr".to_string(),
                "finance".to_string(),
                "legal".to_string(),
                "privacy".to_string(),
                "dmca".to_string(),
                "spam".to_string(),
            ],
            admins: vec![],
            admin: None,
            databases: vec![],
            global_features: GlobalFeatures::default(),
            contact: None,
        })
    }

    /// Load config with environment variable substitution
    pub fn load_config_with_env(path: &str) -> Result<Config, anyhow::Error> {
        let raw = fs::read_to_string(path)?;

        // Allow environment variables based on safe patterns, with content validation for others
        // This prevents problematic environment variables from breaking the substitution
        let mut allowed_env_vars = std::collections::HashMap::new();

        // Patterns that are always safe to use in config substitution
        let safe_patterns = [
            "DATABASE_URL",
            "MYSQL_",
            "BACKUP",
            "PRIMARY_DB_URL",
            "APP_URL",
            "BASE_URL",
            "DEFAULT_LOCALE",
            "TESTING",
            "RUST_ENV",
            "CONFIG_PATH",
            "HOST",
            "PORT",
            "ADMIN_",
            "theme_display_",
            "RUST_",
            "TEST_",
        ];

        // Helper function to check if a value is safe for substitution
        fn is_value_safe(value: &str) -> bool {
            !value.contains("${") && // Skip values that already contain substitution syntax
            !value.contains("}") && // Skip values with closing braces that could break substitution
            !value.contains("{") && // Skip values with opening braces that could break substitution
            !value.contains("$(") && // Skip values with command substitution syntax
            !value.contains("`") && // Skip values with backticks
            !value.contains("\"") && // Skip values with quotes that could break TOML
            !value.contains("'") && // Skip values with single quotes
            !value.contains("\n") && // Skip multiline values
            !value.contains("\r") && // Skip values with carriage returns
            !value.contains("\t") && // Skip values with tabs
            value.len() < 1000 // Skip very long values
        }

        // Process all environment variables
        for (key, value) in std::env::vars() {
            // Check if this key matches any safe pattern
            let is_safe_pattern = safe_patterns.iter().any(|pattern| {
                if pattern.ends_with('_') {
                    key.starts_with(pattern)
                } else {
                    key == *pattern || key.starts_with(&format!("{}_", pattern))
                }
            });

            if is_safe_pattern {
                // For safe patterns, allow the value without additional validation
                allowed_env_vars.insert(key, value);
            } else {
                // For other variables, check if the value is safe
                if is_value_safe(&value) {
                    allowed_env_vars.insert(key, value);
                } else {
                    eprintln!("Warning: Skipping environment variable {} due to potentially problematic content", key);
                }
            }
        }

        // Try substitution with filtered environment variables
        match substitute(&raw, &allowed_env_vars) {
            Ok(substituted) => {
                let config: Config = toml::from_str(&substituted)?;
                Ok(config)
            }
            Err(e) => {
                // If substitution fails, try with the original content (no substitution)
                // This provides a fallback for cases where substitution isn't needed
                eprintln!(
                    "Warning: Environment variable substitution failed: {}. Using original config.",
                    e
                );
                let config: Config = toml::from_str(&raw)?;
                Ok(config)
            }
        }
    }

    /// Get required aliases for a specific domain
    pub fn get_required_aliases_for_domain(&self, _domain: &str) -> Vec<String> {
        self.required_aliases.clone()
    }

    /// Get common aliases for a specific domain
    pub fn get_common_aliases_for_domain(&self, _domain: &str) -> Vec<String> {
        self.common_aliases.clone()
    }

    /// Get all aliases (required + common) for a specific domain
    pub fn get_all_aliases_for_domain(&self, domain: &str) -> Vec<String> {
        let mut all = self.get_required_aliases_for_domain(domain);
        all.extend(self.get_common_aliases_for_domain(domain));
        all
    }

    /// Convert to RequiredAliasConfig for a specific domain
    pub fn to_required_alias_config(&self, domain: &str) -> RequiredAliasConfig {
        RequiredAliasConfig::from_vecs(
            self.get_required_aliases_for_domain(domain),
            self.get_common_aliases_for_domain(domain),
        )
    }

    /// Verify admin credentials and return role if valid
    pub fn verify_admin_credentials(&self, username: &str, password: &str) -> Option<AdminRole> {
        for admin in &self.admins {
            if admin.username == username {
                match bcrypt::verify(password, &admin.password_hash) {
                    Ok(is_valid) => {
                        if is_valid {
                            return Some(admin.role.clone());
                        }
                    }
                    Err(_) => {
                        // Password verification error - continue to next admin
                    }
                }
            }
        }

        None
    }

    /// Get database features for a specific database ID
    pub fn get_database_features(&self, database_id: &str) -> Option<&DatabaseFeatures> {
        self.databases
            .iter()
            .find(|db| db.id == database_id)
            .map(|db| &db.features)
    }

    /// Check if a database is read-only (global or database-specific)
    pub fn is_database_read_only(&self, database_id: &str) -> bool {
        // Check if database is disabled first
        if let Some(features) = self.get_database_features(database_id) {
            if features.disabled {
                return true; // Disabled databases are effectively read-only
            }
        }

        // Check global read-only setting
        if self.global_features.read_only {
            return true;
        }

        // Check database-specific read-only setting
        if let Some(features) = self.get_database_features(database_id) {
            return features.read_only;
        }

        false
    }

    /// Check if new users are blocked (global or database-specific)
    pub fn is_new_users_blocked(&self, database_id: &str) -> bool {
        // Check if database is disabled first
        if let Some(features) = self.get_database_features(database_id) {
            if features.disabled {
                return true; // Disabled databases block all operations
            }
        }

        // Check global setting
        if self.global_features.no_new_users {
            return true;
        }

        // Check database-specific setting
        if let Some(features) = self.get_database_features(database_id) {
            return features.no_new_users;
        }

        false
    }

    /// Check if new domains are blocked (global or database-specific)
    pub fn is_new_domains_blocked(&self, database_id: &str) -> bool {
        // Check if database is disabled first
        if let Some(features) = self.get_database_features(database_id) {
            if features.disabled {
                return true; // Disabled databases block all operations
            }
        }

        // Check global setting
        if self.global_features.no_new_domains {
            return true;
        }

        // Check database-specific setting
        if let Some(features) = self.get_database_features(database_id) {
            return features.no_new_domains;
        }

        false
    }

    /// Check if password updates are blocked (global or database-specific)
    pub fn is_password_updates_blocked(&self, database_id: &str) -> bool {
        // Check if database is disabled first
        if let Some(features) = self.get_database_features(database_id) {
            if features.disabled {
                return true; // Disabled databases block all operations
            }
        }

        // Check global setting
        if self.global_features.no_password_updates {
            return true;
        }

        // Check database-specific setting
        if let Some(features) = self.get_database_features(database_id) {
            return features.no_password_updates;
        }

        false
    }

    /// Check if a database is completely disabled
    pub fn is_database_disabled(&self, database_id: &str) -> bool {
        if let Some(features) = self.get_database_features(database_id) {
            return features.disabled;
        }
        false
    }

    /// Check if seeding is blocked for a database
    pub fn is_seeding_blocked(&self, database_id: &str) -> bool {
        // Check global features first
        if self.global_features.read_only {
            return true;
        }

        // Check database-specific features
        if let Some(features) = self.get_database_features(database_id) {
            return features.no_seeding || features.disabled || features.read_only;
        }

        false
    }

    /// Check if migrations are blocked for a database
    pub fn is_migration_blocked(&self, database_id: &str) -> bool {
        // Check global features first
        if self.global_features.read_only {
            return true;
        }

        // Check database-specific features
        if let Some(features) = self.get_database_features(database_id) {
            return features.no_migrations || features.disabled || features.read_only;
        }

        false
    }

    /// Check if relays table is available (not disabled)
    pub fn is_relays_available(&self, database_id: &str) -> bool {
        // Check if database is disabled first
        if let Some(features) = self.get_database_features(database_id) {
            if features.disabled {
                return false; // Disabled databases don't have any tables
            }
            return !features.no_relays;
        }
        true // Default to available if no features specified
    }

    /// Check if relocated table is available (not disabled)
    pub fn is_relocated_available(&self, database_id: &str) -> bool {
        // Check if database is disabled first
        if let Some(features) = self.get_database_features(database_id) {
            if features.disabled {
                return false; // Disabled databases don't have any tables
            }
            return !features.no_relocated;
        }
        true // Default to available if no features specified
    }

    /// Check if clients table is available (not disabled)
    pub fn is_clients_available(&self, database_id: &str) -> bool {
        // Check if database is disabled first
        if let Some(features) = self.get_database_features(database_id) {
            if features.disabled {
                return false; // Disabled databases don't have any tables
            }
            return !features.no_clients;
        }
        true // Default to available if no features specified
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            required_aliases: vec![
                "postmaster".to_string(),
                "abuse".to_string(),
                "hostmaster".to_string(),
            ],
            common_aliases: vec![
                "webmaster".to_string(),
                "admin".to_string(),
                "support".to_string(),
                "info".to_string(),
                "noreply".to_string(),
                "no-reply".to_string(),
                "security".to_string(),
                "help".to_string(),
                "contact".to_string(),
                "sales".to_string(),
                "marketing".to_string(),
                "hr".to_string(),
                "finance".to_string(),
                "legal".to_string(),
                "privacy".to_string(),
                "dmca".to_string(),
                "spam".to_string(),
            ],
            admins: vec![AdminCredentials {
                username: "admin".to_string(),
                password_hash: "$2a$12$o8thacsiGCRhN1JN8xnW6e0KqNb7KrSgM67xxa62RKoAC9fOPf.aO"
                    .to_string(),
                role: AdminRole::Edit,
            }],
            admin: None,
            databases: vec![],
            global_features: GlobalFeatures::default(),
            contact: None,
        }
    }
}

impl DatabaseConfig {
    /// Get the mapped field name for a table-qualified logical field, or the logical name if not mapped
    ///
    /// Examples:
    /// - `field("users.id")` -> looks for "users.id" in field_map, falls back to "id"
    /// - `field("users.enabled")` -> looks for "users.enabled" in field_map, falls back to "enabled"
    /// - `field("id")` -> looks for "id" in field_map, falls back to "id" (backward compatibility)
    pub fn field<'a>(&'a self, logical: &'a str) -> &'a str {
        // First try the exact logical name (for backward compatibility)
        if let Some(mapped) = self.field_map.get(logical) {
            return mapped.as_str();
        }

        // If not found and it contains a dot, try without the table prefix
        if logical.contains('.') {
            let field_name = logical.split('.').next_back().unwrap_or(logical);
            if let Some(mapped) = self.field_map.get(field_name) {
                return mapped.as_str();
            }
        }

        // Fall back to the original logical name
        logical
    }

    /// Get the mapped field name for a specific table and field
    ///
    /// Examples:
    /// - `field_for_table("users", "id")` -> looks for "users.id" or "id" in field_map
    /// - `field_for_table("domains", "enabled")` -> looks for "domains.enabled" or "enabled" in field_map
    pub fn field_for_table<'a>(&'a self, table: &'a str, field: &'a str) -> &'a str {
        let qualified = format!("{table}.{field}");

        // First try the qualified name
        if let Some(mapped) = self.field_map.get(&qualified) {
            return mapped.as_str();
        }

        // Then try just the field name
        if let Some(mapped) = self.field_map.get(field) {
            return mapped.as_str();
        }

        // Fall back to the original field name
        field
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionPoolConfig {
    #[serde(default = "default_max_size")]
    pub max_size: u32,
    #[serde(default = "default_min_idle")]
    pub min_idle: u32,
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: u64,
    #[serde(default = "default_test_on_acquire")]
    pub test_on_acquire: bool,
    #[serde(default = "default_test_on_return")]
    pub test_on_return: bool,
}

fn default_max_size() -> u32 {
    20 // Increased from 10 to 20 for better concurrency
}

fn default_min_idle() -> u32 {
    5 // Increased from 1 to 5 to keep more connections ready
}

fn default_connection_timeout() -> u64 {
    10 // Reduced from 30 to 10 seconds for faster failure detection
}

fn default_idle_timeout() -> u64 {
    600 // Increased from 300 to 600 seconds (10 minutes) to keep connections longer
}

fn default_max_lifetime() -> u64 {
    3600 // Increased from 1800 to 3600 seconds (1 hour) for better connection reuse
}

fn default_test_on_acquire() -> bool {
    true
}

fn default_test_on_return() -> bool {
    false
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_size: default_max_size(),
            min_idle: default_min_idle(),
            connection_timeout: default_connection_timeout(),
            idle_timeout: default_idle_timeout(),
            max_lifetime: default_max_lifetime(),
            test_on_acquire: default_test_on_acquire(),
            test_on_return: default_test_on_return(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optional_tables_availability() {
        let mut config = Config::default();

        // Add a test database
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures::default(),
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test default availability (should be true)
        assert!(config.is_relays_available("test"));
        assert!(config.is_relocated_available("test"));
        assert!(config.is_clients_available("test"));

        // Test with disabled tables
        let mut config = Config::default();
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures {
                no_relays: true,
                no_relocated: true,
                no_clients: true,
                ..Default::default()
            },
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test disabled availability (should be false)
        assert!(!config.is_relays_available("test"));
        assert!(!config.is_relocated_available("test"));
        assert!(!config.is_clients_available("test"));

        // Test with disabled database
        let mut config = Config::default();
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures {
                disabled: true,
                ..Default::default()
            },
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test disabled database (should be false for all tables)
        assert!(!config.is_relays_available("test"));
        assert!(!config.is_relocated_available("test"));
        assert!(!config.is_clients_available("test"));

        // Test non-existent database (should be true by default)
        assert!(config.is_relays_available("nonexistent"));
        assert!(config.is_relocated_available("nonexistent"));
        assert!(config.is_clients_available("nonexistent"));
    }

    #[test]
    fn test_seeding_blocked() {
        let mut config = Config::default();

        // Add a test database
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures::default(),
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test default (should not be blocked)
        assert!(!config.is_seeding_blocked("test"));

        // Test with global read_only = true
        let mut config = Config::default();
        config.global_features.read_only = true;
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures::default(),
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test global read_only blocks seeding
        assert!(config.is_seeding_blocked("test"));

        // Test with database-specific no_seeding = true
        let mut config = Config::default();
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures {
                no_seeding: true,
                ..Default::default()
            },
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test database-specific no_seeding blocks seeding
        assert!(config.is_seeding_blocked("test"));

        // Test with database-specific disabled = true
        let mut config = Config::default();
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures {
                disabled: true,
                ..Default::default()
            },
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test disabled database blocks seeding
        assert!(config.is_seeding_blocked("test"));

        // Test with database-specific read_only = true
        let mut config = Config::default();
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures {
                read_only: true,
                ..Default::default()
            },
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test database-specific read_only blocks seeding
        assert!(config.is_seeding_blocked("test"));

        // Test non-existent database (should not be blocked by default)
        assert!(!config.is_seeding_blocked("nonexistent"));
    }

    #[test]
    fn test_migration_blocked() {
        let mut config = Config::default();

        // Add a test database
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures::default(),
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test default (should not be blocked)
        assert!(!config.is_migration_blocked("test"));

        // Test with global read_only = true
        let mut config = Config::default();
        config.global_features.read_only = true;
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures::default(),
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test global read_only blocks migrations
        assert!(config.is_migration_blocked("test"));

        // Test with database-specific no_migrations = true
        let mut config = Config::default();
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures {
                no_migrations: true,
                ..Default::default()
            },
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test database-specific no_migrations blocks migrations
        assert!(config.is_migration_blocked("test"));

        // Test with database-specific disabled = true
        let mut config = Config::default();
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures {
                disabled: true,
                ..Default::default()
            },
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test disabled database blocks migrations
        assert!(config.is_migration_blocked("test"));

        // Test with database-specific read_only = true
        let mut config = Config::default();
        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test DB".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures {
                read_only: true,
                ..Default::default()
            },
            field_map: std::collections::HashMap::new(),
            connection_pool: ConnectionPoolConfig::default(),
        };
        config.databases.push(db_config);

        // Test database-specific read_only blocks migrations
        assert!(config.is_migration_blocked("test"));

        // Test non-existent database (should not be blocked by default)
        assert!(!config.is_migration_blocked("nonexistent"));
    }
}
