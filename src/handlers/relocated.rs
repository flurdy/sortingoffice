use crate::templates::relocated::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState, i18n::get_translation};
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
    headers.get("HX-Request").map_or(false, |v| v == "true")
}

// List all relocated entries
pub async fn list_relocated(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated list request");
    
    let relocated = match db::get_relocated(pool) {
        Ok(relocated) => {
            info!("Successfully retrieved {} relocated entries", relocated.len());
            relocated
        }
        Err(e) => {
            error!("Failed to retrieve relocated entries: {:?}", e);
            vec![]
        }
    };

    // Get translations
    let title = get_translation(&state, &locale, "relocated-title").await;
    let add_relocated = get_translation(&state, &locale, "relocated-add").await;
    let table_header_old_address = get_translation(&state, &locale, "relocated-table-header-old-address").await;
    let table_header_new_address = get_translation(&state, &locale, "relocated-table-header-new-address").await;
    let table_header_enabled = get_translation(&state, &locale, "relocated-table-header-enabled").await;
    let table_header_actions = get_translation(&state, &locale, "relocated-table-header-actions").await;
    let status_enabled = get_translation(&state, &locale, "status-enabled").await;
    let status_disabled = get_translation(&state, &locale, "status-disabled").await;
    let action_view = get_translation(&state, &locale, "action-view").await;
    let action_enable = get_translation(&state, &locale, "action-enable").await;
    let action_disable = get_translation(&state, &locale, "action-disable").await;
    let delete_confirm = get_translation(&state, &locale, "relocated-delete-confirm").await;
    let empty_title = get_translation(&state, &locale, "relocated-empty-title").await;
    let empty_description = get_translation(&state, &locale, "relocated-empty-description").await;
    let relocated_list_description = get_translation(&state, &locale, "relocated-list-description").await;

    let content_template = RelocatedListTemplate {
        title: &title,
        add_relocated: &add_relocated,
        table_header_old_address: &table_header_old_address,
        table_header_new_address: &table_header_new_address,
        table_header_enabled: &table_header_enabled,
        table_header_actions: &table_header_actions,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        action_view: &action_view,
        action_enable: &action_enable,
        action_disable: &action_disable,
        delete_confirm: &delete_confirm,
        empty_title: &empty_title,
        empty_description: &empty_description,
        relocated,
        relocated_list_description: &relocated_list_description,
    };
    
    let content = match content_template.render() {
        Ok(content) => {
            debug!("Template rendered successfully, content length: {}", content.len());
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
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "relocated-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

// Show a specific relocated entry
pub async fn show_relocated(
    State(state): State<AppState>,
    Path(relocated_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated show request for ID: {}", relocated_id);
    
    let relocated = match db::get_relocated_by_id(pool, relocated_id) {
        Ok(relocated) => relocated,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "relocated-not-found").await;
            return Html(not_found_msg);
        }
    };

    let title = get_translation(&state, &locale, "relocated-title").await;
    let action_edit = get_translation(&state, &locale, "action-edit").await;
    let action_enable = get_translation(&state, &locale, "action-enable").await;
    let action_disable = get_translation(&state, &locale, "action-disable").await;
    let action_delete = get_translation(&state, &locale, "action-delete").await;
    let delete_confirm = get_translation(&state, &locale, "relocated-delete-confirm").await;
    let back_to_list = get_translation(&state, &locale, "relocated-back-to-list").await;
    let field_id = get_translation(&state, &locale, "relocated-field-id").await;
    let field_old_address = get_translation(&state, &locale, "relocated-field-old-address").await;
    let field_new_address = get_translation(&state, &locale, "relocated-field-new-address").await;
    let field_enabled = get_translation(&state, &locale, "relocated-field-enabled").await;
    let field_created = get_translation(&state, &locale, "relocated-field-created").await;
    let field_modified = get_translation(&state, &locale, "relocated-field-modified").await;
    let status_enabled = get_translation(&state, &locale, "status-enabled").await;
    let status_disabled = get_translation(&state, &locale, "status-disabled").await;
    let view_edit_settings = get_translation(&state, &locale, "relocated-view-edit-settings").await;
    let relocated_show_title = get_translation(&state, &locale, "relocated-show-title").await;
    let relocated_info_title = get_translation(&state, &locale, "relocated-info-title").await;
    let relocated_info_description = get_translation(&state, &locale, "relocated-info-description").await;

    let content_template = RelocatedShowTemplate {
        title: &title,
        action_edit: &action_edit,
        action_enable: &action_enable,
        action_disable: &action_disable,
        action_delete: &action_delete,
        delete_confirm: &delete_confirm,
        back_to_list: &back_to_list,
        field_id: &field_id,
        field_old_address: &field_old_address,
        field_new_address: &field_new_address,
        field_enabled: &field_enabled,
        field_created: &field_created,
        field_modified: &field_modified,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        view_edit_settings: &view_edit_settings,
        relocated_show_title: &relocated_show_title,
        relocated_info_title: &relocated_info_title,
        relocated_info_description: &relocated_info_description,
        relocated,
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
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "relocated-show-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
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

    let title = get_translation(&state, &locale, "relocated-new-relocated").await;
    let action = "/relocated";
    let field_old_address = get_translation(&state, &locale, "relocated-field-old-address").await;
    let field_new_address = get_translation(&state, &locale, "relocated-field-new-address").await;
    let field_enabled = get_translation(&state, &locale, "relocated-field-enabled").await;
    let field_old_address_help = get_translation(&state, &locale, "relocated-field-old-address-help").await;
    let field_new_address_help = get_translation(&state, &locale, "relocated-field-new-address-help").await;
    let action_save = get_translation(&state, &locale, "action-save").await;
    let action_cancel = get_translation(&state, &locale, "action-cancel").await;
    let back_to_list = get_translation(&state, &locale, "relocated-back-to-list").await;
    let placeholder_old_address = get_translation(&state, &locale, "relocated-placeholder-old-address").await;
    let placeholder_new_address = get_translation(&state, &locale, "relocated-placeholder-new-address").await;

    let content_template = RelocatedFormTemplate {
        title: &title,
        action: &action,
        form,
        field_old_address: &field_old_address,
        field_new_address: &field_new_address,
        field_enabled: &field_enabled,
        field_old_address_help: &field_old_address_help,
        field_new_address_help: &field_new_address_help,
        action_save: &action_save,
        action_cancel: &action_cancel,
        back_to_list: &back_to_list,
        placeholder_old_address: &placeholder_old_address,
        placeholder_new_address: &placeholder_new_address,
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
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "relocated-add-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

// Create a new relocated entry
pub async fn create_relocated(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RelocatedForm>,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated create request");
    
    match db::create_relocated(pool, form) {
        Ok(relocated) => {
            info!("Successfully created relocated entry: {}", relocated.old_address);
            Html(format!("<script>window.location.href='/relocated/{}';</script>", relocated.pkid))
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
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated edit form request for ID: {}", relocated_id);
    
    let relocated = match db::get_relocated_by_id(pool, relocated_id) {
        Ok(relocated) => relocated,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "relocated-not-found").await;
            return Html(not_found_msg);
        }
    };

    let form = RelocatedForm {
        old_address: relocated.old_address.clone(),
        new_address: relocated.new_address.clone(),
        enabled: relocated.enabled,
    };

    let title = get_translation(&state, &locale, "relocated-edit-relocated").await;
    let action = format!("/relocated/{}", relocated_id);
    let field_old_address = get_translation(&state, &locale, "relocated-field-old-address").await;
    let field_new_address = get_translation(&state, &locale, "relocated-field-new-address").await;
    let field_enabled = get_translation(&state, &locale, "relocated-field-enabled").await;
    let field_old_address_help = get_translation(&state, &locale, "relocated-field-old-address-help").await;
    let field_new_address_help = get_translation(&state, &locale, "relocated-field-new-address-help").await;
    let action_save = get_translation(&state, &locale, "action-save").await;
    let action_cancel = get_translation(&state, &locale, "action-cancel").await;
    let back_to_list = get_translation(&state, &locale, "relocated-back-to-list").await;
    let placeholder_old_address = get_translation(&state, &locale, "relocated-placeholder-old-address").await;
    let placeholder_new_address = get_translation(&state, &locale, "relocated-placeholder-new-address").await;

    let content_template = RelocatedFormTemplate {
        title: &title,
        action: &action,
        form,
        field_old_address: &field_old_address,
        field_new_address: &field_new_address,
        field_enabled: &field_enabled,
        field_old_address_help: &field_old_address_help,
        field_new_address_help: &field_new_address_help,
        action_save: &action_save,
        action_cancel: &action_cancel,
        back_to_list: &back_to_list,
        placeholder_old_address: &placeholder_old_address,
        placeholder_new_address: &placeholder_new_address,
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
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "relocated-edit-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

// Update a relocated entry
pub async fn update_relocated(
    State(state): State<AppState>,
    Path(relocated_id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<RelocatedForm>,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated update request for ID: {}", relocated_id);
    
    match db::update_relocated(pool, relocated_id, form) {
        Ok(relocated) => {
            info!("Successfully updated relocated entry: {}", relocated.old_address);
            Html(format!("<script>window.location.href='/relocated/{}';</script>", relocated.pkid))
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
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated delete request for ID: {}", relocated_id);
    
    match db::delete_relocated(pool, relocated_id) {
        Ok(_) => {
            info!("Successfully deleted relocated entry ID: {}", relocated_id);
            Html("<script>window.location.href='/relocated';</script>".to_string())
        }
        Err(Error::NotFound) => {
            let not_found_msg = get_translation(&state, &locale, "relocated-not-found").await;
            Html(not_found_msg)
        }
        Err(e) => {
            error!("Failed to delete relocated entry {}: {:?}", relocated_id, e);
            let error_msg = get_translation(&state, &locale, "relocated-delete-error").await;
            Html(error_msg)
        }
    }
}

// Toggle relocated enabled status
pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(relocated_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    debug!("Handling relocated toggle enabled request for ID: {}", relocated_id);
    
    match db::toggle_relocated_enabled(pool, relocated_id) {
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
                let script = if headers.get("hx-target").and_then(|v| v.to_str().ok()).unwrap_or("").contains("relocated-show-status") {
                    format!(
                        "<span class=\"{}\">{}</span><script>document.getElementById('relocated-show-button-{}').textContent = '{}';</script>",
                        badge_class, enabled_text, relocated_id, button_text
                    )
                } else {
                    format!(
                        "<span class=\"{}\">{}</span><script>document.getElementById('relocated-button-{}').textContent = '{}';</script>",
                        badge_class, enabled_text, relocated_id, button_text
                    )
                };
                Html(script)
            } else {
                // For show view, return the full status section
                let status_enabled = get_translation(&state, &locale, "status-enabled").await;
                let status_disabled = get_translation(&state, &locale, "status-disabled").await;
                
                if relocated.enabled {
                    Html(format!("<span class=\"inline-flex rounded-full bg-green-100 dark:bg-green-900 px-2 text-xs font-semibold leading-5 text-green-800 dark:text-green-200\">{}</span>", status_enabled))
                } else {
                    Html(format!("<span class=\"inline-flex rounded-full bg-red-100 dark:bg-red-900 px-2 text-xs font-semibold leading-5 text-red-800 dark:text-red-200\">{}</span>", status_disabled))
                }
            }
        }
        Err(Error::NotFound) => {
            let not_found_msg = get_translation(&state, &locale, "relocated-not-found").await;
            Html(format!("<span class=\"text-danger\">{}</span>", not_found_msg))
        }
        Err(e) => {
            error!("Failed to toggle relocated entry {}: {:?}", relocated_id, e);
            let error_msg = get_translation(&state, &locale, "relocated-toggle-error").await;
            Html(format!("<span class=\"text-danger\">{}</span>", error_msg))
        }
    }
} 
