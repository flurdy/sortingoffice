use crate::models::{Client, PaginatedResult};
use askama::Template;

#[derive(Template)]
#[template(path = "clients/list.html", escape = "html")]
pub struct ClientsListTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub add_client: &'a str,
    pub table_header_client: &'a str,
    pub table_header_status: &'a str,
    pub table_header_enabled: &'a str,
    pub table_header_actions: &'a str,
    pub status_allowed: &'a str,
    pub status_blocked: &'a str,
    pub status_enabled: &'a str,
    pub status_disabled: &'a str,
    pub action_view: &'a str,
    pub action_enable: &'a str,
    pub action_disable: &'a str,
    pub action_delete: &'a str,
    pub delete_confirm: &'a str,
    pub empty_title: &'a str,
    pub empty_description: &'a str,
    pub clients: &'a [Client],
    pub pagination: &'a PaginatedResult<Client>,
    pub page_range: &'a [i64],
    pub max_item: i64,
}

#[derive(Template)]
#[template(path = "clients/show.html", escape = "html")]
pub struct ClientShowTemplate<'a> {
    pub title: &'a str,
    pub client: Client,
    pub view_edit_settings: &'a str,
    pub back_to_clients: &'a str,
    pub client_information: &'a str,
    pub client_details: &'a str,
    pub client_name: &'a str,
    pub status: &'a str,
    pub status_allowed: &'a str,
    pub status_blocked: &'a str,
    pub status_enabled: &'a str,
    pub status_disabled: &'a str,
    pub enabled_label: &'a str,
    pub created: &'a str,
    pub updated: &'a str,
    pub edit_client: &'a str,
    pub action_enable: &'a str,
    pub action_disable: &'a str,
    pub delete_client: &'a str,
    pub delete_confirm: &'a str,
}

#[derive(Template)]
#[template(path = "clients/form.html", escape = "html")]
pub struct ClientFormTemplate<'a> {
    pub title: &'a str,
    pub client: Option<Client>,
    pub form_error: &'a str,
    pub form_client: &'a str,
    pub form_status: &'a str,
    pub form_enabled: &'a str,
    pub form_cancel: &'a str,
    pub form_create_client: &'a str,
    pub form_update_client: &'a str,
    pub form_placeholder_client: &'a str,
    pub form_tooltip_client: &'a str,
    pub form_tooltip_status: &'a str,
    pub form_tooltip_enabled: &'a str,
    pub status_allowed: &'a str,
    pub status_blocked: &'a str,
    pub enabled_yes: &'a str,
    pub enabled_no: &'a str,
} 
