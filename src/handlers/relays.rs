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

    // Get translations
    let title = get_translation(&state, &locale, "relays-title").await;
    let add_relay = get_translation(&state, &locale, "relays-add").await;
    let table_header_recipient =
        get_translation(&state, &locale, "relays-table-header-recipient").await;
    let table_header_status = get_translation(&state, &locale, "relays-table-header-status").await;
    let table_header_enabled =
        get_translation(&state, &locale, "relays-table-header-enabled").await;
    let table_header_actions =
        get_translation(&state, &locale, "relays-table-header-actions").await;
    let status_enabled = get_translation(&state, &locale, "status-enabled").await;
    let status_disabled = get_translation(&state, &locale, "status-disabled").await;
    let status_ok = get_translation(&state, &locale, "status-ok").await;
    let status_reject = get_translation(&state, &locale, "status-reject").await;
    let action_view = get_translation(&state, &locale, "action-view").await;
    let action_enable = get_translation(&state, &locale, "action-enable").await;
    let action_disable = get_translation(&state, &locale, "action-disable").await;
    let delete_confirm = get_translation(&state, &locale, "relays-delete-confirm").await;
    let empty_title = get_translation(&state, &locale, "relays-empty-title").await;
    let empty_description = get_translation(&state, &locale, "relays-empty-description").await;
    let relays_list_description = get_translation(&state, &locale, "relays-list-description").await;

    let content_template = RelayListTemplate {
        title: &title,
        add_relay: &add_relay,
        table_header_recipient: &table_header_recipient,
        table_header_status: &table_header_status,
        table_header_enabled: &table_header_enabled,
        table_header_actions: &table_header_actions,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        status_ok: &status_ok,
        status_reject: &status_reject,
        action_view: &action_view,
        action_enable: &action_enable,
        action_disable: &action_disable,
        delete_confirm: &delete_confirm,
        empty_title: &empty_title,
        empty_description: &empty_description,
        relays,
        relays_list_description: &relays_list_description,
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
            get_translation(&state, &locale, "relays-title").await,
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
            let not_found_msg = get_translation(&state, &locale, "relays-not-found").await;
            return Html(not_found_msg);
        }
    };

    let title = get_translation(&state, &locale, "relays-title").await;
    let action_edit = get_translation(&state, &locale, "action-edit").await;
    let action_enable = get_translation(&state, &locale, "action-enable").await;
    let action_disable = get_translation(&state, &locale, "action-disable").await;
    let action_delete = get_translation(&state, &locale, "action-delete").await;
    let delete_confirm = get_translation(&state, &locale, "relays-delete-confirm").await;
    let back_to_list = get_translation(&state, &locale, "relays-back-to-list").await;
    let field_id = get_translation(&state, &locale, "relays-field-id").await;
    let field_recipient = get_translation(&state, &locale, "relays-field-recipient").await;
    let field_status = get_translation(&state, &locale, "relays-field-status").await;
    let field_enabled = get_translation(&state, &locale, "relays-field-enabled").await;
    let field_created = get_translation(&state, &locale, "relays-field-created").await;
    let field_modified = get_translation(&state, &locale, "relays-field-modified").await;
    let status_enabled = get_translation(&state, &locale, "status-enabled").await;
    let status_disabled = get_translation(&state, &locale, "status-disabled").await;
    let status_ok = get_translation(&state, &locale, "status-ok").await;
    let status_reject = get_translation(&state, &locale, "status-reject").await;
    let view_edit_settings = get_translation(&state, &locale, "relays-view-edit-settings").await;
    let relay_show_title = get_translation(&state, &locale, "relays-show-title-label").await;
    let relay_info_title = get_translation(&state, &locale, "relays-info-title").await;
    let relay_info_description = get_translation(&state, &locale, "relays-info-description").await;

    let content_template = RelayShowTemplate {
        title: &title,
        relay,
        action_edit: &action_edit,
        action_enable: &action_enable,
        action_disable: &action_disable,
        action_delete: &action_delete,
        delete_confirm: &delete_confirm,
        back_to_list: &back_to_list,
        field_id: &field_id,
        field_recipient: &field_recipient,
        field_status: &field_status,
        field_enabled: &field_enabled,
        field_created: &field_created,
        field_modified: &field_modified,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        status_ok: &status_ok,
        status_reject: &status_reject,
        view_edit_settings: &view_edit_settings,
        relay_show_title: &relay_show_title,
        relay_info_title: &relay_info_title,
        relay_info_description: &relay_info_description,
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
            get_translation(&state, &locale, "relays-title").await,
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
        title: &form_translations["relays-new-relay"],
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
        form_translations["relays-add-title"].clone(),
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

    debug!("Handling relay create request");

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
            let not_found_msg = get_translation(&state, &locale, "relays-not-found").await;
            return Html(not_found_msg);
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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relay delete request for ID: {}", relay_id);

    match db::delete_relay(&pool, relay_id) {
        Ok(_) => {
            info!("Successfully deleted relay ID: {}", relay_id);
            Html("<script>window.location.href='/relays';</script>".to_string())
        }
        Err(Error::NotFound) => {
            let not_found_msg = get_translation(&state, &locale, "relays-not-found").await;
            Html(not_found_msg)
        }
        Err(e) => {
            error!("Failed to delete relay {}: {:?}", relay_id, e);
            let error_msg = get_translation(&state, &locale, "relays-delete-error").await;
            Html(error_msg)
        }
    }
}

// Toggle relay enabled status
pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(relay_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relay toggle enabled request for ID: {}", relay_id);

    match db::toggle_relay_enabled(&pool, relay_id) {
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
        Err(Error::NotFound) => {
            let not_found_msg = get_translation(&state, &locale, "relays-not-found").await;
            Html(format!(
                "<span class=\"text-danger\">{not_found_msg}</span>"
            ))
        }
        Err(e) => {
            error!(
                "Failed to toggle relay {} enabled status: {:?}",
                relay_id, e
            );
            let error_msg = get_translation(&state, &locale, "relays-toggle-error").await;
            Html(format!("<span class=\"text-danger\">{error_msg}</span>"))
        }
    }
}
