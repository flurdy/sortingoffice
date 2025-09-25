use crate::models::CacheStats;
use askama::Template;

#[derive(Template)]
#[template(path = "cache_management/index.html")]
pub struct CacheManagementTemplate {
    pub cache_stats: CacheStats,
}

impl CacheManagementTemplate {
    pub fn new(cache_stats: CacheStats) -> Self {
        Self { cache_stats }
    }
}
