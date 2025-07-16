use crate::templates::layout::BaseTemplate;
use crate::templates::users::*;
use crate::{
    db, get_entity_or_not_found, i18n::get_translation, models::*, render_template,
    render_template_with_title, AppState,
};
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Html,
    Form,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChangePasswordForm {
    pub new_password: String,
    pub confirm_password: String,
}

// Helper function to get current database info
async fn get_current_db_info(state: &AppState, headers: &HeaderMap) -> (String, String) {
    let current_db_id = crate::handlers::auth::get_selected_database(headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());
    (current_db_label, current_db_id)
}

async fn build_user_list_template(
    state: &AppState,
    locale: &str,
    users: Vec<User>,
    pagination: PaginatedResult<User>,
) -> UsersListTemplate {
    let title = get_translation(state, locale, "users-title").await;
    let description = get_translation(state, locale, "users-description").await;
    let add_user = get_translation(state, locale, "users-add").await;
    let table_header_username = get_translation(state, locale, "users-table-header-username").await;
    let table_header_domain = get_translation(state, locale, "users-table-header-domain").await;
    let table_header_enabled = get_translation(state, locale, "users-table-header-enabled").await;
    let table_header_actions = get_translation(state, locale, "users-table-header-actions").await;
    let status_active = get_translation(state, locale, "status-active").await;
    let status_inactive = get_translation(state, locale, "status-inactive").await;
    let action_view = get_translation(state, locale, "action-view").await;
    let enable_user = get_translation(state, locale, "users-enable-user").await;
    let disable_user = get_translation(state, locale, "users-disable-user").await;
    let empty_title = get_translation(state, locale, "users-empty-title").await;
    let empty_description = get_translation(state, locale, "users-empty-description").await;
    let page_range: Vec<i64> = (1..=pagination.total_pages).collect();
    let max_item = std::cmp::min(
        pagination.current_page * pagination.per_page,
        pagination.total_count,
    );

    UsersListTemplate {
        title,
        description,
        add_user,
        table_header_username,
        table_header_domain,
        table_header_enabled,
        table_header_actions,
        status_active,
        status_inactive,
        action_view,
        enable_user,
        disable_user,
        empty_title,
        empty_description,
        users,
        pagination,
        page_range,
        max_item,
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
        created: get_translation(state, locale, "users-created").await,
        modified: get_translation(state, locale, "users-modified").await,
        status_active: get_translation(state, locale, "status-active").await,
        status_inactive: get_translation(state, locale, "status-inactive").await,
        edit_user: get_translation(state, locale, "users-edit-user").await,
        enable_user: get_translation(state, locale, "users-enable-user").await,
        disable_user: get_translation(state, locale, "users-disable-user").await,
        delete_user: get_translation(state, locale, "users-delete-user").await,
        delete_confirm: get_translation(state, locale, "users-delete-confirm").await,
        status: get_translation(state, locale, "users-status").await,
        password_change_required_label: get_translation(state, locale, "users-password-change-required-label").await,
        password_change_required_yes: get_translation(state, locale, "users-password-change-required-yes").await,
        password_change_required_no: get_translation(state, locale, "users-password-change-required-no").await,
        password_management_title: get_translation(state, locale, "users-password-management-title").await,
        change_password_button: get_translation(state, locale, "users-change-password-button").await,
        require_password_change_button: get_translation(state, locale, "users-require-password-change-button").await,
        user,
    }
}

async fn build_user_form_template(
    state: &AppState,
    locale: &str,
    user: Option<User>,
    form: UserForm,
    error: Option<String>,
) -> UserFormTemplate {
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
        placeholder_user_email: get_translation(state, locale, "users-placeholder-user-email")
            .await,
        placeholder_name: get_translation(state, locale, "users-placeholder-name").await,

        tooltip_user_id: get_translation(state, locale, "users-tooltip-user-id").await,
        tooltip_password: get_translation(state, locale, "users-tooltip-password").await,
        tooltip_name: get_translation(state, locale, "users-tooltip-name").await,

        tooltip_active: get_translation(state, locale, "users-tooltip-active").await,
        users_change_password: get_translation(state, locale, "users-change-password").await,
        users_change_password_tooltip: get_translation(state, locale, "users-change-password-tooltip").await,
        users_placeholder_password: get_translation(state, locale, "users-placeholder-password").await,
        password_management_title: get_translation(state, locale, "password-management-title").await,
        change_password_button: get_translation(state, locale, "change-password-button").await,
        toggle_change_password_button: get_translation(state, locale, "toggle-change-password-button").await,
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

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::utils::get_user_locale(&headers);
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    let paginated_users = match db::get_users_paginated(&pool, page, per_page) {
        Ok(users) => users,
        Err(_) => PaginatedResult::new(vec![], 0, 1, per_page),
    };
    let translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
        &[
            "users-title",
            "users-description",
            "users-add",
            "users-table-header-username",
            "users-table-header-domain",
            "users-table-header-enabled",
            "users-table-header-actions",
            "status-active",
            "status-inactive",
            "action-view",
            "users-enable-user",
            "users-disable-user",
            "users-empty-title",
            "users-empty-description",
        ],
    )
    .await;
    let paginated = PaginatedResult::new(
        paginated_users.items.clone(),
        paginated_users.total_count,
        paginated_users.current_page,
        paginated_users.per_page,
    );
    let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
    let max_item = std::cmp::min(
        paginated.current_page * paginated.per_page,
        paginated.total_count,
    );
    let content_template = UsersListTemplate {
        title: translations["users-title"].to_string(),
        description: translations["users-description"].to_string(),
        add_user: translations["users-add"].to_string(),
        table_header_username: translations["users-table-header-username"].to_string(),
        table_header_domain: translations["users-table-header-domain"].to_string(),
        table_header_enabled: translations["users-table-header-enabled"].to_string(),
        table_header_actions: translations["users-table-header-actions"].to_string(),
        status_active: translations["status-active"].to_string(),
        status_inactive: translations["status-inactive"].to_string(),
        action_view: translations["action-view"].to_string(),
        enable_user: translations["users-enable-user"].to_string(),
        disable_user: translations["users-disable-user"].to_string(),
        empty_title: translations["users-empty-title"].to_string(),
        empty_description: translations["users-empty-description"].to_string(),
        users: paginated_users.items,
        pagination: paginated,
        page_range,
        max_item,
    };
    render_template_with_title!(
        content_template,
        content_template.title,
        &state,
        &locale,
        &headers
    )
}

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::utils::get_user_locale(&headers);
    let form = UserForm {
        id: "".to_string(),
        password: "".to_string(),
        name: "".to_string(),
        enabled: true,
        change_password: false,
    };
    let translations = crate::handlers::utils::get_translations_batch(
            &state,
            &locale,
        &[
            "users-new-user",
            "form-error",
            "form-username",
            "form-password",
            "form-name",
            "form-domain",
            "form-active",
            "form-cancel",
            "form-create-user",
            "form-update-user",
            "form-placeholder-username",
            "form-placeholder-password",
            "form-placeholder-name",
            "form-placeholder-domain",
            "form-tooltip-username",
            "form-tooltip-password",
            "form-tooltip-name",
            "form-tooltip-domain",
            "form-tooltip-enable",
        ],
        )
    .await;
    let content_template = UserFormTemplate {
        title: translations["users-new-user"].to_string(),
        form_user_id: translations["form-username"].to_string(),
        form_password: translations["form-password"].to_string(),
        form_name: translations["form-name"].to_string(),
        form_active: translations["form-active"].to_string(),
        placeholder_user_email: translations["form-placeholder-username"].to_string(),
        placeholder_name: translations["form-placeholder-name"].to_string(),
        tooltip_user_id: translations["form-tooltip-username"].to_string(),
        tooltip_password: translations["form-tooltip-password"].to_string(),
        tooltip_name: translations["form-tooltip-name"].to_string(),
        tooltip_active: translations["form-tooltip-enable"].to_string(),
        users_change_password: get_translation(&state, &locale, "users-change-password").await,
        users_change_password_tooltip: get_translation(&state, &locale, "users-change-password-tooltip").await,
        users_placeholder_password: get_translation(&state, &locale, "users-placeholder-password").await,
        password_management_title: get_translation(&state, &locale, "password-management-title").await,
        change_password_button: get_translation(&state, &locale, "change-password-button").await,
        toggle_change_password_button: get_translation(&state, &locale, "toggle-change-password-button").await,
        cancel: translations["form-cancel"].to_string(),
        create_user: translations["form-create-user"].to_string(),
        update_user: translations["form-update-user"].to_string(),
        new_user: translations["users-new-user"].to_string(),
        edit_user_title: "".to_string(),
        user: None,
        form,
        error: None,
    };
    render_template!(content_template, &state, &locale, &headers)
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let user = get_entity_or_not_found!(
        db::get_user(&pool, id),
        &state,
        &crate::handlers::utils::get_user_locale(&headers),
        "users-not-found"
    );
    let locale = crate::handlers::utils::get_user_locale(&headers);
    let content_template = build_user_show_template(&state, &locale, user).await;
    render_template!(content_template, &state, &locale, &headers)
}

pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    let user = match db::get_user(&pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    let form = UserForm {
        id: user.id.clone(),
        password: "".to_string(), // Don't populate password for security
        name: user.name.clone(),
        enabled: user.enabled,
        change_password: user.change_password,
    };

    let content_template = build_user_form_template(&state, &locale, Some(user), form, None).await;
    let content = content_template.render().unwrap();

    if crate::handlers::utils::is_htmx_request(&headers) {
        Html(content)
    } else {
        let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "users-edit-title").await,
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

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<UserForm>,
) -> Html<String> {
    // Get current database ID for restriction checks
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Check database restrictions
    if let Err(_status_code) = crate::handlers::utils::check_database_restrictions(
        &state,
        &current_db_id,
        "create_user",
    ) {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let error_msg = get_translation(&state, &locale, "error-operation-not-allowed").await;
        let form_template =
            build_user_form_template(&state, &locale, None, form.clone(), Some(error_msg)).await;
        let content = form_template.render().unwrap();

        if crate::handlers::utils::is_htmx_request(&headers) {
            return Html(content);
        } else {
            let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "users-add-title").await,
                content,
                &state,
                &locale,
                current_db_label,
                current_db_id,
            )
            .await
            .unwrap();
            return Html(template.render().unwrap());
        }
    }

    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Validate required fields
    if form.id.trim().is_empty() {
        let error_msg = get_translation(&state, &locale, "validation-username-required").await;
        let form_template =
            build_user_form_template(&state, &locale, None, form.clone(), Some(error_msg)).await;
        let content = form_template.render().unwrap();

        if crate::handlers::utils::is_htmx_request(&headers) {
            Html(content)
        } else {
            let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "users-add-title").await,
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
    } else {
        // Create user directly (no domain validation needed)
        match db::create_user(&pool, form.clone()) {
            Ok(_) => {
                let users = match db::get_users(&pool) {
                    Ok(users) => users,
                    Err(e) => {
                        eprintln!("Error getting users: {e:?}");
                        vec![]
                    }
                };
                let paginated = PaginatedResult::new(users.clone(), 0, 1, 20);
                let content_template =
                    build_user_list_template(&state, &locale, users, paginated).await;
                let content = content_template.render().unwrap();

                if crate::handlers::utils::is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let (current_db_label, current_db_id) =
                        get_current_db_info(&state, &headers).await;
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, &locale, "users-title").await,
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
            Err(e) => {
                let error_msg = if e.to_string().contains("Duplicate entry") {
                    get_translation(&state, &locale, "error-duplicate-user").await
                } else {
                    get_translation(&state, &locale, "error-unexpected").await
                };

                let form_template =
                    build_user_form_template(&state, &locale, None, form.clone(), Some(error_msg))
                        .await;
                let content = form_template.render().unwrap();

                if crate::handlers::utils::is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let (current_db_label, current_db_id) =
                        get_current_db_info(&state, &headers).await;
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, &locale, "users-add-title").await,
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
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<UserForm>,
) -> Html<String> {
    // Get current database ID for restriction checks
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Check database restrictions
    if let Err(_status_code) = crate::handlers::utils::check_database_restrictions(
        &state,
        &current_db_id,
        "update_user",
    ) {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let error_msg = get_translation(&state, &locale, "error-operation-not-allowed").await;

        // Get existing user for form display
        let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
            .await
            .expect("Failed to get database pool");
        let existing_user = match db::get_user(&pool, id) {
            Ok(user) => user,
            Err(_) => return Html("User not found".to_string()),
        };

        let form_template = build_user_form_template(
            &state,
            &locale,
            Some(existing_user),
            form.clone(),
            Some(error_msg),
        )
        .await;
        let content = form_template.render().unwrap();

        if crate::handlers::utils::is_htmx_request(&headers) {
            return Html(content);
        } else {
            let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "users-edit-title").await,
                content,
                &state,
                &locale,
                current_db_label,
                current_db_id,
            )
            .await
            .unwrap();
            return Html(template.render().unwrap());
        }
    }

    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    // First get the existing user
    let existing_user = match db::get_user(&pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    // Validate required fields
    if form.id.trim().is_empty() {
        let error_msg = get_translation(&state, &locale, "validation-username-required").await;
        let form_template = build_user_form_template(
            &state,
            &locale,
            Some(existing_user),
            form.clone(),
            Some(error_msg),
        )
        .await;
        let content = form_template.render().unwrap();

        if crate::handlers::utils::is_htmx_request(&headers) {
            Html(content)
        } else {
            let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "users-edit-title").await,
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
    } else {
        match db::update_user(&pool, id, form.clone()) {
            Ok(_) => {
                let user = match db::get_user(&pool, id) {
                    Ok(user) => user,
                    Err(_) => return Html("User not found".to_string()),
                };

                let content_template = build_user_show_template(&state, &locale, user).await;
                let content = content_template.render().unwrap();

                if crate::handlers::utils::is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let (current_db_label, current_db_id) =
                        get_current_db_info(&state, &headers).await;
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, &locale, "users-show-title").await,
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
            Err(e) => {
                let error_msg = if e.to_string().contains("Duplicate entry") {
                    get_translation(&state, &locale, "error-duplicate-user").await
                } else {
                    get_translation(&state, &locale, "error-unexpected").await
                };

                let form_template = build_user_form_template(
                    &state,
                    &locale,
                    Some(existing_user),
                    form.clone(),
                    Some(error_msg),
                )
                .await;
                let content = form_template.render().unwrap();

                if crate::handlers::utils::is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let (current_db_label, current_db_id) =
                        get_current_db_info(&state, &headers).await;
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, &locale, "users-edit-title").await,
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
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::delete_user(&pool, id) {
        Ok(_) => {
            let users = db::get_users(&pool).unwrap_or_default();
            let paginated = PaginatedResult::new(users.clone(), 0, 1, 20);
            let content_template =
                build_user_list_template(&state, &locale, users, paginated).await;
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Failed to delete user".to_string()),
    }
}

pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_user_enabled(&pool, id) {
        Ok(_) => {
            let users = db::get_users(&pool).unwrap_or_default();
            let paginated = PaginatedResult::new(users.clone(), 0, 1, 20);
            let content_template =
                build_user_list_template(&state, &locale, users, paginated).await;
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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_user_enabled(&pool, id) {
        Ok(_) => {
            let users = db::get_users(&pool).unwrap_or_default();
            let paginated = PaginatedResult::new(users.clone(), 0, 1, 20);
            let content_template =
                build_user_list_template(&state, &locale, users, paginated).await;
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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_user_enabled(&pool, id) {
        Ok(_) => {
            let user = match db::get_user(&pool, id) {
                Ok(user) => user,
                Err(_) => return Html("User not found".to_string()),
            };

            let content_template = build_user_show_template(&state, &locale, user).await;
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Failed to toggle user status".to_string()),
    }
}

// --- Password management handlers ---

// GET handler for change password form
pub async fn change_password_form(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let user = match db::get_user(&pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };
    let locale = crate::handlers::language::get_user_locale(&headers);
    let content = render_change_password_form(&user, None, &state, &locale).await;
    Html(content)
}

// POST handler for change password form
pub async fn change_password_post(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<ChangePasswordForm>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);
    let user = match db::get_user(&pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };
    if form.new_password != form.confirm_password {
        let error_msg = get_translation(&state, &locale, "error-passwords-do-not-match").await;
        let content = render_change_password_form(&user, Some(error_msg), &state, &locale).await;
        return Html(content);
    }
    if form.new_password.len() < 8 {
        let error_msg = get_translation(&state, &locale, "error-password-too-short").await;
        let content = render_change_password_form(&user, Some(error_msg), &state, &locale).await;
        return Html(content);
    }
    match db::update_user_password(&pool, id, &form.new_password) {
        Ok(_) => {
            let content_template = build_user_show_template(&state, &locale, user).await;
            let content = content_template.render().unwrap();
            Html(content)
        }
        Err(_) => {
            let error_msg = get_translation(&state, &locale, "error-failed-to-update-password").await;
            let content = render_change_password_form(&user, Some(error_msg), &state, &locale).await;
            Html(content)
        }
    }
}

async fn render_change_password_form(user: &User, error: Option<String>, state: &AppState, locale: &str) -> String {
    use crate::templates::users::ChangePasswordTemplate;
    ChangePasswordTemplate {
        user: user.clone(),
        error,
        change_password_title: get_translation(state, locale, "users-change-password-title").await,
        new_password_label: get_translation(state, locale, "users-new-password-label").await,
        new_password_placeholder: get_translation(state, locale, "users-new-password-placeholder").await,
        confirm_password_label: get_translation(state, locale, "users-confirm-password-label").await,
        confirm_password_placeholder: get_translation(state, locale, "users-confirm-password-placeholder").await,
        cancel_button: get_translation(state, locale, "users-cancel-button").await,
        change_password_button: get_translation(state, locale, "users-change-password-button").await,
    }.render().unwrap()
}

pub async fn toggle_change_password(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    let user = match db::get_user(&pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    // Toggle the change_password field
    let form = UserForm {
        id: user.id.clone(),
        password: "".to_string(),
        name: user.name.clone(),
        enabled: user.enabled,
        change_password: !user.change_password, // Toggle the value
    };

    // Update the user with the toggled change_password field
    match db::update_user(&pool, id, form) {
        Ok(_) => {
            // Get the updated user
            let updated_user = match db::get_user(&pool, id) {
                Ok(user) => user,
                Err(_) => return Html("User not found".to_string()),
            };

            let content_template = build_user_show_template(&state, &locale, updated_user).await;
            let content = content_template.render().unwrap();

            if crate::handlers::utils::is_htmx_request(&headers) {
                Html(content)
            } else {
                let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "users-show-title").await,
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
        Err(_) => Html("Failed to toggle change password field".to_string()),
    }
}
