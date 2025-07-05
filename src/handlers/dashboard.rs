use axum::{
    extract::State,
    response::Html,
};
use crate::{AppState, db};
use crate::templates::dashboard::DashboardTemplate;
use crate::templates::layout::BaseTemplate;
use askama::Template;

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;
    
    let stats = match db::get_system_stats(pool) {
        Ok(stats) => stats,
        Err(_) => crate::models::SystemStats {
            total_domains: 0,
            total_users: 0,
            total_aliases: 0,
            total_quota: 0,
            used_quota: 0,
        },
    };
    
    let content_template = DashboardTemplate { title: "Dashboard", stats };
    let content = content_template.render().unwrap();
    
    let template = BaseTemplate { 
        title: "Dashboard".to_string(), 
        content 
    };
    Html(template.render().unwrap())
} 
