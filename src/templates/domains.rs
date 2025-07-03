use askama::Template;
use crate::models::{Domain, DomainForm};

#[derive(Template)]
#[template(path = "domains/list.html", escape = "html")]
pub struct DomainListTemplate<'a> {
    pub title: &'a str,
    pub domains: Vec<Domain>,
}

#[derive(Template)]
#[template(path = "domains/show.html", escape = "html")]
pub struct DomainShowTemplate<'a> {
    pub title: &'a str,
    pub domain: Domain,
}

#[derive(Template)]
#[template(path = "domains/form.html", escape = "html")]
pub struct DomainFormTemplate<'a> {
    pub title: &'a str,
    pub domain: Option<Domain>,
    pub form: DomainForm,
} 
