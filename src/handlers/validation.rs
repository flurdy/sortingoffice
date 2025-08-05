use crate::AppState;
use axum::http::HeaderMap;
use axum::response::Html;

// Import functions from utils for now
use crate::i18n::get_translation;

// Import HTTP helper functions directly
use crate::handlers::http_helpers::{get_user_locale, is_htmx_request};

// Import translation functions directly
use crate::handlers::templates::render_template_safely;
use crate::handlers::translations::{get_entity_error_translations, get_entity_form_translations};

/// Generic form validation with error handling
pub async fn validate_form_and_handle_error<F, V, E>(
    _state: &AppState,
    form: &F,
    validator: V,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
    error_handler: E,
) -> Result<(), Html<String>>
where
    F: Clone,
    V: FnOnce(&F) -> Result<(), String>,
    E: FnOnce(
        &AppState,
        &str,
        &HeaderMap,
        F,
        &str,
        bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Html<String>> + Send>>,
{
    match validator(form) {
        Ok(()) => Ok(()),
        Err(error_message) => {
            let form_clone = form.clone();
            let _ = error_handler(state, locale, headers, form_clone, &error_message, true).await;
            Err(Html(error_message))
        }
    }
}

/// Handle entity not found errors with consistent error handling
pub async fn handle_entity_not_found(
    state: &AppState,
    headers: &HeaderMap,
    entity_type: &str,
    _error_key: &str,
) -> Html<String> {
    let locale = get_user_locale(headers);
    let translations = get_entity_error_translations(state, &locale, entity_type).await;
    let title = translations
        .get("not_found_title")
        .unwrap_or(&"Not Found".to_string())
        .clone();

    let content = format!("{} not found", entity_type);

    let current_db_id = crate::handlers::auth::get_selected_database(headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());

    match crate::templates::error::ErrorTemplate::new(
        &title,
        &content,
        state,
        &locale,
        &current_db_label,
        &current_db_id,
    )
    .await
    {
        Ok(error_template) => match render_template_safely(&error_template) {
            Ok(content) => Html(content),
            Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
        },
        Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
    }
}

/// Validate alias form field with error handling
pub async fn validate_alias_form_field<F>(
    state: &AppState,
    headers: &HeaderMap,
    form: &crate::models::AliasForm,
    validator: F,
    error_key: &str,
) -> Result<(), Html<String>>
where
    F: FnOnce(&crate::models::AliasForm) -> Result<(), crate::validation::ValidationError>,
{
    match validator(form) {
        Ok(_) => Ok(()),
        Err(_) => {
            let locale = get_user_locale(headers);
            let error_msg = get_translation(state, &locale, error_key).await;

            // Get form translations
            let form_translations = get_entity_form_translations(state, &locale, "aliases").await;

            // Get individual translations like the rendering module does
            let mail_address =
                crate::i18n::get_translation(state, &locale, "aliases-field-mail").await;
            let destination =
                crate::i18n::get_translation(state, &locale, "aliases-field-destination").await;
            let placeholder_mail =
                crate::i18n::get_translation(state, &locale, "aliases-placeholder-mail").await;
            let placeholder_destination =
                crate::i18n::get_translation(state, &locale, "aliases-placeholder-destination")
                    .await;
            let tooltip_mail =
                crate::i18n::get_translation(state, &locale, "aliases-field-mail-help").await;
            let tooltip_destination =
                crate::i18n::get_translation(state, &locale, "aliases-field-destination-help")
                    .await;
            let active = crate::i18n::get_translation(state, &locale, "form-enabled").await;
            let tooltip_active =
                crate::i18n::get_translation(state, &locale, "aliases-tooltip-enabled").await;
            let cancel = crate::i18n::get_translation(state, &locale, "form-cancel").await;
            let update_alias =
                crate::i18n::get_translation(state, &locale, "aliases-update-alias").await;
            let create_alias =
                crate::i18n::get_translation(state, &locale, "aliases-create-alias").await;

            let content_template = crate::templates::aliases::AliasFormTemplate {
                title: &form_translations["aliases-add-title"],
                alias: None,
                form: form.clone(),
                error: Some(error_msg),
                return_url: form.return_url.clone(),
                edit_alias: &form_translations["aliases-edit-alias"],
                new_alias: &form_translations["aliases-new-alias"],
                form_error: &form_translations["form-error"],
                mail_address: &mail_address,
                destination: &destination,
                placeholder_mail: &placeholder_mail,
                placeholder_destination: &placeholder_destination,
                tooltip_mail: &tooltip_mail,
                tooltip_destination: &tooltip_destination,
                active: &active,
                tooltip_active: &tooltip_active,
                cancel: &cancel,
                update_alias: &update_alias,
                create_alias: &create_alias,
            };

            let error_html = crate::handlers::utils::render_form_template(
                content_template,
                state,
                &locale,
                headers,
                form_translations["aliases-add-title"].clone(),
            )
            .await;
            Err(error_html)
        }
    }
}

/// Validate user form field with error handling
pub async fn validate_user_form_field<F>(
    state: &AppState,
    headers: &HeaderMap,
    form: &crate::models::UserForm,
    validator: F,
    error_key: &str,
) -> Result<(), Html<String>>
where
    F: FnOnce(&crate::models::UserForm) -> Result<(), crate::validation::ValidationError>,
{
    match validator(form) {
        Ok(_) => Ok(()),
        Err(_) => {
            let locale = get_user_locale(headers);
            let error_msg = get_translation(state, &locale, error_key).await;

            // Build user form template with error
            let form_template = crate::handlers::users::build_user_form_template(
                state,
                &locale,
                None,
                form.clone(),
                Some(error_msg),
            )
            .await;
            let content = match render_template_safely(form_template) {
                Ok(content) => content,
                Err(_) => {
                    return Err(crate::handlers::errors::render_500_page(state, headers).await)
                }
            };

            if is_htmx_request(headers) {
                Err(Html(content))
            } else {
                let (current_db_label, current_db_id) =
                    crate::handlers::utils::get_current_db_info(state, headers).await;
                let template = crate::templates::layout::BaseTemplate::with_i18n(
                    get_translation(state, &locale, "users-new-user").await,
                    content,
                    state,
                    &locale,
                    current_db_label,
                    current_db_id,
                )
                .await
                .unwrap();
                Err(match render_template_safely(template) {
                    Ok(content) => Html(content),
                    Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
                })
            }
        }
    }
}
