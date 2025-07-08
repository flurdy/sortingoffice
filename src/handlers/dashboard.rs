use crate::templates::dashboard::DashboardTemplate;
use crate::templates::layout::BaseTemplate;
use crate::{db, AppState};
use askama::Template;
use axum::{extract::State, response::Html, http::HeaderMap};

pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;

    let stats = match db::get_system_stats(pool) {
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

    // Get user's preferred locale
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get all translations
    let title = crate::i18n::get_translation(&state, &locale, "dashboard-title").await;
    let description = crate::i18n::get_translation(&state, &locale, "dashboard-description").await;
    let total_domains = crate::i18n::get_translation(&state, &locale, "dashboard-total-domains").await;
    let total_backups = crate::i18n::get_translation(&state, &locale, "dashboard-total-backups").await;
    let total_aliases = crate::i18n::get_translation(&state, &locale, "dashboard-total-aliases").await;
    let total_users = crate::i18n::get_translation(&state, &locale, "dashboard-total-users").await;
    let total_relays = crate::i18n::get_translation(&state, &locale, "dashboard-total-relays").await;
    let total_relocated = crate::i18n::get_translation(&state, &locale, "dashboard-total-relocated").await;
    let quick_actions = crate::i18n::get_translation(&state, &locale, "dashboard-quick-actions").await;
    let manage_domains = crate::i18n::get_translation(&state, &locale, "quick-action-manage-domains").await;
    let manage_domains_desc = crate::i18n::get_translation(&state, &locale, "quick-action-manage-domains-desc").await;
    let manage_backups = crate::i18n::get_translation(&state, &locale, "quick-action-manage-backups").await;
    let manage_backups_desc = crate::i18n::get_translation(&state, &locale, "quick-action-manage-backups-desc").await;
    let manage_aliases = crate::i18n::get_translation(&state, &locale, "quick-action-manage-aliases").await;
    let manage_aliases_desc = crate::i18n::get_translation(&state, &locale, "quick-action-manage-aliases-desc").await;
    let manage_users = crate::i18n::get_translation(&state, &locale, "quick-action-manage-users").await;
    let manage_users_desc = crate::i18n::get_translation(&state, &locale, "quick-action-manage-users-desc").await;
    let help_resources = crate::i18n::get_translation(&state, &locale, "dashboard-help-resources").await;
    let help_title = crate::i18n::get_translation(&state, &locale, "help-title").await;
    let help_description = crate::i18n::get_translation(&state, &locale, "help-description").await;
    let help_read_guide = crate::i18n::get_translation(&state, &locale, "help-read-guide").await;

    let content_template = DashboardTemplate {
        title: &title,
        description: &description,
        total_domains: &total_domains,
        total_backups: &total_backups,
        total_aliases: &total_aliases,
        total_users: &total_users,
        total_relays: &total_relays,
        total_relocated: &total_relocated,
        quick_actions: &quick_actions,
        manage_domains: &manage_domains,
        manage_domains_desc: &manage_domains_desc,
        manage_backups: &manage_backups,
        manage_backups_desc: &manage_backups_desc,
        manage_aliases: &manage_aliases,
        manage_aliases_desc: &manage_aliases_desc,
        manage_users: &manage_users,
        manage_users_desc: &manage_users_desc,
        help_resources: &help_resources,
        help_title: &help_title,
        help_description: &help_description,
        help_read_guide: &help_read_guide,
        stats,
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate::with_i18n(
        crate::i18n::get_translation(&state, &locale, "dashboard-title").await,
        content,
        &state,
        &locale,
    ).await.unwrap();
    
    Html(template.render().unwrap())
}
