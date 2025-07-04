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
        form,
        error: None
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

pub async fn edit(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;
    
    let mailbox = match db::get_mailbox(pool, id) {
        Ok(mailbox) => mailbox,
        Err(_) => return Html("Mailbox not found".to_string()),
    };
    
    let form = MailboxForm {
        username: mailbox.username.clone(),
        password: "".to_string(), // Don't populate password for security
        name: mailbox.name.clone(),
        domain: mailbox.domain.clone(),
        quota: mailbox.quota,
        active: mailbox.active,
    };
    
    let content_template = MailboxFormTemplate { 
        title: "Edit Mailbox", 
        mailbox: Some(mailbox), 
        form,
        error: None
    };
    let content = content_template.render().unwrap();
    
    let template = BaseTemplate { 
        title: "Edit Mailbox".to_string(), 
        content 
    };
    Html(template.render().unwrap())
}

pub async fn create(
    State(state): State<AppState>,
    Form(form): Form<MailboxForm>,
) -> Html<String> {
    let pool = &state.pool;
    
    // First check if the domain exists
    match db::get_domain_by_name(pool, &form.domain) {
        Ok(_) => {
            // Domain exists, proceed with mailbox creation
            match db::create_mailbox(pool, form.clone()) {
                Ok(_) => {
                    let mailboxes = match db::get_mailboxes(pool) {
                        Ok(mailboxes) => mailboxes,
                        Err(_) => vec![],
                    };
                    let content_template = MailboxListTemplate { title: "Mailboxes", mailboxes };
                    Html(content_template.render().unwrap())
                }
                Err(e) => {
                    // Handle specific database errors
                    let error_message = match e {
                        diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => {
                            "A mailbox with this username already exists."
                        }
                        diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _) => {
                            "The specified domain does not exist. Please create the domain first."
                        }
                        _ => "Error creating mailbox. Please check your input and try again."
                    };
                    
                    // Return to form with error message
                    let form_template = MailboxFormTemplate { 
                        title: "New Mailbox", 
                        mailbox: None, 
                        form,
                        error: Some(error_message.to_string())
                    };
                    let content = form_template.render().unwrap();
                    let template = BaseTemplate { 
                        title: "New Mailbox".to_string(), 
                        content 
                    };
                    Html(template.render().unwrap())
                }
            }
        }
        Err(_) => {
            // Domain doesn't exist
            let form_template = MailboxFormTemplate { 
                title: "New Mailbox", 
                mailbox: None, 
                form: form.clone(),
                error: Some("The specified domain does not exist. Please create the domain first.".to_string())
            };
            let content = form_template.render().unwrap();
            let template = BaseTemplate { 
                title: "New Mailbox".to_string(), 
                content 
            };
            Html(template.render().unwrap())
        }
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
            Html(content_template.render().unwrap())
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
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error deleting mailbox".to_string()),
    }
} 
