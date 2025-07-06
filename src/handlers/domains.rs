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

    tracing::debug!("Handling domains list request");
    let domains = match db::get_domains(pool) {
        Ok(domains) => {
            tracing::info!("Successfully retrieved {} domains", domains.len());
            domains
        },
        Err(e) => {
            tracing::error!("Failed to retrieve domains: {:?}", e);
            vec![]
        },
    };

    tracing::debug!("Rendering template with {} domains", domains.len());
    let content_template = DomainListTemplate {
        title: "Domains",
        domains,
    };
    let content = match content_template.render() {
        Ok(content) => {
            tracing::debug!("Template rendered successfully, content length: {}", content.len());
            content
        },
        Err(e) => {
            tracing::error!("Failed to render template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    let template = BaseTemplate {
        title: "Domains".to_string(),
        content,
    };
    Html(template.render().unwrap())
}

pub async fn new() -> Html<String> {
    let form = DomainForm {
        domain: "".to_string(),
        transport: "virtual".to_string(),
        enabled: true,
    };

    let content_template = DomainFormTemplate {
        title: "New Domain",
        domain: None,
        form,
        error: None,
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
        transport: domain.transport.clone().unwrap_or_default(),
        enabled: domain.enabled,
    };

    let content_template = DomainFormTemplate {
        title: "Edit Domain",
        domain: Some(domain),
        form,
        error: None,
    };
    Html(content_template.render().unwrap())
}

pub async fn create(State(state): State<AppState>, Form(form): Form<DomainForm>) -> Html<String> {
    let pool = &state.pool;

    // Validate form data
    if form.domain.trim().is_empty() {
        let content_template = DomainFormTemplate {
            title: "New Domain",
            domain: None,
            form,
            error: Some("Domain name is required. Please enter a valid domain name.".to_string()),
        };
        return Html(content_template.render().unwrap());
    }

    let new_domain = NewDomain {
        domain: form.domain.trim().to_string(),
        transport: Some(form.transport.clone()),
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
        Err(e) => {
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("A domain with the name '{}' already exists.", form.domain),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The domain data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while creating the domain. Please try again.".to_string(),
            };

            let content_template = DomainFormTemplate {
                title: "New Domain",
                domain: None,
                form,
                error: Some(error_message),
            };
            Html(content_template.render().unwrap())
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Form(form): Form<DomainForm>,
) -> Html<String> {
    let pool = &state.pool;

    // Validate form data
    if form.domain.trim().is_empty() {
        let content_template = DomainFormTemplate {
            title: "Edit Domain",
            domain: None,
            form,
            error: Some("Domain name is required. Please enter a valid domain name.".to_string()),
        };
        return Html(content_template.render().unwrap());
    }

    let domain_name = form.domain.clone();
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
        Err(e) => {
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("A domain with the name '{}' already exists.", domain_name),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The domain data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while updating the domain. Please try again.".to_string(),
            };

            // Recreate the form for error display
            let error_form = DomainForm {
                domain: domain_name,
                transport: "virtual".to_string(),
                enabled: true,
            };

            let content_template = DomainFormTemplate {
                title: "Edit Domain",
                domain: None,
                form: error_form,
                error: Some(error_message),
            };
            Html(content_template.render().unwrap())
        }
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
