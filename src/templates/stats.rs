use crate::models::SystemStats;
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
    pub total_clients: &'a str,
    pub quota_usage_title: &'a str,
    pub quota_usage_overview: &'a str,
    pub quota_usage_description: &'a str,
    pub quota_usage_percentage: &'a str,
    pub quota_total: &'a str,
    pub quota_used: &'a str,
    pub recent_activity_title: &'a str,
    pub recent_domains: &'a str,
    pub recent_users: &'a str,
    pub recent_aliases: &'a str,
    pub recent_backups: &'a str,
    pub recent_relays: &'a str,
    pub recent_relocated: &'a str,
    pub recent_clients: &'a str,
    pub system_stats: SystemStats,
}
