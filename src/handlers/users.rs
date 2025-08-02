use crate::handlers::utils::get_current_db_info;
use crate::{
    db, get_entity_or_not_found,
    i18n::get_translation,
    models::{PaginatedResult, User, UserForm},
    render_template, render_template_with_title,
    templates::layout::BaseTemplate,
    templates::users::*,
    AppState,
};
use askama::Template;
use axum::{
    extract::{Form, Path, State},
    http::HeaderMap,
    response::Html,
};
use serde::Deserialize;
use tracing::error;

#[derive(Deserialize)]
pub struct ChangePasswordForm {
    pub new_password: String,
    pub confirm_password: String,
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
    let pagination_previous = get_translation(state, locale, "pagination-previous").await;
    let pagination_next = get_translation(state, locale, "pagination-next").await;
    let pagination_showing = get_translation(state, locale, "pagination-showing").await;
    let pagination_to = get_translation(state, locale, "pagination-to").await;
    let pagination_of = get_translation(state, locale, "pagination-of").await;
    let pagination_results = get_translation(state, locale, "pagination-results").await;
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
        pagination_previous,
        pagination_next,
        pagination_showing,
        pagination_to,
        pagination_of,
        pagination_results,
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
        full_name: get_translation(state, locale, "users-form-name").await,
        users_maildir: get_translation(state, locale, "users-maildir").await,
        users_home: get_translation(state, locale, "users-home").await,
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
        password_change_required_label: get_translation(
            state,
            locale,
            "users-password-change-required-label",
        )
        .await,
        password_change_required_yes: get_translation(
            state,
            locale,
            "users-password-change-required-yes",
        )
        .await,
        password_change_required_no: get_translation(
            state,
            locale,
            "users-password-change-required-no",
        )
        .await,
        password_management_title: get_translation(
            state,
            locale,
            "users-password-management-title",
        )
        .await,
        change_password_button: get_translation(state, locale, "users-change-password-button")
            .await,
        require_password_change_button: get_translation(
            state,
            locale,
            "users-require-password-change-button",
        )
        .await,
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
    // Use helper functions to fetch translations in batches
    let form_translations = crate::handlers::utils::get_translations_batch(
        state,
        locale,
        &[
            "users-edit-user-title",
            "users-new-user",
            "users-change-password",
            "users-change-password-tooltip",
            "password-management-title",
            "change-password-button",
            "toggle-change-password-button",
            "form-cancel",
            "action-save",
        ],
    )
    .await;
    let field_translations = crate::handlers::utils::get_field_translations(
        state,
        locale,
        "users",
        &["id", "password", "name", "active", "maildir", "home"],
    )
    .await;

    let title = if user.is_some() {
        form_translations["users-edit-user-title"].clone()
    } else {
        form_translations["users-new-user"].clone()
    };

    UserFormTemplate {
        title,
        form_user_id: field_translations
            .get("users-field-id")
            .unwrap_or(&"User ID".to_string())
            .clone(),
        form_password: field_translations
            .get("users-field-password")
            .unwrap_or(&"Password".to_string())
            .clone(),
        form_name: field_translations
            .get("users-field-name")
            .unwrap_or(&"Name".to_string())
            .clone(),
        form_active: field_translations
            .get("users-field-active")
            .unwrap_or(&"Active".to_string())
            .clone(),
        placeholder_user_email: field_translations
            .get("users-placeholder-user-email")
            .unwrap_or(&"Enter user email".to_string())
            .clone(),
        placeholder_name: field_translations
            .get("users-placeholder-name")
            .unwrap_or(&"Enter name".to_string())
            .clone(),
        tooltip_user_id: field_translations
            .get("users-field-id-help")
            .unwrap_or(&"User ID tooltip".to_string())
            .clone(),
        tooltip_password: field_translations
            .get("users-field-password-help")
            .unwrap_or(&"Password tooltip".to_string())
            .clone(),
        tooltip_name: field_translations
            .get("users-field-name-help")
            .unwrap_or(&"Name tooltip".to_string())
            .clone(),
        tooltip_active: field_translations
            .get("users-field-active-help")
            .unwrap_or(&"Active tooltip".to_string())
            .clone(),
        users_change_password: form_translations
            .get("users-change-password")
            .unwrap_or(&"Change Password".to_string())
            .clone(),
        users_change_password_tooltip: form_translations
            .get("users-change-password-tooltip")
            .unwrap_or(&"Change user password".to_string())
            .clone(),
        users_placeholder_password: field_translations
            .get("users-placeholder-password")
            .unwrap_or(&"Enter password".to_string())
            .clone(),
        password_management_title: form_translations
            .get("password-management-title")
            .unwrap_or(&"Password Management".to_string())
            .clone(),
        change_password_button: form_translations
            .get("change-password-button")
            .unwrap_or(&"Change Password".to_string())
            .clone(),
        toggle_change_password_button: form_translations
            .get("toggle-change-password-button")
            .unwrap_or(&"Toggle Change Password".to_string())
            .clone(),
        cancel: form_translations
            .get("form-cancel")
            .unwrap_or(&"Cancel".to_string())
            .clone(),
        create_user: form_translations
            .get("action-save")
            .unwrap_or(&"Save".to_string())
            .clone(),
        update_user: form_translations
            .get("action-save")
            .unwrap_or(&"Save".to_string())
            .clone(),
        new_user: form_translations
            .get("users-new-user")
            .unwrap_or(&"New User".to_string())
            .clone(),
        edit_user_title: form_translations
            .get("users-edit-user-title")
            .unwrap_or(&"Edit User".to_string())
            .clone(),
        user,
        form,
        error,
        users_maildir: field_translations
            .get("users-field-maildir")
            .unwrap_or(&"Maildir".to_string())
            .clone(),
        users_tooltip_maildir: field_translations
            .get("users-field-maildir-help")
            .unwrap_or(&"Maildir tooltip".to_string())
            .clone(),
        users_placeholder_maildir: field_translations
            .get("users-placeholder-maildir")
            .unwrap_or(&"Enter maildir".to_string())
            .clone(),
        users_home: field_translations
            .get("users-field-home")
            .unwrap_or(&"Home".to_string())
            .clone(),
        users_tooltip_home: field_translations
            .get("users-field-home-help")
            .unwrap_or(&"Home tooltip".to_string())
            .clone(),
        users_placeholder_home: field_translations
            .get("users-placeholder-home")
            .unwrap_or(&"Enter home".to_string())
            .clone(),
    }
}

pub async fn list(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::utils::get_user_locale(&headers);
    // Get users with error handling
    let users = match db::get_users(&pool) {
        Ok(users) => users,
        Err(e) => {
            error!("Failed to retrieve users: {:?}", e);
            vec![]
        }
    };
    // Dummy pagination for now
    let paginated = PaginatedResult::new(users.clone(), users.len() as i64, 1, users.len() as i64);
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
    let page_range: Vec<i64> = vec![1];
    let max_item = users.len() as i64;
    let pagination_translations =
        crate::handlers::utils::get_pagination_translations(&state, &locale).await;
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
        users,
        pagination: paginated,
        page_range,
        max_item,
        pagination_previous: pagination_translations["pagination-previous"].clone(),
        pagination_next: pagination_translations["pagination-next"].clone(),
        pagination_showing: pagination_translations["pagination-showing"].clone(),
        pagination_to: pagination_translations["pagination-to"].clone(),
        pagination_of: pagination_translations["pagination-of"].clone(),
        pagination_results: pagination_translations["pagination-results"].clone(),
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
        maildir: "".to_string(),
        home: "/var/spool/mail/maildir".to_string(),
        enabled: true,
        change_password: false,
    };

    let content_template = build_user_form_template(&state, &locale, None, form, None).await;

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        "users-add-title".to_string(),
    )
    .await
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<String>,
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
    Path(id): Path<String>,
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
        maildir: user.maildir.clone(),
        home: user.home.clone(),
        enabled: user.enabled,
        change_password: user.change_password,
    };

    let content_template = build_user_form_template(&state, &locale, Some(user), form, None).await;

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        "users-edit-title".to_string(),
    )
    .await
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
    if let Err(_status_code) =
        crate::handlers::utils::check_database_restrictions(&state, &current_db_id, "create_user")
    {
        // Return error form for restrictions
        let locale = crate::handlers::utils::get_user_locale(&headers);
        let error_message = get_translation(&state, &locale, "error-database-restriction").await;
        let form_template =
            build_user_form_template(&state, &locale, None, form.clone(), Some(error_message))
                .await;
        let content = form_template.render().unwrap();

        if crate::handlers::utils::is_htmx_request(&headers) {
            return Html(content);
        } else {
            let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "users-new-user").await,
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

    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Validate user ID
    match crate::validation::validate_user_id(&form.id) {
        Ok(_) => {}
        Err(_e) => {
            let error_message =
                get_translation(&state, &locale, "validation-user-id-invalid").await;
            let form_template =
                build_user_form_template(&state, &locale, None, form.clone(), Some(error_message))
                    .await;
            let content = form_template.render().unwrap();

            if crate::handlers::utils::is_htmx_request(&headers) {
                return Html(content);
            } else {
                let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "users-new-user").await,
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
    }

    // Validate password is not empty
    if form.password.trim().is_empty() {
        let error_message = get_translation(&state, &locale, "validation-password-required").await;
        let form_template =
            build_user_form_template(&state, &locale, None, form.clone(), Some(error_message))
                .await;
        let content = form_template.render().unwrap();

        if crate::handlers::utils::is_htmx_request(&headers) {
            return Html(content);
        } else {
            let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "users-new-user").await,
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

    // Validate user paths
    if !form.maildir.is_empty() && !form.home.is_empty() {
        let combined_maildir_path = format!("{}/{}", form.home, form.maildir);
        match crate::validation::validate_user_path(&combined_maildir_path) {
            Ok(_) => {}
            Err(_e) => {
                let error_message =
                    get_translation(&state, &locale, "validation-user-path-invalid").await;
                let form_template = build_user_form_template(
                    &state,
                    &locale,
                    None,
                    form.clone(),
                    Some(error_message),
                )
                .await;
                let content = form_template.render().unwrap();

                if crate::handlers::utils::is_htmx_request(&headers) {
                    return Html(content);
                } else {
                    let (current_db_label, current_db_id) =
                        get_current_db_info(&state, &headers).await;
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, &locale, "users-new-user").await,
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
        }
    }

    match crate::validation::validate_user_path(&form.home) {
        Ok(_) => {}
        Err(_e) => {
            let error_message =
                get_translation(&state, &locale, "validation-user-path-invalid").await;
            let form_template =
                build_user_form_template(&state, &locale, None, form.clone(), Some(error_message))
                    .await;
            let content = form_template.render().unwrap();

            if crate::handlers::utils::is_htmx_request(&headers) {
                return Html(content);
            } else {
                let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "users-new-user").await,
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
    }

    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    // Create user directly (no domain validation needed)
    match db::create_user(&pool, form.clone()) {
        Ok(_) => {
            let users = match db::get_users(&pool) {
                Ok(users) => users,
                Err(e) => {
                    error!("Failed to retrieve users after creation: {:?}", e);
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
                let (current_db_label, current_db_id) = get_current_db_info(&state, &headers).await;
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
            let error_message =
                crate::handlers::utils::handle_database_error(&state, &locale, e, "user", &form.id)
                    .await;

            let form_template =
                build_user_form_template(&state, &locale, None, form.clone(), Some(error_message))
                    .await;

            // Use helper function for template rendering
            crate::handlers::utils::render_form_template(
                form_template,
                &state,
                &locale,
                &headers,
                "users-add-title".to_string(),
            )
            .await
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Form(form): Form<UserForm>,
) -> Html<String> {
    // Get current database ID for restriction checks
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Check database restrictions
    if let Err(_status_code) =
        crate::handlers::utils::check_database_restrictions(&state, &current_db_id, "update_user")
    {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let error_msg = get_translation(&state, &locale, "error-operation-not-allowed").await;

        // Get existing user for form display
        let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
            .await
            .expect("Failed to get database pool");
        let existing_user = match db::get_user(&pool, id.clone()) {
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
    let existing_user = match db::get_user(&pool, id.clone()) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    // Validate required fields
    if form.id.trim().is_empty() {
        let form_translations =
            crate::handlers::utils::get_entity_form_translations(&state, &locale, "users").await;
        let error_msg = form_translations["validation-username-required"].clone();
        let form_template = build_user_form_template(
            &state,
            &locale,
            Some(existing_user),
            form.clone(),
            Some(error_msg),
        )
        .await;

        // Use helper function for template rendering
        crate::handlers::utils::render_form_template(
            form_template,
            &state,
            &locale,
            &headers,
            "users-edit-title".to_string(),
        )
        .await
    } else {
        match db::update_user(&pool, id.clone(), form.clone()) {
            Ok(_) => {
                let user = match db::get_user(&pool, id.clone()) {
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
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::delete_user(&pool, id) {
        Ok(_) => {
            let users = match db::get_users(&pool) {
                Ok(users) => users,
                Err(e) => {
                    error!("Failed to retrieve users after deletion: {:?}", e);
                    vec![]
                }
            };
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
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_user_enabled(&pool, id.clone()) {
        Ok(_) => {
            let users = match db::get_users(&pool) {
                Ok(users) => users,
                Err(e) => {
                    error!("Failed to retrieve users after toggle: {:?}", e);
                    vec![]
                }
            };
            let paginated = PaginatedResult::new(users.clone(), 0, 1, 20);
            let content_template =
                build_user_list_template(&state, &locale, users, paginated).await;
            Html(content_template.render().unwrap())
        }
        Err(e) => {
            error!("Failed to toggle user status: {:?}", e);
            Html("Failed to toggle user status".to_string())
        }
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_user_enabled(&pool, id.clone()) {
        Ok(_) => {
            let users = match db::get_users(&pool) {
                Ok(users) => users,
                Err(e) => {
                    error!("Failed to retrieve users after toggle: {:?}", e);
                    vec![]
                }
            };
            let paginated = PaginatedResult::new(users.clone(), 0, 1, 20);
            let content_template =
                build_user_list_template(&state, &locale, users, paginated).await;
            Html(content_template.render().unwrap())
        }
        Err(e) => {
            error!("Failed to toggle user status: {:?}", e);
            Html("Failed to toggle user status".to_string())
        }
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_user_enabled(&pool, id.clone()) {
        Ok(_) => {
            let user = match db::get_user(&pool, id.clone()) {
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
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let user = match db::get_user(&pool, id.clone()) {
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
    Path(id): Path<String>,
    headers: HeaderMap,
    Form(form): Form<ChangePasswordForm>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);
    let user = match db::get_user(&pool, id.clone()) {
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
    match db::update_user_password(&pool, id.clone(), &form.new_password) {
        Ok(_) => {
            let content_template = build_user_show_template(&state, &locale, user).await;
            let content = content_template.render().unwrap();
            Html(content)
        }
        Err(_) => {
            let error_msg =
                get_translation(&state, &locale, "error-failed-to-update-password").await;
            let content =
                render_change_password_form(&user, Some(error_msg), &state, &locale).await;
            Html(content)
        }
    }
}

async fn render_change_password_form(
    user: &User,
    error: Option<String>,
    state: &AppState,
    locale: &str,
) -> String {
    use crate::templates::users::ChangePasswordTemplate;
    ChangePasswordTemplate {
        user: user.clone(),
        error,
        change_password_title: get_translation(state, locale, "users-change-password-title").await,
        user_email_label: get_translation(state, locale, "users-change-password-user-email").await,
        new_password_label: get_translation(state, locale, "users-new-password-label").await,
        new_password_placeholder: get_translation(state, locale, "users-new-password-placeholder")
            .await,
        confirm_password_label: get_translation(state, locale, "users-confirm-password-label")
            .await,
        confirm_password_placeholder: get_translation(
            state,
            locale,
            "users-confirm-password-placeholder",
        )
        .await,
        cancel_button: get_translation(state, locale, "users-cancel-button").await,
        change_password_button: get_translation(state, locale, "users-change-password-button")
            .await,
    }
    .render()
    .unwrap()
}

pub async fn toggle_change_password(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    let user = match db::get_user(&pool, id.clone()) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    // Toggle the change_password field
    let form = UserForm {
        id: user.id.clone(),
        password: "".to_string(),
        name: user.name.clone(),
        maildir: user.maildir.clone(),
        home: user.home.clone(),
        enabled: user.enabled,
        change_password: !user.change_password, // Toggle the value
    };

    // Update the user with the toggled change_password field
    match db::update_user(&pool, id.clone(), form) {
        Ok(_) => {
            // Get the updated user
            let updated_user = match db::get_user(&pool, id.clone()) {
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
