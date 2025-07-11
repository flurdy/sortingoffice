use crate::templates::config::ConfigTemplate;
use crate::templates::layout::BaseTemplate;
use crate::{config::Config, i18n::get_translation, AppState};
use askama::Template;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Html,
};

pub async fn view_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get translations
    let title = get_translation(&state, &locale, "config-title").await;
    let description = get_translation(&state, &locale, "config-description").await;
    let required_aliases_header =
        get_translation(&state, &locale, "config-required-aliases-header").await;
    let common_aliases_header =
        get_translation(&state, &locale, "config-common-aliases-header").await;
    let domain_overrides_header =
        get_translation(&state, &locale, "config-domain-overrides-header").await;
    let save_button = get_translation(&state, &locale, "config-save-button").await;
    let cancel_button = get_translation(&state, &locale, "config-cancel-button").await;
    let add_required_alias_button =
        get_translation(&state, &locale, "config-add-required-alias-button").await;
    let add_common_alias_button =
        get_translation(&state, &locale, "config-add-common-alias-button").await;
    let remove_alias_button = get_translation(&state, &locale, "config-remove-alias-button").await;
    let promote_button = get_translation(&state, &locale, "config-promote-button").await;
    let demote_button = get_translation(&state, &locale, "config-demote-button").await;
    let required_aliases_description =
        get_translation(&state, &locale, "config-required-aliases-description").await;
    let common_aliases_description =
        get_translation(&state, &locale, "config-common-aliases-description").await;
    let domain_overrides_description =
        get_translation(&state, &locale, "config-domain-overrides-description").await;
    let add_domain_override_button =
        get_translation(&state, &locale, "config-add-domain-override-button").await;
    let remove_domain_button =
        get_translation(&state, &locale, "config-remove-domain-button").await;
    let required_aliases_label =
        get_translation(&state, &locale, "config-required-aliases-label").await;
    let common_aliases_label =
        get_translation(&state, &locale, "config-common-aliases-label").await;
    let remove_button = get_translation(&state, &locale, "config-remove-button").await;
    let add_alias_button = get_translation(&state, &locale, "config-add-alias-button").await;
    let placeholder_required_alias =
        get_translation(&state, &locale, "config-placeholder-required-alias").await;
    let placeholder_common_alias =
        get_translation(&state, &locale, "config-placeholder-common-alias").await;
    let placeholder_domain = get_translation(&state, &locale, "config-placeholder-domain").await;
    let placeholder_domain_alias =
        get_translation(&state, &locale, "config-placeholder-domain-alias").await;

    // Global Feature Toggles translations
    let global_features_header = get_translation(&state, &locale, "config-global-features-header").await;
    let global_features_description = get_translation(&state, &locale, "config-global-features-description").await;
    let feature_read_only = get_translation(&state, &locale, "config-feature-read-only").await;
    let feature_no_new_users = get_translation(&state, &locale, "config-feature-no-new-users").await;
    let feature_no_new_domains = get_translation(&state, &locale, "config-feature-no-new-domains").await;
    let feature_no_password_updates = get_translation(&state, &locale, "config-feature-no-password-updates").await;
    let feature_database_disabled = get_translation(&state, &locale, "config-feature-database-disabled").await;
    let status_enabled = get_translation(&state, &locale, "config-status-enabled").await;
    let status_disabled = get_translation(&state, &locale, "config-status-disabled").await;

    // Database Feature Toggles translations
    let database_features_header = get_translation(&state, &locale, "config-database-features-header").await;
    let database_features_description = get_translation(&state, &locale, "config-database-features-description").await;
    let database_disabled_badge = get_translation(&state, &locale, "config-database-disabled-badge").await;

    // Load current configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Error loading config: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the config template
    let domain_overrides_vec: Vec<(&String, &crate::config::DomainOverride)> =
        config.domain_overrides.iter().collect();
    let content_template = ConfigTemplate {
        title: &title,
        description: &description,
        required_aliases_header: &required_aliases_header,
        common_aliases_header: &common_aliases_header,
        domain_overrides_header: &domain_overrides_header,
        save_button: &save_button,
        cancel_button: &cancel_button,
        add_required_alias_button: &add_required_alias_button,
        add_common_alias_button: &add_common_alias_button,
        remove_alias_button: &remove_alias_button,
        promote_button: &promote_button,
        demote_button: &demote_button,
        required_aliases_description: &required_aliases_description,
        common_aliases_description: &common_aliases_description,
        domain_overrides_description: &domain_overrides_description,
        add_domain_override_button: &add_domain_override_button,
        remove_domain_button: &remove_domain_button,
        required_aliases_label: &required_aliases_label,
        common_aliases_label: &common_aliases_label,
        remove_button: &remove_button,
        add_alias_button: &add_alias_button,
        placeholder_required_alias: &placeholder_required_alias,
        placeholder_common_alias: &placeholder_common_alias,
        placeholder_domain: &placeholder_domain,
        placeholder_domain_alias: &placeholder_domain_alias,
        // Global Feature Toggles
        global_features_header: &global_features_header,
        global_features_description: &global_features_description,
        feature_read_only: &feature_read_only,
        feature_no_new_users: &feature_no_new_users,
        feature_no_new_domains: &feature_no_new_domains,
        feature_no_password_updates: &feature_no_password_updates,
        feature_database_disabled: &feature_database_disabled,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        // Database Feature Toggles
        database_features_header: &database_features_header,
        database_features_description: &database_features_description,
        database_disabled_badge: &database_disabled_badge,
        config: &config,
        domain_overrides_vec,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Error rendering config template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the base template
    // Get current database id from session/cookie or default
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    // Get current database label from db_manager
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());

    let template = match BaseTemplate::with_i18n(
        title,
        content,
        &state,
        &locale,
        current_db_label,
        current_db_id,
    )
    .await
    {
        Ok(template) => template,
        Err(e) => {
            tracing::error!("Error creating base template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match template.render() {
        Ok(content) => Ok(Html(content)),
        Err(e) => {
            tracing::error!("Error rendering final template: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
