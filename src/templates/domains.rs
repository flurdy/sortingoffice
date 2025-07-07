use crate::models::{Domain, DomainForm};
use askama::Template;

#[derive(Template)]
#[template(path = "domains/list.html", escape = "html")]
pub struct DomainListTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub add_domain: &'a str,
    pub table_header_domain: &'a str,
    pub table_header_transport: &'a str,
    pub table_header_status: &'a str,
    pub table_header_actions: &'a str,
    pub status_active: &'a str,
    pub status_inactive: &'a str,
    pub action_view: &'a str,
    pub action_enable: &'a str,
    pub action_disable: &'a str,
    pub empty_title: &'a str,
    pub empty_description: &'a str,
    pub domains: Vec<Domain>,
}

#[derive(Template)]
#[template(path = "domains/show.html", escape = "html")]
pub struct DomainShowTemplate<'a> {
    pub title: &'a str,
    pub domain: Domain,
    pub view_edit_settings: &'a str,
    pub back_to_domains: &'a str,
    pub domain_information: &'a str,
    pub domain_details: &'a str,
    pub domain_name: &'a str,
    pub transport: &'a str,
    pub status: &'a str,
    pub status_active: &'a str,
    pub status_inactive: &'a str,
    pub created: &'a str,
    pub modified: &'a str,
    pub edit_domain_button: &'a str,
    pub enable_domain: &'a str,
    pub disable_domain: &'a str,
    pub delete_domain: &'a str,
    pub delete_confirm: &'a str,
}

#[derive(Template)]
#[template(path = "domains/form.html", escape = "html")]
pub struct DomainFormTemplate<'a> {
    pub title: &'a str,
    pub domain: Option<Domain>,
    pub form: DomainForm,
    pub error: Option<String>,
    pub form_error: &'a str,
    pub form_domain: &'a str,
    pub form_transport: &'a str,
    pub form_active: &'a str,
    pub form_cancel: &'a str,
    pub form_create_domain: &'a str,
    pub form_update_domain: &'a str,
    pub form_placeholder_domain: &'a str,
    pub form_placeholder_transport: &'a str,
    pub form_tooltip_domain: &'a str,
    pub form_tooltip_transport: &'a str,
    pub form_tooltip_enable: &'a str,
}
