use axum::{
    extract::{Path, State},
    response::Html,
    Form,
};
use crate::{AppState, db, models::*};
use crate::templates::aliases::*;
use crate::templates::layout::BaseTemplate;
use askama::Template;

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;
    
    let aliases = match db::get_aliases(pool) {
        Ok(aliases) => aliases,
        Err(_) => vec![],
    };
    
    let content_template = AliasListTemplate { title: "Aliases", aliases };
    let content = content_template.render().unwrap();
    
    let template = BaseTemplate { 
        title: "Aliases".to_string(), 
        content 
    };
    Html(template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;
    
    let alias = match db::get_alias(pool, id) {
        Ok(alias) => alias,
        Err(_) => return Html("Alias not found".to_string()),
    };
    
    let content_template = AliasShowTemplate { title: "Show Alias", alias };
    let content = content_template.render().unwrap();
    
    let template = BaseTemplate { 
        title: "Show Alias".to_string(), 
        content 
    };
    Html(template.render().unwrap())
}

pub async fn create(
    State(state): State<AppState>,
    Form(form): Form<AliasForm>,
) -> Html<String> {
    let pool = &state.pool;
    
    match db::create_alias(pool, form) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(_) => vec![],
            };
            let content_template = AliasListTemplate { title: "Aliases", aliases };
            let content = content_template.render().unwrap();
            
            let template = BaseTemplate { 
                title: "Aliases".to_string(), 
                content 
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error creating alias".to_string()),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Form(form): Form<AliasForm>,
) -> Html<String> {
    let pool = &state.pool;
    
    match db::update_alias(pool, id, form) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(_) => vec![],
            };
            let content_template = AliasListTemplate { title: "Aliases", aliases };
            let content = content_template.render().unwrap();
            
            let template = BaseTemplate { 
                title: "Aliases".to_string(), 
                content 
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error updating alias".to_string()),
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;
    
    match db::delete_alias(pool, id) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(_) => vec![],
            };
            let content_template = AliasListTemplate { title: "Aliases", aliases };
            let content = content_template.render().unwrap();
            
            let template = BaseTemplate { 
                title: "Aliases".to_string(), 
                content 
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error deleting alias".to_string()),
    }
} 
