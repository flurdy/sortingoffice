use crate::{db, i18n::get_translation, models::*, AppState};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
    Form,
};
use diesel::result::Error;
use tracing::{debug, error, info};

use crate::handlers::database_ops::handle_entity_operation;

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some_and(|v| v == "true")
}

// List all relays
pub async fn list_relays(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
    let locale = crate::handlers::language::get_user_locale(&headers);
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    debug!("Handling relays list request");

    // Check if relays table is available for this database
    if !state.config.is_relays_available(&current_db_id) {
        let not_available_msg = get_translation(&state, &locale, "relays-not-available").await;
        return Html(not_available_msg);
    }

    let relays = crate::handlers::database_ops::get_entity_list_with_fallback(
        || async { db::get_relays(&pool) },
        "retrieve relays",
    )
    .await;

    // Use the new resource-specific helper function
    crate::handlers::rendering::render_relay_list_page(relays, &state, &locale, &headers).await
}

// Show a specific relay
pub async fn show_relay(
    State(state): State<AppState>,
    Path(relay_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
    let locale = crate::handlers::language::get_user_locale(&headers);
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    debug!("Handling relay show request for ID: {}", relay_id);

    // Check if relays table is available for this database
    if !state.config.is_relays_available(&current_db_id) {
        let not_available_msg = get_translation(&state, &locale, "relays-not-available").await;
        return Html(not_available_msg);
    }

    let relay = match crate::handlers::database_ops::get_entity_with_not_found(
        || async { db::get_relay(&pool, relay_id) },
        &state,
        &locale,
        "relay",
        "relays-not-found",
    )
    .await
    {
        Ok(relay) => relay,
        Err(error_response) => return error_response,
    };

    // Use the new resource-specific helper function
    crate::handlers::rendering::render_relay_show_page(relay, &state, &locale, &headers).await
}

// Show form for creating a new relay
pub async fn create_form(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relay create form request");

    let form = RelayForm {
        recipient: "".to_string(),
        status: "OK".to_string(),
        enabled: true,
    };

    // Use the new resource-specific helper function
    crate::handlers::rendering::render_relay_form_page(
        form,
        "relays-add-title",
        "relays-add-title",
        None, // No ID for new form
        &state,
        &locale,
        &headers,
    )
    .await
}

// Create a new relay
pub async fn create_relay(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RelayForm>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
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
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
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

    // Use the new resource-specific helper function
    crate::handlers::rendering::render_relay_form_page(
        form,
        "relays-edit-title",
        "relays-edit-title",
        Some(relay_id), // Pass the ID for edit form
        &state,
        &locale,
        &headers,
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
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
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
            crate::handlers::errors::render_relay_not_found_page(&state, &headers).await
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
    match handle_entity_operation(
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
    match handle_entity_operation(
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

// Toggle relay enabled status for domain show page
pub async fn toggle_enabled_domain_show(
    State(state): State<AppState>,
    Path(relay_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };

    match db::toggle_relay_enabled(&pool, relay_id) {
        Ok(_) => {
            let relay = match db::get_relay(&pool, relay_id) {
                Ok(relay) => relay,
                Err(_) => {
                    return crate::handlers::errors::render_relay_not_found_page(&state, &headers)
                        .await
                }
            };

            // Extract domain from relay recipient and look it up
            let domain_name = relay.recipient.split('@').next_back().unwrap_or("");
            let domain = match db::get_domain_by_name(&pool, domain_name) {
                Ok(domain) => domain,
                Err(_) => {
                    return crate::handlers::errors::render_domain_not_found_page(&state, &headers)
                        .await
                }
            };

            let locale = crate::handlers::language::get_user_locale(&headers);

            // Use resource-specific helper for domain show page with proper relay data
            let alias_report = db::get_domain_alias_report(&pool, &domain.domain).ok();
            let existing_aliases =
                db::get_aliases_for_domain(&pool, &domain.domain).unwrap_or_default();
            let analytics_common_aliases =
                crate::analytics::find_database_common_aliases(&state, &headers, 10, 3).await;

            // Get relays for this domain
            let domain_relays =
                db::get_relays_for_domain(&pool, &domain.domain).unwrap_or_default();

            crate::handlers::rendering::render_domain_show_page(
                domain,
                alias_report,
                existing_aliases,
                analytics_common_aliases,
                domain_relays,
                vec![], // domain_users - empty for now
                &state,
                &locale,
                &headers,
            )
            .await
        }
        Err(_) => return crate::handlers::errors::render_500_page(&state, &headers).await,
    }
}
