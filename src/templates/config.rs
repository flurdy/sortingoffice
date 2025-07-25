use crate::config::Config;
use askama::Template;

#[derive(Template)]
#[template(path = "config.html", escape = "html")]
pub struct ConfigTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub domain_overrides_header: &'a str,
    pub domain_overrides_description: &'a str,
    pub cancel_button: &'a str,
    pub add_domain_override_button: &'a str,
    pub remove_domain_button: &'a str,
    pub required_aliases_label: &'a str,
    pub common_aliases_label: &'a str,
    pub remove_button: &'a str,
    pub placeholder_domain: &'a str,
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
    // Database Feature Toggles
    pub database_features_header: &'a str,
    pub database_features_description: &'a str,
    pub database_disabled_badge: &'a str,
    pub config: &'a Config,
    pub domain_overrides_vec: Vec<(&'a String, &'a crate::config::DomainOverride)>,
}
