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
    let content_template = ConfigTemplate {
        title: &form_translations["config-title"],
        description: &form_translations["config-description"],
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
