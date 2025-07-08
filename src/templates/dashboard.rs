use crate::models::SystemStats;
use askama::Template;

#[derive(Template)]
#[template(path = "dashboard.html", escape = "html")]
pub struct DashboardTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub total_domains: &'a str,
    pub total_backups: &'a str,
    pub total_aliases: &'a str,
    pub total_users: &'a str,
    pub total_relays: &'a str,
    pub total_relocated: &'a str,
    pub quick_actions: &'a str,
    pub manage_domains: &'a str,
    pub manage_domains_desc: &'a str,
    pub manage_backups: &'a str,
    pub manage_backups_desc: &'a str,
    pub manage_aliases: &'a str,
    pub manage_aliases_desc: &'a str,
    pub manage_users: &'a str,
    pub manage_users_desc: &'a str,
    pub help_resources: &'a str,
    pub help_title: &'a str,
    pub help_description: &'a str,
    pub help_read_guide: &'a str,
    pub stats: SystemStats,
}
