use crate::templates::dashboard::DashboardTemplate;
use crate::{db, get_system_stats_or_default, render_template_with_title, AppState};
use askama::Template;
use axum::{extract::State, http::HeaderMap, response::Html};

pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    // Get the current database pool based on session selection
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    // Use the new macro for SystemStats retrieval
    let stats = get_system_stats_or_default!(db::get_system_stats(&pool));

    // Get user's preferred locale
    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Use the batch translation fetcher for common translations
    let common_translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
        &[
            "dashboard-title",
            "dashboard-description",
            "dashboard-total-domains",
            "dashboard-total-backups",
            "dashboard-total-aliases",
            "dashboard-total-users",
            "dashboard-total-relays",
            "dashboard-total-relocated",
            "dashboard-total-clients",
            "dashboard-enabled-domains-and-backups",
            "dashboard-enabled-aliases",
            "dashboard-enabled-users",
            "dashboard-quick-actions",
            "dashboard-primary-actions",
            "dashboard-advanced-management",
            "dashboard-analytics-reports",
            "quick-action-manage-domains-and-backups",
            "quick-action-manage-domains-and-backups-desc",
            "quick-action-manage-email",
            "quick-action-manage-email-desc",
            "quick-action-manage-users",
            "quick-action-manage-users-desc",
            "quick-action-manage-relays",
            "quick-action-manage-relocated",
            "quick-action-manage-clients",
            "quick-action-manage-config",
            "quick-action-manage-config-desc",
            "quick-action-view-statistics",
            "quick-action-view-statistics-desc",
            "quick-action-view-reports",
            "quick-action-view-reports-desc",
            "resource-domains",
            "resource-backups",
            "resource-aliases",
            "resource-users",
            "resource-relays",
            "resource-relocated",
            "resource-clients",
            "quick-action-manage-domains",
            "quick-action-manage-domains-desc",
            "quick-action-manage-backups",
            "quick-action-manage-backups-desc",
            "quick-action-manage-aliases",
            "quick-action-manage-aliases-desc",
            "quick-action-manage-clients-desc",
            "dashboard-help-resources",
            "help-title",
            "help-description",
            "help-read-guide",
        ],
    )
    .await;

    let content_template = DashboardTemplate {
        title: &common_translations["dashboard-title"],
        description: &common_translations["dashboard-description"],
        total_domains: &common_translations["dashboard-total-domains"],
        total_backups: &common_translations["dashboard-total-backups"],
        total_aliases: &common_translations["dashboard-total-aliases"],
        total_users: &common_translations["dashboard-total-users"],
        total_relays: &common_translations["dashboard-total-relays"],
        total_relocated: &common_translations["dashboard-total-relocated"],
        total_clients: &common_translations["dashboard-total-clients"],
        enabled_domains_and_backups: &common_translations["dashboard-enabled-domains-and-backups"],
        enabled_aliases: &common_translations["dashboard-enabled-aliases"],
        enabled_users: &common_translations["dashboard-enabled-users"],
        quick_actions: &common_translations["dashboard-quick-actions"],
        primary_actions: &common_translations["dashboard-primary-actions"],
        advanced_management: &common_translations["dashboard-advanced-management"],
        analytics_reports: &common_translations["dashboard-analytics-reports"],
        manage_domains_and_backups: &common_translations["quick-action-manage-domains-and-backups"],
        manage_domains_and_backups_desc: &common_translations
            ["quick-action-manage-domains-and-backups-desc"],
        manage_email: &common_translations["quick-action-manage-email"],
        manage_email_desc: &common_translations["quick-action-manage-email-desc"],
        manage_users: &common_translations["quick-action-manage-users"],
        manage_users_desc: &common_translations["quick-action-manage-users-desc"],
        manage_relays: &common_translations["quick-action-manage-relays"],
        manage_relocated: &common_translations["quick-action-manage-relocated"],
        manage_clients: &common_translations["quick-action-manage-clients"],
        manage_config: &common_translations["quick-action-manage-config"],
        manage_config_desc: &common_translations["quick-action-manage-config-desc"],
        view_statistics: &common_translations["quick-action-view-statistics"],
        view_statistics_desc: &common_translations["quick-action-view-statistics-desc"],
        view_reports: &common_translations["quick-action-view-reports"],
        view_reports_desc: &common_translations["quick-action-view-reports-desc"],
        domains: &common_translations["resource-domains"],
        backups: &common_translations["resource-backups"],
        aliases: &common_translations["resource-aliases"],
        users: &common_translations["resource-users"],
        relays: &common_translations["resource-relays"],
        relocated: &common_translations["resource-relocated"],
        clients: &common_translations["resource-clients"],
        manage_domains: &common_translations["quick-action-manage-domains"],
        manage_domains_desc: &common_translations["quick-action-manage-domains-desc"],
        manage_backups: &common_translations["quick-action-manage-backups"],
        manage_backups_desc: &common_translations["quick-action-manage-backups-desc"],
        manage_aliases: &common_translations["quick-action-manage-aliases"],
        manage_aliases_desc: &common_translations["quick-action-manage-aliases-desc"],
        manage_clients_desc: &common_translations["quick-action-manage-clients-desc"],
        help_resources: &common_translations["dashboard-help-resources"],
        help_title: &common_translations["help-title"],
        help_description: &common_translations["help-description"],
        help_read_guide: &common_translations["help-read-guide"],
        stats,
    };

    // Use the new render template macro with title
    render_template_with_title!(
        content_template,
        content_template.title,
        &state,
        &locale,
        &headers
    )
}
