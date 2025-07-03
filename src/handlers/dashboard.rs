use axum::{
    extract::State,
    response::Html,
};
use crate::{AppState, db};
use crate::templates::dashboard::DashboardTemplate;

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;
    
    let stats = match db::get_system_stats(pool) {
        Ok(stats) => stats,
        Err(_) => crate::models::SystemStats {
            total_domains: 0,
            total_users: 0,
            total_aliases: 0,
            total_mailboxes: 0,
            total_quota: 0,
            used_quota: 0,
        },
    };
    
    let template = DashboardTemplate { stats };
    Html(template.render().unwrap())
} 
