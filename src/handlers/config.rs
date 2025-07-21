use crate::templates::config::ConfigTemplate;
use crate::templates::layout::BaseTemplate;
use crate::{config::Config, AppState};
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

    // Use helper functions to fetch translations in batches
    let form_translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
        &[
            "config-title",
            "config-description",
            "config-required-aliases-header",
            "config-common-aliases-header",
            "config-domain-overrides-header",
            "config-save-button",
            "config-cancel-button",
            "config-add-required-alias-button",
            "config-add-common-alias-button",
            "config-remove-alias-button",
            "config-promote-button",
            "config-demote-button",
            "config-required-aliases-description",
            "config-common-aliases-description",
            "config-domain-overrides-description",
            "config-add-domain-override-button",
            "config-remove-domain-button",
            "config-required-aliases-label",
            "config-common-aliases-label",
            "config-remove-button",
            "config-add-alias-button",
            "config-placeholder-required-alias",
            "config-placeholder-common-alias",
            "config-placeholder-domain",
            "config-placeholder-domain-alias",
            "config-global-features-header",
            "config-global-features-description",
            "config-feature-read-only",
            "config-feature-no-new-users",
            "config-feature-no-new-domains",
            "config-feature-no-password-updates",
            "config-feature-database-disabled",
            "config-status-enabled",
            "config-status-disabled",
            "config-database-features-header",
            "config-database-features-description",
            "config-database-disabled-badge",
        ],
    )
    .await;

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
        title: &form_translations["config-title"],
        description: &form_translations["config-description"],
        required_aliases_header: &form_translations["config-required-aliases-header"],
        common_aliases_header: &form_translations["config-common-aliases-header"],
        domain_overrides_header: &form_translations["config-domain-overrides-header"],
        save_button: &form_translations["config-save-button"],
        cancel_button: &form_translations["config-cancel-button"],
        add_required_alias_button: &form_translations["config-add-required-alias-button"],
        add_common_alias_button: &form_translations["config-add-common-alias-button"],
        remove_alias_button: &form_translations["config-remove-alias-button"],
        promote_button: &form_translations["config-promote-button"],
        demote_button: &form_translations["config-demote-button"],
        required_aliases_description: &form_translations["config-required-aliases-description"],
        common_aliases_description: &form_translations["config-common-aliases-description"],
        domain_overrides_description: &form_translations["config-domain-overrides-description"],
        add_domain_override_button: &form_translations["config-add-domain-override-button"],
        remove_domain_button: &form_translations["config-remove-domain-button"],
        required_aliases_label: &form_translations["config-required-aliases-label"],
        common_aliases_label: &form_translations["config-common-aliases-label"],
        remove_button: &form_translations["config-remove-button"],
        add_alias_button: &form_translations["config-add-alias-button"],
        placeholder_required_alias: &form_translations["config-placeholder-required-alias"],
        placeholder_common_alias: &form_translations["config-placeholder-common-alias"],
        placeholder_domain: &form_translations["config-placeholder-domain"],
        placeholder_domain_alias: &form_translations["config-placeholder-domain-alias"],
        // Global Feature Toggles
        global_features_header: &form_translations["config-global-features-header"],
        global_features_description: &form_translations["config-global-features-description"],
        feature_read_only: &form_translations["config-feature-read-only"],
        feature_no_new_users: &form_translations["config-feature-no-new-users"],
        feature_no_new_domains: &form_translations["config-feature-no-new-domains"],
        feature_no_password_updates: &form_translations["config-feature-no-password-updates"],
        feature_database_disabled: &form_translations["config-feature-database-disabled"],
        status_enabled: &form_translations["config-status-enabled"],
        status_disabled: &form_translations["config-status-disabled"],
        // Database Feature Toggles
        database_features_header: &form_translations["config-database-features-header"],
        database_features_description: &form_translations["config-database-features-description"],
        database_disabled_badge: &form_translations["config-database-disabled-badge"],
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
        form_translations["config-title"].clone(),
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
