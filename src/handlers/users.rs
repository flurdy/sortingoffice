use crate::templates::layout::BaseTemplate;
use crate::templates::users::*;
use crate::{db, models::*, AppState};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
    Form,
};

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;

    let users = match db::get_users(pool) {
        Ok(users) => users,
        Err(_) => vec![],
    };

    let content_template = UserListTemplate {
        title: "Users",
        users,
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate {
        title: "Users".to_string(),
        content,
    };
    Html(template.render().unwrap())
}

pub async fn new() -> Html<String> {
    let form = UserForm {
        username: "".to_string(),
        password: "".to_string(),
        name: "".to_string(),
        domain: "example.com".to_string(),
        quota: 1073741824, // 1GB
        active: true,
    };

    let content_template = UserFormTemplate {
        title: "New User",
        user: None,
        form,
        error: None,
    };
    Html(content_template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
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

    let template = BaseTemplate {
        title: "Show User".to_string(),
        content,
    };
    Html(template.render().unwrap())
}

pub async fn edit(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    let user = match db::get_user(pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };

    let form = UserForm {
        username: user.username.clone(),
        password: "".to_string(), // Don't populate password for security
        name: user.name.clone(),
        domain: user.domain.clone(),
        quota: user.quota,
        active: user.active,
    };

    let content_template = UserFormTemplate {
        title: "Edit User",
        user: Some(user),
        form,
        error: None,
    };
    Html(content_template.render().unwrap())
}

pub async fn create(State(state): State<AppState>, Form(form): Form<UserForm>) -> Html<String> {
    let pool = &state.pool;

    // First check if the domain exists
    match db::get_domain_by_name(pool, &form.domain) {
        Ok(_) => {
            // Domain exists, proceed with user creation
            match db::create_user(pool, form.clone()) {
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
                Err(e) => {
                    // Handle specific database errors
                    let error_message = match e {
                        diesel::result::Error::DatabaseError(
                            diesel::result::DatabaseErrorKind::UniqueViolation,
                            _,
                        ) => "A user with this username already exists.",
                        diesel::result::Error::DatabaseError(
                            diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                            _,
                        ) => "The specified domain does not exist. Please create the domain first.",
                        _ => "Error creating user. Please check your input and try again.",
                    };

                    // Return to form with error message
                    let form_template = UserFormTemplate {
                        title: "New User",
                        user: None,
                        form: form.clone(),
                        error: Some(error_message.to_string()),
                    };
                    let content = form_template.render().unwrap();
                    let template = BaseTemplate {
                        title: "New User".to_string(),
                        content,
                    };
                    Html(template.render().unwrap())
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
                    "The specified domain does not exist. Please create the domain first."
                        .to_string(),
                ),
            };
            let content = form_template.render().unwrap();
            let template = BaseTemplate {
                title: "New User".to_string(),
                content,
            };
            Html(template.render().unwrap())
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Form(form): Form<UserForm>,
) -> Html<String> {
    let pool = &state.pool;

    match db::update_user(pool, id, form) {
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
        Err(_) => Html("Error updating user".to_string()),
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

pub async fn toggle_active(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    match db::toggle_user_active(pool, id) {
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

            let template = BaseTemplate {
                title: "Show User".to_string(),
                content,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error toggling user status".to_string()),
    }
}

// Toggle from list: returns updated list
pub async fn toggle_active_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_user_active(pool, id) {
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
pub async fn toggle_active_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_user_active(pool, id) {
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
