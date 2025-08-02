use crate::{db, get_entity_or_not_found, i18n::get_translation, models::*, AppState};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
    Form,
};
use diesel::result::Error;
use tracing::{debug, error, info};

use crate::handlers::utils::{
    get_current_db_pool, render_relocated_form_page, render_relocated_list_page,
    render_relocated_show_page,
};

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some_and(|v| v == "true")
}

// List all relocated entries
pub async fn list_relocated(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
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

    render_relocated_list_page(relocated, &state, &locale, &headers).await
}

// Show a specific relocated entry
pub async fn show_relocated(
    State(state): State<AppState>,
    Path(relocated_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    let relocated = get_entity_or_not_found!(
        db::get_relocated_by_id(&pool, relocated_id),
        &state,
        &locale,
        "relocated-not-found"
    );

    render_relocated_show_page(relocated, &state, &locale, &headers).await
}

// Show form for creating a new relocated entry
pub async fn create_form(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let form = RelocatedForm {
        old_address: String::new(),
        new_address: String::new(),
        enabled: true,
    };

    render_relocated_form_page(
        form,
        "relocated-add-title",
        "relocated-add-action",
        &state,
        &locale,
        &headers,
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
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    let relocated = get_entity_or_not_found!(
        db::get_relocated_by_id(&pool, relocated_id),
        &state,
        &locale,
        "relocated-not-found"
    );

    let form = RelocatedForm {
        old_address: relocated.old_address.clone(),
        new_address: relocated.new_address.clone(),
        enabled: relocated.enabled,
    };

    render_relocated_form_page(
        form,
        "relocated-edit-title",
        "relocated-edit-action",
        &state,
        &locale,
        &headers,
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
