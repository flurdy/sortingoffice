use askama::Template;
use crate::models::Relocated;

#[derive(Template)]
#[template(path = "relocated/list.html", escape = "html")]
pub struct RelocatedListTemplate<'a> {
    pub title: &'a str,
    pub add_relocated: &'a str,
    pub table_header_old_address: &'a str,
    pub table_header_new_address: &'a str,
    pub table_header_enabled: &'a str,
    pub table_header_actions: &'a str,
    pub status_enabled: &'a str,
    pub status_disabled: &'a str,
    pub action_view: &'a str,
    pub action_enable: &'a str,
    pub action_disable: &'a str,
    pub delete_confirm: &'a str,
    pub empty_title: &'a str,
    pub empty_description: &'a str,
    pub relocated: Vec<Relocated>,
    pub relocated_list_description: &'a str,
}

#[derive(Template)]
#[template(path = "relocated/show.html")]
pub struct RelocatedShowTemplate<'a> {
    pub title: &'a str,
    pub action_edit: &'a str,
    pub action_enable: &'a str,
    pub action_disable: &'a str,
    pub action_delete: &'a str,
    pub delete_confirm: &'a str,
    pub back_to_list: &'a str,
    pub field_id: &'a str,
    pub field_old_address: &'a str,
    pub field_new_address: &'a str,
    pub field_enabled: &'a str,
    pub field_created: &'a str,
    pub field_modified: &'a str,
    pub status_enabled: &'a str,
    pub status_disabled: &'a str,
    pub view_edit_settings: &'a str,
    pub relocated_show_title: &'a str,
    pub relocated_info_title: &'a str,
    pub relocated_info_description: &'a str,
    pub relocated: Relocated,
}

#[derive(Template)]
#[template(path = "relocated/form.html", escape = "html")]
pub struct RelocatedFormTemplate<'a> {
    pub title: &'a str,
    pub action: &'a str,
    pub form: crate::models::RelocatedForm,
    pub field_old_address: &'a str,
    pub field_new_address: &'a str,
    pub field_enabled: &'a str,
    pub field_old_address_help: &'a str,
    pub field_new_address_help: &'a str,
    pub action_save: &'a str,
    pub action_cancel: &'a str,
    pub back_to_list: &'a str,
    pub placeholder_old_address: &'a str,
    pub placeholder_new_address: &'a str,
} 
