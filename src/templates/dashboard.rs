use askama::Template;
use crate::models::SystemStats;

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub stats: SystemStats,
} 
