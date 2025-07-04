use axum::{
    extract::{Path, State},
    response::Html,
    Form,
};
use crate::{AppState, db, models::*};
use crate::templates::users::*;
use crate::templates::layout::BaseTemplate;
use askama::Template;

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;
    
    let users = match db::get_users(pool) {
        Ok(users) => users,
        Err(_) => vec![],
    };
    
    let content_template = UserListTemplate { title: "Users", users };
    let content = content_template.render().unwrap();
    
    let template = BaseTemplate { 
        title: "Users".to_string(), 
        content 
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
        form 
    };
    Html(content_template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;
    
    let user = match db::get_user(pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };
    
    let content_template = UserShowTemplate { title: "Show User", user };
    let content = content_template.render().unwrap();
    
    let template = BaseTemplate { 
        title: "Show User".to_string(), 
        content 
    };
    Html(template.render().unwrap())
}

pub async fn create(
    State(state): State<AppState>,
    Form(form): Form<UserForm>,
) -> Html<String> {
    let pool = &state.pool;
    
    match db::create_user(pool, form) {
        Ok(_) => {
            let users = match db::get_users(pool) {
                Ok(users) => users,
                Err(_) => vec![],
            };
            let content_template = UserListTemplate { title: "Users", users };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error creating user".to_string()),
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
            let users = match db::get_users(pool) {
                Ok(users) => users,
                Err(_) => vec![],
            };
            let content_template = UserListTemplate { title: "Users", users };
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
            let content_template = UserListTemplate { title: "Users", users };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error deleting user".to_string()),
    }
} 
