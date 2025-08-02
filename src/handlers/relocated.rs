use crate::templates::layout::BaseTemplate;
use crate::templates::relocated::*;
use crate::{
    db, get_entity_or_not_found, i18n::get_translation, models::*, render_template, AppState,
};
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

// List all relocated entries
pub async fn list_relocated(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    debug!("Handling relocated list request");

    // Check if relocated table is available for this database
    if !state.config.is_relocated_available(&current_db_id) {
        let not_available_msg = get_translation(&state, &locale, "relocated-not-available").await;
        return Html(not_available_msg);
    }

    let relocated = match db::get_relocated(&pool) {
        Ok(relocated) => {
            info!(
                "Successfully retrieved {} relocated entries",
                relocated.len()
            );
            relocated
        }
        Err(e) => {
            error!("Failed to retrieve relocated entries: {:?}", e);
            vec![]
        }
    };

    // Get all translations using consolidated helper functions
    let translations =
        crate::handlers::utils::get_entity_all_translations(&state, &locale, "relocated").await;

    let content_template = RelocatedListTemplate {
        title: &translations["relocated-title"],
        add_relocated: &translations["relocated-add"],
        table_header_old_address: &translations["relocated-table-header-old-address"],
        table_header_new_address: &translations["relocated-table-header-new-address"],
        table_header_enabled: &translations["relocated-table-header-enabled"],
        table_header_actions: &translations["relocated-table-header-actions"],
        status_enabled: &translations["status-enabled"],
        status_disabled: &translations["status-disabled"],
        action_view: &translations["action-view"],
        action_enable: &translations["action-enable"],
        action_disable: &translations["action-disable"],
        delete_confirm: &translations["relocated-delete-confirm"],
        empty_title: &translations["relocated-empty-title"],
        empty_description: &translations["relocated-empty-description"],
        relocated,
        relocated_list_description: &translations["relocated-list-description"],
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

        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "relocated-title").await,
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

// Show a specific relocated entry
pub async fn show_relocated(
    State(state): State<AppState>,
    Path(relocated_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::utils::get_user_locale(&headers);

    debug!("Handling relocated show request for ID: {}", relocated_id);

    // Use the new macro for "not found" error handling
    let relocated = get_entity_or_not_found!(
        db::get_relocated_by_id(&pool, relocated_id),
        &state,
        &locale,
        "relocated-not-found"
    );

    // Use the batch translation fetcher for common translations
    let translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
        &[
            "relocated-title",
            "action-edit",
            "action-enable",
            "action-disable",
            "action-delete",
            "relocated-delete-confirm",
            "relocated-back-to-list",
            "relocated-field-id",
            "relocated-field-old-address",
            "relocated-field-new-address",
            "relocated-field-enabled",
            "relocated-field-created",
            "relocated-field-modified",
            "status-enabled",
            "status-disabled",
            "relocated-view-edit-settings",
            "relocated-show-title",
            "relocated-info-title",
            "relocated-info-description",
        ],
    )
    .await;

    let content_template = RelocatedShowTemplate {
        title: &translations["relocated-title"],
        action_edit: &translations["action-edit"],
        action_enable: &translations["action-enable"],
        action_disable: &translations["action-disable"],
        action_delete: &translations["action-delete"],
        delete_confirm: &translations["relocated-delete-confirm"],
        back_to_list: &translations["relocated-back-to-list"],
        field_id: &translations["relocated-field-id"],
        field_old_address: &translations["relocated-field-old-address"],
        field_new_address: &translations["relocated-field-new-address"],
        field_enabled: &translations["relocated-field-enabled"],
        field_created: &translations["relocated-field-created"],
        field_modified: &translations["relocated-field-modified"],
        status_enabled: &translations["status-enabled"],
        status_disabled: &translations["status-disabled"],
        view_edit_settings: &translations["relocated-view-edit-settings"],
        relocated_show_title: &translations["relocated-show-title"],
        relocated_info_title: &translations["relocated-info-title"],
        relocated_info_description: &translations["relocated-info-description"],
        relocated,
    };

    // Use the new render template macro
    render_template!(content_template, &state, &locale, &headers)
}

// Show form for creating a new relocated entry
pub async fn create_form(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated create form request");

    let form = RelocatedForm {
        old_address: "".to_string(),
        new_address: "".to_string(),
        enabled: true,
    };

    // Use helper functions to fetch translations in batches
    let form_translations =
        crate::handlers::utils::get_entity_form_translations(&state, &locale, "relocated").await;
    let field_translations = crate::handlers::utils::get_field_translations(
        &state,
        &locale,
        "relocated",
        &["old_address", "new_address", "enabled"],
    )
    .await;

    let content_template = RelocatedFormTemplate {
        title: &form_translations["relocated-new-relocated"],
        action: "/relocated",
        form,
        field_old_address: &field_translations["relocated-field-old-address"],
        field_new_address: &field_translations["relocated-field-new-address"],
        field_enabled: &field_translations["relocated-field-enabled"],
        field_old_address_help: &field_translations["relocated-field-old-address-help"],
        field_new_address_help: &field_translations["relocated-field-new-address-help"],
        action_save: &form_translations["action-save"],
        action_cancel: &form_translations["action-cancel"],
        back_to_list: &form_translations["relocated-back-to-list"],
        placeholder_old_address: &field_translations["relocated-placeholder-old-address"],
        placeholder_new_address: &field_translations["relocated-placeholder-new-address"],
    };

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations["relocated-add-title"].clone(),
    )
    .await
}

// Create a new relocated entry
pub async fn create_relocated(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RelocatedForm>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated create request");

    match db::create_relocated(&pool, form) {
        Ok(relocated) => {
            info!(
                "Successfully created relocated entry: {}",
                relocated.old_address
            );
            Html(format!(
                "<script>window.location.href='/relocated/{}';</script>",
                relocated.pkid
            ))
        }
        Err(e) => {
            error!("Failed to create relocated entry: {:?}", e);
            let error_msg = get_translation(&state, &locale, "relocated-create-error").await;
            Html(error_msg)
        }
    }
}

// Show form for editing a relocated entry
pub async fn edit_form(
    State(state): State<AppState>,
    Path(relocated_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!(
        "Handling relocated edit form request for ID: {}",
        relocated_id
    );

    let relocated = match db::get_relocated_by_id(&pool, relocated_id) {
        Ok(relocated) => relocated,
        Err(_) => {
            return crate::handlers::utils::handle_entity_not_found(
                &state,
                &headers,
                "relocated",
                "relocated-not-found",
            )
            .await;
        }
    };

    let form = RelocatedForm {
        old_address: relocated.old_address.clone(),
        new_address: relocated.new_address.clone(),
        enabled: relocated.enabled,
    };

    // Use helper functions to fetch translations in batches
    let form_translations =
        crate::handlers::utils::get_entity_form_translations(&state, &locale, "relocated").await;
    let field_translations = crate::handlers::utils::get_field_translations(
        &state,
        &locale,
        "relocated",
        &["old_address", "new_address", "enabled"],
    )
    .await;

    let content_template = RelocatedFormTemplate {
        title: &form_translations["relocated-edit-relocated"],
        action: &format!("/relocated/{relocated_id}"),
        form,
        field_old_address: &field_translations["relocated-field-old-address"],
        field_new_address: &field_translations["relocated-field-new-address"],
        field_enabled: &field_translations["relocated-field-enabled"],
        field_old_address_help: &field_translations["relocated-field-old-address-help"],
        field_new_address_help: &field_translations["relocated-field-new-address-help"],
        action_save: &form_translations["action-save"],
        action_cancel: &form_translations["action-cancel"],
        back_to_list: &form_translations["relocated-back-to-list"],
        placeholder_old_address: &field_translations["relocated-placeholder-old-address"],
        placeholder_new_address: &field_translations["relocated-placeholder-new-address"],
    };

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations["relocated-edit-title"].clone(),
    )
    .await
}

// Update a relocated entry
pub async fn update_relocated(
    State(state): State<AppState>,
    Path(relocated_id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<RelocatedForm>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated update request for ID: {}", relocated_id);

    match db::update_relocated(&pool, relocated_id, form) {
        Ok(relocated) => {
            info!(
                "Successfully updated relocated entry: {}",
                relocated.old_address
            );
            Html(format!(
                "<script>window.location.href='/relocated/{}';</script>",
                relocated.pkid
            ))
        }
        Err(Error::NotFound) => {
            let not_found_msg = get_translation(&state, &locale, "relocated-not-found").await;
            Html(not_found_msg)
        }
        Err(e) => {
            error!("Failed to update relocated entry {}: {:?}", relocated_id, e);
            let error_msg = get_translation(&state, &locale, "relocated-update-error").await;
            Html(error_msg)
        }
    }
}

// Delete a relocated entry
pub async fn delete_relocated(
    State(state): State<AppState>,
    Path(relocated_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return error,
    };

    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated delete request for ID: {}", relocated_id);

    // Use helper function for entity operation
    match crate::handlers::utils::handle_entity_operation(
        || async { db::delete_relocated(&pool, relocated_id) },
        &state,
        &locale,
        "delete relocated",
        &relocated_id.to_string(),
        "Successfully deleted relocated",
    )
    .await
    {
        Ok(_) => {
            info!("Successfully deleted relocated entry ID: {}", relocated_id);
            Html("<script>window.location.href='/relocated';</script>".to_string())
        }
        Err(error) => error,
    }
}

// Toggle relocated enabled status
pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(relocated_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return error,
    };

    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!(
        "Handling relocated toggle enabled request for ID: {}",
        relocated_id
    );

    // Use helper function for entity operation
    match crate::handlers::utils::handle_entity_operation(
        || async { db::toggle_relocated_enabled(&pool, relocated_id) },
        &state,
        &locale,
        "toggle relocated",
        &relocated_id.to_string(),
        "Successfully toggled relocated",
    )
    .await
    {
        Ok(relocated) => {
            let enabled_text = if relocated.enabled {
                get_translation(&state, &locale, "status-enabled").await
            } else {
                get_translation(&state, &locale, "status-disabled").await
            };

            // Check if this is a list view toggle (targeting relocated-status-{id})
            if is_htmx_request(&headers) {
                // For list view, return status badge and update button text
                let badge_class = if relocated.enabled {
                    "inline-flex rounded-full bg-green-100 px-2 text-xs font-semibold leading-5 text-green-800"
                } else {
                    "inline-flex rounded-full bg-red-100 px-2 text-xs font-semibold leading-5 text-red-800"
                };

                let button_text = if relocated.enabled {
                    get_translation(&state, &locale, "action-disable").await
                } else {
                    get_translation(&state, &locale, "action-enable").await
                };

                // Check if this is a show view toggle (targeting relocated-show-status-{id})
                let script = if headers
                    .get("hx-target")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .contains("relocated-show-status")
                {
                    format!(
                        "<span class=\"{badge_class}\">{enabled_text}</span><script>document.getElementById('relocated-show-button-{relocated_id}').textContent = '{button_text}';</script>"
                    )
                } else {
                    format!(
                        "<span class=\"{badge_class}\">{enabled_text}</span><script>document.getElementById('relocated-button-{relocated_id}').textContent = '{button_text}';</script>"
                    )
                };
                Html(script)
            } else {
                // For show view, return the full status section
                let status_enabled = get_translation(&state, &locale, "status-enabled").await;
                let status_disabled = get_translation(&state, &locale, "status-disabled").await;

                if relocated.enabled {
                    Html(format!("<span class=\"inline-flex rounded-full bg-green-100 dark:bg-green-900 px-2 text-xs font-semibold leading-5 text-green-800 dark:text-green-200\">{status_enabled}</span>"))
                } else {
                    Html(format!("<span class=\"inline-flex rounded-full bg-red-100 dark:bg-red-900 px-2 text-xs font-semibold leading-5 text-red-800 dark:text-red-200\">{status_disabled}</span>"))
                }
            }
        }
        Err(error) => error,
    }
}
