use crate::models::AliasReport;
use askama::Template;

#[derive(Template)]
#[template(path = "reports.html", escape = "html")]
pub struct ReportsTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub domain_header: &'a str,
    pub catch_all_header: &'a str,
    pub destination_header: &'a str,
    pub required_aliases_header: &'a str,
    pub missing_aliases_header: &'a str,
    pub missing_required_aliases_header: &'a str,
    pub missing_common_aliases_header: &'a str,
    pub mail_header: &'a str,
    pub status_header: &'a str,
    pub status_active: &'a str,
    pub status_inactive: &'a str,
    pub no_catch_all_domains: &'a str,
    pub no_catch_all_domains_description: &'a str,
    pub no_required_aliases: &'a str,
    pub no_missing_aliases: &'a str,
    pub domains_with_catch_all_header: &'a str,
    pub domains_without_catch_all_header: &'a str,
    pub report: &'a AliasReport,
} 
