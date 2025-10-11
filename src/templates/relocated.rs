use crate::models::{PaginatedResult, Relocated};
use askama::Template;

#[derive(Template)]
#[template(path = "relocated/list.html", escape = "html")]
pub struct RelocatedListTemplate<'a> {
    pub title: &'a str,
    pub relocated_list_description: &'a str,
    pub relocated_add: &'a str,
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
    pub pagination: &'a PaginatedResult<Relocated>,
    pub page_range: &'a [i64],
    pub max_item: i64,
    pub pagination_showing: &'a str,
    pub pagination_to: &'a str,
    pub pagination_of: &'a str,
    pub pagination_results: &'a str,
    pub pagination_previous: &'a str,
    pub pagination_next: &'a str,
    pub page_size_label: &'a str,
    pub page_size_10: &'a str,
    pub page_size_20: &'a str,
    pub page_size_50: &'a str,
    pub status_filter_label: &'a str,
    // Filter controls
    pub enabled_filter: &'a str,
    pub filter_all_label: &'a str,
    pub filter_enabled_label: &'a str,
    pub filter_disabled_label: &'a str,
    pub filters_label: &'a str,
    // Database read-only status
    pub current_db_read_only: bool,
    pub read_only_tooltip: &'a str,
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
    pub delete_relocated_disabled_tooltip: &'a str,
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
    pub current_db_read_only: bool,
    pub read_only_tooltip: &'a str,
    pub not_available: &'a str,
    pub relocated: Relocated,
}

#[derive(Template)]
#[template(path = "relocated/form.html", escape = "html")]
pub struct RelocatedFormTemplate<'a> {
    pub title: &'a str,
    pub action: &'a str,
    pub form: crate::models::RelocatedForm,
    pub relocated_id: Option<i32>,
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
