use axum::{
    extract::{Path, State},
    response::Html,
    Form,
};
use crate::{AppState, db, models::*};
use crate::templates::domains::*;

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;
    
    let domains = match db::get_domains(pool) {
        Ok(domains) => domains,
        Err(_) => vec![],
    };
    
    let template = DomainListTemplate { domains };
    Html(template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;
    
    let domain = match db::get_domain(pool, id) {
        Ok(domain) => domain,
        Err(_) => return Html("Domain not found".to_string()),
    };
    
    let template = DomainShowTemplate { domain };
    Html(template.render().unwrap())
}

pub async fn create(
    State(state): State<AppState>,
    Form(form): Form<DomainForm>,
) -> Html<String> {
    let pool = &state.pool;
    
    let new_domain = NewDomain {
        domain: form.domain,
        description: form.description,
        aliases: form.aliases,
        mailboxes: form.mailboxes,
        maxquota: form.maxquota,
        quota: form.quota,
        transport: form.transport,
        backupmx: form.backupmx,
        active: form.active,
    };
    
    match db::create_domain(pool, new_domain) {
        Ok(_) => {
            let domains = match db::get_domains(pool) {
                Ok(domains) => domains,
                Err(_) => vec![],
            };
            let template = DomainListTemplate { domains };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error creating domain".to_string()),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Form(form): Form<DomainForm>,
) -> Html<String> {
    let pool = &state.pool;
    
    match db::update_domain(pool, id, form) {
        Ok(_) => {
            let domains = match db::get_domains(pool) {
                Ok(domains) => domains,
                Err(_) => vec![],
            };
            let template = DomainListTemplate { domains };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error updating domain".to_string()),
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;
    
    match db::delete_domain(pool, id) {
        Ok(_) => {
            let domains = match db::get_domains(pool) {
                Ok(domains) => domains,
                Err(_) => vec![],
            };
            let template = DomainListTemplate { domains };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error deleting domain".to_string()),
    }
} 
