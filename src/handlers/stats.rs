use axum::{
    extract::State,
    response::Html,
};
use crate::{AppState, db};
use crate::templates::stats::StatsTemplate;
use crate::templates::layout::BaseTemplate;
use askama::Template;

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;
    
    let system_stats = match db::get_system_stats(pool) {
        Ok(stats) => stats,
        Err(_) => crate::models::SystemStats {
            total_domains: 0,
            total_users: 0,
            total_aliases: 0,
            total_quota: 0,
            used_quota: 0,
        },
    };
    
    let domain_stats = match db::get_domain_stats(pool) {
        Ok(stats) => stats,
        Err(_) => vec![],
    };
    
    let content_template = StatsTemplate {
        title: "Stats",
        system_stats,
        domain_stats,
    };
    let content = content_template.render().unwrap();
    
    let template = BaseTemplate { 
        title: "Statistics".to_string(), 
        content 
    };
    Html(template.render().unwrap())
} 
