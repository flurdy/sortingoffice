use askama::Template;
use crate::models::{SystemStats, DomainStats};

#[derive(Template)]
#[template(path = "stats.html")]
pub struct StatsTemplate {
    pub system_stats: SystemStats,
    pub domain_stats: Vec<DomainStats>,
} 
