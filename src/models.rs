use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Text};
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_checkbox<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(matches!(
        opt.as_deref(),
        Some("on") | Some("true") | Some("1")
    ))
}

#[derive(Debug, Serialize, Deserialize)]
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
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    pub created: NaiveDateTime,
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    pub modified: NaiveDateTime,
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
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Bool)]
    pub enabled: bool,
    pub crypt: String,
    pub name: String,
    pub maildir: String,
    pub home: String,
    pub uid: u16,
    pub gid: u16,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
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
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    pub created: NaiveDateTime,
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    pub modified: NaiveDateTime,
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
#[derive(Deserialize, Debug)]
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
}

// Statistics models
#[derive(Debug, Serialize, Deserialize)]
pub struct DomainStats {
    pub domain: String,
    pub user_count: i64,
    pub alias_count: i64,
    pub total_quota: i64,
    pub used_quota: i64,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = backups)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Backup {
    pub pkid: i32,
    pub domain: String,
    pub transport: Option<String>,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
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
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = relays)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Relay {
    pub pkid: i32,
    pub recipient: String,
    pub status: String,
    pub enabled: bool,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
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
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = relocated)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Relocated {
    pub pkid: i32,
    pub old_address: String,
    pub new_address: String,
    pub enabled: bool,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
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
    pub id: i32,
    pub client: String,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub enabled: bool,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct CatchAllReport {
    pub domain: String,
    pub catch_all_alias: String,
    pub catch_all_destination: String,
    pub required_aliases: Vec<RequiredAlias>,
}

#[derive(Debug, Serialize, Deserialize)]
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
        // Try to read from environment variables first
        let required_from_env = std::env::var("REQUIRED_ALIASES").ok();
        let common_from_env = std::env::var("COMMON_ALIASES").ok();

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
#[derive(Debug, Serialize, Deserialize)]
pub struct DomainAliasReport {
    pub domain: String,
    pub has_catch_all: bool,
    pub catch_all_alias: Option<String>,
    pub catch_all_destination: Option<String>,
    pub required_aliases: Vec<RequiredAlias>,
    pub missing_required_aliases: Vec<String>,
    pub missing_common_aliases: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasReport {
    pub domains_with_catch_all: Vec<DomainAliasReport>,
    pub domains_without_catch_all: Vec<DomainAliasReport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainAliasMatrixReport {
    pub domains: Vec<DomainAliasMatrixRow>,
    pub required_aliases_list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainAliasMatrixRow {
    pub domain: String,
    pub catch_all_status: AliasStatus,
    pub required_aliases: Vec<RequiredAliasMatrixItem>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub per_page: Option<i64>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct OrphanedAliasReport {
    pub orphaned_aliases: Vec<OrphanedAlias>,
    pub orphaned_users: Vec<OrphanedUser>,
    pub users_without_aliases: Vec<UserWithoutAlias>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrphanedAlias {
    pub mail: String,
    pub destination: String,
    pub domain: String,
    pub enabled: bool,
    pub created: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrphanedUser {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub enabled: bool,
    pub created: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithoutAlias {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub enabled: bool,
    pub created: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalForwarderReport {
    pub external_forwarders: Vec<ExternalForwarder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalForwarder {
    pub mail: String,
    pub destination: String,
    pub domain: String,
    pub enabled: bool,
    pub created: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissingAliasReport {
    pub domains_missing_aliases: Vec<DomainMissingAliases>,
}

#[derive(Debug, Serialize, Deserialize)]
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
