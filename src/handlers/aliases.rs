use crate::templates::aliases::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState, i18n::get_translation};
use askama::Template;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
    Form,
};

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").map_or(false, |v| v == "true")
}

pub async fn list(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;

    let aliases = match db::get_aliases(pool) {
        Ok(aliases) => aliases,
        Err(_) => vec![],
    };

    let locale = crate::handlers::language::get_user_locale(&headers);
    let title = get_translation(&state, &locale, "aliases-title").await;
    let description = get_translation(&state, &locale, "aliases-description").await;
    let add_alias = get_translation(&state, &locale, "aliases-add").await;
    let table_header_mail = get_translation(&state, &locale, "aliases-table-header-mail").await;
    let table_header_destination = get_translation(&state, &locale, "aliases-table-header-destination").await;
    let table_header_status = get_translation(&state, &locale, "aliases-table-header-status").await;
    let table_header_actions = get_translation(&state, &locale, "aliases-table-header-actions").await;
    let status_active = get_translation(&state, &locale, "status-active").await;
    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
    let action_view = get_translation(&state, &locale, "action-view").await;
    let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
    let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;
    let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
    let empty_description = get_translation(&state, &locale, "aliases-empty-description").await;

    let content_template = AliasListTemplate {
        title: &title,
        aliases,
        description: &description,
        add_alias: &add_alias,
        table_header_mail: &table_header_mail,
        table_header_destination: &table_header_destination,
        table_header_status: &table_header_status,
        table_header_actions: &table_header_actions,
        status_active: &status_active,
        status_inactive: &status_inactive,
        action_view: &action_view,
        enable_alias: &enable_alias,
        disable_alias: &disable_alias,
        empty_title: &empty_title,
        empty_description: &empty_description,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "aliases-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let form = AliasForm {
        mail: "".to_string(),
        destination: "".to_string(),
        enabled: true,
    };

    let locale = crate::handlers::language::get_user_locale(&headers);
    let title = get_translation(&state, &locale, "aliases-add-title").await;
    let edit_alias = get_translation(&state, &locale, "aliases-edit-alias").await;
    let new_alias = get_translation(&state, &locale, "aliases-new-alias").await;
    let form_error = get_translation(&state, &locale, "aliases-form-error").await;
    let mail_address = get_translation(&state, &locale, "aliases-mail-address").await;
    let destination = get_translation(&state, &locale, "aliases-destination").await;
    let placeholder_mail = get_translation(&state, &locale, "aliases-placeholder-mail").await;
    let placeholder_destination = get_translation(&state, &locale, "aliases-placeholder-destination").await;
    let tooltip_mail = get_translation(&state, &locale, "aliases-tooltip-mail").await;
    let tooltip_destination = get_translation(&state, &locale, "aliases-tooltip-destination").await;
    let active = get_translation(&state, &locale, "aliases-active").await;
    let tooltip_active = get_translation(&state, &locale, "aliases-tooltip-active").await;
    let cancel = get_translation(&state, &locale, "aliases-cancel").await;
    let update_alias = get_translation(&state, &locale, "aliases-update-alias").await;
    let create_alias = get_translation(&state, &locale, "aliases-create-alias").await;

    let content_template = AliasFormTemplate {
        title: &title,
        alias: None,
        form,
        error: None,
        edit_alias: &edit_alias,
        new_alias: &new_alias,
        form_error: &form_error,
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
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "aliases-add-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    let alias = match db::get_alias(pool, id) {
        Ok(alias) => alias,
        Err(_) => return Html("Alias not found".to_string()),
    };

    let locale = crate::handlers::language::get_user_locale(&headers);
    let title = get_translation(&state, &locale, "aliases-show-title").await;
    let view_edit_settings = get_translation(&state, &locale, "aliases-view-edit-settings").await;
    let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
    let alias_information = get_translation(&state, &locale, "aliases-alias-information").await;
    let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
    let mail = get_translation(&state, &locale, "aliases-mail").await;
    let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
    let status = get_translation(&state, &locale, "aliases-status").await;
    let status_active = get_translation(&state, &locale, "status-active").await;
    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
    let created = get_translation(&state, &locale, "aliases-created").await;
    let modified = get_translation(&state, &locale, "aliases-modified").await;
    let edit_alias_button = get_translation(&state, &locale, "aliases-edit-alias-button").await;
    let enable_alias_button = get_translation(&state, &locale, "aliases-enable-alias-button").await;
    let disable_alias_button = get_translation(&state, &locale, "aliases-disable-alias-button").await;
    let delete_alias = get_translation(&state, &locale, "aliases-delete-alias").await;
    let delete_confirm = get_translation(&state, &locale, "aliases-delete-confirm").await;

    let content_template = AliasShowTemplate {
        title: &title,
        alias,
        view_edit_settings: &view_edit_settings,
        back_to_aliases: &back_to_aliases,
        alias_information: &alias_information,
        alias_details: &alias_details,
        mail: &mail,
        forward_to: &forward_to,
        status: &status,
        status_active: &status_active,
        status_inactive: &status_inactive,
        created: &created,
        modified: &modified,
        edit_alias_button: &edit_alias_button,
        enable_alias_button: &enable_alias_button,
        disable_alias_button: &disable_alias_button,
        delete_alias: &delete_alias,
        delete_confirm: &delete_confirm,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "aliases-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    let alias = match db::get_alias(pool, id) {
        Ok(alias) => alias,
        Err(_) => return Html("Alias not found".to_string()),
    };

    let form = AliasForm {
        mail: alias.mail.clone(),
        destination: alias.destination.clone(),
        enabled: alias.enabled,
    };

    let locale = crate::handlers::language::get_user_locale(&headers);
    let title = get_translation(&state, &locale, "aliases-edit-title").await;
    let edit_alias = get_translation(&state, &locale, "aliases-edit-alias").await;
    let new_alias = get_translation(&state, &locale, "aliases-new-alias").await;
    let form_error = get_translation(&state, &locale, "aliases-form-error").await;
    let mail_address = get_translation(&state, &locale, "aliases-mail-address").await;
    let destination = get_translation(&state, &locale, "aliases-destination").await;
    let placeholder_mail = get_translation(&state, &locale, "aliases-placeholder-mail").await;
    let placeholder_destination = get_translation(&state, &locale, "aliases-placeholder-destination").await;
    let tooltip_mail = get_translation(&state, &locale, "aliases-tooltip-mail").await;
    let tooltip_destination = get_translation(&state, &locale, "aliases-tooltip-destination").await;
    let active = get_translation(&state, &locale, "aliases-active").await;
    let tooltip_active = get_translation(&state, &locale, "aliases-tooltip-active").await;
    let cancel = get_translation(&state, &locale, "aliases-cancel").await;
    let update_alias = get_translation(&state, &locale, "aliases-update-alias").await;
    let create_alias = get_translation(&state, &locale, "aliases-create-alias").await;

    let content_template = AliasFormTemplate {
        title: &title,
        alias: Some(alias),
        form,
        error: None,
        edit_alias: &edit_alias,
        new_alias: &new_alias,
        form_error: &form_error,
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
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "aliases-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<AliasForm>,
) -> Html<String> {
    let pool = &state.pool;

    match db::create_alias(pool, form.clone()) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(e) => {
                    eprintln!("Error getting aliases: {:?}", e);
                    vec![]
                }
            };
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-title").await;
            let description = get_translation(&state, &locale, "aliases-description").await;
            let add_alias = get_translation(&state, &locale, "aliases-add").await;
            let table_header_mail = get_translation(&state, &locale, "aliases-table-header-mail").await;
            let table_header_destination = get_translation(&state, &locale, "aliases-table-header-destination").await;

            let table_header_status = get_translation(&state, &locale, "aliases-table-header-status").await;
            let table_header_actions = get_translation(&state, &locale, "aliases-table-header-actions").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_view = get_translation(&state, &locale, "action-view").await;
            let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
            let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;
            let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
            let empty_description = get_translation(&state, &locale, "aliases-empty-description").await;
            let content_template = AliasListTemplate {
                title: &title,
                aliases,
                description: &description,
                add_alias: &add_alias,
                table_header_mail: &table_header_mail,
                table_header_destination: &table_header_destination,
                table_header_status: &table_header_status,
                table_header_actions: &table_header_actions,
                status_active: &status_active,
                status_inactive: &status_inactive,
                action_view: &action_view,
                enable_alias: &enable_alias,
                disable_alias: &disable_alias,
                empty_title: &empty_title,
                empty_description: &empty_description,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(e) => {
            eprintln!("Error creating alias: {:?}", e);
            
            // Handle specific database errors with user-friendly messages
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("An alias with the email '{}' already exists.", form.mail),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The alias data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while creating the alias. Please try again.".to_string(),
            };

            // Return to form with error message
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-add-title").await;
            let edit_alias = get_translation(&state, &locale, "aliases-edit-alias").await;
            let new_alias = get_translation(&state, &locale, "aliases-new-alias").await;
            let form_error = get_translation(&state, &locale, "aliases-form-error").await;
            let mail_address = get_translation(&state, &locale, "aliases-mail-address").await;
            let destination = get_translation(&state, &locale, "aliases-destination").await;
            let placeholder_mail = get_translation(&state, &locale, "aliases-placeholder-mail").await;
            let placeholder_destination = get_translation(&state, &locale, "aliases-placeholder-destination").await;
            let tooltip_mail = get_translation(&state, &locale, "aliases-tooltip-mail").await;
            let tooltip_destination = get_translation(&state, &locale, "aliases-tooltip-destination").await;
            let tooltip_active = get_translation(&state, &locale, "aliases-tooltip-active").await;
            let cancel = get_translation(&state, &locale, "aliases-cancel").await;
            let update_alias = get_translation(&state, &locale, "aliases-update-alias").await;
            let create_alias = get_translation(&state, &locale, "aliases-create-alias").await;
            
            let active = get_translation(&state, &locale, "aliases-active").await;
            let content_template = AliasFormTemplate {
                title: &title,
                alias: None,
                form: form.clone(),
                error: Some(error_message),
                edit_alias: &edit_alias,
                new_alias: &new_alias,
                form_error: &form_error,
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
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-add-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<AliasForm>,
) -> Html<String> {
    let pool = &state.pool;

    match db::update_alias(pool, id, form.clone()) {
        Ok(_) => {
            let alias = match db::get_alias(pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };
            
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-show-title").await;
            let view_edit_settings = get_translation(&state, &locale, "aliases-view-edit-settings").await;
            let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
            let alias_information = get_translation(&state, &locale, "aliases-alias-information").await;
            let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
            let mail = get_translation(&state, &locale, "aliases-mail").await;
            let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
            let status = get_translation(&state, &locale, "aliases-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "aliases-created").await;
            let modified = get_translation(&state, &locale, "aliases-modified").await;
            let edit_alias_button = get_translation(&state, &locale, "aliases-edit-alias-button").await;
            let enable_alias_button = get_translation(&state, &locale, "aliases-enable-alias-button").await;
            let disable_alias_button = get_translation(&state, &locale, "aliases-disable-alias-button").await;
            let delete_alias = get_translation(&state, &locale, "aliases-delete-alias").await;
            let delete_confirm = get_translation(&state, &locale, "aliases-delete-confirm").await;
            
            let content_template = AliasShowTemplate {
                title: &title,
                alias,
                view_edit_settings: &view_edit_settings,
                back_to_aliases: &back_to_aliases,
                alias_information: &alias_information,
                alias_details: &alias_details,
                mail: &mail,
                forward_to: &forward_to,
                status: &status,
                status_active: &status_active,
                status_inactive: &status_inactive,
                created: &created,
                modified: &modified,
                edit_alias_button: &edit_alias_button,
                enable_alias_button: &enable_alias_button,
                disable_alias_button: &disable_alias_button,
                delete_alias: &delete_alias,
                delete_confirm: &delete_confirm,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-show-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(e) => {
            eprintln!("Error updating alias: {:?}", e);
            
            // Handle specific database errors with user-friendly messages
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                    _,
                ) => "The domain does not exist. Please create the domain first before adding aliases.".to_string(),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("An alias with the email '{}' already exists.", form.mail),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The alias data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while updating the alias. Please try again.".to_string(),
            };

            // Get the original alias for the form
            let original_alias = match db::get_alias(pool, id) {
                Ok(alias) => Some(alias),
                Err(_) => None,
            };

            // Return to form with error message
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-edit-title").await;
            let edit_alias = get_translation(&state, &locale, "aliases-edit-alias").await;
            let new_alias = get_translation(&state, &locale, "aliases-new-alias").await;
            let form_error = get_translation(&state, &locale, "aliases-form-error").await;
            let mail_address = get_translation(&state, &locale, "aliases-mail-address").await;
            let destination = get_translation(&state, &locale, "aliases-destination").await;
            let placeholder_mail = get_translation(&state, &locale, "aliases-placeholder-mail").await;
            let placeholder_destination = get_translation(&state, &locale, "aliases-placeholder-destination").await;
            let tooltip_mail = get_translation(&state, &locale, "aliases-tooltip-mail").await;
            let tooltip_destination = get_translation(&state, &locale, "aliases-tooltip-destination").await;
            let tooltip_active = get_translation(&state, &locale, "aliases-tooltip-active").await;
            let cancel = get_translation(&state, &locale, "aliases-cancel").await;
            let update_alias = get_translation(&state, &locale, "aliases-update-alias").await;
            let create_alias = get_translation(&state, &locale, "aliases-create-alias").await;
            
            let active = get_translation(&state, &locale, "aliases-active").await;
            let content_template = AliasFormTemplate {
                title: &title,
                alias: original_alias,
                form: form.clone(),
                error: Some(error_message),
                edit_alias: &edit_alias,
                new_alias: &new_alias,
                form_error: &form_error,
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
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-edit-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    match db::delete_alias(pool, id) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(e) => {
                    eprintln!("Error getting aliases: {:?}", e);
                    vec![]
                }
            };
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-title").await;
            let description = get_translation(&state, &locale, "aliases-description").await;
            let add_alias = get_translation(&state, &locale, "aliases-add").await;
            let table_header_mail = get_translation(&state, &locale, "aliases-table-header-mail").await;
            let table_header_destination = get_translation(&state, &locale, "aliases-table-header-destination").await;
            let table_header_status = get_translation(&state, &locale, "aliases-table-header-status").await;
            let table_header_actions = get_translation(&state, &locale, "aliases-table-header-actions").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_view = get_translation(&state, &locale, "action-view").await;
            let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
            let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;
            let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
            let empty_description = get_translation(&state, &locale, "aliases-empty-description").await;
            let content_template = AliasListTemplate {
                title: &title,
                aliases,
                description: &description,
                add_alias: &add_alias,
                table_header_mail: &table_header_mail,
                table_header_destination: &table_header_destination,
                table_header_status: &table_header_status,
                table_header_actions: &table_header_actions,
                status_active: &status_active,
                status_inactive: &status_inactive,
                action_view: &action_view,
                enable_alias: &enable_alias,
                disable_alias: &disable_alias,
                empty_title: &empty_title,
                empty_description: &empty_description,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error deleting alias".to_string()),
    }
}

pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    match db::toggle_alias_enabled(pool, id) {
        Ok(_) => {
            let alias = match db::get_alias(pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };
            
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-show-title").await;
            let view_edit_settings = get_translation(&state, &locale, "aliases-view-edit-settings").await;
            let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
            let alias_information = get_translation(&state, &locale, "aliases-alias-information").await;
            let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
            let mail = get_translation(&state, &locale, "aliases-mail").await;
            let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
            let status = get_translation(&state, &locale, "aliases-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "aliases-created").await;
            let modified = get_translation(&state, &locale, "aliases-modified").await;
            let edit_alias_button = get_translation(&state, &locale, "aliases-edit-alias-button").await;
            let enable_alias_button = get_translation(&state, &locale, "aliases-enable-alias-button").await;
            let disable_alias_button = get_translation(&state, &locale, "aliases-disable-alias-button").await;
            let delete_alias = get_translation(&state, &locale, "aliases-delete-alias").await;
            let delete_confirm = get_translation(&state, &locale, "aliases-delete-confirm").await;
            
            let content_template = AliasShowTemplate {
                title: &title,
                alias,
                view_edit_settings: &view_edit_settings,
                back_to_aliases: &back_to_aliases,
                alias_information: &alias_information,
                alias_details: &alias_details,
                mail: &mail,
                forward_to: &forward_to,
                status: &status,
                status_active: &status_active,
                status_inactive: &status_inactive,
                created: &created,
                modified: &modified,
                edit_alias_button: &edit_alias_button,
                enable_alias_button: &enable_alias_button,
                disable_alias_button: &disable_alias_button,
                delete_alias: &delete_alias,
                delete_confirm: &delete_confirm,
            };
            let content = content_template.render().unwrap();
            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-show-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_alias_enabled(pool, id) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(e) => {
                    eprintln!("Error getting aliases: {:?}", e);
                    vec![]
                }
            };
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-title").await;
            let description = get_translation(&state, &locale, "aliases-description").await;
            let add_alias = get_translation(&state, &locale, "aliases-add").await;
            let table_header_mail = get_translation(&state, &locale, "aliases-table-header-mail").await;
            let table_header_destination = get_translation(&state, &locale, "aliases-table-header-destination").await;
            let table_header_status = get_translation(&state, &locale, "aliases-table-header-status").await;
            let table_header_actions = get_translation(&state, &locale, "aliases-table-header-actions").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_view = get_translation(&state, &locale, "action-view").await;
            let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
            let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;
            let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
            let empty_description = get_translation(&state, &locale, "aliases-empty-description").await;
            let content_template = AliasListTemplate {
                title: &title,
                aliases,
                description: &description,
                add_alias: &add_alias,
                table_header_mail: &table_header_mail,
                table_header_destination: &table_header_destination,
                table_header_status: &table_header_status,
                table_header_actions: &table_header_actions,
                status_active: &status_active,
                status_inactive: &status_inactive,
                action_view: &action_view,
                enable_alias: &enable_alias,
                disable_alias: &disable_alias,
                empty_title: &empty_title,
                empty_description: &empty_description,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_alias_enabled(pool, id) {
        Ok(_) => {
            let alias = match db::get_alias(pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };
            
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-show-title").await;
            let view_edit_settings = get_translation(&state, &locale, "aliases-view-edit-settings").await;
            let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
            let alias_information = get_translation(&state, &locale, "aliases-alias-information").await;
            let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
            let mail = get_translation(&state, &locale, "aliases-mail").await;
            let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
            let status = get_translation(&state, &locale, "aliases-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "aliases-created").await;
            let modified = get_translation(&state, &locale, "aliases-modified").await;
            let edit_alias_button = get_translation(&state, &locale, "aliases-edit-alias-button").await;
            let enable_alias_button = get_translation(&state, &locale, "aliases-enable-alias-button").await;
            let disable_alias_button = get_translation(&state, &locale, "aliases-disable-alias-button").await;
            let delete_alias = get_translation(&state, &locale, "aliases-delete-alias").await;
            let delete_confirm = get_translation(&state, &locale, "aliases-delete-confirm").await;
            
            let content_template = AliasShowTemplate {
                title: &title,
                alias,
                view_edit_settings: &view_edit_settings,
                back_to_aliases: &back_to_aliases,
                alias_information: &alias_information,
                alias_details: &alias_details,
                mail: &mail,
                forward_to: &forward_to,
                status: &status,
                status_active: &status_active,
                status_inactive: &status_inactive,
                created: &created,
                modified: &modified,
                edit_alias_button: &edit_alias_button,
                enable_alias_button: &enable_alias_button,
                disable_alias_button: &disable_alias_button,
                delete_alias: &delete_alias,
                delete_confirm: &delete_confirm,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-show-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}
