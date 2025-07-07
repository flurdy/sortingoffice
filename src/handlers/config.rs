use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Html,
};
use crate::{AppState, config::Config, i18n::get_translation};
use crate::templates::layout::BaseTemplate;
use crate::templates::config::ConfigTemplate;
use askama::Template;

pub async fn view_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    
    // Get translations
    let title = get_translation(&state, &locale, "config-title").await;
    let description = get_translation(&state, &locale, "config-description").await;
    let required_aliases_header = get_translation(&state, &locale, "config-required-aliases-header").await;
    let common_aliases_header = get_translation(&state, &locale, "config-common-aliases-header").await;
    let domain_overrides_header = get_translation(&state, &locale, "config-domain-overrides-header").await;
    let save_button = get_translation(&state, &locale, "config-save-button").await;
    let cancel_button = get_translation(&state, &locale, "config-cancel-button").await;
    let add_required_alias_button = get_translation(&state, &locale, "config-add-required-alias-button").await;
    let add_common_alias_button = get_translation(&state, &locale, "config-add-common-alias-button").await;
    let remove_alias_button = get_translation(&state, &locale, "config-remove-alias-button").await;
    let promote_button = get_translation(&state, &locale, "config-promote-button").await;
    let demote_button = get_translation(&state, &locale, "config-demote-button").await;

    // Load current configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Error loading config: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create the config template
    let domain_overrides_vec: Vec<(&String, &crate::config::DomainOverride)> = config.domain_overrides.iter().collect();
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
    let template = match BaseTemplate::with_i18n(title, content, &state, &locale).await {
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
