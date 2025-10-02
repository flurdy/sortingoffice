use crate::models::CacheStats;
use askama::Template;

#[derive(Template)]
#[template(path = "cache_management/index.html")]
pub struct CacheManagementTemplate {
    pub cache_stats: CacheStats,
    pub cache_title: String,
    pub cache_description: String,
    pub stats_title: String,
    pub report_caches_title: String,
    pub report_caches_desc: String,
    pub pagination_caches_title: String,
    pub pagination_caches_desc: String,
    pub dns_caches_title: String,
    pub dns_caches_desc: String,
    pub cached_label: String,
    pub not_cached_label: String,
    pub total_entries_label: String,
    pub actions_title: String,
    pub actions_desc: String,
    pub clear_all_label: String,
    pub clear_reports_label: String,
    pub clear_pagination_label: String,
    pub clear_system_stats_label: String,
    pub clear_dns_label: String,
    pub refresh_stats_label: String,
}

impl CacheManagementTemplate {
    pub fn new(cache_stats: CacheStats) -> Self {
        Self {
            cache_stats,
            cache_title: String::new(),
            cache_description: String::new(),
            stats_title: String::new(),
            report_caches_title: String::new(),
            report_caches_desc: String::new(),
            pagination_caches_title: String::new(),
            pagination_caches_desc: String::new(),
            dns_caches_title: String::new(),
            dns_caches_desc: String::new(),
            cached_label: String::new(),
            not_cached_label: String::new(),
            total_entries_label: String::new(),
            actions_title: String::new(),
            actions_desc: String::new(),
            clear_all_label: String::new(),
            clear_reports_label: String::new(),
            clear_pagination_label: String::new(),
            clear_system_stats_label: String::new(),
            clear_dns_label: String::new(),
            refresh_stats_label: String::new(),
        }
    }
}
