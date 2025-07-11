use crate::models::RequiredAliasConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum AdminRole {
    #[serde(rename = "read-only")]
    ReadOnly,
    #[serde(rename = "edit")]
    Edit,
}

impl Default for AdminRole {
    fn default() -> Self {
        AdminRole::Edit
    }
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
                password_hash: "$2a$12$o8thacsiGCRhN1JN8xnW6e0KqNb7KrSgM67xxa62RKoAC9fOPf.aO".to_string(),
                role: AdminRole::Edit,
            }],
            admin: None,
            databases: vec![],
        }
    }
}
