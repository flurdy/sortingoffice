use crate::templates::domains::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
    Form,
};

pub async fn list(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;

    let domains = match db::get_domains(pool) {
        Ok(domains) => domains,
        Err(_) => vec![],
    };

    let content_template = DomainListTemplate {
        title: "Domains",
        domains,
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate {
        title: "Domains".to_string(),
        content,
    };
    Html(template.render().unwrap())
}

pub async fn new() -> Html<String> {
    let form = DomainForm {
        domain: "".to_string(),
        description: "".to_string(),
        aliases: 10,
        maxquota: 0,
        quota: 0,
        transport: "virtual".to_string(),
        backupmx: false,
        enabled: true,
    };

    let content_template = DomainFormTemplate {
        title: "New Domain",
        domain: None,
        form,
    };
    Html(content_template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    let domain = match db::get_domain(pool, id) {
        Ok(domain) => domain,
        Err(_) => return Html("Domain not found".to_string()),
    };

    let content_template = DomainShowTemplate {
        title: "Show Domain",
        domain,
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate {
        title: "Show Domain".to_string(),
        content,
    };
    Html(template.render().unwrap())
}

pub async fn edit(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    let domain = match db::get_domain(pool, id) {
        Ok(domain) => domain,
        Err(_) => return Html("Domain not found".to_string()),
    };

    let form = DomainForm {
        domain: domain.domain.clone(),
        description: domain.description.clone().unwrap_or_default(),
        aliases: domain.aliases,
        maxquota: domain.maxquota,
        quota: domain.quota,
        transport: domain.transport.clone().unwrap_or_default(),
        backupmx: domain.backupmx,
        enabled: domain.enabled,
    };

    let content_template = DomainFormTemplate {
        title: "Edit Domain",
        domain: Some(domain),
        form,
    };
    Html(content_template.render().unwrap())
}

pub async fn create(State(state): State<AppState>, Form(form): Form<DomainForm>) -> Html<String> {
    let pool = &state.pool;

    let new_domain = NewDomain {
        domain: form.domain,
        description: Some(form.description),
        aliases: form.aliases,
        maxquota: form.maxquota,
        quota: form.quota,
        transport: Some(form.transport),
        backupmx: form.backupmx,
        enabled: form.enabled,
    };

    match db::create_domain(pool, new_domain) {
        Ok(_) => {
            let domains = match db::get_domains(pool) {
                Ok(domains) => domains,
                Err(_) => vec![],
            };
            let template = DomainListTemplate {
                title: "Domains",
                domains,
            };
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
            let domain = match db::get_domain(pool, id) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };
            let content_template = DomainShowTemplate {
                title: "Show Domain",
                domain,
            };
            Html(content_template.render().unwrap())
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
            let template = DomainListTemplate {
                title: "Domains",
                domains,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error deleting domain".to_string()),
    }
}

pub async fn toggle_enabled(State(state): State<AppState>, Path(id): Path<i32>) -> Html<String> {
    let pool = &state.pool;

    match db::toggle_domain_enabled(pool, id) {
        Ok(_) => {
            let domain = match db::get_domain(pool, id) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };

            let content_template = DomainShowTemplate {
                title: "Show Domain",
                domain,
            };
            let content = content_template.render().unwrap();

            let template = BaseTemplate {
                title: "Show Domain".to_string(),
                content,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error toggling domain status".to_string()),
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_domain_enabled(pool, id) {
        Ok(_) => {
            let domains = match db::get_domains(pool) {
                Ok(domains) => domains,
                Err(_) => vec![],
            };
            let template = DomainListTemplate {
                title: "Domains",
                domains,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error toggling domain status".to_string()),
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_domain_enabled(pool, id) {
        Ok(_) => {
            let domain = match db::get_domain(pool, id) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };
            let content_template = DomainShowTemplate {
                title: "Show Domain",
                domain,
            };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error toggling domain status".to_string()),
    }
}
