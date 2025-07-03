use axum::{
    extract::{Path, State},
    response::Html,
    Form,
};
use crate::{AppState, db, models::*};
use crate::templates::mailboxes::*;
use crate::templates::layout::BaseTemplate;
use askama::Template;

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;
    
    let mailboxes = match db::get_mailboxes(pool) {
        Ok(mailboxes) => mailboxes,
        Err(_) => vec![],
    };
    
    let content_template = MailboxListTemplate { title: "Mailboxes", mailboxes };
    let content = content_template.render().unwrap();
    
    let template = BaseTemplate { 
        title: "Mailboxes".to_string(), 
        content 
    };
    Html(template.render().unwrap())
}

pub async fn new() -> Html<String> {
    let form = MailboxForm {
        username: "".to_string(),
        password: "".to_string(),
        name: "".to_string(),
        domain: "example.com".to_string(),
        quota: 1073741824, // 1GB
        active: true,
    };
    
    let content_template = MailboxFormTemplate { 
        title: "New Mailbox", 
        mailbox: None, 
        form 
    };
    Html(content_template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;
    
    let mailbox = match db::get_mailbox(pool, id) {
        Ok(mailbox) => mailbox,
        Err(_) => return Html("Mailbox not found".to_string()),
    };
    
    let content_template = MailboxShowTemplate { title: "Show Mailbox", mailbox };
    let content = content_template.render().unwrap();
    
    let template = BaseTemplate { 
        title: "Show Mailbox".to_string(), 
        content 
    };
    Html(template.render().unwrap())
}

pub async fn create(
    State(state): State<AppState>,
    Form(form): Form<MailboxForm>,
) -> Html<String> {
    let pool = &state.pool;
    
    match db::create_mailbox(pool, form) {
        Ok(_) => {
            let mailboxes = match db::get_mailboxes(pool) {
                Ok(mailboxes) => mailboxes,
                Err(_) => vec![],
            };
            let content_template = MailboxListTemplate { title: "Mailboxes", mailboxes };
            let content = content_template.render().unwrap();
            
            let template = BaseTemplate { 
                title: "Mailboxes".to_string(), 
                content 
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error creating mailbox".to_string()),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Form(form): Form<MailboxForm>,
) -> Html<String> {
    let pool = &state.pool;
    
    match db::update_mailbox(pool, id, form) {
        Ok(_) => {
            let mailboxes = match db::get_mailboxes(pool) {
                Ok(mailboxes) => mailboxes,
                Err(_) => vec![],
            };
            let content_template = MailboxListTemplate { title: "Mailboxes", mailboxes };
            let content = content_template.render().unwrap();
            
            let template = BaseTemplate { 
                title: "Mailboxes".to_string(), 
                content 
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error updating mailbox".to_string()),
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;
    
    match db::delete_mailbox(pool, id) {
        Ok(_) => {
            let mailboxes = match db::get_mailboxes(pool) {
                Ok(mailboxes) => mailboxes,
                Err(_) => vec![],
            };
            let content_template = MailboxListTemplate { title: "Mailboxes", mailboxes };
            let content = content_template.render().unwrap();
            
            let template = BaseTemplate { 
                title: "Mailboxes".to_string(), 
                content 
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error deleting mailbox".to_string()),
    }
} 
