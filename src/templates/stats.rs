use crate::models::{DomainStats, SystemStats};
use askama::Template;

#[derive(Template)]
#[template(path = "stats.html", escape = "html")]
pub struct StatsTemplate<'a> {
    pub title: &'a str,
    pub system_stats: SystemStats,
    pub domain_stats: Vec<DomainStats>,
}
