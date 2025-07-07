use crate::models::{Relay, RelayForm};
use askama::Template;

#[derive(Template)]
#[template(path = "relays/list.html", escape = "html")]
pub struct RelayListTemplate<'a> {
    pub title: &'a str,
    pub add_relay: &'a str,
    pub table_header_recipient: &'a str,
    pub table_header_status: &'a str,
    pub table_header_enabled: &'a str,
    pub table_header_modified: &'a str,
    pub table_header_actions: &'a str,
    pub status_enabled: &'a str,
    pub status_disabled: &'a str,
    pub action_view: &'a str,
    pub action_enable: &'a str,
    pub action_disable: &'a str,
    pub delete_confirm: &'a str,
    pub empty_title: &'a str,
    pub empty_description: &'a str,
    pub relays: Vec<Relay>,
    pub relays_list_description: &'a str,
}

#[derive(Template)]
#[template(path = "relays/show.html", escape = "html")]
pub struct RelayShowTemplate<'a> {
    pub title: &'a str,
    pub relay: Relay,
    pub action_edit: &'a str,
    pub action_enable: &'a str,
    pub action_disable: &'a str,
    pub action_delete: &'a str,
    pub delete_confirm: &'a str,
    pub back_to_list: &'a str,
    pub field_id: &'a str,
    pub field_recipient: &'a str,
    pub field_status: &'a str,
    pub field_enabled: &'a str,
    pub field_created: &'a str,
    pub field_modified: &'a str,
    pub status_enabled: &'a str,
    pub status_disabled: &'a str,
    pub view_edit_settings: &'a str,
    pub relay_show_title: &'a str,
    pub relay_info_title: &'a str,
    pub relay_info_description: &'a str,
}

#[derive(Template)]
#[template(path = "relays/form.html", escape = "html")]
pub struct RelayFormTemplate<'a> {
    pub title: &'a str,
    pub action: &'a str,
    pub form: RelayForm,
    pub field_recipient: &'a str,
    pub field_status: &'a str,
    pub field_enabled: &'a str,
    pub field_recipient_help: &'a str,
    pub field_status_help: &'a str,
    pub action_save: &'a str,
    pub action_cancel: &'a str,
    pub back_to_list: &'a str,
    pub placeholder_recipient: &'a str,
    pub placeholder_status: &'a str,
} 
