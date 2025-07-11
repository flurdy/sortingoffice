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
    pub total_clients: &'a str,
    pub quick_actions: &'a str,

    // New quick action sections
    pub primary_actions: &'a str,
    pub advanced_management: &'a str,
    pub analytics_reports: &'a str,

    // Combined domain & backup management
    pub manage_domains_and_backups: &'a str,
    pub manage_domains_and_backups_desc: &'a str,

    // Email management
    pub manage_email: &'a str,
    pub manage_email_desc: &'a str,

    // User management
    pub manage_users: &'a str,
    pub manage_users_desc: &'a str,

    // Advanced management
    pub manage_relays: &'a str,
    pub manage_relocated: &'a str,
    pub manage_clients: &'a str,
    pub manage_config: &'a str,
    pub manage_config_desc: &'a str,

    // Analytics & reports
    pub view_statistics: &'a str,
    pub view_statistics_desc: &'a str,
    pub view_reports: &'a str,
    pub view_reports_desc: &'a str,

    // Resource labels
    pub domains: &'a str,
    pub backups: &'a str,
    pub aliases: &'a str,
    pub users: &'a str,
    pub relays: &'a str,
    pub relocated: &'a str,
    pub clients: &'a str,

    // Legacy translations (keeping for backward compatibility)
    pub manage_domains: &'a str,
    pub manage_domains_desc: &'a str,
    pub manage_backups: &'a str,
    pub manage_backups_desc: &'a str,
    pub manage_aliases: &'a str,
    pub manage_aliases_desc: &'a str,
    pub manage_clients_desc: &'a str,
    pub help_resources: &'a str,
    pub help_title: &'a str,
    pub help_description: &'a str,
    pub help_read_guide: &'a str,
    pub stats: SystemStats,
}
