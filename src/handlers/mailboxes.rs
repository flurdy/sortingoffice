use axum::{
    extract::{Path, State},
    response::Html,
    Form,
};
use crate::{AppState, db, models::*};
use crate::templates::mailboxes::*;

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;
    
    let mailboxes = match db::get_mailboxes(pool) {
        Ok(mailboxes) => mailboxes,
        Err(_) => vec![],
    };
    
    let template = MailboxListTemplate { mailboxes };
    Html(template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;
    
    let mailbox = match db::get_mailbox(pool, id) {
        Ok(mailbox) => mailbox,
        Err(_) => return Html("Mailbox not found".to_string()),
    };
    
    let template = MailboxShowTemplate { mailbox };
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
            let template = MailboxListTemplate { mailboxes };
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
            let template = MailboxListTemplate { mailboxes };
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
            let template = MailboxListTemplate { mailboxes };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error deleting mailbox".to_string()),
    }
} 
