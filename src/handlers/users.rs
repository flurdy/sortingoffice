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

pub async fn list(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = "en-US"; // For now, use default locale

    let users = match db::get_users(pool) {
        Ok(users) => users,
        Err(_) => vec![],
    };

    let title = get_translation(&state, locale, "users-title").await;
    let content_template = UserListTemplate {
        title: &title,
        users,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = "en-US"; // For now, use default locale
        let template = BaseTemplate::with_i18n(
            get_translation(&state, locale, "users-title").await,
            content,
            &state,
            locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let form = UserForm {
        id: "".to_string(),
        password: "".to_string(),
        name: "".to_string(),
        domain: "example.com".to_string(),
        enabled: true,
    };

    let content_template = UserFormTemplate {
        title: "New User",
        user: None,
        form,
        error: None,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = "en-US"; // For now, use default locale
        let template = BaseTemplate::with_i18n(
            get_translation(&state, locale, "users-add-title").await,
            content,
            &state,
            locale,
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

    let user = match db::get_user(pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    let content_template = UserShowTemplate {
        title: "Show User",
        user,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = "en-US"; // For now, use default locale
        let template = BaseTemplate::with_i18n(
            get_translation(&state, locale, "users-show-title").await,
            content,
            &state,
            locale,
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

    let user = match db::get_user(pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    let form = UserForm {
        id: user.id.clone(),
        password: "".to_string(), // Don't populate password for security
        name: user.name.clone(),
        domain: user.domain.clone(),
        enabled: user.enabled,
    };

    let content_template = UserFormTemplate {
        title: "Edit User",
        user: Some(user),
        form,
        error: None,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = "en-US"; // For now, use default locale
        let template = BaseTemplate::with_i18n(
            get_translation(&state, locale, "users-edit-title").await,
            content,
            &state,
            locale,
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

    // Validate required fields
    if form.id.trim().is_empty() {
        let form_template = UserFormTemplate {
            title: "New User",
            user: None,
            form: form.clone(),
            error: Some("User ID is required.".to_string()),
        };
        let content = form_template.render().unwrap();

        if is_htmx_request(&headers) {
            Html(content)
        } else {
            let locale = "en-US"; // For now, use default locale
            let template = BaseTemplate::with_i18n(
                get_translation(&state, locale, "users-add-title").await,
                content,
                &state,
                locale,
            ).await.unwrap();
            Html(template.render().unwrap())
        }
    } else {
        // First check if the domain exists
        match db::get_domain_by_name(pool, &form.domain) {
            Ok(_) => {
                // Domain exists, proceed with user creation
                match db::create_user(pool, form.clone()) {
                    Ok(_) => {
                        let users = match db::get_users(pool) {
                            Ok(users) => users,
                            Err(e) => {
                                eprintln!("Error getting users: {:?}", e);
                                vec![]
                            }
                        };
                        let content_template = UserListTemplate {
                            title: "Users",
                            users,
                        };
                        let content = content_template.render().unwrap();

                        if is_htmx_request(&headers) {
                            Html(content)
                        } else {
                            let locale = "en-US"; // For now, use default locale
                            let template = BaseTemplate::with_i18n(
                                get_translation(&state, locale, "users-title").await,
                                content,
                                &state,
                                locale,
                            ).await.unwrap();
                            Html(template.render().unwrap())
                        }
                    }
                    Err(e) => {
                        eprintln!("Error creating user: {:?}", e);
                        
                        // Handle specific database errors with user-friendly messages
                        let error_message = match e {
                            diesel::result::Error::DatabaseError(
                                diesel::result::DatabaseErrorKind::UniqueViolation,
                                _,
                            ) => format!("A user with the email '{}' already exists.", form.id),
                            diesel::result::Error::DatabaseError(
                                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                                _,
                            ) => format!("The domain '{}' does not exist. Please create the domain first before adding users.", form.domain),
                            diesel::result::Error::DatabaseError(
                                diesel::result::DatabaseErrorKind::CheckViolation,
                                _,
                            ) => "The user data does not meet the required constraints. Please check your input.".to_string(),
                            _ => "An unexpected error occurred while creating the user. Please try again.".to_string(),
                        };

                        // Return to form with error message
                        let form_template = UserFormTemplate {
                            title: "New User",
                            user: None,
                            form: form.clone(),
                            error: Some(error_message),
                        };
                        let content = form_template.render().unwrap();

                        if is_htmx_request(&headers) {
                            Html(content)
                        } else {
                            let locale = "en-US"; // For now, use default locale
                            let template = BaseTemplate::with_i18n(
                                get_translation(&state, locale, "users-add-title").await,
                                content,
                                &state,
                                locale,
                            ).await.unwrap();
                            Html(template.render().unwrap())
                        }
                    }
                }
            }
            Err(_) => {
                // Domain doesn't exist
                let form_template = UserFormTemplate {
                    title: "New User",
                    user: None,
                    form: form.clone(),
                    error: Some(
                        format!("The domain '{}' does not exist. Please create the domain first before adding users.", form.domain)
                    ),
                };
                let content = form_template.render().unwrap();

                if is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let locale = "en-US"; // For now, use default locale
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, locale, "users-add-title").await,
                        content,
                        &state,
                        locale,
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

    // Validate required fields
    if form.id.trim().is_empty() {
        let form_template = UserFormTemplate {
            title: "Edit User",
            user: None,
            form: form.clone(),
            error: Some("User ID is required.".to_string()),
        };
        let content = form_template.render().unwrap();

        if is_htmx_request(&headers) {
            Html(content)
        } else {
            let locale = "en-US"; // For now, use default locale
            let template = BaseTemplate::with_i18n(
                get_translation(&state, locale, "users-edit-title").await,
                content,
                &state,
                locale,
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
                let content_template = UserShowTemplate {
                    title: "Show User",
                    user,
                };
                let content = content_template.render().unwrap();

                if is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let locale = "en-US"; // For now, use default locale
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, locale, "users-show-title").await,
                        content,
                        &state,
                        locale,
                    ).await.unwrap();
                    Html(template.render().unwrap())
                }
            }
            Err(e) => {
                eprintln!("Error updating user: {:?}", e);
                
                // Handle specific database errors with user-friendly messages
                let error_message = match e {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        _,
                    ) => format!("A user with the email '{}' already exists.", form.id),
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                        _,
                    ) => format!("The domain '{}' does not exist. Please create the domain first before updating the user.", form.domain),
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::CheckViolation,
                        _,
                    ) => "The user data does not meet the required constraints. Please check your input.".to_string(),
                    _ => "An unexpected error occurred while updating the user. Please try again.".to_string(),
                };

                // Return to form with error message
                let form_template = UserFormTemplate {
                    title: "Edit User",
                    user: None,
                    form: form.clone(),
                    error: Some(error_message),
                };
                let content = form_template.render().unwrap();

                if is_htmx_request(&headers) {
                    Html(content)
                } else {
                    let locale = "en-US"; // For now, use default locale
                    let template = BaseTemplate::with_i18n(
                        get_translation(&state, locale, "users-edit-title").await,
                        content,
                        &state,
                        locale,
                    ).await.unwrap();
                    Html(template.render().unwrap())
                }
            }
        }
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    match db::delete_user(pool, id) {
        Ok(_) => {
            let users = match db::get_users(pool) {
                Ok(users) => users,
                Err(_) => vec![],
            };
            let content_template = UserListTemplate {
                title: "Users",
                users,
            };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error deleting user".to_string()),
    }
}

pub async fn toggle_enabled(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    match db::toggle_user_enabled(pool, id) {
        Ok(_) => {
            // Redirect back to the show page
            let user = match db::get_user(pool, id) {
                Ok(user) => user,
                Err(_) => return Html("User not found".to_string()),
            };

            let content_template = UserShowTemplate {
                title: "Show User",
                user,
            };
            let content = content_template.render().unwrap();

            let locale = "en-US"; // For now, use default locale
            let template = BaseTemplate::with_i18n(
                get_translation(&state, locale, "users-show-title").await,
                content,
                &state,
                locale,
            ).await.unwrap();
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error toggling user status".to_string()),
    }
}

// Toggle from list: returns updated list
pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_user_enabled(pool, id) {
        Ok(_) => {
            let users = match db::get_users(pool) {
                Ok(users) => users,
                Err(_) => vec![],
            };
            let content_template = UserListTemplate {
                title: "Users",
                users,
            };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error toggling user status".to_string()),
    }
}

// Toggle from show: returns updated show
pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_user_enabled(pool, id) {
        Ok(_) => {
            let user = match db::get_user(pool, id) {
                Ok(user) => user,
                Err(_) => return Html("User not found".to_string()),
            };
            let content_template = UserShowTemplate {
                title: "Show User",
                user,
            };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error toggling user status".to_string()),
    }
}
