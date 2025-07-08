use crate::models::{Alias, AliasForm};
use askama::Template;

#[derive(Template)]
#[template(path = "aliases/list.html", escape = "html")]
pub struct AliasListTemplate<'a> {
    pub title: &'a str,
    pub aliases: Vec<Alias>,
    pub description: &'a str,
    pub add_alias: &'a str,
    pub table_header_mail: &'a str,
    pub table_header_destination: &'a str,
    pub table_header_enabled: &'a str,
    pub table_header_actions: &'a str,
    pub status_active: &'a str,
    pub status_inactive: &'a str,
    pub action_view: &'a str,
    pub enable_alias: &'a str,
    pub disable_alias: &'a str,
    pub empty_title: &'a str,
    pub empty_description: &'a str,
}

#[derive(Template)]
#[template(path = "aliases/show.html", escape = "html")]
pub struct AliasShowTemplate<'a> {
    pub title: &'a str,
    pub alias: Alias,
    pub view_edit_settings: &'a str,
    pub back_to_aliases: &'a str,
    pub alias_information: &'a str,
    pub alias_details: &'a str,
    pub mail: &'a str,
    pub forward_to: &'a str,
    pub status: &'a str,
    pub status_active: &'a str,
    pub status_inactive: &'a str,
    pub created: &'a str,
    pub modified: &'a str,
    pub edit_alias_button: &'a str,
    pub enable_alias_button: &'a str,
    pub disable_alias_button: &'a str,
    pub delete_alias: &'a str,
    pub delete_confirm: &'a str,
}

#[derive(Template)]
#[template(path = "aliases/form.html", escape = "html")]
pub struct AliasFormTemplate<'a> {
    pub title: &'a str,
    pub alias: Option<Alias>,
    pub form: AliasForm,
    pub error: Option<String>,
    pub edit_alias: &'a str,
    pub new_alias: &'a str,
    pub form_error: &'a str,
    pub mail_address: &'a str,
    pub destination: &'a str,
    pub placeholder_mail: &'a str,
    pub placeholder_destination: &'a str,
    pub tooltip_mail: &'a str,
    pub tooltip_destination: &'a str,
    pub active: &'a str,
    pub tooltip_active: &'a str,
    pub cancel: &'a str,
    pub update_alias: &'a str,
    pub create_alias: &'a str,
}
