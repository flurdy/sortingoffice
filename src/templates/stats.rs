use crate::models::{DomainStats, SystemStats};
use askama::Template;

#[derive(Template)]
#[template(path = "stats.html", escape = "html")]
pub struct StatsTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub system_overview: &'a str,
    pub system_description: &'a str,
    pub total_domains: &'a str,
    pub total_backups: &'a str,
    pub total_aliases: &'a str,
    pub total_users: &'a str,
    pub total_relays: &'a str,
    pub total_relocated: &'a str,
    pub domain_statistics: &'a str,
    pub table_header_domain: &'a str,
    pub table_header_users: &'a str,
    pub table_header_aliases: &'a str,
    pub table_header_total_quota: &'a str,
    pub table_header_used_quota: &'a str,
    pub system_stats: SystemStats,
    pub domain_stats: Vec<DomainStats>,
}
