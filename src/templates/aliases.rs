use crate::models::{Alias, AliasForm, Domain, PaginatedResult};
use askama::Template;

#[derive(Template)]
#[template(path = "aliases/list.html")]
pub struct AliasesListTemplate<'a> {
    pub title: &'a str,
    pub aliases: &'a [Alias],
    pub pagination: &'a PaginatedResult<Alias>,
    pub page_range: &'a [i64], // Changed back to reference
    pub max_item: i64,
    pub description: &'a str,
    pub add_alias: &'a str,
    pub table_header_mail: &'a str,
    pub table_header_domain: &'a str,
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
    pub current_sort_by: &'a str,
    pub current_sort_order: &'a str,
    // Pagination translations
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
    // Search
    pub search_term: &'a str,
    // Database read-only status
    pub current_db_read_only: bool,
    pub read_only_tooltip: &'a str,
    // Filter controls
    pub enabled_filter: &'a str,
    pub filter_all_label: &'a str,
    pub filter_enabled_label: &'a str,
    pub filter_disabled_label: &'a str,
    pub filters_label: &'a str,
}

#[derive(Template)]
#[template(path = "aliases/form.html")]
pub struct AliasFormTemplate<'a> {
    pub title: &'a str,
    pub alias: Option<Alias>,
    pub form: AliasForm,
    pub error: Option<String>,
    pub return_url: Option<String>,
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
    pub not_available: &'a str,
}

#[derive(Template)]
#[template(path = "aliases/show.html")]
pub struct AliasShowTemplate<'a> {
    pub title: &'a str,
    pub alias: Alias,
    pub view_edit_settings: &'a str,
    pub back_to_aliases: &'a str,
    pub alias_information: &'a str,
    pub alias_details: &'a str,
    pub mail: &'a str,
    pub forward_to: &'a str,
    pub domain: &'a str,
    pub domain_info: Option<Domain>,
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
    pub current_db_read_only: bool,
    pub read_only_tooltip: &'a str,
    pub delete_alias_disabled_tooltip: &'a str,
    pub not_available: &'a str,
    pub cross_domain_report: Option<&'a crate::models::AliasCrossDomainReport>,
}

#[derive(Template)]
#[template(path = "aliases/search_results.html")]
pub struct AliasSearchResultsTemplate<'a> {
    pub aliases: &'a [Alias],
    pub no_results: &'a str,
    pub select_text: &'a str,
}

#[derive(Template)]
#[template(path = "aliases/domain_search_results.html")]
pub struct DomainSearchResultsTemplate<'a> {
    pub domains: &'a [Domain],
    pub no_results: &'a str,
    pub select_text: &'a str,
    pub status_active: &'a str,
    pub status_inactive: &'a str,
}
