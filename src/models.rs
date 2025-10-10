use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

fn deserialize_checkbox<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        Some("on") | Some("true") | Some("1") => Ok(true),
        Some("false") | Some("off") | Some("0") => Ok(false),
        _ => Ok(false), // Default to false for any other value or None
    }
}

// Custom deserializer that can handle multiple values with the same field name
fn deserialize_duplicate_fields<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;

    match value {
        Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                if let Value::String(s) = item {
                    result.push(s);
                }
            }
            Ok(result)
        }
        Value::String(s) => Ok(vec![s]),
        _ => Ok(Vec::new()),
    }
}

fn deserialize_radio_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s == "true")
}

fn deserialize_transport<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // Normalize "virtual" to "virtual:" since "virtual" is not a valid transport format
    if s == "virtual" {
        Ok("virtual:".to_string())
    } else {
        Ok(s)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CacheStats {
    pub system_stats_cached: bool,
    pub catch_all_report_cached: bool,
    pub alias_report_cached: bool,
    pub domain_alias_matrix_report_cached: bool,
    pub orphaned_aliases_report_cached: bool,
    pub external_forwarders_report_cached: bool,
    pub missing_aliases_report_cached: bool,
    // DNS caches
    pub dns_ns_count: usize,
    pub dns_mx_count: usize,
    pub dns_txt_count: usize,
    pub dns_dkim_count: usize,
    pub total_dns_entries: usize,
    pub domains_paginated_count: usize,
    pub aliases_paginated_count: usize,
    pub users_paginated_count: usize,
    pub clients_paginated_count: usize,
    pub relays_paginated_count: usize,
    pub relocated_paginated_count: usize,
    pub total_pagination_entries: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SystemStats {
    pub total_domains: i64,
    pub enabled_domains: i64,
    pub disabled_domains: i64,
    pub recent_domains: i64,
    pub total_users: i64,
    pub enabled_users: i64,
    pub disabled_users: i64,
    pub recent_users: i64,
    pub total_aliases: i64,
    pub enabled_aliases: i64,
    pub disabled_aliases: i64,
    pub recent_aliases: i64,
    pub total_backups: i64,
    pub enabled_backups: i64,
    pub disabled_backups: i64,
    pub recent_backups: i64,
    pub total_relays: i64,
    pub enabled_relays: i64,
    pub disabled_relays: i64,
    pub recent_relays: i64,
    pub total_relocated: i64,
    pub enabled_relocated: i64,
    pub disabled_relocated: i64,
    pub recent_relocated: i64,
    pub total_clients: i64,
    pub enabled_clients: i64,
    pub disabled_clients: i64,
    pub recent_clients: i64,
    pub total_quota: i64,
    pub used_quota: i64,
    pub quota_usage_percent: f64,
    // Combined enabled stats for dashboard
    pub enabled_domains_and_backups: i64,
}

#[derive(
    Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Clone, QueryableByName,
)]
#[diesel(table_name = domains)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Domain {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub pkid: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub domain: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub transport: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub created: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub modified: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub enabled: bool,
}

impl Domain {
    pub fn transport_display(&self) -> String {
        self.transport.clone().unwrap_or_else(|| "-".to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = domains)]
pub struct NewDomain {
    pub domain: String,
    pub transport: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, QueryableByName)]
#[diesel(table_name = users)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub id: String,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub crypt: String,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub maildir: String,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub home: String,
    #[diesel(sql_type = diesel::sql_types::Unsigned<diesel::sql_types::Smallint>)]
    pub uid: u16,
    #[diesel(sql_type = diesel::sql_types::Unsigned<diesel::sql_types::Smallint>)]
    pub gid: u16,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub created: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub modified: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub enabled: bool,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub change_password: bool,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: String,
    pub crypt: String,
    pub name: String,
    pub maildir: String,
    pub home: String,
    pub uid: u16,
    pub gid: u16,
    pub enabled: bool,
    pub change_password: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserForm {
    pub id: String,
    pub password: String,
    pub name: String,
    pub maildir: String,
    pub home: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub enabled: bool,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub change_password: bool,
}

#[derive(
    Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Clone, QueryableByName,
)]
#[diesel(table_name = aliases)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Alias {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub pkid: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub mail: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub destination: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub created: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub modified: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub enabled: bool,
}

impl Alias {
    pub fn domain(&self) -> String {
        self.mail.split('@').nth(1).unwrap_or("").to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = aliases)]
pub struct NewAlias {
    pub mail: String,
    pub destination: String,
    pub enabled: bool,
}

// Form data structures for HTMX
#[derive(Deserialize, Debug, Clone)]
pub struct DomainForm {
    pub domain: String,
    pub transport: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AliasForm {
    pub mail: String,
    pub destination: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub enabled: bool,
    pub return_url: Option<String>,
    pub redirect_to: Option<String>, // "aliases" or "domain" (default)
}

// Statistics models
#[derive(Debug, Serialize, Deserialize)]
pub struct DomainStats {
    pub domain_id: i32,
    pub domain: String,
    pub user_count: i64,
    pub alias_count: i64,
    pub relay_count: i64,
    pub relay_enabled_count: i64,
    pub relay_disabled_count: i64,
    pub relocated_count: i64,
    pub relocated_enabled_count: i64,
    pub relocated_disabled_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrossDatabaseDomainInfo {
    pub database_id: String,
    pub database_label: String,
    pub is_primary_domain: bool,
    pub is_backup_domain: bool,
    pub enabled: bool,
    pub user_count: i64,
    pub alias_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentChange {
    pub resource_type: String, // "domain", "user", "alias", "backup", "relay", "relocated", "client"
    pub resource_id: String,   // ID or identifier of the resource
    pub resource_name: String, // Human-readable name (domain name, user email, etc.)
    pub action: String,        // "created", "updated", "deleted"
    pub timestamp: NaiveDateTime,
    pub enabled: Option<bool>, // For resources that have enabled status
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentChangesReport {
    pub changes: Vec<RecentChange>,
    pub total_count: usize,
}

// Search models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub resource_type: String, // "domain", "user", "alias", "backup", "relay", "relocated", "client"
    pub resource_id: String,
    pub resource_name: String,
    pub resource_description: String, // Additional context about the resource
    pub enabled: Option<bool>,
    pub created: Option<NaiveDateTime>,
    pub modified: Option<NaiveDateTime>,
    pub match_fields: Vec<String>, // Fields that matched the search
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResults {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_count: usize,
    pub search_time_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SearchForm {
    pub query: String,
    pub resource_types: Option<Vec<String>>, // Filter by resource types
    pub enabled_only: Option<bool>,          // Filter by enabled status
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = backups)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Backup {
    pub pkid: i32,
    pub domain: String,
    pub transport: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub created: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub modified: Option<NaiveDateTime>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = backups)]
pub struct NewBackup {
    pub domain: String,
    pub transport: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupForm {
    pub domain: String,
    pub transport: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub enabled: bool,
}

// Relay models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = relays)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Relay {
    pub pkid: i32,
    pub recipient: String,
    pub status: String,
    pub enabled: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub created: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub modified: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = relays)]
pub struct NewRelay {
    pub recipient: String,
    pub status: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelayForm {
    pub recipient: String,
    pub status: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub enabled: bool,
}

// Relocated models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = relocated)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Relocated {
    pub pkid: i32,
    pub old_address: String,
    pub new_address: String,
    pub enabled: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub created: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub modified: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = relocated)]
pub struct NewRelocated {
    pub old_address: String,
    pub new_address: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelocatedForm {
    pub old_address: String,
    pub new_address: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub enabled: bool,
}

// Client models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = clients)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Client {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub id: i32,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub client: String,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub status: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub enabled: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub created_at: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = clients)]
pub struct NewClient {
    pub client: String,
    pub status: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientForm {
    pub client: String,
    pub status: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub enabled: bool,
}

// Catch-all report models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatchAllReport {
    pub domain: String,
    pub catch_all_alias: String,
    pub catch_all_destination: String,
    pub required_aliases: Vec<RequiredAlias>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequiredAlias {
    pub mail: String,
    pub destination: String,
    pub enabled: bool,
}

// Configuration for required and common aliases
#[derive(Debug, Clone)]
pub struct RequiredAliasConfig {
    pub required_aliases: Vec<String>,
    pub common_aliases: Vec<String>,
}

impl Default for RequiredAliasConfig {
    fn default() -> Self {
        // Use centralized config utilities
        let required_from_env =
            crate::config_utils::ConfigUtils::get_env_var_opt("REQUIRED_ALIASES");
        let common_from_env = crate::config_utils::ConfigUtils::get_env_var_opt("COMMON_ALIASES");

        let required_aliases = if let Some(aliases_str) = required_from_env {
            let aliases: Vec<String> = aliases_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !aliases.is_empty() {
                aliases
            } else {
                Self::default_required_aliases()
            }
        } else {
            Self::default_required_aliases()
        };

        let common_aliases = if let Some(aliases_str) = common_from_env {
            let aliases: Vec<String> = aliases_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !aliases.is_empty() {
                aliases
            } else {
                Self::default_common_aliases()
            }
        } else {
            Self::default_common_aliases()
        };

        Self {
            required_aliases,
            common_aliases,
        }
    }
}

impl RequiredAliasConfig {
    /// Default required aliases (essential for email standards)
    fn default_required_aliases() -> Vec<String> {
        vec![
            "postmaster".to_string(),
            "abuse".to_string(),
            "hostmaster".to_string(),
        ]
    }

    /// Default common aliases (frequently used but not strictly required)
    fn default_common_aliases() -> Vec<String> {
        vec![
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
        ]
    }

    /// Get all aliases (required + common)
    pub fn get_all_aliases(&self) -> Vec<String> {
        let mut all = self.required_aliases.clone();
        all.extend(self.common_aliases.clone());
        all
    }

    /// Create a new configuration from comma-separated strings
    pub fn from_strings(required_str: &str, common_str: &str) -> Self {
        let required_aliases: Vec<String> = required_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let common_aliases: Vec<String> = common_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            required_aliases,
            common_aliases,
        }
    }

    /// Create a new configuration from vectors
    pub fn from_vecs(required: Vec<String>, common: Vec<String>) -> Self {
        Self {
            required_aliases: required,
            common_aliases: common,
        }
    }

    /// Get the list of required aliases
    pub fn get_required_aliases(&self) -> &[String] {
        &self.required_aliases
    }

    /// Get the list of common aliases
    pub fn get_common_aliases(&self) -> &[String] {
        &self.common_aliases
    }

    /// Add a new required alias
    pub fn add_required_alias(&mut self, alias: String) {
        if !self.required_aliases.contains(&alias) {
            self.required_aliases.push(alias);
        }
    }

    /// Add a new common alias
    pub fn add_common_alias(&mut self, alias: String) {
        if !self.common_aliases.contains(&alias) {
            self.common_aliases.push(alias);
        }
    }

    /// Remove a required alias
    pub fn remove_required_alias(&mut self, alias: &str) {
        self.required_aliases.retain(|a| a != alias);
    }

    /// Remove a common alias
    pub fn remove_common_alias(&mut self, alias: &str) {
        self.common_aliases.retain(|a| a != alias);
    }

    /// Move an alias from common to required
    pub fn promote_to_required(&mut self, alias: &str) {
        if let Some(index) = self.common_aliases.iter().position(|a| a == alias) {
            let alias = self.common_aliases.remove(index);
            self.add_required_alias(alias);
        }
    }

    /// Move an alias from required to common
    pub fn demote_to_common(&mut self, alias: &str) {
        if let Some(index) = self.required_aliases.iter().position(|a| a == alias) {
            let alias = self.required_aliases.remove(index);
            self.add_common_alias(alias);
        }
    }
}

// Enhanced report models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainAliasReport {
    pub domain: String,
    pub has_catch_all: bool,
    pub catch_all_alias: Option<String>,
    pub catch_all_destination: Option<String>,
    pub required_aliases: Vec<RequiredAlias>,
    pub missing_required_aliases: Vec<String>,
    pub missing_common_aliases: Vec<String>,
    pub disabled_required_aliases: Vec<Alias>,
    pub disabled_common_aliases: Vec<Alias>,
    pub disabled_catch_all: Option<Alias>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AliasReport {
    pub domains_with_catch_all: Vec<DomainAliasReport>,
    pub domains_without_catch_all: Vec<DomainAliasReport>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainAliasMatrixReport {
    pub domains: Vec<DomainAliasMatrixRow>,
    pub required_aliases_list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainAliasMatrixRow {
    pub id: i32,
    pub domain: String,
    pub catch_all_status: AliasStatus,
    pub required_aliases: Vec<RequiredAliasMatrixItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequiredAliasMatrixItem {
    pub alias: String,
    pub status: AliasStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AliasStatus {
    Present,
    Missing,
    Disabled,
}

impl AliasStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            AliasStatus::Present => "✅",
            AliasStatus::Missing => "❌",
            AliasStatus::Disabled => "⚠️",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            AliasStatus::Present => "text-green-600 dark:text-green-400",
            AliasStatus::Missing => "text-red-600 dark:text-red-400",
            AliasStatus::Disabled => "text-yellow-600 dark:text-yellow-400",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            AliasStatus::Present => "Present and enabled",
            AliasStatus::Missing => "Missing",
            AliasStatus::Disabled => "Present but disabled",
        }
    }
}

// Pagination models
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub backup_page: Option<i64>,
    pub per_page: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub search: Option<String>,
    pub enabled_filter: Option<String>, // "all", "enabled", or "disabled"
    pub exclude_subdomains: Option<bool>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            backup_page: Some(1),
            per_page: Some(20),
            sort_by: Some("mail".to_string()),
            sort_order: Some("asc".to_string()),
            search: None,
            enabled_filter: Some("all".to_string()),
            exclude_subdomains: Some(false),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub current_page: i64,
    pub per_page: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

// Additional report models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrphanedAliasReport {
    pub orphaned_aliases: Vec<OrphanedAlias>,
    pub orphaned_users: Vec<OrphanedUser>,
    pub users_without_aliases: Vec<UserWithoutAlias>,
    pub orphaned_relays: Vec<OrphanedRelay>,
    pub orphaned_relocated: Vec<OrphanedRelocated>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrphanedReportParams {
    pub hide_disabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MxServersReportParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub exclude_disabled: Option<bool>,
    pub exclude_subdomains: Option<bool>,
    pub filter_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrphanedAlias {
    pub id: i32,
    pub mail: String,
    pub destination: String,
    pub domain: String,
    pub domain_id: Option<i32>,
    pub domain_enabled: Option<bool>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrphanedUser {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub domain_id: Option<i32>,
    pub domain_enabled: Option<bool>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserWithoutAlias {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub domain_id: Option<i32>,
    pub domain_enabled: Option<bool>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrphanedRelay {
    pub id: i32,
    pub recipient: String,
    pub status: String,
    pub domain: String,
    pub domain_id: Option<i32>,
    pub domain_enabled: Option<bool>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrphanedRelocated {
    pub id: i32,
    pub old_address: String,
    pub new_address: String,
    pub domain: String,
    pub domain_id: Option<i32>,
    pub domain_enabled: Option<bool>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalForwarderReport {
    pub external_forwarders: Vec<ExternalForwarder>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalForwarder {
    pub id: i32,
    pub mail: String,
    pub destination: String,
    pub domain: String,
    pub domain_id: Option<i32>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MissingAliasReport {
    pub domains_missing_aliases: Vec<DomainMissingAliases>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainMissingAliases {
    pub domain: String,
    pub missing_required_aliases: Vec<String>,
    pub has_catch_all: bool,
    pub catch_all_alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasCrossDomainReport {
    pub alias: String,
    pub occurrences: Vec<AliasOccurrence>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasOccurrence {
    pub id: i32,
    pub domain: String,
    pub mail: String,
    pub destination: String,
    pub enabled: bool,
}

// Cross-database domain matrix models
#[derive(Debug, Serialize, Deserialize)]
pub struct CrossDatabaseDomainMatrixReport {
    pub databases: Vec<DatabaseInfo>,
    pub domains: Vec<CrossDatabaseDomainRow>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossDatabaseDomainRow {
    pub domain: String,
    pub domain_id: i32, // ID in database where domain exists, 0 if not exists
    pub domain_database_id: String, // Database ID where domain exists, empty if not exists
    pub presence: Vec<DomainPresence>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainPresence {
    pub database_id: String,
    pub database_label: String,
    pub presence_type: DomainPresenceType,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DomainPresenceType {
    Primary, // Domain exists as primary domain
    Backup,  // Domain exists as backup domain
    Missing, // Domain doesn't exist in this database
}

impl DomainPresenceType {
    pub fn symbol(&self) -> &'static str {
        match self {
            DomainPresenceType::Primary => "",
            DomainPresenceType::Backup => "",
            DomainPresenceType::Missing => "",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            DomainPresenceType::Primary => "text-blue-600 dark:text-blue-400",
            DomainPresenceType::Backup => "text-green-600 dark:text-green-400",
            DomainPresenceType::Missing => "text-gray-400 dark:text-gray-500",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            DomainPresenceType::Primary => "Primary domain",
            DomainPresenceType::Backup => "Backup domain",
            DomainPresenceType::Missing => "Not present",
        }
    }
}

// Cross-database User Distribution Report models
#[derive(Debug, Serialize, Deserialize)]
pub struct CrossDatabaseUserDistributionReport {
    pub databases: Vec<DatabaseInfo>,
    pub users: Vec<CrossDatabaseUserRow>,
    pub summary: UserDistributionSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossDatabaseUserRow {
    pub user_id: String,
    pub user_name: String,
    pub presence: Vec<UserPresence>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPresence {
    pub database_id: String,
    pub database_label: String,
    pub present: bool,
    pub enabled: bool,
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDistributionSummary {
    pub total_users: i64,
    pub users_in_multiple_dbs: i64,
    pub users_in_single_db: i64,
    pub enabled_users: i64,
    pub disabled_users: i64,
}

// Cross-database Feature Toggle Compliance Report models
#[derive(Debug, Serialize, Deserialize)]
pub struct CrossDatabaseFeatureToggleReport {
    pub databases: Vec<DatabaseFeatureInfo>,
    pub compliance_summary: FeatureComplianceSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseFeatureInfo {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub features: DatabaseFeatures,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseFeatures {
    pub read_only: bool,
    pub no_new_users: bool,
    pub no_new_domains: bool,
    pub no_password_updates: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureComplianceSummary {
    pub total_databases: i64,
    pub databases_with_read_only: i64,
    pub databases_with_no_new_users: i64,
    pub databases_with_no_new_domains: i64,
    pub databases_with_no_password_updates: i64,
    pub fully_restricted_databases: i64,
}

// Cross-database Migration Status Report models
#[derive(Debug, Serialize, Deserialize)]
pub struct CrossDatabaseMigrationReport {
    pub databases: Vec<DatabaseMigrationInfo>,
    pub migration_summary: MigrationSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseMigrationInfo {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub migration_status: MigrationStatus,
    pub last_migration: String,
    pub migration_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MigrationStatus {
    UpToDate,
    Behind,
    Error,
    Unknown,
}

impl MigrationStatus {
    pub fn css_class(&self) -> &'static str {
        match self {
            MigrationStatus::UpToDate => "text-green-600 dark:text-green-400",
            MigrationStatus::Behind => "text-yellow-600 dark:text-yellow-400",
            MigrationStatus::Error => "text-red-600 dark:text-red-400",
            MigrationStatus::Unknown => "text-gray-400 dark:text-gray-500",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            MigrationStatus::UpToDate => "✓",
            MigrationStatus::Behind => "⚠",
            MigrationStatus::Error => "✗",
            MigrationStatus::Unknown => "?",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            MigrationStatus::UpToDate => "Up to date",
            MigrationStatus::Behind => "Behind on migrations",
            MigrationStatus::Error => "Migration error",
            MigrationStatus::Unknown => "Unknown status",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationSummary {
    pub total_databases: i64,
    pub up_to_date: i64,
    pub behind: i64,
    pub errors: i64,
    pub unknown: i64,
    pub latest_migration: String,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total_count: i64, current_page: i64, per_page: i64) -> Self {
        let total_pages = if per_page > 0 {
            (total_count + per_page - 1) / per_page // Ceiling division
        } else {
            1 // Default to 1 page if per_page is 0
        };
        let has_next = current_page < total_pages;
        let has_prev = current_page > 1;

        Self {
            items,
            total_count,
            current_page,
            per_page: per_page.max(1), // Ensure per_page is at least 1
            total_pages,
            has_next,
            has_prev,
        }
    }
}

// Wizard Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WizardStep {
    DomainConfig,
    AliasConfig,
    Review,
    Executing,
    Complete,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AliasType {
    Required, // postmaster, abuse, etc.
    Common,   // admin, webmaster, etc.
    Custom,   // user-defined
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AliasWizardData {
    pub alias: String,
    pub destination: String,
    pub enabled: bool,
    pub alias_type: AliasType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainWizardData {
    pub domain: String,
    pub transport: Option<String>, // Override common transport
    pub enabled: bool,
    pub aliases: Vec<AliasWizardData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainWizardSession {
    pub step: WizardStep,
    pub domains: Vec<DomainWizardData>,
    pub common_aliases: Vec<String>,
    pub custom_aliases: Vec<String>,
    pub common_destination: String,
    pub transport: String,
    pub enabled: bool,
    pub catchall_enabled: bool,
}

// Form models for wizard steps
#[derive(Debug, Deserialize)]
pub struct DomainConfigForm {
    pub domains: String, // Comma-separated domain string
    pub transport: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_radio_bool")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct AliasConfigForm {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duplicate_fields")]
    pub required_aliases: Vec<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duplicate_fields")]
    pub common_aliases: Vec<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duplicate_fields")]
    pub custom_aliases: Vec<String>,
    pub common_destination: String,
    #[serde(default)]
    pub alias_destinations: HashMap<String, String>, // alias -> destination
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub catchall_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct WizardConfirmForm {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub confirmed: bool,
}

// Wizard result models
#[derive(Debug, Serialize, Deserialize)]
pub struct WizardResult {
    pub success: bool,
    pub domains_created: i32,
    pub aliases_created: i32,
    pub errors: Vec<WizardError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WizardError {
    pub domain: Option<String>,
    pub alias: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WizardSummary {
    pub total_domains: i32,
    pub total_aliases: i32,
    pub domains_list: Vec<String>,
    pub aliases_list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WizardProgress {
    pub current_step: String,
    pub total_steps: i32,
    pub current_step_number: i32,
    pub message: String,
}

// MX Servers Report models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MxServersReport {
    pub domains: Vec<DomainMxStatus>,
    pub mail_servers: Vec<String>,
    pub summary: MxServersSummary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainMxStatus {
    pub domain: String,
    pub domain_id: i32,
    pub enabled: bool,
    pub is_backup: bool,
    pub mx_records: Vec<String>,
    pub mx_status: MxStatus,
    pub missing_servers: Vec<String>,
    pub unexpected_servers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MxStatus {
    Compliant,    // All MX records point to configured mail servers
    NonCompliant, // Some MX records don't point to configured mail servers
    Empty,        // No MX records found
    Error,        // Error during DNS lookup
}

impl MxStatus {
    pub fn css_class(&self) -> &'static str {
        match self {
            MxStatus::Compliant => "text-green-600 dark:text-green-400",
            MxStatus::NonCompliant => "text-red-600 dark:text-red-400",
            MxStatus::Empty => "text-yellow-600 dark:text-yellow-400",
            MxStatus::Error => "text-gray-600 dark:text-gray-400",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            MxStatus::Compliant => "✅",
            MxStatus::NonCompliant => "❌",
            MxStatus::Empty => "⚠️",
            MxStatus::Error => "❓",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            MxStatus::Compliant => "MX records point to configured mail servers",
            MxStatus::NonCompliant => "Some MX records don't point to configured mail servers",
            MxStatus::Empty => "No MX records found",
            MxStatus::Error => "Error during DNS lookup",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "compliant" => Some(MxStatus::Compliant),
            "noncompliant" | "non-compliant" | "non_compliant" => Some(MxStatus::NonCompliant),
            "empty" => Some(MxStatus::Empty),
            "error" => Some(MxStatus::Error),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MxStatus::Compliant => "compliant",
            MxStatus::NonCompliant => "noncompliant",
            MxStatus::Empty => "empty",
            MxStatus::Error => "error",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MxServersSummary {
    pub total_domains: i64,
    pub compliant_domains: i64,
    pub non_compliant_domains: i64,
    pub empty_mx_domains: i64,
    pub error_domains: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DatabaseConfig, DatabaseFeatures};
    use serde_json;

    #[test]
    fn test_domain_serialization() {
        let domain = Domain {
            pkid: 1,
            domain: "example.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            created: Some(chrono::Utc::now().naive_utc()),
            modified: Some(chrono::Utc::now().naive_utc()),
            enabled: true,
        };

        let json = serde_json::to_string(&domain).unwrap();
        let deserialized: Domain = serde_json::from_str(&json).unwrap();

        assert_eq!(domain.pkid, deserialized.pkid);
        assert_eq!(domain.domain, deserialized.domain);
        assert_eq!(domain.transport, deserialized.transport);
        assert_eq!(domain.created, deserialized.created);
        assert_eq!(domain.modified, deserialized.modified);
        assert_eq!(domain.enabled, deserialized.enabled);
    }

    #[test]
    fn test_new_domain_creation() {
        let new_domain = NewDomain {
            domain: "test.com".to_string(),
            transport: Some("smtp:localhost".to_string()),
            enabled: true,
        };

        assert_eq!(new_domain.domain, "test.com");
        assert_eq!(new_domain.transport, Some("smtp:localhost".to_string()));
        assert!(new_domain.enabled);
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: "testuser@example.com".to_string(),
            crypt: "hashed_password".to_string(),
            name: "Test User".to_string(),
            maildir: "testuser/".to_string(),
            home: "/var/spool/mail/virtual".to_string(),
            uid: 5000,
            gid: 5000,
            created: Some(chrono::Utc::now().naive_utc()),
            modified: Some(chrono::Utc::now().naive_utc()),
            enabled: true,
            change_password: false,
        };

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();

        assert_eq!(user.id, deserialized.id);
        assert_eq!(user.name, deserialized.name);
        assert_eq!(user.maildir, deserialized.maildir);
        assert_eq!(user.enabled, deserialized.enabled);
    }

    #[test]
    fn test_alias_serialization() {
        let alias = Alias {
            pkid: 1,
            mail: "test@example.com".to_string(),
            destination: "user@example.com".to_string(),
            created: Some(chrono::Utc::now().naive_utc()),
            modified: Some(chrono::Utc::now().naive_utc()),
            enabled: true,
        };

        let json = serde_json::to_string(&alias).unwrap();
        let deserialized: Alias = serde_json::from_str(&json).unwrap();

        assert_eq!(alias.pkid, deserialized.pkid);
        assert_eq!(alias.mail, deserialized.mail);
        assert_eq!(alias.destination, deserialized.destination);
        assert_eq!(alias.enabled, deserialized.enabled);
    }

    #[test]
    fn test_domain_form_deserialization() {
        let form_data = "domain=test.com&transport=smtp%3Alocalhost&enabled=on";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.domain, "test.com");
        assert_eq!(form.transport, "smtp:localhost");
        assert!(form.enabled);
    }

    #[test]
    fn test_domain_form_deserialization_without_checkboxes() {
        let form_data = "domain=test.com&transport=smtp%3Alocalhost";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.domain, "test.com");
        assert_eq!(form.transport, "smtp:localhost");
        assert!(!form.enabled); // Default value
    }

    #[test]
    fn test_user_form_deserialization() {
        let form_data =
            "id=testuser@example.com&password=secret123&name=Test+User&maildir=testdir&home=/var/spool/mail/virtual&enabled=on";
        let form: UserForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.id, "testuser@example.com");
        assert_eq!(form.password, "secret123");
        assert_eq!(form.name, "Test User");
        assert_eq!(form.maildir, "testdir");
        assert!(form.enabled);
    }

    #[test]
    fn test_alias_form_deserialization() {
        let form_data = "mail=test%40example.com&destination=user%40example.com&enabled=on";
        let form: AliasForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.mail, "test@example.com");
        assert_eq!(form.destination, "user@example.com");
        assert!(form.enabled);
    }

    #[test]
    fn test_checkbox_deserialization() {
        // Test form data deserialization with "on" value
        let form_data = "domain=test.com&transport=smtp:localhost&enabled=on";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert!(form.enabled);

        // Test form data deserialization with "true" value
        let form_data = "domain=test.com&transport=smtp:localhost&enabled=true";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert!(form.enabled);

        // Test form data deserialization with "1" value
        let form_data = "domain=test.com&transport=smtp:localhost&enabled=1";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert!(form.enabled);

        // Test form data deserialization with missing values
        let form_data = "domain=test.com&transport=smtp:localhost";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert!(!form.enabled);

        // Test form data deserialization with "off" value
        let form_data = "domain=test.com&transport=smtp:localhost&enabled=off";
        let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
        assert!(!form.enabled);
    }

    #[test]
    fn test_system_stats_serialization() {
        let stats = SystemStats {
            total_domains: 5,
            enabled_domains: 3,
            disabled_domains: 2,
            recent_domains: 1,
            total_users: 10,
            enabled_users: 8,
            disabled_users: 2,
            recent_users: 3,
            total_aliases: 15,
            enabled_aliases: 12,
            disabled_aliases: 3,
            recent_aliases: 4,
            total_backups: 2,
            enabled_backups: 1,
            disabled_backups: 1,
            recent_backups: 0,
            total_relays: 3,
            enabled_relays: 2,
            disabled_relays: 1,
            recent_relays: 1,
            total_relocated: 4,
            enabled_relocated: 3,
            disabled_relocated: 1,
            recent_relocated: 2,
            total_clients: 6,
            enabled_clients: 5,
            disabled_clients: 1,
            recent_clients: 2,
            total_quota: 1000000000,
            used_quota: 250000000,
            quota_usage_percent: 25.0,
            enabled_domains_and_backups: 4,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: SystemStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.total_domains, deserialized.total_domains);
        assert_eq!(stats.total_users, deserialized.total_users);
        assert_eq!(stats.total_aliases, deserialized.total_aliases);
        assert_eq!(stats.total_backups, deserialized.total_backups);
        assert_eq!(stats.total_relays, deserialized.total_relays);
        assert_eq!(stats.total_relocated, deserialized.total_relocated);
        assert_eq!(stats.total_clients, deserialized.total_clients);
        assert_eq!(stats.total_quota, deserialized.total_quota);
        assert_eq!(stats.used_quota, deserialized.used_quota);
    }

    #[test]
    fn test_domain_stats_serialization() {
        let stats = DomainStats {
            domain_id: 1,
            domain: "example.com".to_string(),
            user_count: 10,
            alias_count: 20,
            relay_count: 5,
            relay_enabled_count: 4,
            relay_disabled_count: 1,
            relocated_count: 3,
            relocated_enabled_count: 2,
            relocated_disabled_count: 1,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: DomainStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.domain_id, deserialized.domain_id);
        assert_eq!(stats.domain, deserialized.domain);
        assert_eq!(stats.user_count, deserialized.user_count);
        assert_eq!(stats.alias_count, deserialized.alias_count);
        assert_eq!(stats.relay_count, deserialized.relay_count);
        assert_eq!(stats.relay_enabled_count, deserialized.relay_enabled_count);
        assert_eq!(
            stats.relay_disabled_count,
            deserialized.relay_disabled_count
        );
        assert_eq!(stats.relocated_count, deserialized.relocated_count);
        assert_eq!(
            stats.relocated_enabled_count,
            deserialized.relocated_enabled_count
        );
        assert_eq!(
            stats.relocated_disabled_count,
            deserialized.relocated_disabled_count
        );
    }

    #[test]
    fn test_backup_serialization() {
        let backup = Backup {
            pkid: 1,
            domain: "backup.example.com".to_string(),
            transport: Some("smtp:[]".to_string()),
            created: Some(chrono::Utc::now().naive_utc()),
            modified: Some(chrono::Utc::now().naive_utc()),
            enabled: true,
        };

        let json = serde_json::to_string(&backup).unwrap();
        let deserialized: Backup = serde_json::from_str(&json).unwrap();

        assert_eq!(backup.pkid, deserialized.pkid);
        assert_eq!(backup.domain, deserialized.domain);
        assert_eq!(backup.transport, deserialized.transport);
        assert_eq!(backup.enabled, deserialized.enabled);
    }

    #[test]
    fn test_new_backup_creation() {
        let new_backup = NewBackup {
            domain: "backup.test.com".to_string(),
            transport: Some("smtp:relay.test.com".to_string()),
            enabled: true,
        };

        assert_eq!(new_backup.domain, "backup.test.com");
        assert_eq!(
            new_backup.transport,
            Some("smtp:relay.test.com".to_string())
        );
        assert!(new_backup.enabled);
    }

    #[test]
    fn test_backup_form_deserialization() {
        let form_data = "domain=backup.test.com&transport=smtp%3Arelay.test.com&enabled=on";
        let form: BackupForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.domain, "backup.test.com");
        assert_eq!(form.transport, "smtp:relay.test.com");
        assert!(form.enabled);
    }

    #[test]
    fn test_backup_form_deserialization_without_checkboxes() {
        let form_data = "domain=backup.test.com&transport=smtp%3Arelay.test.com";
        let form: BackupForm = serde_urlencoded::from_str(form_data).unwrap();

        assert_eq!(form.domain, "backup.test.com");
        assert_eq!(form.transport, "smtp:relay.test.com");
        assert!(!form.enabled); // Default value
    }

    #[test]
    fn test_paginated_result_empty() {
        let paginated: PaginatedResult<i32> = PaginatedResult::new(vec![], 0, 1, 10);
        assert_eq!(paginated.items.len(), 0);
        assert_eq!(paginated.total_count, 0);
        assert_eq!(paginated.current_page, 1);
        assert_eq!(paginated.per_page, 10);
        assert_eq!(paginated.total_pages, 0);
        assert!(!paginated.has_next);
        assert!(!paginated.has_prev);
    }

    #[test]
    fn test_paginated_result_single_page() {
        let paginated = PaginatedResult::new(vec![1, 2, 3], 3, 1, 10);
        assert_eq!(paginated.total_pages, 1);
        assert!(!paginated.has_next);
        assert!(!paginated.has_prev);
    }

    #[test]
    fn test_paginated_result_multiple_pages() {
        let paginated = PaginatedResult::new((1..21).collect(), 100, 2, 20);
        assert_eq!(paginated.total_pages, 5);
        assert!(paginated.has_next);
        assert!(paginated.has_prev);
        assert_eq!(paginated.current_page, 2);
        assert_eq!(paginated.per_page, 20);
    }

    #[test]
    fn test_paginated_result_last_page() {
        let paginated = PaginatedResult::new((81..101).collect(), 100, 5, 20);
        assert_eq!(paginated.total_pages, 5);
        assert!(!paginated.has_next);
        assert!(paginated.has_prev);
    }

    #[test]
    fn test_paginated_result_out_of_bounds_page() {
        let paginated: PaginatedResult<i32> = PaginatedResult::new(vec![], 30, 5, 10);
        assert_eq!(paginated.total_pages, 3);
        assert!(!paginated.has_next);
        assert!(paginated.has_prev);
    }

    #[test]
    fn test_table_qualified_field_mapping() {
        use std::collections::HashMap;

        // Create a database config with table-qualified field mappings
        let mut field_map = HashMap::new();
        field_map.insert("users.id".to_string(), "user_id".to_string());
        field_map.insert("users.enabled".to_string(), "is_active".to_string());
        field_map.insert("domains.id".to_string(), "domain_id".to_string());
        field_map.insert("domains.enabled".to_string(), "is_enabled".to_string());
        field_map.insert("aliases.id".to_string(), "alias_id".to_string());

        let db_config = DatabaseConfig {
            id: "test".to_string(),
            label: "Test Database".to_string(),
            url: "mysql://test:test@localhost/test".to_string(),
            features: DatabaseFeatures::default(),
            field_map,
            connection_pool: crate::config::ConnectionPoolConfig::default(),
        };

        // Test table-qualified field mapping
        assert_eq!(db_config.field_for_table("users", "id"), "user_id");
        assert_eq!(db_config.field_for_table("users", "enabled"), "is_active");
        assert_eq!(db_config.field_for_table("domains", "id"), "domain_id");
        assert_eq!(
            db_config.field_for_table("domains", "enabled"),
            "is_enabled"
        );
        assert_eq!(db_config.field_for_table("aliases", "id"), "alias_id");

        // Test fallback to original field name when not mapped
        assert_eq!(db_config.field_for_table("users", "name"), "name");
        assert_eq!(db_config.field_for_table("domains", "domain"), "domain");

        // Test backward compatibility with simple field names
        assert_eq!(db_config.field("users.id"), "user_id");
        assert_eq!(db_config.field("domains.enabled"), "is_enabled");
        assert_eq!(db_config.field("unknown"), "unknown");
    }
}

// Duplicate Domain Wizard Models
#[derive(Debug, Deserialize)]
pub struct DuplicateDomainForm {
    #[serde(default)]
    pub source_domain: String,
    #[serde(default)]
    pub new_domain: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_transport")]
    pub transport: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub enabled: bool,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub duplicate_aliases: bool,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub duplicate_relays: bool,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub confirmed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateDomainSession {
    pub step: DuplicateWizardStep,
    pub source_domain: Option<Domain>,
    pub source_is_backup: bool, // Track if source domain is a backup domain
    pub new_domain: String,
    pub transport: String,
    pub enabled: bool,
    pub duplicate_aliases: bool,
    pub duplicate_relays: bool,
    pub aliases_to_duplicate: Vec<Alias>,
    pub relays_to_duplicate: Vec<Relay>,
    pub target_is_backup: Option<bool>, // Future: allow user to choose target domain type
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DuplicateWizardStep {
    DomainSelection,
    Configuration,
    Review,
    Executing,
    Complete,
}
