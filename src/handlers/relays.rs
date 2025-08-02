use crate::templates::layout::BaseTemplate;
use crate::templates::relays::*;
use crate::{db, i18n::get_translation, models::*, AppState};
use askama::Template;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
    Form,
};
use diesel::result::Error;
use tracing::{debug, error, info};

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some_and(|v| v == "true")
}

// List all relays
pub async fn list_relays(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    debug!("Handling relays list request");

    // Check if relays table is available for this database
    if !state.config.is_relays_available(&current_db_id) {
        let not_available_msg = get_translation(&state, &locale, "relays-not-available").await;
        return Html(not_available_msg);
    }

    let relays = match db::get_relays(&pool) {
        Ok(relays) => {
            info!("Successfully retrieved {} relays", relays.len());
            relays
        }
        Err(e) => {
            error!("Failed to retrieve relays: {:?}", e);
            vec![]
        }
    };

    // Get all translations using consolidated helper functions
    let translations =
        crate::handlers::utils::get_entity_all_translations(&state, &locale, "relays").await;

    let content_template = RelayListTemplate {
        title: &translations["relays-title"],
        add_relay: &translations["relays-add"],
        table_header_recipient: &translations["relays-table-header-recipient"],
        table_header_status: &translations["relays-table-header-status"],
        table_header_enabled: &translations["relays-table-header-enabled"],
        table_header_actions: &translations["relays-table-header-actions"],
        status_enabled: &translations["status-enabled"],
        status_disabled: &translations["status-disabled"],
        status_ok: &translations["status-ok"],
        status_reject: &translations["status-reject"],
        action_view: &translations["action-view"],
        action_enable: &translations["action-enable"],
        action_disable: &translations["action-disable"],
        delete_confirm: &translations["relays-delete-confirm"],
        empty_title: &translations["relays-empty-title"],
        empty_description: &translations["relays-empty-description"],
        relays,
        relays_list_description: &translations["relays-list-description"],
    };

    let content = match content_template.render() {
        Ok(content) => {
            debug!(
                "Template rendered successfully, content length: {}",
                content.len()
            );
            content
        }
        Err(e) => {
            error!("Failed to render template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let current_db_id = crate::handlers::auth::get_selected_database(&headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = BaseTemplate::with_i18n(
            translations["relays-title"].clone(),
            content,
            &state,
            &locale,
            current_db_label,
            current_db_id,
        )
        .await
        .unwrap();

        Html(template.render().unwrap())
    }
}

// Show a specific relay
pub async fn show_relay(
    State(state): State<AppState>,
    Path(relay_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    debug!("Handling relay show request for ID: {}", relay_id);

    // Check if relays table is available for this database
    if !state.config.is_relays_available(&current_db_id) {
        let not_available_msg = get_translation(&state, &locale, "relays-not-available").await;
        return Html(not_available_msg);
    }

    let relay = match db::get_relay(&pool, relay_id) {
        Ok(relay) => relay,
        Err(_) => {
            return crate::handlers::utils::handle_entity_not_found(
                &state,
                &headers,
                "relays",
                "relays-not-found",
            )
            .await;
        }
    };

    // Get all translations using consolidated helper functions
    let translations =
        crate::handlers::utils::get_entity_all_translations(&state, &locale, "relays").await;

    let content_template = RelayShowTemplate {
        title: &translations["relays-title"],
        relay,
        action_edit: &translations["action-edit"],
        action_enable: &translations["action-enable"],
        action_disable: &translations["action-disable"],
        action_delete: &translations["action-delete"],
        delete_confirm: &translations["relays-delete-confirm"],
        back_to_list: &translations["relays-back-to-list"],
        field_id: &translations["relays-field-id"],
        field_recipient: &translations["relays-field-recipient"],
        field_status: &translations["relays-field-status"],
        field_enabled: &translations["relays-field-enabled"],
        field_created: &translations["relays-field-created"],
        field_modified: &translations["relays-field-modified"],
        status_enabled: &translations["status-enabled"],
        status_disabled: &translations["status-disabled"],
        status_ok: &translations["status-ok"],
        status_reject: &translations["status-reject"],
        view_edit_settings: &translations["relays-view-edit-settings"],
        relay_show_title: &translations["relays-show-title-label"],
        relay_info_title: &translations["relays-info-title"],
        relay_info_description: &translations["relays-info-description"],
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to render template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let current_db_id = crate::handlers::auth::get_selected_database(&headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = BaseTemplate::with_i18n(
            translations["relays-title"].clone(),
            content,
            &state,
            &locale,
            current_db_label,
            current_db_id,
        )
        .await
        .unwrap();

        Html(template.render().unwrap())
    }
}

// Show form for creating a new relay
pub async fn create_form(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relay create form request");

    let form = RelayForm {
        recipient: "".to_string(),
        status: "".to_string(),
        enabled: true,
    };

    // Use helper functions to fetch translations in batches
    let form_translations =
        crate::handlers::utils::get_entity_form_translations(&state, &locale, "relays").await;
    let field_translations = crate::handlers::utils::get_field_translations(
        &state,
        &locale,
        "relays",
        &["recipient", "status", "enabled"],
    )
    .await;

    let content_template = RelayFormTemplate {
        title: &form_translations["relays-add-relay"],
        action: "/relays",
        form,
        field_recipient: &field_translations["relays-field-recipient"],
        field_status: &field_translations["relays-field-status"],
        field_enabled: &field_translations["relays-field-enabled"],
        field_recipient_help: &field_translations["relays-field-recipient-help"],
        field_status_help: &field_translations["relays-field-status-help"],
        action_save: &form_translations["action-save"],
        action_cancel: &form_translations["action-cancel"],
        back_to_list: &form_translations["relays-back-to-list"],
        placeholder_recipient: &field_translations["relays-placeholder-recipient"],
        placeholder_status: &field_translations["relays-placeholder-status"],
    };

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations["relays-add-relay"].clone(),
    )
    .await
}

// Create a new relay
pub async fn create_relay(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RelayForm>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relay creation request");

    match db::create_relay(&pool, form) {
        Ok(relay) => {
            info!("Successfully created relay: {}", relay.recipient);
            Html(format!(
                "<script>window.location.href='/relays/{}';</script>",
                relay.pkid
            ))
        }
        Err(e) => {
            error!("Failed to create relay: {:?}", e);
            let error_msg = get_translation(&state, &locale, "relays-create-error").await;
            Html(error_msg)
        }
    }
}

// Show form for editing a relay
pub async fn edit_form(
    State(state): State<AppState>,
    Path(relay_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relay edit form request for ID: {}", relay_id);

    let relay = match db::get_relay(&pool, relay_id) {
        Ok(relay) => relay,
        Err(_) => {
            return crate::handlers::utils::handle_entity_not_found(
                &state,
                &headers,
                "relays",
                "relays-not-found",
            )
            .await;
        }
    };

    let form = RelayForm {
        recipient: relay.recipient.clone(),
        status: relay.status.clone(),
        enabled: relay.enabled,
    };

    // Use helper functions to fetch translations in batches
    let form_translations =
        crate::handlers::utils::get_entity_form_translations(&state, &locale, "relays").await;
    let field_translations = crate::handlers::utils::get_field_translations(
        &state,
        &locale,
        "relays",
        &["recipient", "status", "enabled"],
    )
    .await;

    let content_template = RelayFormTemplate {
        title: &form_translations["relays-edit-relay"],
        action: &format!("/relays/{relay_id}"),
        form,
        field_recipient: &field_translations["relays-field-recipient"],
        field_status: &field_translations["relays-field-status"],
        field_enabled: &field_translations["relays-field-enabled"],
        field_recipient_help: &field_translations["relays-field-recipient-help"],
        field_status_help: &field_translations["relays-field-status-help"],
        action_save: &form_translations["action-save"],
        action_cancel: &form_translations["action-cancel"],
        back_to_list: &form_translations["relays-back-to-list"],
        placeholder_recipient: &field_translations["relays-placeholder-recipient"],
        placeholder_status: &field_translations["relays-placeholder-status"],
    };

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations["relays-edit-relay"].clone(),
    )
    .await
}

// Update a relay
pub async fn update_relay(
    State(state): State<AppState>,
    Path(relay_id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<RelayForm>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relay update request for ID: {}", relay_id);

    match db::update_relay(&pool, relay_id, form) {
        Ok(relay) => {
            info!("Successfully updated relay: {}", relay.recipient);
            Html(format!(
                "<script>window.location.href='/relays/{}';</script>",
                relay.pkid
            ))
        }
        Err(Error::NotFound) => {
            let not_found_msg = get_translation(&state, &locale, "relays-not-found").await;
            Html(not_found_msg)
        }
        Err(e) => {
            error!("Failed to update relay {}: {:?}", relay_id, e);
            let error_msg = get_translation(&state, &locale, "relays-update-error").await;
            Html(error_msg)
        }
    }
}

// Delete a relay
pub async fn delete_relay(
    State(state): State<AppState>,
    Path(relay_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return error,
    };

    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relay delete request for ID: {}", relay_id);

    // Use helper function for entity operation
    match crate::handlers::utils::handle_entity_operation(
        || async { db::delete_relay(&pool, relay_id) },
        &state,
        &locale,
        "delete relay",
        &relay_id.to_string(),
        "Successfully deleted relay",
    )
    .await
    {
        Ok(_) => {
            info!("Successfully deleted relay ID: {}", relay_id);
            Html("<script>window.location.href='/relays';</script>".to_string())
        }
        Err(error) => error,
    }
}

// Toggle relay enabled status
pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(relay_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return error,
    };

    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relay toggle enabled request for ID: {}", relay_id);

    // Use helper function for entity operation
    match crate::handlers::utils::handle_entity_operation(
        || async { db::toggle_relay_enabled(&pool, relay_id) },
        &state,
        &locale,
        "toggle relay",
        &relay_id.to_string(),
        "Successfully toggled relay",
    )
    .await
    {
        Ok(relay) => {
            let enabled_text = if relay.enabled {
                get_translation(&state, &locale, "status-enabled").await
            } else {
                get_translation(&state, &locale, "status-disabled").await
            };

            // Check if this is a list view toggle (targeting relay-status-{id})
            if is_htmx_request(&headers) {
                // For list view, return status badge and update button text
                let badge_class = if relay.enabled {
                    "inline-flex rounded-full bg-green-100 px-2 text-xs font-semibold leading-5 text-green-800"
                } else {
                    "inline-flex rounded-full bg-red-100 px-2 text-xs font-semibold leading-5 text-red-800"
                };

                let button_text = if relay.enabled {
                    get_translation(&state, &locale, "action-disable").await
                } else {
                    get_translation(&state, &locale, "action-enable").await
                };

                // Check if this is a show view toggle (targeting relay-show-status-{id})
                let script = if headers
                    .get("hx-target")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .contains("relay-show-status")
                {
                    format!(
                        "<span class=\"{badge_class}\">{enabled_text}</span><script>document.getElementById('relay-show-button-{relay_id}').textContent = '{button_text}';</script>"
                    )
                } else {
                    format!(
                        "<span class=\"{badge_class}\">{enabled_text}</span><script>document.getElementById('relay-button-{relay_id}').textContent = '{button_text}';</script>"
                    )
                };
                Html(script)
            } else {
                // For show view, return the full status section
                let status_enabled = get_translation(&state, &locale, "status-enabled").await;
                let status_disabled = get_translation(&state, &locale, "status-disabled").await;

                if relay.enabled {
                    Html(format!("<span class=\"inline-flex rounded-full bg-green-100 dark:bg-green-900 px-2 text-xs font-semibold leading-5 text-green-800 dark:text-green-200\">{status_enabled}</span>"))
                } else {
                    Html(format!("<span class=\"inline-flex rounded-full bg-red-100 dark:bg-red-900 px-2 text-xs font-semibold leading-5 text-red-800 dark:text-red-200\">{status_disabled}</span>"))
                }
            }
        }
        Err(error) => error,
    }
}
