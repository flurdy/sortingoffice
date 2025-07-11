use crate::templates::stats::StatsTemplate;
use crate::{db, AppState, get_system_stats_or_default, render_template};
use askama::Template;
use axum::{extract::State, http::HeaderMap, response::Html};

pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Use the new macro for SystemStats retrieval
    let system_stats = get_system_stats_or_default!(db::get_system_stats(pool));

    let domain_stats = match db::get_domain_stats(pool) {
        Ok(stats) => stats,
        Err(_) => vec![],
    };

    // Use the batch translation fetcher for all statistics translations
    let translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
        &[
            "stats-title",
            "stats-description",
            "stats-system-overview",
            "stats-system-description",
            "stats-total-domains",
            "stats-total-backups",
            "stats-total-aliases",
            "stats-total-users",
            "stats-total-relays",
            "stats-total-relocated",
            "stats-total-clients",
            "stats-domain-statistics",
            "stats-table-header-domain",
            "stats-table-header-users",
            "stats-table-header-aliases",
            "stats-table-header-total-quota",
            "stats-table-header-used-quota",
            "stats-quota-usage-title",
            "stats-quota-usage-overview",
            "stats-quota-usage-description",
            "stats-quota-usage-percentage",
            "stats-quota-total",
            "stats-quota-used",
            "stats-recent-activity-title",
            "stats-recent-domains",
            "stats-recent-users",
            "stats-recent-aliases",
            "stats-recent-backups",
            "stats-recent-relays",
            "stats-recent-relocated",
            "stats-recent-clients",
        ],
    )
    .await;

    let content_template = StatsTemplate {
        title: &translations["stats-title"],
        description: &translations["stats-description"],
        system_overview: &translations["stats-system-overview"],
        system_description: &translations["stats-system-description"],
        total_domains: &translations["stats-total-domains"],
        total_backups: &translations["stats-total-backups"],
        total_aliases: &translations["stats-total-aliases"],
        total_users: &translations["stats-total-users"],
        total_relays: &translations["stats-total-relays"],
        total_relocated: &translations["stats-total-relocated"],
        total_clients: &translations["stats-total-clients"],
        domain_statistics: &translations["stats-domain-statistics"],
        table_header_domain: &translations["stats-table-header-domain"],
        table_header_users: &translations["stats-table-header-users"],
        table_header_aliases: &translations["stats-table-header-aliases"],
        table_header_total_quota: &translations["stats-table-header-total-quota"],
        table_header_used_quota: &translations["stats-table-header-used-quota"],
        quota_usage_title: &translations["stats-quota-usage-title"],
        quota_usage_overview: &translations["stats-quota-usage-overview"],
        quota_usage_description: &translations["stats-quota-usage-description"],
        quota_usage_percentage: &translations["stats-quota-usage-percentage"],
        quota_total: &translations["stats-quota-total"],
        quota_used: &translations["stats-quota-used"],
        recent_activity_title: &translations["stats-recent-activity-title"],
        recent_domains: &translations["stats-recent-domains"],
        recent_users: &translations["stats-recent-users"],
        recent_aliases: &translations["stats-recent-aliases"],
        recent_backups: &translations["stats-recent-backups"],
        recent_relays: &translations["stats-recent-relays"],
        recent_relocated: &translations["stats-recent-relocated"],
        recent_clients: &translations["stats-recent-clients"],
        system_stats,
        domain_stats: domain_stats,
    };

    // Use the new render template macro
    render_template!(content_template, &state, &locale, &headers)
}
