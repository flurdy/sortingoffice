use crate::models::RequiredAliasConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub struct DomainOverride {
    pub required: Vec<String>,
    pub common: Vec<String>,
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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
}

impl Default for DatabaseFeatures {
    fn default() -> Self {
        DatabaseFeatures {
            read_only: false,
            no_new_users: false,
            no_new_domains: false,
            no_password_updates: false,
            disabled: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

impl Default for GlobalFeatures {
    fn default() -> Self {
        GlobalFeatures {
            read_only: false,
            no_new_users: false,
            no_new_domains: false,
            no_password_updates: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub required_aliases: Vec<String>,
    pub common_aliases: Vec<String>,
    #[serde(default)]
    pub domain_overrides: HashMap<String, DomainOverride>,
    #[serde(default)]
    pub admins: Vec<AdminCredentials>,
    #[serde(default)]
    pub admin: Option<AdminCredentials>,
    #[serde(default)]
    pub databases: Vec<DatabaseConfig>,
    #[serde(default)]
    pub global_features: GlobalFeatures,
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
            domain_overrides: HashMap::new(),
            admins: vec![],
            admin: None,
            databases: vec![],
            global_features: GlobalFeatures::default(),
        })
    }

    /// Get required aliases for a specific domain
    pub fn get_required_aliases_for_domain(&self, domain: &str) -> Vec<String> {
        if let Some(overrides) = self.domain_overrides.get(domain) {
            overrides.required.clone()
        } else {
            self.required_aliases.clone()
        }
    }

    /// Get common aliases for a specific domain
    pub fn get_common_aliases_for_domain(&self, domain: &str) -> Vec<String> {
        if let Some(overrides) = self.domain_overrides.get(domain) {
            overrides.common.clone()
        } else {
            self.common_aliases.clone()
        }
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
            domain_overrides: HashMap::new(),
            admins: vec![AdminCredentials {
                username: "admin".to_string(),
                password_hash: "$2a$12$o8thacsiGCRhN1JN8xnW6e0KqNb7KrSgM67xxa62RKoAC9fOPf.aO"
                    .to_string(),
                role: AdminRole::Edit,
            }],
            admin: None,
            databases: vec![],
            global_features: GlobalFeatures::default(),
        }
    }
}

impl DatabaseConfig {
    /// Get the mapped field name for a logical field, or the logical name if not mapped
    pub fn field<'a>(&'a self, logical: &'a str) -> &'a str {
        self.field_map
            .get(logical)
            .map(|s| s.as_str())
            .unwrap_or(logical)
    }
}
