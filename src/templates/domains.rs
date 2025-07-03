use askama::Template;
use crate::models::Domain;

#[derive(Template)]
#[template(path = "domains/list.html")]
pub struct DomainListTemplate {
    pub domains: Vec<Domain>,
}

#[derive(Template)]
#[template(path = "domains/show.html")]
pub struct DomainShowTemplate {
    pub domain: Domain,
}

#[derive(Template)]
#[template(path = "domains/form.html")]
pub struct DomainFormTemplate {
    pub domain: Option<Domain>,
} 
