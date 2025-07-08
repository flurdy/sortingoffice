use crate::templates::layout::BaseTemplate;
use crate::templates::stats::StatsTemplate;
use crate::{db, AppState, i18n::get_translation};
use askama::Template;
use axum::{extract::State, response::Html, http::HeaderMap};

pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    let system_stats = match db::get_system_stats(pool) {
        Ok(stats) => stats,
        Err(_) => crate::models::SystemStats {
            total_domains: 0,
            total_users: 0,
            total_aliases: 0,
            total_backups: 0,
            total_relays: 0,
            total_relocated: 0,
            total_quota: 0,
            used_quota: 0,
        },
    };

    let domain_stats = match db::get_domain_stats(pool) {
        Ok(stats) => stats,
        Err(_) => vec![],
    };

    // Get all translations
    let title = get_translation(&state, &locale, "stats-title").await;
    let description = get_translation(&state, &locale, "stats-description").await;
    let system_overview = get_translation(&state, &locale, "stats-system-overview").await;
    let system_description = get_translation(&state, &locale, "stats-system-description").await;
    let total_domains = get_translation(&state, &locale, "stats-total-domains").await;
    let total_backups = get_translation(&state, &locale, "stats-total-backups").await;
    let total_aliases = get_translation(&state, &locale, "stats-total-aliases").await;
    let total_users = get_translation(&state, &locale, "stats-total-users").await;
    let total_relays = get_translation(&state, &locale, "stats-total-relays").await;
    let total_relocated = get_translation(&state, &locale, "stats-total-relocated").await;
    let domain_statistics = get_translation(&state, &locale, "stats-domain-statistics").await;
    let table_header_domain = get_translation(&state, &locale, "stats-table-header-domain").await;
    let table_header_users = get_translation(&state, &locale, "stats-table-header-users").await;
    let table_header_aliases = get_translation(&state, &locale, "stats-table-header-aliases").await;
    let table_header_total_quota = get_translation(&state, &locale, "stats-table-header-total-quota").await;
    let table_header_used_quota = get_translation(&state, &locale, "stats-table-header-used-quota").await;

    let content_template = StatsTemplate {
        title: &title,
        description: &description,
        system_overview: &system_overview,
        system_description: &system_description,
        total_domains: &total_domains,
        total_backups: &total_backups,
        total_aliases: &total_aliases,
        total_users: &total_users,
        total_relays: &total_relays,
        total_relocated: &total_relocated,
        domain_statistics: &domain_statistics,
        table_header_domain: &table_header_domain,
        table_header_users: &table_header_users,
        table_header_aliases: &table_header_aliases,
        table_header_total_quota: &table_header_total_quota,
        table_header_used_quota: &table_header_used_quota,
        system_stats,
        domain_stats,
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate::with_i18n(
        get_translation(&state, &locale, "stats-title").await,
        content,
        &state,
        &locale,
    ).await.unwrap();
    
    Html(template.render().unwrap())
}
