use crate::config::Config;
use askama::Template;

#[derive(Template)]
#[template(path = "config.html", escape = "html")]
pub struct ConfigTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    // Global Feature Toggles
    pub global_features_header: &'a str,
    pub global_features_description: &'a str,
    pub feature_read_only: &'a str,
    pub feature_no_new_users: &'a str,
    pub feature_no_new_domains: &'a str,
    pub feature_no_password_updates: &'a str,
    pub feature_database_disabled: &'a str,
    pub status_enabled: &'a str,
    pub status_disabled: &'a str,
    // Table column headers
    pub table_feature_column: &'a str,
    pub table_status_column: &'a str,
    // Database Feature Toggles
    pub database_features_header: &'a str,
    pub database_features_description: &'a str,
    pub database_disabled_badge: &'a str,
    // System Management section
    pub system_management_header: &'a str,
    pub system_management_description: &'a str,
    pub cache_management_header: &'a str,
    pub cache_management_description: &'a str,
    pub cache_management_button: &'a str,
    pub config: &'a Config,
}
