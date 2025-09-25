use crate::templates::stats::StatsTemplate;
use crate::{render_template, AppState};
use askama::Template;
use axum::{extract::State, http::HeaderMap, response::Html};

pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
    let locale = crate::handlers::http_helpers::get_user_locale(&headers);

    // Use cached system stats for better performance
    let system_stats = match state.db_manager.get_system_stats_cached(&pool).await {
        Ok(stats) => stats,
        Err(e) => {
            tracing::error!("Failed to get system stats: {:?}", e);
            crate::models::SystemStats::default()
        }
    };

    // Use the batch translation fetcher for all statistics translations
    let translations = crate::handlers::translations::get_translations_batch(
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
    };

    // Use the new render template macro
    render_template!(content_template, &state, &locale, &headers)
}
