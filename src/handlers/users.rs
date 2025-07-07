use crate::templates::layout::BaseTemplate;
use crate::templates::users::*;
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

async fn build_user_list_template(state: &AppState, locale: &str, users: Vec<User>) -> UserListTemplate {
    UserListTemplate {
        title: get_translation(state, locale, "users-title").await,
        description: get_translation(state, locale, "users-description").await,
        add_user: get_translation(state, locale, "users-add").await,
        table_header_user_id: get_translation(state, locale, "users-table-header-user-id").await,
        table_header_name: get_translation(state, locale, "users-table-header-name").await,

        table_header_status: get_translation(state, locale, "users-table-header-status").await,
        table_header_actions: get_translation(state, locale, "users-table-header-actions").await,
        status_active: get_translation(state, locale, "status-active").await,
        status_inactive: get_translation(state, locale, "status-inactive").await,
        view: get_translation(state, locale, "users-view").await,
        enable: get_translation(state, locale, "users-enable").await,
        disable: get_translation(state, locale, "users-disable").await,
        empty_no_users: get_translation(state, locale, "users-empty-no-users").await,
        empty_get_started: get_translation(state, locale, "users-empty-get-started").await,
        users,
    }
}

async fn build_user_show_template(state: &AppState, locale: &str, user: User) -> UserShowTemplate {
    UserShowTemplate {
        title: get_translation(state, locale, "users-show-user-title").await,
        view_edit_settings: get_translation(state, locale, "users-view-edit-settings").await,
        back_to_users: get_translation(state, locale, "users-back-to-users").await,
        user_information: get_translation(state, locale, "users-user-information").await,
        user_details: get_translation(state, locale, "users-user-details").await,
        user_id: get_translation(state, locale, "users-user-id").await,
        full_name: get_translation(state, locale, "users-full-name").await,

        status: get_translation(state, locale, "users-status").await,
        created: get_translation(state, locale, "users-created").await,
        modified: get_translation(state, locale, "users-modified").await,
        status_active: get_translation(state, locale, "status-active").await,
        status_inactive: get_translation(state, locale, "status-inactive").await,
        edit_user: get_translation(state, locale, "users-edit-user").await,
        enable_user: get_translation(state, locale, "users-enable-user").await,
        disable_user: get_translation(state, locale, "users-disable-user").await,
        delete_user: get_translation(state, locale, "users-delete-user").await,
        delete_confirm: get_translation(state, locale, "users-delete-confirm").await,
        user,
    }
}

async fn build_user_form_template(state: &AppState, locale: &str, user: Option<User>, form: UserForm, error: Option<String>) -> UserFormTemplate {
    let title = if user.is_some() {
        get_translation(state, locale, "users-edit-user-title").await
    } else {
        get_translation(state, locale, "users-new-user").await
    };

    UserFormTemplate {
        title,
        form_user_id: get_translation(state, locale, "users-form-user-id").await,
        form_password: get_translation(state, locale, "users-form-password").await,
        form_name: get_translation(state, locale, "users-form-name").await,

        form_active: get_translation(state, locale, "users-form-active").await,
        placeholder_user_email: get_translation(state, locale, "users-placeholder-user-email").await,
        placeholder_name: get_translation(state, locale, "users-placeholder-name").await,

        tooltip_user_id: get_translation(state, locale, "users-tooltip-user-id").await,
        tooltip_password: get_translation(state, locale, "users-tooltip-password").await,
        tooltip_name: get_translation(state, locale, "users-tooltip-name").await,

        tooltip_active: get_translation(state, locale, "users-tooltip-active").await,
        cancel: get_translation(state, locale, "users-cancel").await,
        create_user: get_translation(state, locale, "users-create-user").await,
        update_user: get_translation(state, locale, "users-update-user").await,
        new_user: get_translation(state, locale, "users-new-user").await,
        edit_user_title: get_translation(state, locale, "users-edit-user-title").await,
        user,
        form,
        error,
    }
}

pub async fn list(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    let users = match db::get_users(pool) {
        Ok(users) => users,
        Err(_) => vec![],
    };

    let content_template = build_user_list_template(&state, &locale, users).await;
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "users-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let form = UserForm {
        id: "".to_string(),
        password: "".to_string(),
        name: "".to_string(),
        enabled: true,
    };

    let content_template = build_user_form_template(&state, &locale, None, form, None).await;
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "users-add-title").await,
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
    let locale = crate::handlers::language::get_user_locale(&headers);

    let user = match db::get_user(pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    let content_template = build_user_show_template(&state, &locale, user).await;
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "users-show-title").await,
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
    let locale = crate::handlers::language::get_user_locale(&headers);

    let user = match db::get_user(pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    let form = UserForm {
        id: user.id.clone(),
        password: "".to_string(), // Don't populate password for security
        name: user.name.clone(),
        enabled: user.enabled,
    };

    let content_template = build_user_form_template(&state, &locale, Some(user), form, None).await;
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "users-edit-title").await,
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
    Form(form): Form<UserForm>,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Validate required fields
    if form.id.trim().is_empty() {
        let error_msg = get_translation(&state, &locale, "validation-username-required").await;
        let form_template = build_user_form_template(&state, &locale, None, form.clone(), Some(error_msg)).await;
        let content = form_template.render().unwrap();

        if is_htmx_request(&headers) {
            Html(content)
        } else {
            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "users-add-title").await,
                content,
                &state,
                &locale,
            ).await.unwrap();
            Html(template.render().unwrap())
        }
    } else {
        // Create user directly (no domain validation needed)
        match db::create_user(pool, form.clone()) {
            Ok(_) => {
                let users = match db::get_users(pool) {
                    Ok(users) => users,
                    Err(e) => {
                        eprintln!("Error getting users: {:?}", e);
                        vec![]
                    }
                };
                let content_template = build_user_list_template(&state, &locale, users).await;
                let content = content_template.render().unwrap();

                if is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, &locale, "users-title").await,
                        content,
                        &state,
                        &locale,
                    ).await.unwrap();
                    Html(template.render().unwrap())
                }
            }
            Err(e) => {
                let error_msg = if e.to_string().contains("Duplicate entry") {
                    get_translation(&state, &locale, "error-duplicate-user").await
                } else {
                    get_translation(&state, &locale, "error-unexpected").await
                };
                
                let form_template = build_user_form_template(&state, &locale, None, form.clone(), Some(error_msg)).await;
                let content = form_template.render().unwrap();

                if is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, &locale, "users-add-title").await,
                        content,
                        &state,
                        &locale,
                    ).await.unwrap();
                    Html(template.render().unwrap())
                }
            }
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<UserForm>,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    // First get the existing user
    let existing_user = match db::get_user(pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    // Validate required fields
    if form.id.trim().is_empty() {
        let error_msg = get_translation(&state, &locale, "validation-username-required").await;
        let form_template = build_user_form_template(&state, &locale, Some(existing_user), form.clone(), Some(error_msg)).await;
        let content = form_template.render().unwrap();

        if is_htmx_request(&headers) {
            Html(content)
        } else {
            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "users-edit-title").await,
                content,
                &state,
                &locale,
            ).await.unwrap();
            Html(template.render().unwrap())
        }
    } else {
        match db::update_user(pool, id, form.clone()) {
            Ok(_) => {
                let user = match db::get_user(pool, id) {
                    Ok(user) => user,
                    Err(_) => return Html("User not found".to_string()),
                };

                let content_template = build_user_show_template(&state, &locale, user).await;
                let content = content_template.render().unwrap();

                if is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, &locale, "users-show-title").await,
                        content,
                        &state,
                        &locale,
                    ).await.unwrap();
                    Html(template.render().unwrap())
                }
            }
            Err(e) => {
                let error_msg = if e.to_string().contains("Duplicate entry") {
                    get_translation(&state, &locale, "error-duplicate-user").await
                } else {
                    get_translation(&state, &locale, "error-unexpected").await
                };

                let form_template = build_user_form_template(&state, &locale, Some(existing_user), form.clone(), Some(error_msg)).await;
                let content = form_template.render().unwrap();

                if is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, &locale, "users-edit-title").await,
                        content,
                        &state,
                        &locale,
                    ).await.unwrap();
                    Html(template.render().unwrap())
                }
            }
        }
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i32>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::delete_user(pool, id) {
        Ok(_) => {
            let users = match db::get_users(pool) {
                Ok(users) => users,
                Err(_) => vec![],
            };

            let content_template = build_user_list_template(&state, &locale, users).await;
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Failed to delete user".to_string()),
    }
}

pub async fn toggle_enabled(State(state): State<AppState>, Path(id): Path<i32>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_user_enabled(pool, id) {
        Ok(_) => {
            let users = match db::get_users(pool) {
                Ok(users) => users,
                Err(_) => vec![],
            };

            let content_template = build_user_list_template(&state, &locale, users).await;
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Failed to toggle user status".to_string()),
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_user_enabled(pool, id) {
        Ok(_) => {
            let users = match db::get_users(pool) {
                Ok(users) => users,
                Err(_) => vec![],
            };

            let content_template = build_user_list_template(&state, &locale, users).await;
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Failed to toggle user status".to_string()),
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_user_enabled(pool, id) {
        Ok(_) => {
            let user = match db::get_user(pool, id) {
                Ok(user) => user,
                Err(_) => return Html("User not found".to_string()),
            };

            let content_template = build_user_show_template(&state, &locale, user).await;
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Failed to toggle user status".to_string()),
    }
}
