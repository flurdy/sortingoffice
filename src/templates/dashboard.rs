use askama::Template;
use crate::models::SystemStats;

#[derive(Template)]
#[template(path = "dashboard.html", escape = "html")]
pub struct DashboardTemplate<'a> {
    pub title: &'a str,
    pub stats: SystemStats,
} 
