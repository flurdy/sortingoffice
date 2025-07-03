use axum::{
    extract::{Path, State},
    response::Html,
    Form,
};
use crate::{AppState, db, models::*};
use crate::templates::users::*;
use askama::Template;

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;
    
    let users = match db::get_users(pool) {
        Ok(users) => users,
        Err(_) => vec![],
    };
    
    let template = UserListTemplate { title: "Users", users };
    Html(template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;
    
    let user = match db::get_user(pool, id) {
        Ok(user) => user,
        Err(_) => return Html("User not found".to_string()),
    };
    
    let template = UserShowTemplate { title: "Show User", user };
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
            let template = UserListTemplate { title: "Users", users };
            Html(template.render().unwrap())
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
            let template = UserListTemplate { title: "Users", users };
            Html(template.render().unwrap())
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
            let template = UserListTemplate { title: "Users", users };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error deleting user".to_string()),
    }
} 
