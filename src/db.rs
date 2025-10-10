use crate::config::DatabaseConfig;
use crate::models::*;
use crate::schema::*;
use crate::DbPool;
use chrono::{NaiveDateTime, Utc};
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::result::Error;
use diesel::sql_query;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Global cache invalidation callback
type CacheInvalidationCallback = Box<dyn Fn(&str) + Send + Sync>;

/// Global cache invalidation callback storage
static CACHE_INVALIDATION_CALLBACK: Mutex<Option<CacheInvalidationCallback>> = Mutex::new(None);

/// Set the global cache invalidation callback
pub fn set_cache_invalidation_callback(callback: CacheInvalidationCallback) {
    let mut cb = CACHE_INVALIDATION_CALLBACK.lock().unwrap();
    *cb = Some(callback);
}

/// Call the global cache invalidation callback
pub fn invalidate_cache_for_data_type(data_type: &str) {
    let cb = CACHE_INVALIDATION_CALLBACK.lock().unwrap();
    if let Some(ref callback) = *cb {
        callback(data_type);
    }
}

/// Simple cache entry with TTL
#[derive(Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Simple in-memory cache for frequently accessed data
#[derive(Clone)]
pub struct DataCache {
    system_stats: Arc<RwLock<Option<CacheEntry<SystemStats>>>>,
    // Report caches - using String keys for different report types
    catch_all_report: Arc<RwLock<Option<CacheEntry<Vec<CatchAllReport>>>>>,
    alias_report: Arc<RwLock<Option<CacheEntry<AliasReport>>>>,
    domain_alias_matrix_report: Arc<RwLock<Option<CacheEntry<DomainAliasMatrixReport>>>>,
    orphaned_aliases_report: Arc<RwLock<Option<CacheEntry<OrphanedAliasReport>>>>,
    external_forwarders_report: Arc<RwLock<Option<CacheEntry<ExternalForwarderReport>>>>,
    missing_aliases_report: Arc<RwLock<Option<CacheEntry<MissingAliasReport>>>>,
    // DNS caches
    dns_ns: Arc<RwLock<HashMap<String, CacheEntry<Vec<String>>>>>,
    dns_mx: Arc<RwLock<HashMap<String, CacheEntry<Vec<crate::services::dns_lookup::MxRecord>>>>>,
    dns_txt: Arc<RwLock<HashMap<String, CacheEntry<Vec<String>>>>>,
    dns_dkim: Arc<RwLock<HashMap<String, CacheEntry<Vec<String>>>>>,
    whois: Arc<RwLock<HashMap<String, CacheEntry<String>>>>,
    // Pagination caches - using HashMap for different pagination parameters
    domains_paginated: Arc<RwLock<HashMap<String, CacheEntry<PaginatedResult<Domain>>>>>,
    aliases_paginated: Arc<RwLock<HashMap<String, CacheEntry<PaginatedResult<Alias>>>>>,
    users_paginated: Arc<RwLock<HashMap<String, CacheEntry<PaginatedResult<User>>>>>,
    clients_paginated: Arc<RwLock<HashMap<String, CacheEntry<PaginatedResult<Client>>>>>,
    relays_paginated: Arc<RwLock<HashMap<String, CacheEntry<PaginatedResult<Relay>>>>>,
    relocated_paginated: Arc<RwLock<HashMap<String, CacheEntry<PaginatedResult<Relocated>>>>>,
}

impl DataCache {
    pub fn new() -> Self {
        Self {
            system_stats: Arc::new(RwLock::new(None)),
            catch_all_report: Arc::new(RwLock::new(None)),
            alias_report: Arc::new(RwLock::new(None)),
            domain_alias_matrix_report: Arc::new(RwLock::new(None)),
            orphaned_aliases_report: Arc::new(RwLock::new(None)),
            external_forwarders_report: Arc::new(RwLock::new(None)),
            missing_aliases_report: Arc::new(RwLock::new(None)),
            dns_ns: Arc::new(RwLock::new(HashMap::new())),
            dns_mx: Arc::new(RwLock::new(HashMap::new())),
            dns_txt: Arc::new(RwLock::new(HashMap::new())),
            dns_dkim: Arc::new(RwLock::new(HashMap::new())),
            whois: Arc::new(RwLock::new(HashMap::new())),
            domains_paginated: Arc::new(RwLock::new(HashMap::new())),
            aliases_paginated: Arc::new(RwLock::new(HashMap::new())),
            users_paginated: Arc::new(RwLock::new(HashMap::new())),
            clients_paginated: Arc::new(RwLock::new(HashMap::new())),
            relays_paginated: Arc::new(RwLock::new(HashMap::new())),
            relocated_paginated: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached system stats if available and not expired
    pub async fn get_system_stats(&self) -> Option<SystemStats> {
        let cache = self.system_stats.read().await;
        if let Some(entry) = cache.as_ref() {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Cache system stats with TTL
    pub async fn set_system_stats(&self, stats: SystemStats, ttl: Duration) {
        let mut cache = self.system_stats.write().await;
        *cache = Some(CacheEntry::new(stats, ttl));
    }

    /// Clear system stats cache
    pub async fn clear_system_stats(&self) {
        let mut cache = self.system_stats.write().await;
        *cache = None;
    }

    // Generic cache methods for reports
    async fn get_cached_report<T: Clone>(cache: &Arc<RwLock<Option<CacheEntry<T>>>>) -> Option<T> {
        let cache_guard = cache.read().await;
        if let Some(entry) = cache_guard.as_ref() {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    async fn set_cached_report<T: Clone>(
        cache: &Arc<RwLock<Option<CacheEntry<T>>>>,
        data: T,
        ttl: Duration,
    ) {
        let mut cache_guard = cache.write().await;
        *cache_guard = Some(CacheEntry::new(data, ttl));
    }

    async fn clear_cached_report<T>(cache: &Arc<RwLock<Option<CacheEntry<T>>>>) {
        let mut cache_guard = cache.write().await;
        *cache_guard = None;
    }

    // Catch-all report cache methods
    pub async fn get_catch_all_report(&self) -> Option<Vec<CatchAllReport>> {
        Self::get_cached_report(&self.catch_all_report).await
    }

    pub async fn set_catch_all_report(&self, data: Vec<CatchAllReport>, ttl: Duration) {
        Self::set_cached_report(&self.catch_all_report, data, ttl).await;
    }

    pub async fn clear_catch_all_report(&self) {
        Self::clear_cached_report(&self.catch_all_report).await;
    }

    // Alias report cache methods
    pub async fn get_alias_report(&self) -> Option<AliasReport> {
        Self::get_cached_report(&self.alias_report).await
    }

    pub async fn set_alias_report(&self, data: AliasReport, ttl: Duration) {
        Self::set_cached_report(&self.alias_report, data, ttl).await;
    }

    pub async fn clear_alias_report(&self) {
        Self::clear_cached_report(&self.alias_report).await;
    }

    // Domain alias matrix report cache methods
    pub async fn get_domain_alias_matrix_report(&self) -> Option<DomainAliasMatrixReport> {
        Self::get_cached_report(&self.domain_alias_matrix_report).await
    }

    pub async fn set_domain_alias_matrix_report(
        &self,
        data: DomainAliasMatrixReport,
        ttl: Duration,
    ) {
        Self::set_cached_report(&self.domain_alias_matrix_report, data, ttl).await;
    }

    pub async fn clear_domain_alias_matrix_report(&self) {
        Self::clear_cached_report(&self.domain_alias_matrix_report).await;
    }

    // Orphaned aliases report cache methods
    pub async fn get_orphaned_aliases_report(&self) -> Option<OrphanedAliasReport> {
        Self::get_cached_report(&self.orphaned_aliases_report).await
    }

    pub async fn set_orphaned_aliases_report(&self, data: OrphanedAliasReport, ttl: Duration) {
        Self::set_cached_report(&self.orphaned_aliases_report, data, ttl).await;
    }

    pub async fn clear_orphaned_aliases_report(&self) {
        Self::clear_cached_report(&self.orphaned_aliases_report).await;
    }

    // External forwarders report cache methods
    pub async fn get_external_forwarders_report(&self) -> Option<ExternalForwarderReport> {
        Self::get_cached_report(&self.external_forwarders_report).await
    }

    pub async fn set_external_forwarders_report(
        &self,
        data: ExternalForwarderReport,
        ttl: Duration,
    ) {
        Self::set_cached_report(&self.external_forwarders_report, data, ttl).await;
    }

    pub async fn clear_external_forwarders_report(&self) {
        Self::clear_cached_report(&self.external_forwarders_report).await;
    }

    // Missing aliases report cache methods
    pub async fn get_missing_aliases_report(&self) -> Option<MissingAliasReport> {
        Self::get_cached_report(&self.missing_aliases_report).await
    }

    pub async fn set_missing_aliases_report(&self, data: MissingAliasReport, ttl: Duration) {
        Self::set_cached_report(&self.missing_aliases_report, data, ttl).await;
    }

    pub async fn clear_missing_aliases_report(&self) {
        Self::clear_cached_report(&self.missing_aliases_report).await;
    }

    /// Clear all caches
    pub async fn clear_all_caches(&self) {
        self.clear_system_stats().await;
        self.clear_catch_all_report().await;
        self.clear_alias_report().await;
        self.clear_domain_alias_matrix_report().await;
        self.clear_orphaned_aliases_report().await;
        self.clear_external_forwarders_report().await;
        self.clear_missing_aliases_report().await;
        self.clear_all_pagination_caches().await;
        self.clear_all_dns_caches().await;
    }

    // Generic pagination cache methods
    async fn get_cached_pagination<T: Clone>(
        cache: &Arc<RwLock<HashMap<String, CacheEntry<PaginatedResult<T>>>>>,
        key: &str,
    ) -> Option<PaginatedResult<T>> {
        let cache_guard = cache.read().await;
        if let Some(entry) = cache_guard.get(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    async fn set_cached_pagination<T: Clone>(
        cache: &Arc<RwLock<HashMap<String, CacheEntry<PaginatedResult<T>>>>>,
        key: String,
        data: PaginatedResult<T>,
        ttl: Duration,
    ) {
        let mut cache_guard = cache.write().await;
        cache_guard.insert(key, CacheEntry::new(data, ttl));
    }

    async fn clear_cached_pagination<T>(
        cache: &Arc<RwLock<HashMap<String, CacheEntry<PaginatedResult<T>>>>>,
        key: &str,
    ) {
        let mut cache_guard = cache.write().await;
        cache_guard.remove(key);
    }

    // Domains pagination cache methods
    pub async fn get_domains_paginated(&self, key: &str) -> Option<PaginatedResult<Domain>> {
        Self::get_cached_pagination(&self.domains_paginated, key).await
    }

    pub async fn set_domains_paginated(
        &self,
        key: String,
        data: PaginatedResult<Domain>,
        ttl: Duration,
    ) {
        Self::set_cached_pagination(&self.domains_paginated, key, data, ttl).await;
    }

    pub async fn clear_domains_paginated(&self, key: &str) {
        Self::clear_cached_pagination(&self.domains_paginated, key).await;
    }

    // Aliases pagination cache methods
    pub async fn get_aliases_paginated(&self, key: &str) -> Option<PaginatedResult<Alias>> {
        Self::get_cached_pagination(&self.aliases_paginated, key).await
    }

    pub async fn set_aliases_paginated(
        &self,
        key: String,
        data: PaginatedResult<Alias>,
        ttl: Duration,
    ) {
        Self::set_cached_pagination(&self.aliases_paginated, key, data, ttl).await;
    }

    pub async fn clear_aliases_paginated(&self, key: &str) {
        Self::clear_cached_pagination(&self.aliases_paginated, key).await;
    }

    // Users pagination cache methods
    pub async fn get_users_paginated(&self, key: &str) -> Option<PaginatedResult<User>> {
        Self::get_cached_pagination(&self.users_paginated, key).await
    }

    pub async fn set_users_paginated(
        &self,
        key: String,
        data: PaginatedResult<User>,
        ttl: Duration,
    ) {
        Self::set_cached_pagination(&self.users_paginated, key, data, ttl).await;
    }

    pub async fn clear_users_paginated(&self, key: &str) {
        Self::clear_cached_pagination(&self.users_paginated, key).await;
    }

    // Clients pagination cache methods
    pub async fn get_clients_paginated(&self, key: &str) -> Option<PaginatedResult<Client>> {
        Self::get_cached_pagination(&self.clients_paginated, key).await
    }

    pub async fn set_clients_paginated(
        &self,
        key: String,
        data: PaginatedResult<Client>,
        ttl: Duration,
    ) {
        Self::set_cached_pagination(&self.clients_paginated, key, data, ttl).await;
    }

    pub async fn clear_clients_paginated(&self, key: &str) {
        Self::clear_cached_pagination(&self.clients_paginated, key).await;
    }

    // Relays pagination cache methods
    pub async fn get_relays_paginated(&self, key: &str) -> Option<PaginatedResult<Relay>> {
        Self::get_cached_pagination(&self.relays_paginated, key).await
    }

    pub async fn set_relays_paginated(
        &self,
        key: String,
        data: PaginatedResult<Relay>,
        ttl: Duration,
    ) {
        Self::set_cached_pagination(&self.relays_paginated, key, data, ttl).await;
    }

    pub async fn clear_relays_paginated(&self, key: &str) {
        Self::clear_cached_pagination(&self.relays_paginated, key).await;
    }

    // Relocated pagination cache methods
    pub async fn get_relocated_paginated(&self, key: &str) -> Option<PaginatedResult<Relocated>> {
        Self::get_cached_pagination(&self.relocated_paginated, key).await
    }

    pub async fn set_relocated_paginated(
        &self,
        key: String,
        data: PaginatedResult<Relocated>,
        ttl: Duration,
    ) {
        Self::set_cached_pagination(&self.relocated_paginated, key, data, ttl).await;
    }

    pub async fn clear_relocated_paginated(&self, key: &str) {
        Self::clear_cached_pagination(&self.relocated_paginated, key).await;
    }

    /// Clear all pagination caches
    pub async fn clear_all_pagination_caches(&self) {
        let mut domains_cache = self.domains_paginated.write().await;
        domains_cache.clear();
        let mut aliases_cache = self.aliases_paginated.write().await;
        aliases_cache.clear();
        let mut users_cache = self.users_paginated.write().await;
        users_cache.clear();
        let mut clients_cache = self.clients_paginated.write().await;
        clients_cache.clear();
        let mut relays_cache = self.relays_paginated.write().await;
        relays_cache.clear();
        let mut relocated_cache = self.relocated_paginated.write().await;
        relocated_cache.clear();
    }

    /// Get cache statistics for monitoring
    pub async fn get_stats(&self) -> CacheStats {
        let system_stats_cached = self.system_stats.read().await.is_some();
        let catch_all_report_cached = self.catch_all_report.read().await.is_some();
        let alias_report_cached = self.alias_report.read().await.is_some();
        let domain_alias_matrix_report_cached =
            self.domain_alias_matrix_report.read().await.is_some();
        let orphaned_aliases_report_cached = self.orphaned_aliases_report.read().await.is_some();
        let external_forwarders_report_cached =
            self.external_forwarders_report.read().await.is_some();
        let missing_aliases_report_cached = self.missing_aliases_report.read().await.is_some();

        let domains_paginated_count = self.domains_paginated.read().await.len();
        let aliases_paginated_count = self.aliases_paginated.read().await.len();
        let users_paginated_count = self.users_paginated.read().await.len();
        let clients_paginated_count = self.clients_paginated.read().await.len();
        let relays_paginated_count = self.relays_paginated.read().await.len();
        let relocated_paginated_count = self.relocated_paginated.read().await.len();

        let total_pagination_entries = domains_paginated_count
            + aliases_paginated_count
            + users_paginated_count
            + clients_paginated_count
            + relays_paginated_count
            + relocated_paginated_count;

        // DNS stats
        let dns_ns_count = self.dns_ns.read().await.len();
        let dns_mx_count = self.dns_mx.read().await.len();
        let dns_txt_count = self.dns_txt.read().await.len();
        let dns_dkim_count = self.dns_dkim.read().await.len();
        let total_dns_entries = dns_ns_count + dns_mx_count + dns_txt_count + dns_dkim_count;

        CacheStats {
            system_stats_cached,
            catch_all_report_cached,
            alias_report_cached,
            domain_alias_matrix_report_cached,
            orphaned_aliases_report_cached,
            external_forwarders_report_cached,
            missing_aliases_report_cached,
            dns_ns_count,
            dns_mx_count,
            dns_txt_count,
            dns_dkim_count,
            total_dns_entries,
            domains_paginated_count,
            aliases_paginated_count,
            users_paginated_count,
            clients_paginated_count,
            relays_paginated_count,
            relocated_paginated_count,
            total_pagination_entries,
        }
    }

    // DNS cache helpers
    pub async fn get_dns_ns(&self, domain: &str) -> Option<Vec<String>> {
        let cache = self.dns_ns.read().await;
        cache.get(domain).and_then(|e| {
            if e.is_expired() {
                None
            } else {
                Some(e.data.clone())
            }
        })
    }
    pub async fn set_dns_ns(&self, domain: &str, data: Vec<String>, ttl: Duration) {
        let mut cache = self.dns_ns.write().await;
        cache.insert(domain.to_string(), CacheEntry::new(data, ttl));
    }
    pub async fn get_dns_mx(
        &self,
        domain: &str,
    ) -> Option<Vec<crate::services::dns_lookup::MxRecord>> {
        let cache = self.dns_mx.read().await;
        cache.get(domain).and_then(|e| {
            if e.is_expired() {
                None
            } else {
                Some(e.data.clone())
            }
        })
    }
    pub async fn set_dns_mx(
        &self,
        domain: &str,
        data: Vec<crate::services::dns_lookup::MxRecord>,
        ttl: Duration,
    ) {
        let mut cache = self.dns_mx.write().await;
        cache.insert(domain.to_string(), CacheEntry::new(data, ttl));
    }
    pub async fn get_dns_txt(&self, domain: &str) -> Option<Vec<String>> {
        let cache = self.dns_txt.read().await;
        cache.get(domain).and_then(|e| {
            if e.is_expired() {
                None
            } else {
                Some(e.data.clone())
            }
        })
    }
    pub async fn set_dns_txt(&self, domain: &str, data: Vec<String>, ttl: Duration) {
        let mut cache = self.dns_txt.write().await;
        cache.insert(domain.to_string(), CacheEntry::new(data, ttl));
    }
    pub async fn get_dns_dkim(&self, key: &str) -> Option<Vec<String>> {
        let cache = self.dns_dkim.read().await;
        cache.get(key).and_then(|e| {
            if e.is_expired() {
                None
            } else {
                Some(e.data.clone())
            }
        })
    }
    pub async fn set_dns_dkim(&self, key: &str, data: Vec<String>, ttl: Duration) {
        let mut cache = self.dns_dkim.write().await;
        cache.insert(key.to_string(), CacheEntry::new(data, ttl));
    }
    pub async fn get_whois(&self, domain: &str) -> Option<String> {
        let cache = self.whois.read().await;
        cache.get(domain).and_then(|e| {
            if e.is_expired() {
                None
            } else {
                Some(e.data.clone())
            }
        })
    }
    pub async fn set_whois(&self, domain: &str, data: String, ttl: Duration) {
        let mut cache = self.whois.write().await;
        cache.insert(domain.to_string(), CacheEntry::new(data, ttl));
    }
    pub async fn clear_all_dns_caches(&self) {
        self.dns_ns.write().await.clear();
        self.dns_mx.write().await.clear();
        self.dns_txt.write().await.clear();
        self.dns_dkim.write().await.clear();
        self.whois.write().await.clear();
    }
}

/// Manages multiple database connections
#[derive(Clone)]
pub struct DatabaseManager {
    pools: Arc<RwLock<HashMap<String, DbPool>>>,
    configs: Vec<DatabaseConfig>,
    default_db: String,
    cache: DataCache,
}

impl DatabaseManager {
    /// Create a new database manager with multiple database connections
    pub async fn new(configs: Vec<DatabaseConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut pools = HashMap::new();
        let default_db = configs
            .first()
            .map(|c| c.id.clone())
            .unwrap_or_else(|| "primary".to_string());

        for config in &configs {
            let manager = ConnectionManager::<MysqlConnection>::new(&config.url);

            // Use connection pool configuration from the database config
            match r2d2::Pool::builder()
                .max_size(config.connection_pool.max_size)
                .min_idle(Some(config.connection_pool.min_idle))
                .connection_timeout(std::time::Duration::from_secs(
                    config.connection_pool.connection_timeout,
                ))
                .idle_timeout(Some(std::time::Duration::from_secs(
                    config.connection_pool.idle_timeout,
                )))
                .max_lifetime(Some(std::time::Duration::from_secs(
                    config.connection_pool.max_lifetime,
                )))
                .build(manager)
            {
                Ok(pool) => {
                    tracing::info!(
                        "Created connection pool for database '{}' with max_size={}, min_idle={}",
                        config.id,
                        config.connection_pool.max_size,
                        config.connection_pool.min_idle
                    );
                    pools.insert(config.id.clone(), pool);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to create pool for database '{}': {}. Application will start but this database will be unavailable.",
                        config.id, e
                    );
                    // Don't fail the entire application startup, just log the warning
                }
            }
        }

        let db_manager = DatabaseManager {
            pools: Arc::new(RwLock::new(pools)),
            configs,
            default_db,
            cache: DataCache::new(),
        };

        // Set up the global cache invalidation callback
        let db_manager_clone = db_manager.clone();
        set_cache_invalidation_callback(Box::new(move |data_type| {
            let data_type = data_type.to_string(); // Clone to get owned String
            let db_manager = db_manager_clone.clone(); // Clone for the async block
            let rt = tokio::runtime::Handle::current();
            rt.spawn(async move {
                db_manager.clear_caches_for_data_type(&data_type).await;
            });
        }));

        Ok(db_manager)
    }

    /// Get database configurations
    pub fn get_configs(&self) -> &Vec<DatabaseConfig> {
        &self.configs
    }

    /// Get a database pool by ID
    pub async fn get_pool(&self, db_id: &str) -> Option<DbPool> {
        let pools = self.pools.read().await;
        pools.get(db_id).cloned()
    }

    /// Lazily create a pool on first use with brief retries. This avoids startup races
    /// where the database is not reachable yet when the application boots.
    pub async fn get_or_create_pool(&self, db_id: &str) -> Option<DbPool> {
        // Fast path: already present
        if let Some(existing) = self.get_pool(db_id).await {
            return Some(existing);
        }

        // Find config for this database id
        let db_config = match self.configs.iter().find(|c| c.id == db_id) {
            Some(c) => c.clone(),
            None => return None,
        };

        // Small retry loop to allow MySQL service/DNS to become ready
        let mut last_err: Option<String> = None;
        for attempt in 0..10 {
            let manager = ConnectionManager::<MysqlConnection>::new(&db_config.url);
            match r2d2::Pool::builder()
                .max_size(db_config.connection_pool.max_size)
                .min_idle(Some(db_config.connection_pool.min_idle))
                .connection_timeout(std::time::Duration::from_secs(
                    db_config.connection_pool.connection_timeout,
                ))
                .idle_timeout(Some(std::time::Duration::from_secs(
                    db_config.connection_pool.idle_timeout,
                )))
                .max_lifetime(Some(std::time::Duration::from_secs(
                    db_config.connection_pool.max_lifetime,
                )))
                .build(manager)
            {
                Ok(pool) => {
                    // Insert into map
                    let mut pools = self.pools.write().await;
                    pools.insert(db_id.to_string(), pool.clone());
                    tracing::info!(
                        "Initialized connection pool for '{}' on attempt {}",
                        db_id,
                        attempt + 1
                    );
                    return Some(pool);
                }
                Err(e) => {
                    last_err = Some(e.to_string());
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                }
            }
        }

        tracing::warn!(
            "Failed to initialize connection pool for '{}' after retries: {}",
            db_id,
            last_err.unwrap_or_else(|| "unknown error".to_string())
        );
        None
    }

    /// Get the default database pool
    pub async fn get_default_pool(&self) -> Option<DbPool> {
        self.get_pool(&self.default_db).await
    }

    /// Get all available database configurations

    /// Get only enabled database configurations (not disabled)
    pub fn get_enabled_configs(&self) -> Vec<DatabaseConfig> {
        self.configs
            .iter()
            .filter(|config| !config.features.disabled)
            .cloned()
            .collect()
    }

    /// Get the default database ID
    pub fn get_default_db_id(&self) -> &str {
        &self.default_db
    }

    /// Check if a database ID exists
    pub async fn has_database(&self, db_id: &str) -> bool {
        let pools = self.pools.read().await;
        pools.contains_key(db_id)
    }

    /// Get connection pool statistics for monitoring
    pub async fn get_pool_stats(&self, db_id: &str) -> Option<PoolStats> {
        let pools = self.pools.read().await;
        pools.get(db_id).map(|pool| PoolStats {
            max_size: pool.max_size(),
            size: pool.max_size(),      // r2d2 doesn't expose current size
            available: pool.max_size(), // Simplified for now
            in_use: 0,                  // Simplified for now
        })
    }

    /// Get connection pool statistics for all databases
    pub async fn get_all_pool_stats(&self) -> HashMap<String, PoolStats> {
        let pools = self.pools.read().await;
        let mut stats = HashMap::new();

        for (db_id, pool) in pools.iter() {
            stats.insert(
                db_id.clone(),
                PoolStats {
                    max_size: pool.max_size(),
                    size: pool.max_size(), // r2d2 doesn't expose current size
                    available: pool.max_size(), // Simplified for now
                    in_use: 0,             // Simplified for now
                },
            );
        }

        stats
    }

    /// Health check for a specific database pool
    pub async fn health_check(&self, db_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let pools = self.pools.read().await;
        if let Some(pool) = pools.get(db_id) {
            match pool.get() {
                Ok(mut conn) => {
                    // Test the connection with a simple query
                    match diesel::sql_query("SELECT 1").execute(&mut conn) {
                        Ok(_) => {
                            tracing::debug!("Health check passed for database: {}", db_id);
                            Ok(true)
                        }
                        Err(e) => {
                            tracing::error!("Health check failed for database {}: {:?}", db_id, e);
                            Ok(false)
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to get connection for health check on {}: {:?}",
                        db_id,
                        e
                    );
                    Ok(false)
                }
            }
        } else {
            Err(format!("Database pool not found: {db_id}").into())
        }
    }

    /// Health check all databases
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();
        let pools = self.pools.read().await;

        for config in &self.configs {
            let is_healthy = if let Some(_pool) = pools.get(&config.id) {
                (self.health_check(&config.id).await).unwrap_or_default()
            } else {
                // Database pool not available (failed to create)
                false
            };
            results.insert(config.id.clone(), is_healthy);
        }

        results
    }

    /// Run migrations on all configured databases
    pub async fn run_migrations_on_all_databases(
        &self,
        config: &crate::config::Config,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

        let pools = self.pools.read().await;

        for db_config in &self.configs {
            // Check if migrations are blocked for this database
            if config.is_migration_blocked(&db_config.id) {
                tracing::warn!(
                    "⚠️  Migrations blocked for database: {} (read-only, disabled, or no_migrations=true)",
                    db_config.id
                );
                continue;
            }

            if let Some(pool) = pools.get(&db_config.id) {
                tracing::info!("Running migrations on database: {}", db_config.id);

                match pool.get() {
                    Ok(mut conn) => match conn.run_pending_migrations(MIGRATIONS) {
                        Ok(_) => tracing::info!(
                            "✅ Migrations completed successfully for database: {}",
                            db_config.id
                        ),
                        Err(e) => {
                            tracing::error!(
                                "❌ Failed to run migrations on database {}: {}",
                                db_config.id,
                                e
                            );
                            return Err(format!(
                                "Failed to run migrations on database {}: {}",
                                db_config.id, e
                            )
                            .into());
                        }
                    },
                    Err(e) => {
                        tracing::error!(
                            "❌ Failed to get connection for database {}: {}",
                            db_config.id,
                            e
                        );
                        return Err(format!(
                            "Failed to get connection for database {}: {}",
                            db_config.id, e
                        )
                        .into());
                    }
                }
            } else {
                tracing::warn!("⚠️  No pool found for database: {}", db_config.id);
            }
        }

        tracing::info!("✅ Migrations completed on all databases");
        Ok(())
    }

    /// Run migrations on a specific database
    pub async fn run_migrations_on_database(
        &self,
        db_id: &str,
        config: &crate::config::Config,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

        // Check if migrations are blocked for this database
        if config.is_migration_blocked(db_id) {
            return Err(format!(
                "Migrations blocked for database: {db_id} (read-only, disabled, or no_migrations=true)"
            )
            .into());
        }

        let pools = self.pools.read().await;

        if let Some(pool) = pools.get(db_id) {
            tracing::info!("Running migrations on database: {}", db_id);

            let mut conn = pool
                .get()
                .map_err(|e| format!("Failed to get connection for database {db_id}: {e}"))?;

            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|e| format!("Failed to run migrations on database {db_id}: {e}"))?;

            tracing::info!(
                "✅ Migrations completed successfully for database: {}",
                db_id
            );
            Ok(())
        } else {
            Err(format!("No pool found for database: {db_id}").into())
        }
    }

    /// Get system stats with caching (5 minute TTL)
    pub async fn get_system_stats_cached(&self, pool: &DbPool) -> Result<SystemStats, Error> {
        // Try to get from cache first
        if let Some(cached_stats) = self.cache.get_system_stats().await {
            tracing::debug!("Returning cached system stats");
            return Ok(cached_stats);
        }

        // Cache miss, fetch from database
        tracing::debug!("Cache miss, fetching system stats from database");
        let stats = get_system_stats(pool)?;

        // Cache the result for 5 minutes
        self.cache
            .set_system_stats(stats.clone(), Duration::from_secs(300))
            .await;

        Ok(stats)
    }

    /// Clear system stats cache (useful after data modifications)
    pub async fn clear_system_stats_cache(&self) {
        self.cache.clear_system_stats().await;
    }

    // Cached report functions (10-minute TTL for reports)
    pub async fn get_catch_all_report_cached(
        &self,
        pool: &DbPool,
    ) -> Result<Vec<CatchAllReport>, Error> {
        if let Some(cached_report) = self.cache.get_catch_all_report().await {
            tracing::debug!("Returning cached catch-all report");
            return Ok(cached_report);
        }

        tracing::debug!("Cache miss, fetching catch-all report from database");
        let report = get_catch_all_report(pool)?;

        self.cache
            .set_catch_all_report(report.clone(), Duration::from_secs(600))
            .await;
        Ok(report)
    }

    pub async fn get_alias_report_cached(&self, pool: &DbPool) -> Result<AliasReport, Error> {
        if let Some(cached_report) = self.cache.get_alias_report().await {
            tracing::debug!("Returning cached alias report");
            return Ok(cached_report);
        }

        tracing::debug!("Cache miss, fetching alias report from database");
        let report = get_alias_report(pool)?;

        self.cache
            .set_alias_report(report.clone(), Duration::from_secs(600))
            .await;
        Ok(report)
    }

    pub async fn get_domain_alias_matrix_report_cached(
        &self,
        pool: &DbPool,
    ) -> Result<DomainAliasMatrixReport, Error> {
        if let Some(cached_report) = self.cache.get_domain_alias_matrix_report().await {
            tracing::debug!("Returning cached domain alias matrix report");
            return Ok(cached_report);
        }

        tracing::debug!("Cache miss, fetching domain alias matrix report from database");
        let report = get_domain_alias_matrix_report(pool)?;

        self.cache
            .set_domain_alias_matrix_report(report.clone(), Duration::from_secs(600))
            .await;
        Ok(report)
    }

    pub async fn get_orphaned_aliases_report_cached(
        &self,
        pool: &DbPool,
    ) -> Result<OrphanedAliasReport, Error> {
        if let Some(cached_report) = self.cache.get_orphaned_aliases_report().await {
            tracing::debug!("Returning cached orphaned aliases report");
            return Ok(cached_report);
        }

        tracing::debug!("Cache miss, fetching orphaned aliases report from database");
        let report = get_orphaned_aliases_report(pool)?;

        self.cache
            .set_orphaned_aliases_report(report.clone(), Duration::from_secs(600))
            .await;
        Ok(report)
    }

    pub async fn get_external_forwarders_report_cached(
        &self,
        pool: &DbPool,
    ) -> Result<ExternalForwarderReport, Error> {
        if let Some(cached_report) = self.cache.get_external_forwarders_report().await {
            tracing::debug!("Returning cached external forwarders report");
            return Ok(cached_report);
        }

        tracing::debug!("Cache miss, fetching external forwarders report from database");
        let report = get_external_forwarders_report(pool)?;

        self.cache
            .set_external_forwarders_report(report.clone(), Duration::from_secs(600))
            .await;
        Ok(report)
    }

    pub async fn get_missing_aliases_report_cached(
        &self,
        pool: &DbPool,
    ) -> Result<MissingAliasReport, Error> {
        if let Some(cached_report) = self.cache.get_missing_aliases_report().await {
            tracing::debug!("Returning cached missing aliases report");
            return Ok(cached_report);
        }

        tracing::debug!("Cache miss, fetching missing aliases report from database");
        let report = get_missing_aliases_report(pool)?;

        self.cache
            .set_missing_aliases_report(report.clone(), Duration::from_secs(600))
            .await;
        Ok(report)
    }

    /// Clear all caches (useful after data modifications)
    pub async fn clear_all_caches(&self) {
        self.cache.clear_all_caches().await;
    }

    /// Get cache statistics for monitoring
    pub async fn get_cache_stats(&self) -> CacheStats {
        self.cache.get_stats().await
    }

    // DNS cache wrappers
    pub async fn get_dns_ns(&self, domain: &str) -> Option<Vec<String>> {
        self.cache.get_dns_ns(domain).await
    }

    pub async fn set_dns_ns(&self, domain: &str, data: Vec<String>, ttl: Duration) {
        self.cache.set_dns_ns(domain, data, ttl).await;
    }

    pub async fn get_dns_mx(
        &self,
        domain: &str,
    ) -> Option<Vec<crate::services::dns_lookup::MxRecord>> {
        self.cache.get_dns_mx(domain).await
    }

    pub async fn set_dns_mx(
        &self,
        domain: &str,
        data: Vec<crate::services::dns_lookup::MxRecord>,
        ttl: Duration,
    ) {
        self.cache.set_dns_mx(domain, data, ttl).await;
    }

    pub async fn get_dns_txt(&self, domain: &str) -> Option<Vec<String>> {
        self.cache.get_dns_txt(domain).await
    }

    pub async fn set_dns_txt(&self, domain: &str, data: Vec<String>, ttl: Duration) {
        self.cache.set_dns_txt(domain, data, ttl).await;
    }

    pub async fn get_dns_dkim(&self, key: &str) -> Option<Vec<String>> {
        self.cache.get_dns_dkim(key).await
    }

    pub async fn set_dns_dkim(&self, key: &str, data: Vec<String>, ttl: Duration) {
        self.cache.set_dns_dkim(key, data, ttl).await;
    }

    pub async fn get_whois(&self, domain: &str) -> Option<String> {
        self.cache.get_whois(domain).await
    }

    pub async fn set_whois(&self, domain: &str, data: String, ttl: Duration) {
        self.cache.set_whois(domain, data, ttl).await;
    }

    /// Clear specific cache types
    pub async fn clear_cache_by_type(&self, cache_type: &str) {
        match cache_type {
            "system_stats" => self.cache.clear_system_stats().await,
            "reports" => {
                self.cache.clear_catch_all_report().await;
                self.cache.clear_alias_report().await;
                self.cache.clear_domain_alias_matrix_report().await;
                self.cache.clear_orphaned_aliases_report().await;
                self.cache.clear_external_forwarders_report().await;
                self.cache.clear_missing_aliases_report().await;
            }
            "pagination" => self.cache.clear_all_pagination_caches().await,
            "dns" => self.cache.clear_all_dns_caches().await,
            "all" => self.cache.clear_all_caches().await,
            _ => {
                tracing::warn!("Unknown cache type '{}' for cache clearing", cache_type);
            }
        }
    }

    /// Clear caches based on the type of data that was modified
    pub async fn clear_caches_for_data_type(&self, data_type: &str) {
        match data_type {
            "domain" => {
                // Domains affect system stats, reports, and domain pagination
                self.cache.clear_system_stats().await;
                self.cache.clear_alias_report().await;
                self.cache.clear_domain_alias_matrix_report().await;
                self.cache.clear_orphaned_aliases_report().await;
                self.cache.clear_external_forwarders_report().await;
                self.cache.clear_missing_aliases_report().await;
                self.cache.clear_all_pagination_caches().await; // Clear all pagination as domains affect all reports
            }
            "user" => {
                // Users affect system stats, reports, and user pagination
                self.cache.clear_system_stats().await;
                self.cache.clear_orphaned_aliases_report().await;
                self.cache.clear_external_forwarders_report().await;
                self.cache.clear_missing_aliases_report().await;
                self.cache.clear_all_pagination_caches().await; // Clear all pagination as users affect all reports
            }
            "alias" => {
                // Aliases affect system stats, reports, and alias pagination
                self.cache.clear_system_stats().await;
                self.cache.clear_catch_all_report().await;
                self.cache.clear_alias_report().await;
                self.cache.clear_domain_alias_matrix_report().await;
                self.cache.clear_orphaned_aliases_report().await;
                self.cache.clear_external_forwarders_report().await;
                self.cache.clear_missing_aliases_report().await;
                self.cache.clear_all_pagination_caches().await; // Clear all pagination as aliases affect all reports
            }
            "backup" => {
                // Backups affect system stats
                self.cache.clear_system_stats().await;
            }
            "relay" => {
                // Relays affect system stats and relay pagination
                self.cache.clear_system_stats().await;
                self.cache.clear_all_pagination_caches().await; // Clear all pagination as relays affect all reports
            }
            "relocated" => {
                // Relocated affect system stats and relocated pagination
                self.cache.clear_system_stats().await;
                self.cache.clear_all_pagination_caches().await; // Clear all pagination as relocated affect all reports
            }
            "client" => {
                // Clients affect system stats and client pagination
                self.cache.clear_system_stats().await;
                self.cache.clear_all_pagination_caches().await; // Clear all pagination as clients affect all reports
            }
            _ => {
                // Unknown data type, clear all caches to be safe
                tracing::warn!(
                    "Unknown data type '{}' for cache invalidation, clearing all caches",
                    data_type
                );
                self.cache.clear_all_caches().await;
            }
        }
    }

    // Cached pagination functions (5-minute TTL for pagination)
    pub async fn get_domains_paginated_cached(
        &self,
        pool: &DbPool,
        page: i64,
        per_page: i64,
        db_id: &str,
        search: Option<&str>,
        enabled_filter: &str,
        exclude_subdomains: bool,
    ) -> Result<PaginatedResult<Domain>, Error> {
        let search_key = search.unwrap_or("");
        let cache_key = format!(
            "domains_{}_{}_{}_{}_{}_{}",
            db_id, page, per_page, search_key, enabled_filter, exclude_subdomains
        );

        if let Some(cached_result) = self.cache.get_domains_paginated(&cache_key).await {
            tracing::debug!("Returning cached domains pagination for key: {}", cache_key);
            return Ok(cached_result);
        }

        tracing::debug!(
            "Cache miss, fetching domains pagination from database for key: {}",
            cache_key
        );
        let result = get_domains_paginated(
            pool,
            page,
            per_page,
            search,
            enabled_filter,
            exclude_subdomains,
        )?;

        self.cache
            .set_domains_paginated(cache_key, result.clone(), Duration::from_secs(300))
            .await;
        Ok(result)
    }

    pub async fn get_aliases_paginated_cached(
        &self,
        pool: &DbPool,
        page: i64,
        per_page: i64,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
        db_id: &str,
        search: Option<&str>,
        enabled_filter: &str,
    ) -> Result<PaginatedResult<Alias>, Error> {
        let search_key = search.unwrap_or("");
        let cache_key = format!(
            "aliases_{}_{}_{}_{}_{}_{}_{}",
            db_id,
            page,
            per_page,
            sort_by.unwrap_or(""),
            sort_order.unwrap_or(""),
            search_key,
            enabled_filter
        );

        if let Some(cached_result) = self.cache.get_aliases_paginated(&cache_key).await {
            tracing::debug!("Returning cached aliases pagination for key: {}", cache_key);
            return Ok(cached_result);
        }

        tracing::debug!(
            "Cache miss, fetching aliases pagination from database for key: {}",
            cache_key
        );
        let result = get_aliases_paginated(
            pool,
            page,
            per_page,
            sort_by,
            sort_order,
            search,
            enabled_filter,
        )?;

        self.cache
            .set_aliases_paginated(cache_key, result.clone(), Duration::from_secs(300))
            .await;
        Ok(result)
    }

    pub async fn get_users_paginated_cached(
        &self,
        pool: &DbPool,
        page: i64,
        per_page: i64,
        db_id: &str,
        enabled_filter: &str,
    ) -> Result<PaginatedResult<User>, Error> {
        let cache_key = format!("users_{}_{}_{}_{}", db_id, page, per_page, enabled_filter);

        if let Some(cached_result) = self.cache.get_users_paginated(&cache_key).await {
            tracing::debug!("Returning cached users pagination for key: {}", cache_key);
            return Ok(cached_result);
        }

        tracing::debug!(
            "Cache miss, fetching users pagination from database for key: {}",
            cache_key
        );
        let result = get_users_paginated(pool, page, per_page, enabled_filter)?;

        self.cache
            .set_users_paginated(cache_key, result.clone(), Duration::from_secs(300))
            .await;
        Ok(result)
    }

    pub async fn get_clients_paginated_cached(
        &self,
        pool: &DbPool,
        page: i64,
        per_page: i64,
        db_id: &str,
        enabled_filter: &str,
    ) -> Result<PaginatedResult<Client>, Error> {
        let cache_key = format!("clients_{}_{}_{}_{}", db_id, page, per_page, enabled_filter);

        if let Some(cached_result) = self.cache.get_clients_paginated(&cache_key).await {
            tracing::debug!("Returning cached clients pagination for key: {}", cache_key);
            return Ok(cached_result);
        }

        tracing::debug!(
            "Cache miss, fetching clients pagination from database for key: {}",
            cache_key
        );
        let result = get_clients_paginated(pool, page, per_page, enabled_filter)?;

        self.cache
            .set_clients_paginated(cache_key, result.clone(), Duration::from_secs(300))
            .await;
        Ok(result)
    }

    pub async fn get_relays_paginated_cached(
        &self,
        pool: &DbPool,
        page: i64,
        per_page: i64,
        db_id: &str,
        enabled_filter: &str,
    ) -> Result<PaginatedResult<Relay>, Error> {
        let cache_key = format!("relays_{}_{}_{}_ {}", db_id, page, per_page, enabled_filter);

        if let Some(cached_result) = self.cache.get_relays_paginated(&cache_key).await {
            tracing::debug!("Returning cached relays pagination for key: {}", cache_key);
            return Ok(cached_result);
        }

        tracing::debug!(
            "Cache miss, fetching relays pagination from database for key: {}",
            cache_key
        );
        let result = get_relays_paginated(pool, page, per_page, enabled_filter)?;

        self.cache
            .set_relays_paginated(cache_key, result.clone(), Duration::from_secs(300))
            .await;
        Ok(result)
    }

    pub async fn get_relocated_paginated_cached(
        &self,
        pool: &DbPool,
        page: i64,
        per_page: i64,
        db_id: &str,
        enabled_filter: &str,
    ) -> Result<PaginatedResult<Relocated>, Error> {
        let cache_key = format!(
            "relocated_{}_{}_{}_{}",
            db_id, page, per_page, enabled_filter
        );

        if let Some(cached_result) = self.cache.get_relocated_paginated(&cache_key).await {
            tracing::debug!(
                "Returning cached relocated pagination for key: {}",
                cache_key
            );
            return Ok(cached_result);
        }

        tracing::debug!(
            "Cache miss, fetching relocated pagination from database for key: {}",
            cache_key
        );
        let result = get_relocated_paginated(pool, page, per_page, enabled_filter)?;

        self.cache
            .set_relocated_paginated(cache_key, result.clone(), Duration::from_secs(300))
            .await;
        Ok(result)
    }
}

/// Connection pool statistics for monitoring
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub max_size: u32,
    pub size: u32,
    pub available: u32,
    pub in_use: u32,
}

impl PoolStats {
    /// Get the utilization percentage of the pool
    pub fn utilization_percentage(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.in_use as f64 / self.max_size as f64) * 100.0
        }
    }

    /// Check if the pool is under high load (more than 80% utilization)
    pub fn is_under_high_load(&self) -> bool {
        self.utilization_percentage() > 80.0
    }

    /// Check if the pool has available connections
    pub fn has_available_connections(&self) -> bool {
        self.available > 0
    }
}

pub fn get_domains(pool: &DbPool) -> Result<Vec<Domain>, Error> {
    let mut conn = pool.get().map_err(|e| {
        tracing::error!("Failed to get connection from pool: {:?}", e);
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    tracing::debug!("Executing get_domains query");
    let result = domains::table
        .select(Domain::as_select())
        .order(domains::domain.asc())
        .load::<Domain>(&mut conn);

    match &result {
        Ok(domains) => tracing::debug!("Retrieved {} domains", domains.len()),
        Err(e) => tracing::error!("Error retrieving domains: {:?}", e),
    }

    result
}

pub fn get_domain(pool: &DbPool, domain_id: i32) -> Result<Domain, Error> {
    let mut conn = pool.get().map_err(|e| {
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(format!("Failed to get database connection: {e:?}")),
        )
    })?;
    domains::table
        .find(domain_id)
        .select(Domain::as_select())
        .first::<Domain>(&mut conn)
}

pub fn get_domain_by_name(pool: &DbPool, domain_name: &str) -> Result<Domain, Error> {
    let mut conn = pool.get().map_err(|e| {
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(format!("Failed to get database connection: {e:?}")),
        )
    })?;
    domains::table
        .filter(domains::domain.eq(domain_name))
        .select(Domain::as_select())
        .first::<Domain>(&mut conn)
}

pub fn create_domain(pool: &DbPool, new_domain: NewDomain) -> Result<Domain, Error> {
    let mut conn = pool.get().map_err(|e| {
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(format!("Failed to get database connection: {e:?}")),
        )
    })?;
    let now = Utc::now().naive_utc();

    diesel::insert_into(domains::table)
        .values((
            domains::domain.eq(new_domain.domain),
            domains::transport.eq(new_domain.transport),
            domains::enabled.eq(new_domain.enabled),
            domains::created.eq(now),
            domains::modified.eq(now),
        ))
        .execute(&mut conn)?;

    let domain = domains::table
        .order(domains::pkid.desc())
        .select(Domain::as_select())
        .first::<Domain>(&mut conn)?;

    // Invalidate caches after domain creation
    invalidate_cache_for_data_type("domain");

    Ok(domain)
}

pub fn update_domain(
    pool: &DbPool,
    domain_id: i32,
    domain_data: DomainForm,
) -> Result<Domain, Error> {
    let mut conn = pool.get().unwrap();
    diesel::update(domains::table.find(domain_id))
        .set((
            domains::domain.eq(domain_data.domain),
            domains::transport.eq(domain_data.transport),
            domains::enabled.eq(domain_data.enabled),
            domains::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    let domain = get_domain(pool, domain_id)?;

    // Invalidate caches after domain update
    invalidate_cache_for_data_type("domain");

    Ok(domain)
}

pub fn delete_domain(pool: &DbPool, domain_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    let result = diesel::delete(domains::table.find(domain_id)).execute(&mut conn)?;

    // Invalidate caches after domain deletion
    invalidate_cache_for_data_type("domain");

    Ok(result)
}

pub fn get_users(pool: &DbPool) -> Result<Vec<User>, Error> {
    let mut conn = pool.get().unwrap();
    users::table
        .select(User::as_select())
        .order(users::id.asc())
        .load::<User>(&mut conn)
}

pub fn get_user(pool: &DbPool, user_id: String) -> Result<User, Error> {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().unwrap();
    users
        .filter(id.eq(user_id))
        .select(User::as_select())
        .first::<User>(&mut conn)
}

pub fn get_user_by_id(pool: &DbPool, user_id: &str) -> Result<User, Error> {
    let mut conn = pool.get().unwrap();
    users::table
        .filter(users::id.eq(user_id))
        .select(User::as_select())
        .first::<User>(&mut conn)
}

pub fn create_user(pool: &DbPool, user_data: UserForm) -> Result<User, Error> {
    let mut conn = pool.get().unwrap();

    // Hash the password
    let hashed_password = bcrypt::hash(user_data.password.as_bytes(), bcrypt::DEFAULT_COST)
        .map_err(|e| {
            Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

    let maildir = format!("{}/", user_data.id);

    let new_user = NewUser {
        id: user_data.id,
        crypt: hashed_password,
        name: user_data.name,
        maildir,
        home: "/var/spool/mail/virtual".to_string(),
        uid: 5000,
        gid: 5000,
        enabled: user_data.enabled,
        change_password: false,
    };

    let now = Utc::now().naive_utc();

    diesel::insert_into(users::table)
        .values((
            users::id.eq(new_user.id),
            users::crypt.eq(new_user.crypt),
            users::name.eq(new_user.name),
            users::maildir.eq(new_user.maildir),
            users::home.eq(new_user.home),
            users::uid.eq(new_user.uid),
            users::gid.eq(new_user.gid),
            users::enabled.eq(new_user.enabled),
            users::change_password.eq(new_user.change_password),
            users::created.eq(now),
            users::modified.eq(now),
        ))
        .execute(&mut conn)?;

    let user = users::table
        .order(users::id.desc())
        .select(User::as_select())
        .first::<User>(&mut conn)?;

    // Invalidate caches after user creation
    invalidate_cache_for_data_type("user");

    Ok(user)
}

pub fn update_user(pool: &DbPool, user_id: String, user_data: UserForm) -> Result<User, Error> {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().unwrap();

    // First get the current user to preserve unchanged fields
    let _current_user = get_user_by_id(pool, &user_id)?;

    // Update the user - include id if it's different from the current one
    if user_data.id != user_id {
        let new_id = user_data.id.clone();
        diesel::update(users.filter(id.eq(&user_id)))
            .set((
                id.eq(new_id),
                name.eq(user_data.name),
                enabled.eq(user_data.enabled),
                change_password.eq(user_data.change_password),
                modified.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut conn)?;
    } else {
        diesel::update(users.filter(id.eq(&user_id)))
            .set((
                name.eq(user_data.name),
                enabled.eq(user_data.enabled),
                change_password.eq(user_data.change_password),
                modified.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut conn)?;
    }

    // Return the updated user using the new ID if it changed
    let final_user_id = if user_data.id != user_id {
        user_data.id
    } else {
        user_id
    };
    let user = get_user(pool, final_user_id)?;

    // Invalidate caches after user update
    invalidate_cache_for_data_type("user");

    Ok(user)
}

pub fn update_user_password(
    pool: &DbPool,
    user_id: String,
    new_password: &str,
) -> Result<(), Error> {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().unwrap();

    // Hash the new password
    let hashed_password = bcrypt::hash(new_password, bcrypt::DEFAULT_COST).map_err(|e| {
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    // Update the password
    diesel::update(users.filter(id.eq(user_id)))
        .set((
            crypt.eq(hashed_password),
            modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    Ok(())
}

pub fn delete_user(pool: &DbPool, user_id: String) -> Result<usize, Error> {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().unwrap();

    let result = diesel::delete(users.filter(id.eq(user_id))).execute(&mut conn)?;

    // Invalidate caches after user deletion
    invalidate_cache_for_data_type("user");

    Ok(result)
}

pub fn get_aliases(pool: &DbPool) -> Result<Vec<Alias>, Error> {
    let mut conn = pool.get().unwrap();
    aliases::table
        .select(Alias::as_select())
        .order(aliases::mail.asc())
        .load::<Alias>(&mut conn)
}

pub fn get_alias(pool: &DbPool, alias_id: i32) -> Result<Alias, Error> {
    let mut conn = pool.get().unwrap();
    aliases::table
        .find(alias_id)
        .select(Alias::as_select())
        .first::<Alias>(&mut conn)
}

pub fn create_alias(pool: &DbPool, alias_data: AliasForm) -> Result<Alias, Error> {
    let mut conn = pool.get().unwrap();

    let now = Utc::now().naive_utc();

    diesel::insert_into(aliases::table)
        .values((
            aliases::mail.eq(alias_data.mail),
            aliases::destination.eq(alias_data.destination),
            aliases::enabled.eq(alias_data.enabled),
            aliases::created.eq(now),
            aliases::modified.eq(now),
        ))
        .execute(&mut conn)?;

    let alias = aliases::table
        .order(aliases::pkid.desc())
        .select(Alias::as_select())
        .first::<Alias>(&mut conn)?;

    // Invalidate caches after alias creation
    invalidate_cache_for_data_type("alias");

    Ok(alias)
}

pub fn update_alias(pool: &DbPool, alias_id: i32, alias_data: AliasForm) -> Result<Alias, Error> {
    let mut conn = pool.get().unwrap();
    diesel::update(aliases::table.find(alias_id))
        .set((
            aliases::mail.eq(alias_data.mail),
            aliases::destination.eq(alias_data.destination),
            aliases::enabled.eq(alias_data.enabled),
            aliases::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    get_alias(pool, alias_id)
}

pub fn delete_alias(pool: &DbPool, alias_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    let result = diesel::delete(aliases::table.find(alias_id)).execute(&mut conn)?;

    // Invalidate caches after alias deletion
    invalidate_cache_for_data_type("alias");

    Ok(result)
}

// Toggle functions for enable/disable functionality
pub fn toggle_domain_enabled(pool: &DbPool, domain_id: i32) -> Result<Domain, Error> {
    let mut conn = pool.get().unwrap();

    // First get the current domain to check its enabled status
    let current_domain = get_domain(pool, domain_id)?;
    let new_enabled_status = !current_domain.enabled;

    diesel::update(domains::table.find(domain_id))
        .set((
            domains::enabled.eq(new_enabled_status),
            domains::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    let domain = get_domain(pool, domain_id)?;

    // Invalidate caches after domain toggle
    invalidate_cache_for_data_type("domain");

    Ok(domain)
}

pub fn toggle_user_enabled(pool: &DbPool, user_id: String) -> Result<User, Error> {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().unwrap();

    // Get current user
    let current_user = get_user(pool, user_id.clone())?;

    // Toggle the enabled status
    diesel::update(users.filter(id.eq(user_id.clone())))
        .set((
            enabled.eq(!current_user.enabled),
            modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    // Return the updated user
    let user = get_user(pool, user_id)?;

    // Invalidate caches after user toggle
    invalidate_cache_for_data_type("user");

    Ok(user)
}

pub fn toggle_alias_enabled(pool: &DbPool, alias_id: i32) -> Result<Alias, Error> {
    let mut conn = pool.get().unwrap();

    // First get the current alias to check its enabled status
    let current_alias = get_alias(pool, alias_id)?;
    let new_enabled_status = !current_alias.enabled;

    diesel::update(aliases::table.find(alias_id))
        .set((
            aliases::enabled.eq(new_enabled_status),
            aliases::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    let alias = get_alias(pool, alias_id)?;

    // Invalidate caches after alias toggle
    invalidate_cache_for_data_type("alias");

    Ok(alias)
}

// Statistics functions
pub fn get_system_stats(pool: &DbPool) -> Result<SystemStats, Error> {
    use chrono::Duration;
    use chrono::Utc;
    let mut conn = pool.get().map_err(|e| {
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(format!("Failed to get database connection: {e:?}")),
        )
    })?;
    let now = Utc::now().naive_utc();
    let week_ago = now - Duration::days(7);

    tracing::debug!(
        "Getting system stats - now: {}, week_ago: {}",
        now,
        week_ago
    );

    // Optimized queries - still using individual queries but with better performance
    // Domains - use parallel execution where possible
    let total_domains: i64 = domains::table.count().get_result(&mut conn)?;
    let enabled_domains: i64 = domains::table
        .filter(domains::enabled.eq(true))
        .count()
        .get_result(&mut conn)?;
    let disabled_domains: i64 = domains::table
        .filter(domains::enabled.eq(false))
        .count()
        .get_result(&mut conn)?;
    let recent_domains: i64 = domains::table
        .filter(domains::created.is_not_null())
        .filter(domains::created.gt(Some(week_ago)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    // Users
    let total_users: i64 = users::table.count().get_result(&mut conn)?;
    let enabled_users: i64 = users::table
        .filter(users::enabled.eq(true))
        .count()
        .get_result(&mut conn)?;
    let disabled_users: i64 = users::table
        .filter(users::enabled.eq(false))
        .count()
        .get_result(&mut conn)?;
    let recent_users: i64 = users::table
        .filter(users::created.is_not_null())
        .filter(users::created.gt(Some(week_ago)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    // Aliases
    let total_aliases: i64 = aliases::table.count().get_result(&mut conn)?;
    let enabled_aliases: i64 = aliases::table
        .filter(aliases::enabled.eq(true))
        .count()
        .get_result(&mut conn)?;
    let disabled_aliases: i64 = aliases::table
        .filter(aliases::enabled.eq(false))
        .count()
        .get_result(&mut conn)?;
    let recent_aliases: i64 = aliases::table
        .filter(aliases::created.is_not_null())
        .filter(aliases::created.gt(Some(week_ago)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    // Backups
    let total_backups: i64 = backups::table.count().get_result(&mut conn)?;
    let enabled_backups: i64 = backups::table
        .filter(backups::enabled.eq(true))
        .count()
        .get_result(&mut conn)?;
    let disabled_backups: i64 = backups::table
        .filter(backups::enabled.eq(false))
        .count()
        .get_result(&mut conn)?;
    let recent_backups: i64 = backups::table
        .filter(backups::created.is_not_null())
        .filter(backups::created.gt(Some(week_ago)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    // Relays (optional table)
    let (total_relays, enabled_relays, disabled_relays, recent_relays) =
        if relays_table_exists(pool) {
            let total: i64 = relays::table.count().get_result(&mut conn)?;
            let enabled: i64 = relays::table
                .filter(relays::enabled.eq(true))
                .count()
                .get_result(&mut conn)?;
            let disabled: i64 = relays::table
                .filter(relays::enabled.eq(false))
                .count()
                .get_result(&mut conn)?;
            let recent: i64 = relays::table
                .filter(relays::created.is_not_null())
                .filter(relays::created.gt(Some(week_ago)))
                .count()
                .get_result(&mut conn)
                .unwrap_or(0);
            (total, enabled, disabled, recent)
        } else {
            (0, 0, 0, 0) // Table doesn't exist
        };

    // Relocated (optional table)
    let (total_relocated, enabled_relocated, disabled_relocated, recent_relocated) =
        if relocated_table_exists(pool) {
            let total: i64 = relocated::table.count().get_result(&mut conn)?;
            let enabled: i64 = relocated::table
                .filter(relocated::enabled.eq(true))
                .count()
                .get_result(&mut conn)?;
            let disabled: i64 = relocated::table
                .filter(relocated::enabled.eq(false))
                .count()
                .get_result(&mut conn)?;
            let recent: i64 = relocated::table
                .filter(relocated::created.is_not_null())
                .filter(relocated::created.gt(Some(week_ago)))
                .count()
                .get_result(&mut conn)
                .unwrap_or(0);
            (total, enabled, disabled, recent)
        } else {
            (0, 0, 0, 0) // Table doesn't exist
        };

    // Clients (optional table)
    let (total_clients, enabled_clients, disabled_clients, recent_clients) =
        if clients_table_exists(pool) {
            let total: i64 = clients::table.count().get_result(&mut conn)?;
            let enabled: i64 = clients::table
                .filter(clients::enabled.eq(true))
                .count()
                .get_result(&mut conn)?;
            let disabled: i64 = clients::table
                .filter(clients::enabled.eq(false))
                .count()
                .get_result(&mut conn)?;
            let recent: i64 = clients::table
                .filter(clients::created_at.is_not_null())
                .filter(clients::created_at.gt(Some(week_ago)))
                .count()
                .get_result(&mut conn)
                .unwrap_or(0);
            (total, enabled, disabled, recent)
        } else {
            (0, 0, 0, 0) // Table doesn't exist
        };

    // Quota (still 0, as not implemented)
    let total_quota: i64 = 0;
    let used_quota: i64 = 0;
    let quota_usage_percent: f64 = 0.0;

    // Combined enabled stats for dashboard
    let enabled_domains_and_backups = enabled_domains + enabled_backups;

    Ok(SystemStats {
        total_domains,
        enabled_domains,
        disabled_domains,
        recent_domains,
        total_users,
        enabled_users,
        disabled_users,
        recent_users,
        total_aliases,
        enabled_aliases,
        disabled_aliases,
        recent_aliases,
        total_backups,
        enabled_backups,
        disabled_backups,
        recent_backups,
        total_relays,
        enabled_relays,
        disabled_relays,
        recent_relays,
        total_relocated,
        enabled_relocated,
        disabled_relocated,
        recent_relocated,
        total_clients,
        enabled_clients,
        disabled_clients,
        recent_clients,
        total_quota,
        used_quota,
        quota_usage_percent,
        enabled_domains_and_backups,
    })
}

pub fn get_domain_stats(pool: &DbPool) -> Result<Vec<DomainStats>, Error> {
    let mut conn = pool.get().unwrap();

    // This is a simplified version - in a real implementation you'd want to use proper SQL aggregation
    let domains = get_domains(pool)?;
    let mut stats = Vec::new();

    for domain in domains {
        // Count aliases for this domain by checking the domain part of the mail field
        let alias_count: i64 = aliases::table
            .filter(aliases::mail.like(format!("%@{}", domain.domain)))
            .count()
            .get_result(&mut conn)?;

        // Count users for this domain by checking the domain part of the id field
        let user_count: i64 = users::table
            .filter(users::id.like(format!("%@{}", domain.domain)))
            .count()
            .get_result(&mut conn)?;

        // Count relays for this domain by checking the domain part of the recipient field
        let relay_count: i64 = relays::table
            .filter(relays::recipient.like(format!("%@{}", domain.domain)))
            .count()
            .get_result(&mut conn)?;

        let relay_enabled_count: i64 = relays::table
            .filter(relays::recipient.like(format!("%@{}", domain.domain)))
            .filter(relays::enabled.eq(true))
            .count()
            .get_result(&mut conn)?;

        let relay_disabled_count: i64 = relay_count - relay_enabled_count;

        // Count relocated for this domain by checking the domain part of the old_address field
        let relocated_count: i64 = relocated::table
            .filter(relocated::old_address.like(format!("%@{}", domain.domain)))
            .count()
            .get_result(&mut conn)?;

        let relocated_enabled_count: i64 = relocated::table
            .filter(relocated::old_address.like(format!("%@{}", domain.domain)))
            .filter(relocated::enabled.eq(true))
            .count()
            .get_result(&mut conn)?;

        let relocated_disabled_count: i64 = relocated_count - relocated_enabled_count;

        stats.push(DomainStats {
            domain_id: domain.pkid,
            domain: domain.domain,
            user_count,
            alias_count,
            relay_count,
            relay_enabled_count,
            relay_disabled_count,
            relocated_count,
            relocated_enabled_count,
            relocated_disabled_count,
        });
    }

    Ok(stats)
}

// Cross-database domain information
pub async fn get_cross_database_domain_info(
    db_manager: &crate::DatabaseManager,
    domain_name: &str,
) -> Result<Vec<crate::models::CrossDatabaseDomainInfo>, Box<dyn std::error::Error>> {
    let configs = db_manager.get_configs();
    let mut cross_db_info = Vec::new();

    for config in configs {
        if let Some(pool) = db_manager.get_pool(&config.id).await {
            let mut is_primary_domain = false;
            let mut is_backup_domain = false;
            let enabled;
            let user_count;
            let alias_count;

            // Check if domain exists as primary domain
            match get_domain_by_name(&pool, domain_name) {
                Ok(domain) => {
                    is_primary_domain = true;
                    enabled = domain.enabled;
                }
                Err(_) => {
                    // Check if domain exists as backup domain
                    match get_backup_by_name(&pool, domain_name) {
                        Ok(backup) => {
                            is_backup_domain = true;
                            enabled = backup.enabled;
                        }
                        Err(_) => {
                            // Domain doesn't exist in this database
                            continue;
                        }
                    }
                }
            }

            // Get user count for this domain
            let mut conn = pool.get().map_err(|e| {
                tracing::error!("Failed to get connection from pool: {:?}", e);
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::Unknown,
                    Box::new(format!("Failed to get database connection: {e:?}")),
                )
            })?;

            user_count = users::table
                .filter(users::id.like(format!("%@{}", domain_name)))
                .count()
                .get_result(&mut conn)
                .unwrap_or(0);

            // Get alias count for this domain
            alias_count = aliases::table
                .filter(aliases::mail.like(format!("%@{}", domain_name)))
                .count()
                .get_result(&mut conn)
                .unwrap_or(0);

            cross_db_info.push(crate::models::CrossDatabaseDomainInfo {
                database_id: config.id.clone(),
                database_label: config.label.clone(),
                is_primary_domain,
                is_backup_domain,
                enabled,
                user_count,
                alias_count,
            });
        }
    }

    Ok(cross_db_info)
}

// Backup functions
pub fn get_backups(pool: &DbPool) -> Result<Vec<Backup>, Error> {
    let mut conn = pool.get().unwrap();
    backups::table
        .select(Backup::as_select())
        .order(backups::domain.asc())
        .load::<Backup>(&mut conn)
}

pub fn get_backups_paginated(
    pool: &DbPool,
    page: i64,
    per_page: i64,
    search: Option<&str>,
    enabled_filter: &str,
    exclude_subdomains: bool,
) -> Result<PaginatedResult<Backup>, Error> {
    let mut conn = pool.get().map_err(|e| {
        tracing::error!("Failed to get connection from pool: {:?}", e);
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    let offset = (page - 1) * per_page;

    // Build query with optional filters
    let mut query = backups::table.into_boxed();

    if let Some(search_term) = search {
        if !search_term.trim().is_empty() {
            let search_pattern = format!("%{}%", search_term);
            query = query.filter(backups::domain.like(search_pattern));
        }
    }

    match enabled_filter {
        "enabled" => {
            query = query.filter(backups::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(backups::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    if exclude_subdomains {
        query = query.filter(backups::domain.not_like("%.%.%"));
    }

    // Get total count with filters
    let total_count: i64 = query.count().get_result(&mut conn)?;

    // Rebuild query for paginated results with same filters
    let mut query = backups::table.into_boxed();

    if let Some(search_term) = search {
        if !search_term.trim().is_empty() {
            let search_pattern = format!("%{}%", search_term);
            query = query.filter(backups::domain.like(search_pattern));
        }
    }

    match enabled_filter {
        "enabled" => {
            query = query.filter(backups::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(backups::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    if exclude_subdomains {
        query = query.filter(backups::domain.not_like("%.%.%"));
    }

    // Get paginated results with filters
    let backups = query
        .select(Backup::as_select())
        .order(backups::domain.asc())
        .limit(per_page)
        .offset(offset)
        .load::<Backup>(&mut conn)?;

    Ok(PaginatedResult::new(backups, total_count, page, per_page))
}

pub fn get_backup(pool: &DbPool, backup_id: i32) -> Result<Backup, Error> {
    let mut conn = pool.get().unwrap();
    backups::table
        .find(backup_id)
        .select(Backup::as_select())
        .first::<Backup>(&mut conn)
}

pub fn get_backup_by_name(pool: &DbPool, backup_name: &str) -> Result<Backup, Error> {
    let mut conn = pool.get().unwrap();
    backups::table
        .filter(backups::domain.eq(backup_name))
        .select(Backup::as_select())
        .first::<Backup>(&mut conn)
}

pub fn create_backup(pool: &DbPool, new_backup: NewBackup) -> Result<Backup, Error> {
    let mut conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();

    diesel::insert_into(backups::table)
        .values((
            backups::domain.eq(new_backup.domain),
            backups::transport.eq(new_backup.transport),
            backups::enabled.eq(new_backup.enabled),
            backups::created.eq(now),
            backups::modified.eq(now),
        ))
        .execute(&mut conn)?;

    let backup = backups::table
        .order(backups::pkid.desc())
        .select(Backup::as_select())
        .first::<Backup>(&mut conn)?;

    // Invalidate caches after backup creation
    invalidate_cache_for_data_type("backup");

    Ok(backup)
}

pub fn update_backup(
    pool: &DbPool,
    backup_id: i32,
    backup_data: BackupForm,
) -> Result<Backup, Error> {
    let mut conn = pool.get().unwrap();
    diesel::update(backups::table.find(backup_id))
        .set((
            backups::domain.eq(backup_data.domain),
            backups::transport.eq(backup_data.transport),
            backups::enabled.eq(backup_data.enabled),
            backups::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    get_backup(pool, backup_id)
}

pub fn delete_backup(pool: &DbPool, backup_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    let result = diesel::delete(backups::table.find(backup_id)).execute(&mut conn)?;

    // Invalidate caches after backup deletion
    invalidate_cache_for_data_type("backup");

    Ok(result)
}

pub fn toggle_backup_enabled(pool: &DbPool, backup_id: i32) -> Result<Backup, Error> {
    let mut conn = pool.get().unwrap();

    // First get the current backup to check its enabled status
    let current_backup = get_backup(pool, backup_id)?;
    let new_enabled_status = !current_backup.enabled;

    diesel::update(backups::table.find(backup_id))
        .set((
            backups::enabled.eq(new_enabled_status),
            backups::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    let backup = get_backup(pool, backup_id)?;

    // Invalidate caches after backup toggle
    invalidate_cache_for_data_type("backup");

    Ok(backup)
}

/// Convert a primary domain to a backup domain
/// This moves the domain from the domains table to the backups table
pub fn convert_domain_to_backup(pool: &DbPool, domain_id: i32) -> Result<Backup, Error> {
    let mut conn = pool.get().map_err(|e| {
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(format!("Failed to get database connection: {e:?}")),
        )
    })?;

    // Get the existing domain
    let domain = get_domain(pool, domain_id)?;

    // Insert into backups table
    diesel::insert_into(backups::table)
        .values((
            backups::domain.eq(&domain.domain),
            backups::transport.eq(&domain.transport),
            backups::enabled.eq(domain.enabled),
            backups::created.eq(domain.created),
            backups::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    // Get the newly created backup
    let backup = get_backup_by_name(pool, &domain.domain)?;

    // Delete from domains table
    diesel::delete(domains::table.find(domain_id)).execute(&mut conn)?;

    // Invalidate caches
    invalidate_cache_for_data_type("domain");
    invalidate_cache_for_data_type("backup");

    Ok(backup)
}

/// Convert a backup domain to a primary domain
/// This moves the domain from the backups table to the domains table
pub fn convert_backup_to_domain(pool: &DbPool, backup_id: i32) -> Result<Domain, Error> {
    let mut conn = pool.get().map_err(|e| {
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(format!("Failed to get database connection: {e:?}")),
        )
    })?;

    // Get the existing backup
    let backup = get_backup(pool, backup_id)?;

    // Insert into domains table
    diesel::insert_into(domains::table)
        .values((
            domains::domain.eq(&backup.domain),
            domains::transport.eq(&backup.transport),
            domains::enabled.eq(backup.enabled),
            domains::created.eq(backup.created),
            domains::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    // Get the newly created domain
    let domain = get_domain_by_name(pool, &backup.domain)?;

    // Delete from backups table
    diesel::delete(backups::table.find(backup_id)).execute(&mut conn)?;

    // Invalidate caches
    invalidate_cache_for_data_type("domain");
    invalidate_cache_for_data_type("backup");

    Ok(domain)
}

// Relay database functions
pub fn get_relays(pool: &DbPool) -> Result<Vec<Relay>, Error> {
    let mut conn = pool.get().unwrap();
    relays::table
        .select(Relay::as_select())
        .order(relays::recipient.asc())
        .load::<Relay>(&mut conn)
}

pub fn get_relay(pool: &DbPool, relay_id: i32) -> Result<Relay, Error> {
    let mut conn = pool.get().unwrap();
    relays::table
        .find(relay_id)
        .select(Relay::as_select())
        .first::<Relay>(&mut conn)
}

pub fn get_relay_by_recipient(pool: &DbPool, recipient: &str) -> Result<Relay, Error> {
    let mut conn = pool.get().unwrap();
    relays::table
        .filter(relays::recipient.eq(recipient))
        .select(Relay::as_select())
        .first::<Relay>(&mut conn)
}

pub fn get_relays_for_domain(pool: &DbPool, domain: &str) -> Result<Vec<Relay>, Error> {
    let mut conn = pool.get().unwrap();
    relays::table
        .filter(relays::recipient.like(format!("%@{}", domain)))
        .select(Relay::as_select())
        .order(relays::recipient.asc())
        .load::<Relay>(&mut conn)
}

pub fn create_relay(pool: &DbPool, relay_data: RelayForm) -> Result<Relay, Error> {
    let mut conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();

    let new_relay = NewRelay {
        recipient: relay_data.recipient,
        status: relay_data.status,
        enabled: relay_data.enabled,
    };

    diesel::insert_into(relays::table)
        .values((
            relays::recipient.eq(new_relay.recipient),
            relays::status.eq(new_relay.status),
            relays::enabled.eq(new_relay.enabled),
            relays::created.eq(now),
            relays::modified.eq(now),
        ))
        .execute(&mut conn)?;

    relays::table
        .order(relays::pkid.desc())
        .select(Relay::as_select())
        .first::<Relay>(&mut conn)
}

pub fn update_relay(pool: &DbPool, relay_id: i32, relay_data: RelayForm) -> Result<Relay, Error> {
    let mut conn = pool.get().unwrap();
    diesel::update(relays::table.find(relay_id))
        .set((
            relays::recipient.eq(relay_data.recipient),
            relays::status.eq(relay_data.status),
            relays::enabled.eq(relay_data.enabled),
            relays::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    get_relay(pool, relay_id)
}

pub fn delete_relay(pool: &DbPool, relay_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    diesel::delete(relays::table.find(relay_id)).execute(&mut conn)
}

pub fn toggle_relay_enabled(pool: &DbPool, relay_id: i32) -> Result<Relay, Error> {
    let mut conn = pool.get().unwrap();

    // Get current relay
    let current_relay = get_relay(pool, relay_id)?;

    // Toggle enabled status
    diesel::update(relays::table.find(relay_id))
        .set((
            relays::enabled.eq(!current_relay.enabled),
            relays::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    get_relay(pool, relay_id)
}

// Relocated functions
pub fn get_relocated(pool: &DbPool) -> Result<Vec<Relocated>, Error> {
    let mut conn = pool.get().unwrap();
    relocated::table
        .select(Relocated::as_select())
        .order(relocated::old_address.asc())
        .load::<Relocated>(&mut conn)
}

pub fn get_relocated_by_id(pool: &DbPool, relocated_id: i32) -> Result<Relocated, Error> {
    let mut conn = pool.get().unwrap();
    relocated::table
        .find(relocated_id)
        .select(Relocated::as_select())
        .first::<Relocated>(&mut conn)
}

pub fn get_relocated_by_old_address(pool: &DbPool, old_address: &str) -> Result<Relocated, Error> {
    let mut conn = pool.get().unwrap();
    relocated::table
        .filter(relocated::old_address.eq(old_address))
        .select(Relocated::as_select())
        .first::<Relocated>(&mut conn)
}

pub fn create_relocated(pool: &DbPool, relocated_data: RelocatedForm) -> Result<Relocated, Error> {
    let mut conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();

    let new_relocated = NewRelocated {
        old_address: relocated_data.old_address,
        new_address: relocated_data.new_address,
        enabled: relocated_data.enabled,
    };

    diesel::insert_into(relocated::table)
        .values((
            relocated::old_address.eq(new_relocated.old_address),
            relocated::new_address.eq(new_relocated.new_address),
            relocated::enabled.eq(new_relocated.enabled),
            relocated::created.eq(now),
            relocated::modified.eq(now),
        ))
        .execute(&mut conn)?;

    relocated::table
        .order(relocated::pkid.desc())
        .select(Relocated::as_select())
        .first::<Relocated>(&mut conn)
}

pub fn update_relocated(
    pool: &DbPool,
    relocated_id: i32,
    relocated_data: RelocatedForm,
) -> Result<Relocated, Error> {
    let mut conn = pool.get().unwrap();
    diesel::update(relocated::table.find(relocated_id))
        .set((
            relocated::old_address.eq(relocated_data.old_address),
            relocated::new_address.eq(relocated_data.new_address),
            relocated::enabled.eq(relocated_data.enabled),
            relocated::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    get_relocated_by_id(pool, relocated_id)
}

pub fn delete_relocated(pool: &DbPool, relocated_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    diesel::delete(relocated::table.find(relocated_id)).execute(&mut conn)
}

pub fn toggle_relocated_enabled(pool: &DbPool, relocated_id: i32) -> Result<Relocated, Error> {
    let mut conn = pool.get().unwrap();

    // Get current relocated
    let current_relocated = get_relocated_by_id(pool, relocated_id)?;

    // Toggle enabled status
    diesel::update(relocated::table.find(relocated_id))
        .set((
            relocated::enabled.eq(!current_relocated.enabled),
            relocated::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    get_relocated_by_id(pool, relocated_id)
}

// Catch-all report functions
pub fn get_catch_all_report(pool: &DbPool) -> Result<Vec<CatchAllReport>, Error> {
    let mut conn = pool.get().unwrap();

    // Get all domains that have catch-all aliases (@domain.com)
    let catch_all_aliases = aliases::table
        .filter(aliases::mail.like("@%"))
        .filter(aliases::enabled.eq(true))
        .select(Alias::as_select())
        .load::<Alias>(&mut conn)?;

    let mut reports = Vec::new();

    for catch_all_alias in catch_all_aliases {
        let domain = catch_all_alias.domain();

        // Get all other aliases for this domain (excluding the catch-all)
        let required_aliases = aliases::table
            .filter(aliases::mail.like(format!("%@{domain}")))
            .filter(aliases::mail.ne(&catch_all_alias.mail))
            .filter(aliases::enabled.eq(true))
            .select(Alias::as_select())
            .load::<Alias>(&mut conn)?;

        let required_aliases: Vec<RequiredAlias> = required_aliases
            .into_iter()
            .map(|alias| RequiredAlias {
                mail: alias.mail,
                destination: alias.destination,
                enabled: alias.enabled,
            })
            .collect();

        reports.push(CatchAllReport {
            domain,
            catch_all_alias: catch_all_alias.mail,
            catch_all_destination: catch_all_alias.destination,
            required_aliases,
        });
    }

    Ok(reports)
}

// Helper function to load configuration with fallback
fn load_config_with_fallback() -> crate::config::Config {
    match crate::config::Config::load() {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("Failed to load config, using defaults: {:?}", e);
            crate::config::Config::default()
        }
    }
}

// Helper function to get catch-all alias for a domain
fn get_catch_all_alias_for_domain(
    conn: &mut diesel::MysqlConnection,
    domain: &str,
) -> Result<Option<Alias>, Error> {
    aliases::table
        .filter(aliases::mail.eq(format!("@{}", domain)))
        .filter(aliases::enabled.eq(true))
        .select(Alias::as_select())
        .first::<Alias>(conn)
        .optional()
}

// Helper function to get all aliases for a domain
fn get_domain_aliases(
    conn: &mut diesel::MysqlConnection,
    domain: &str,
) -> Result<Vec<Alias>, Error> {
    aliases::table
        .filter(aliases::mail.like(format!("%@{}", domain)))
        .filter(aliases::enabled.eq(true))
        .select(Alias::as_select())
        .load::<Alias>(conn)
}

// Helper function to convert aliases to RequiredAlias format
fn convert_aliases_to_required(aliases: &[Alias]) -> Vec<RequiredAlias> {
    aliases
        .iter()
        .map(|alias| RequiredAlias {
            mail: alias.mail.clone(),
            destination: alias.destination.clone(),
            enabled: alias.enabled,
        })
        .collect()
}

// Helper function to find missing aliases
fn find_missing_aliases(
    domain_aliases: &[Alias],
    domain_required_aliases: &[String],
    domain_common_aliases: &[String],
) -> (Vec<String>, Vec<String>) {
    let existing_aliases: std::collections::HashSet<String> = domain_aliases
        .iter()
        .map(|alias| alias.mail.split('@').next().unwrap_or("").to_string())
        .collect();

    let mut missing_required = domain_required_aliases
        .iter()
        .filter(|required| !existing_aliases.contains(*required))
        .cloned()
        .collect::<Vec<String>>();
    missing_required.sort();

    let mut missing_common = domain_common_aliases
        .iter()
        .filter(|common| !existing_aliases.contains(*common))
        .cloned()
        .collect::<Vec<String>>();
    missing_common.sort();

    (missing_required, missing_common)
}

// Helper function to create domain alias report
fn create_domain_alias_report(
    domain: Domain,
    catch_all_alias: Option<Alias>,
    domain_aliases: Vec<Alias>,
    config: &crate::config::Config,
) -> DomainAliasReport {
    let required_aliases = convert_aliases_to_required(&domain_aliases);

    let domain_required_aliases = config.get_required_aliases_for_domain(&domain.domain);
    let domain_common_aliases = config.get_common_aliases_for_domain(&domain.domain);

    // Find missing required aliases only if there's no catch-all
    let (missing_required_aliases, missing_common_aliases) = if catch_all_alias.is_none() {
        find_missing_aliases(
            &domain_aliases,
            &domain_required_aliases,
            &domain_common_aliases,
        )
    } else {
        (Vec::new(), Vec::new())
    };

    DomainAliasReport {
        domain: domain.domain,
        has_catch_all: catch_all_alias.is_some(),
        catch_all_alias: catch_all_alias.as_ref().map(|ca| ca.mail.clone()),
        catch_all_destination: catch_all_alias.as_ref().map(|ca| ca.destination.clone()),
        required_aliases,
        missing_required_aliases,
        missing_common_aliases,
        disabled_required_aliases: Vec::new(),
        disabled_common_aliases: Vec::new(),
        disabled_catch_all: None,
    }
}

// Enhanced alias report functions
pub fn get_alias_report(pool: &DbPool) -> Result<AliasReport, Error> {
    let mut conn = pool.get().unwrap();
    let config = load_config_with_fallback();
    let domains = get_domains(pool)?;

    let mut domains_with_catch_all = Vec::new();
    let mut domains_without_catch_all = Vec::new();

    for domain in domains {
        let catch_all_alias = get_catch_all_alias_for_domain(&mut conn, &domain.domain)?;
        let domain_aliases = get_domain_aliases(&mut conn, &domain.domain)?;

        let domain_report =
            create_domain_alias_report(domain, catch_all_alias, domain_aliases, &config);

        if domain_report.has_catch_all {
            domains_with_catch_all.push(domain_report);
        } else {
            domains_without_catch_all.push(domain_report);
        }
    }

    Ok(AliasReport {
        domains_with_catch_all,
        domains_without_catch_all,
    })
}

// Matrix report functions
pub fn get_domain_alias_matrix_report(pool: &DbPool) -> Result<DomainAliasMatrixReport, Error> {
    let mut conn = pool.get().unwrap();

    // Load configuration
    let config = match crate::config::Config::load() {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("Failed to load config, using defaults: {:?}", e);
            crate::config::Config::default()
        }
    };

    // Get all domains
    let domains = get_domains(pool)?;
    let mut matrix_rows = Vec::new();

    for domain in domains {
        // Check catch-all status
        let catch_all_alias = aliases::table
            .filter(aliases::mail.eq(format!("@{}", domain.domain)))
            .select(Alias::as_select())
            .first::<Alias>(&mut conn)
            .optional()?;

        let catch_all_status = match catch_all_alias {
            Some(alias) if alias.enabled => AliasStatus::Present,
            Some(_) => AliasStatus::Disabled,
            None => AliasStatus::Missing,
        };

        // Get all aliases for this domain
        let domain_aliases = aliases::table
            .filter(aliases::mail.like(format!("%@{}", domain.domain)))
            .select(Alias::as_select())
            .load::<Alias>(&mut conn)?;

        // Create a map of existing aliases for quick lookup
        let existing_aliases: std::collections::HashMap<String, bool> = domain_aliases
            .iter()
            .map(|alias| {
                let local_part = alias.mail.split('@').next().unwrap_or("").to_string();
                (local_part, alias.enabled)
            })
            .collect();

        // Check required aliases
        let required_aliases = config.get_required_aliases_for_domain(&domain.domain);
        let required_matrix_items: Vec<RequiredAliasMatrixItem> = required_aliases
            .iter()
            .map(|alias| {
                let status = match existing_aliases.get(alias) {
                    Some(&enabled) if enabled => AliasStatus::Present,
                    Some(_) => AliasStatus::Disabled,
                    None => AliasStatus::Missing,
                };
                RequiredAliasMatrixItem {
                    alias: alias.clone(),
                    status,
                }
            })
            .collect();

        matrix_rows.push(DomainAliasMatrixRow {
            id: domain.pkid,
            domain: domain.domain,
            catch_all_status,
            required_aliases: required_matrix_items,
        });
    }

    // Get the list of required aliases for the header
    let required_aliases_list = config.get_required_aliases_for_domain(""); // Get global required aliases

    Ok(DomainAliasMatrixReport {
        domains: matrix_rows,
        required_aliases_list,
    })
}

// Get alias report for a specific domain
pub fn get_domain_alias_report(
    pool: &DbPool,
    domain_name: &str,
) -> Result<DomainAliasReport, Error> {
    let mut conn = pool.get().unwrap();

    // Load configuration
    let config = match crate::config::Config::load() {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("Failed to load config, using defaults: {:?}", e);
            crate::config::Config::default()
        }
    };

    // Check if this domain has a catch-all alias (enabled or disabled)
    let catch_all_alias = aliases::table
        .filter(aliases::mail.eq(format!("@{domain_name}")))
        .select(Alias::as_select())
        .first::<Alias>(&mut conn)
        .optional()?;

    // Check if this domain has an enabled catch-all alias
    let enabled_catch_all_alias = aliases::table
        .filter(aliases::mail.eq(format!("@{domain_name}")))
        .filter(aliases::enabled.eq(true))
        .select(Alias::as_select())
        .first::<Alias>(&mut conn)
        .optional()?;

    // Get all enabled aliases for this domain
    let enabled_aliases = aliases::table
        .filter(aliases::mail.like(format!("%@{domain_name}")))
        .filter(aliases::enabled.eq(true))
        .select(Alias::as_select())
        .load::<Alias>(&mut conn)?;

    // Get all disabled aliases for this domain (excluding catch-all)
    let disabled_aliases = aliases::table
        .filter(aliases::mail.like(format!("%@{domain_name}")))
        .filter(aliases::enabled.eq(false))
        .filter(aliases::mail.ne(format!("@{domain_name}"))) // Exclude catch-all
        .select(Alias::as_select())
        .load::<Alias>(&mut conn)?;

    // Convert enabled aliases to RequiredAlias format and sort by mail
    let mut required_aliases: Vec<RequiredAlias> = enabled_aliases
        .iter()
        .map(|alias| RequiredAlias {
            mail: alias.mail.clone(),
            destination: alias.destination.clone(),
            enabled: alias.enabled,
        })
        .collect();
    required_aliases.sort_by(|a, b| a.mail.cmp(&b.mail));

    // Get required aliases for this specific domain
    let domain_required_aliases = config.get_required_aliases_for_domain(domain_name);
    let domain_common_aliases = config.get_common_aliases_for_domain(domain_name);

    // Separate disabled aliases into required and common based on their local part
    let mut disabled_required_aliases: Vec<Alias> = Vec::new();
    let mut disabled_common_aliases: Vec<Alias> = Vec::new();

    for alias in &disabled_aliases {
        let local_part = alias.mail.split('@').next().unwrap_or("").to_string();
        if domain_required_aliases.contains(&local_part) {
            disabled_required_aliases.push(alias.clone());
        } else if domain_common_aliases.contains(&local_part) {
            disabled_common_aliases.push(alias.clone());
        }
    }

    disabled_required_aliases.sort_by(|a, b| a.mail.cmp(&b.mail));
    disabled_common_aliases.sort_by(|a, b| a.mail.cmp(&b.mail));

    // Find missing required aliases only if there's no enabled catch-all
    let (missing_required_aliases, missing_common_aliases) = if enabled_catch_all_alias.is_none() {
        let existing_aliases: std::collections::HashSet<String> = enabled_aliases
            .iter()
            .map(|alias| alias.mail.split('@').next().unwrap_or("").to_string())
            .collect();

        let mut missing_required = domain_required_aliases
            .iter()
            .filter(|required| !existing_aliases.contains(*required))
            .cloned()
            .collect::<Vec<String>>();
        missing_required.sort();

        let mut missing_common = domain_common_aliases
            .iter()
            .filter(|common| !existing_aliases.contains(*common))
            .cloned()
            .collect::<Vec<String>>();
        missing_common.sort();

        (missing_required, missing_common)
    } else {
        (Vec::new(), Vec::new())
    };

    // Determine if there's a disabled catch-all
    let disabled_catch_all = if let Some(ca) = catch_all_alias {
        if !ca.enabled {
            Some(ca)
        } else {
            None
        }
    } else {
        None
    };

    Ok(DomainAliasReport {
        domain: domain_name.to_string(),
        has_catch_all: enabled_catch_all_alias.is_some(),
        catch_all_alias: enabled_catch_all_alias.as_ref().map(|ca| ca.mail.clone()),
        catch_all_destination: enabled_catch_all_alias
            .as_ref()
            .map(|ca| ca.destination.clone()),
        required_aliases,
        missing_required_aliases,
        missing_common_aliases,
        disabled_required_aliases,
        disabled_common_aliases,
        disabled_catch_all,
    })
}

// Client functions
pub fn get_clients(pool: &DbPool) -> Result<Vec<Client>, Error> {
    let mut conn = pool.get().unwrap();
    clients::table
        .select(Client::as_select())
        .order(clients::client.asc())
        .load::<Client>(&mut conn)
}

pub fn get_client(pool: &DbPool, client_id: i32) -> Result<Client, Error> {
    let mut conn = pool.get().unwrap();
    clients::table
        .find(client_id)
        .select(Client::as_select())
        .first::<Client>(&mut conn)
}

pub fn get_client_by_name(pool: &DbPool, client_name: &str) -> Result<Client, Error> {
    let mut conn = pool.get().unwrap();
    clients::table
        .filter(clients::client.eq(client_name))
        .select(Client::as_select())
        .first::<Client>(&mut conn)
}

pub fn create_client(pool: &DbPool, client_data: ClientForm) -> Result<Client, Error> {
    let mut conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();

    diesel::insert_into(clients::table)
        .values((
            clients::client.eq(client_data.client),
            clients::status.eq(client_data.status),
            clients::enabled.eq(client_data.enabled),
            clients::created_at.eq(now),
            clients::updated_at.eq(now),
        ))
        .execute(&mut conn)?;

    clients::table
        .order(clients::id.desc())
        .select(Client::as_select())
        .first::<Client>(&mut conn)
}

pub fn update_client(
    pool: &DbPool,
    client_id: i32,
    client_data: ClientForm,
) -> Result<Client, Error> {
    let mut conn = pool.get().unwrap();

    diesel::update(clients::table.find(client_id))
        .set((
            clients::client.eq(client_data.client),
            clients::status.eq(client_data.status),
            clients::enabled.eq(client_data.enabled),
            clients::updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    get_client(pool, client_id)
}

pub fn delete_client(pool: &DbPool, client_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    diesel::delete(clients::table.find(client_id)).execute(&mut conn)
}

pub fn toggle_client_enabled(pool: &DbPool, client_id: i32) -> Result<Client, Error> {
    let mut conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();

    // First get the current client to check its enabled status
    let current_client = clients::table
        .filter(clients::id.eq(client_id))
        .select(Client::as_select())
        .first::<Client>(&mut conn)?;

    // Toggle the enabled status
    let new_enabled = !current_client.enabled;

    diesel::update(clients::table.filter(clients::id.eq(client_id)))
        .set((
            clients::enabled.eq(new_enabled),
            clients::updated_at.eq(now),
        ))
        .execute(&mut conn)?;

    // Return the updated client
    clients::table
        .filter(clients::id.eq(client_id))
        .select(Client::as_select())
        .first::<Client>(&mut conn)
}

// Function to create multiple aliases for a domain
pub fn create_domain_aliases(
    pool: &DbPool,
    domain: &str,
    aliases: Vec<(String, String)>,
) -> Result<Vec<Alias>, Error> {
    let mut conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();
    let mut created_aliases = Vec::new();

    for (local_part, destination) in aliases {
        let mail = if local_part == "@" {
            format!("@{domain}")
        } else {
            format!("{local_part}@{domain}")
        };

        // Check if alias already exists
        let existing = aliases::table
            .filter(aliases::mail.eq(&mail))
            .select(Alias::as_select())
            .first::<Alias>(&mut conn)
            .optional()?;

        if existing.is_none() {
            diesel::insert_into(aliases::table)
                .values((
                    aliases::mail.eq(&mail),
                    aliases::destination.eq(&destination),
                    aliases::enabled.eq(true),
                    aliases::created.eq(now),
                    aliases::modified.eq(now),
                ))
                .execute(&mut conn)?;

            // Get the created alias
            let created_alias = aliases::table
                .filter(aliases::mail.eq(&mail))
                .select(Alias::as_select())
                .first::<Alias>(&mut conn)?;

            created_aliases.push(created_alias);
        }
    }

    Ok(created_aliases)
}

pub fn get_aliases_for_domain(pool: &DbPool, domain_name: &str) -> Result<Vec<Alias>, Error> {
    let mut conn = pool.get().unwrap();
    aliases::table
        .filter(aliases::mail.like(format!("%@{domain_name}")))
        .select(Alias::as_select())
        .order(aliases::mail.asc())
        .load::<Alias>(&mut conn)
}

/// Get users belonging to a specific domain by matching the user id's domain part
pub fn get_users_for_domain(pool: &DbPool, domain_name: &str) -> Result<Vec<User>, Error> {
    let mut conn = pool.get().unwrap();
    users::table
        .filter(users::id.like(format!("%@{domain_name}")))
        .select(User::as_select())
        .order(users::id.asc())
        .load::<User>(&mut conn)
}

pub fn search_aliases(pool: &DbPool, query: &str, limit: i64) -> Result<Vec<Alias>, Error> {
    let mut conn = pool.get().unwrap();
    let search_pattern = format!("%{query}%");

    aliases::table
        .filter(
            aliases::destination
                .like(&search_pattern)
                .or(aliases::mail.like(&search_pattern)),
        )
        .select(Alias::as_select())
        .order(aliases::destination.asc())
        .limit(limit)
        .load::<Alias>(&mut conn)
}

pub fn search_aliases_by_name(pool: &DbPool, query: &str, limit: i64) -> Result<Vec<Alias>, Error> {
    let mut conn = pool.get().unwrap();
    let search_pattern = format!("{query}%@%");

    aliases::table
        .filter(aliases::mail.like(&search_pattern))
        .select(Alias::as_select())
        .order(aliases::mail.asc())
        .limit(limit)
        .load::<Alias>(&mut conn)
}

pub fn search_domains(pool: &DbPool, query: &str, limit: i64) -> Result<Vec<Domain>, Error> {
    let mut conn = pool.get().unwrap();
    let search_pattern = format!("%{query}%");

    domains::table
        .filter(domains::domain.like(&search_pattern))
        .select(Domain::as_select())
        .order(domains::domain.asc())
        .limit(limit)
        .load::<Domain>(&mut conn)
}

// Paginated functions
pub fn get_domains_paginated(
    pool: &DbPool,
    page: i64,
    per_page: i64,
    search: Option<&str>,
    enabled_filter: &str,
    exclude_subdomains: bool,
) -> Result<PaginatedResult<Domain>, Error> {
    let mut conn = pool.get().map_err(|e| {
        tracing::error!("Failed to get connection from pool: {:?}", e);
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    let offset = (page - 1) * per_page;

    // Build query with optional filters
    let mut query = domains::table.into_boxed();

    if let Some(search_term) = search {
        if !search_term.trim().is_empty() {
            let search_pattern = format!("%{}%", search_term);
            query = query.filter(domains::domain.like(search_pattern));
        }
    }

    match enabled_filter {
        "enabled" => {
            query = query.filter(domains::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(domains::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    if exclude_subdomains {
        query = query.filter(domains::domain.not_like("%.%.%"));
    }

    // Get total count with filters
    let total_count: i64 = query.count().get_result(&mut conn)?;

    // Rebuild query for paginated results with same filters
    let mut query = domains::table.into_boxed();

    if let Some(search_term) = search {
        if !search_term.trim().is_empty() {
            let search_pattern = format!("%{}%", search_term);
            query = query.filter(domains::domain.like(search_pattern));
        }
    }

    match enabled_filter {
        "enabled" => {
            query = query.filter(domains::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(domains::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    if exclude_subdomains {
        query = query.filter(domains::domain.not_like("%.%.%"));
    }

    // Get paginated results with filters
    let domains = query
        .select(Domain::as_select())
        .order(domains::domain.asc())
        .limit(per_page)
        .offset(offset)
        .load::<Domain>(&mut conn)?;

    Ok(PaginatedResult::new(domains, total_count, page, per_page))
}

pub fn get_aliases_paginated(
    pool: &DbPool,
    page: i64,
    per_page: i64,
    sort_by: Option<&str>,
    sort_order: Option<&str>,
    search: Option<&str>,
    enabled_filter: &str,
) -> Result<PaginatedResult<Alias>, Error> {
    let mut conn = pool.get().unwrap();

    let offset = (page - 1) * per_page;

    // Build the query with search filter
    let mut query = aliases::table.select(Alias::as_select()).into_boxed();

    if let Some(search_term) = search {
        if !search_term.trim().is_empty() {
            let search_pattern = format!("%{}%", search_term);
            query = query.filter(
                aliases::mail
                    .like(search_pattern.clone())
                    .or(aliases::destination.like(search_pattern)),
            );
        }
    }

    match enabled_filter {
        "enabled" => {
            query = query.filter(aliases::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(aliases::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    // Get total count with search filter
    let total_count: i64 = query.count().get_result(&mut conn)?;

    // Rebuild query for paginated results
    let mut query = aliases::table.select(Alias::as_select()).into_boxed();

    if let Some(search_term) = search {
        if !search_term.trim().is_empty() {
            let search_pattern = format!("%{}%", search_term);
            query = query.filter(
                aliases::mail
                    .like(search_pattern.clone())
                    .or(aliases::destination.like(search_pattern)),
            );
        }
    }

    match enabled_filter {
        "enabled" => {
            query = query.filter(aliases::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(aliases::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    // Apply sorting
    match sort_by {
        Some("mail") => {
            if sort_order == Some("desc") {
                query = query.order(aliases::mail.desc());
            } else {
                query = query.order(aliases::mail.asc());
            }
        }
        Some("destination") => {
            if sort_order == Some("desc") {
                query = query
                    .order(aliases::destination.desc())
                    .order(aliases::mail.desc());
            } else {
                query = query
                    .order(aliases::destination.asc())
                    .order(aliases::mail.asc());
            }
        }
        Some("domain") => {
            // For domain sorting, we need to sort by the domain part of the mail field
            // Since we can't easily use SQL functions in Diesel, we'll sort by mail
            // and then post-process to group by domain. This is a limitation of the current setup.
            if sort_order == Some("desc") {
                query = query.order(aliases::mail.desc());
            } else {
                query = query.order(aliases::mail.asc());
            }
        }
        _ => {
            // Default sorting by mail ascending
            query = query.order(aliases::mail.asc());
        }
    }

    // Get paginated results
    let mut aliases = query
        .limit(per_page)
        .offset(offset)
        .load::<Alias>(&mut conn)?;

    // Post-process for domain sorting if needed
    if sort_by == Some("domain") {
        aliases.sort_by(|a, b| {
            let domain_a = a.mail.split('@').next_back().unwrap_or("");
            let domain_b = b.mail.split('@').next_back().unwrap_or("");
            let domain_cmp = if sort_order == Some("desc") {
                domain_b.cmp(domain_a)
            } else {
                domain_a.cmp(domain_b)
            };
            // If domains are equal, sort by mail as secondary
            if domain_cmp == std::cmp::Ordering::Equal {
                if sort_order == Some("desc") {
                    b.mail.cmp(&a.mail)
                } else {
                    a.mail.cmp(&b.mail)
                }
            } else {
                domain_cmp
            }
        });
    }

    Ok(PaginatedResult::new(aliases, total_count, page, per_page))
}

pub fn get_users_paginated(
    pool: &DbPool,
    page: i64,
    per_page: i64,
    enabled_filter: &str,
) -> Result<PaginatedResult<User>, Error> {
    let mut conn = pool.get().unwrap();

    let offset = (page - 1) * per_page;

    // Build query with enabled filter
    let mut query = users::table.into_boxed();

    match enabled_filter {
        "enabled" => {
            query = query.filter(users::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(users::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    // Get total count
    let total_count: i64 = query.count().get_result(&mut conn)?;

    // Rebuild query for paginated results
    let mut query = users::table.into_boxed();

    match enabled_filter {
        "enabled" => {
            query = query.filter(users::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(users::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    // Get paginated results
    let users = query
        .select(User::as_select())
        .order(users::id.asc())
        .limit(per_page)
        .offset(offset)
        .load::<User>(&mut conn)?;

    Ok(PaginatedResult::new(users, total_count, page, per_page))
}

pub fn get_clients_paginated(
    pool: &DbPool,
    page: i64,
    per_page: i64,
    enabled_filter: &str,
) -> Result<PaginatedResult<Client>, Error> {
    let mut conn = pool.get().unwrap();

    let offset = (page - 1) * per_page;

    // Build query with enabled filter
    let mut query = clients::table.into_boxed();

    match enabled_filter {
        "enabled" => {
            query = query.filter(clients::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(clients::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    // Get total count
    let total_count: i64 = query.count().get_result(&mut conn)?;

    // Rebuild query for paginated results
    let mut query = clients::table.into_boxed();

    match enabled_filter {
        "enabled" => {
            query = query.filter(clients::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(clients::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    // Get paginated results
    let clients = query
        .select(Client::as_select())
        .order(clients::client.asc())
        .limit(per_page)
        .offset(offset)
        .load::<Client>(&mut conn)?;

    Ok(PaginatedResult::new(clients, total_count, page, per_page))
}

pub fn get_relays_paginated(
    pool: &DbPool,
    page: i64,
    per_page: i64,
    enabled_filter: &str,
) -> Result<PaginatedResult<Relay>, Error> {
    let mut conn = pool.get().unwrap();

    let offset = (page - 1) * per_page;

    // Build query with enabled filter
    let mut query = relays::table.into_boxed();

    match enabled_filter {
        "enabled" => {
            query = query.filter(relays::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(relays::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    // Get total count
    let total_count: i64 = query.count().get_result(&mut conn)?;

    // Rebuild query for paginated results
    let mut query = relays::table.into_boxed();

    match enabled_filter {
        "enabled" => {
            query = query.filter(relays::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(relays::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    // Get paginated results
    let relays = query
        .select(Relay::as_select())
        .order(relays::recipient.asc())
        .limit(per_page)
        .offset(offset)
        .load::<Relay>(&mut conn)?;

    Ok(PaginatedResult::new(relays, total_count, page, per_page))
}

pub fn get_relocated_paginated(
    pool: &DbPool,
    page: i64,
    per_page: i64,
    enabled_filter: &str,
) -> Result<PaginatedResult<Relocated>, Error> {
    let mut conn = pool.get().unwrap();

    let offset = (page - 1) * per_page;

    // Build query with enabled filter
    let mut query = relocated::table.into_boxed();

    match enabled_filter {
        "enabled" => {
            query = query.filter(relocated::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(relocated::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    // Get total count
    let total_count: i64 = query.count().get_result(&mut conn)?;

    // Rebuild query for paginated results
    let mut query = relocated::table.into_boxed();

    match enabled_filter {
        "enabled" => {
            query = query.filter(relocated::enabled.eq(true));
        }
        "disabled" => {
            query = query.filter(relocated::enabled.eq(false));
        }
        _ => {
            // "all" - no filter
        }
    }

    // Get paginated results
    let relocated = query
        .select(Relocated::as_select())
        .order(relocated::old_address.asc())
        .limit(per_page)
        .offset(offset)
        .load::<Relocated>(&mut conn)?;

    Ok(PaginatedResult::new(relocated, total_count, page, per_page))
}

// Helper functions for optional tables
pub fn table_exists(pool: &DbPool, table_name: &str) -> bool {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return false,
    };

    // Try to query the table to see if it exists
    let query = format!("SELECT 1 FROM {table_name} LIMIT 1");
    diesel::sql_query(query).execute(&mut conn).is_ok()
}

pub fn relays_table_exists(pool: &DbPool) -> bool {
    table_exists(pool, "relays")
}

pub fn relocated_table_exists(pool: &DbPool) -> bool {
    table_exists(pool, "relocated")
}

pub fn clients_table_exists(pool: &DbPool) -> bool {
    table_exists(pool, "clients")
}

// Additional report functions
pub fn get_orphaned_aliases_report(pool: &DbPool) -> Result<OrphanedAliasReport, Error> {
    let mut conn = pool.get().unwrap();

    // Find aliases where the mail domain doesn't exist in the domains table
    let mut orphaned_aliases: Vec<OrphanedAlias> = aliases::table
        .select((
            aliases::pkid,
            aliases::mail,
            aliases::destination,
            aliases::enabled,
            aliases::created,
        ))
        .order_by(aliases::mail.asc())
        .load::<(i32, String, String, bool, Option<NaiveDateTime>)>(&mut conn)?
        .into_iter()
        .filter(|(_, mail, _, _, _)| {
            // Extract the domain from the alias mail address
            if let Some(at_pos) = mail.rfind('@') {
                let mail_domain = &mail[at_pos + 1..];
                // Check if this domain exists and is enabled in our domains table
                let domain_exists: Option<(String, bool)> = domains::table
                    .filter(domains::domain.eq(mail_domain))
                    .select((domains::domain, domains::enabled))
                    .first::<(String, bool)>(&mut conn)
                    .optional()
                    .unwrap_or(None);
                // Consider orphaned if domain doesn't exist or is disabled
                domain_exists.is_none_or(|(_, enabled)| !enabled)
            } else {
                false
            }
        })
        .map(|(id, mail, destination, enabled, _created)| {
            let domain = mail.split('@').nth(1).unwrap_or("").to_string();
            OrphanedAlias {
                id,
                mail,
                destination,
                domain,
                domain_id: None,      // Will be populated later
                domain_enabled: None, // Will be populated later
                enabled,
            }
        })
        .collect::<Vec<_>>();

    // Sort by domain first, then by mail
    orphaned_aliases.sort_by(|a, b| a.domain.cmp(&b.domain).then_with(|| a.mail.cmp(&b.mail)));

    // Populate domain IDs and enabled status for orphaned aliases
    for alias in &mut orphaned_aliases {
        if let Some(at_pos) = alias.mail.rfind('@') {
            let mail_domain = &alias.mail[at_pos + 1..];
            if let Ok((domain_id, domain_enabled)) = domains::table
                .filter(domains::domain.eq(mail_domain))
                .select((domains::pkid, domains::enabled))
                .first::<(i32, bool)>(&mut conn)
            {
                alias.domain_id = Some(domain_id);
                alias.domain_enabled = Some(domain_enabled);
            }
        }
    }

    // Find users where the domain doesn't exist or is disabled in the domains table
    let mut orphaned_users: Vec<OrphanedUser> = users::table
        .select((users::id, users::name, users::enabled, users::created))
        .order_by(users::id.asc())
        .load::<(String, String, bool, Option<NaiveDateTime>)>(&mut conn)?
        .into_iter()
        .filter(|(id, _, _, _)| {
            // Extract the domain from the user ID
            if let Some(at_pos) = id.rfind('@') {
                let user_domain = &id[at_pos + 1..];
                // Check if this domain exists and is enabled in our domains table
                let domain_exists: Option<(String, bool)> = domains::table
                    .filter(domains::domain.eq(user_domain))
                    .select((domains::domain, domains::enabled))
                    .first::<(String, bool)>(&mut conn)
                    .optional()
                    .unwrap_or(None);
                // Consider orphaned if domain doesn't exist or is disabled
                domain_exists.is_none_or(|(_, enabled)| !enabled)
            } else {
                false
            }
        })
        .map(|(id, name, enabled, _created)| {
            let domain = id.split('@').nth(1).unwrap_or("").to_string();
            OrphanedUser {
                id,
                name,
                domain,
                domain_id: None,      // Will be populated later
                domain_enabled: None, // Will be populated later
                enabled,
            }
        })
        .collect::<Vec<_>>();

    // Sort by domain first, then by id
    orphaned_users.sort_by(|a, b| a.domain.cmp(&b.domain).then_with(|| a.id.cmp(&b.id)));

    // Populate domain IDs and enabled status for orphaned users
    for user in &mut orphaned_users {
        if let Some(at_pos) = user.id.rfind('@') {
            let user_domain = &user.id[at_pos + 1..];
            if let Ok((domain_id, domain_enabled)) = domains::table
                .filter(domains::domain.eq(user_domain))
                .select((domains::pkid, domains::enabled))
                .first::<(i32, bool)>(&mut conn)
            {
                user.domain_id = Some(domain_id);
                user.domain_enabled = Some(domain_enabled);
            }
        }
    }

    // Find users who don't have a corresponding alias
    let mut users_without_aliases: Vec<UserWithoutAlias> = users::table
        .select((users::id, users::name, users::enabled, users::created))
        .order_by(users::id.asc())
        .load::<(String, String, bool, Option<NaiveDateTime>)>(&mut conn)?
        .into_iter()
        .filter(|(id, _, _, _)| {
            // Check if there's an alias for this user
            let alias_exists: Option<String> = aliases::table
                .filter(aliases::mail.eq(id))
                .select(aliases::mail)
                .first::<String>(&mut conn)
                .optional()
                .unwrap_or(None);
            alias_exists.is_none()
        })
        .map(|(id, name, enabled, _created)| {
            let domain = id.split('@').nth(1).unwrap_or("").to_string();
            UserWithoutAlias {
                id,
                name,
                domain,
                domain_id: None,      // Will be populated later
                domain_enabled: None, // Will be populated later
                enabled,
            }
        })
        .collect::<Vec<_>>();

    // Sort by domain first, then by id
    users_without_aliases.sort_by(|a, b| a.domain.cmp(&b.domain).then_with(|| a.id.cmp(&b.id)));

    // Populate domain IDs and enabled status for users without aliases
    for user in &mut users_without_aliases {
        if let Some(at_pos) = user.id.rfind('@') {
            let user_domain = &user.id[at_pos + 1..];
            if let Ok((domain_id, domain_enabled)) = domains::table
                .filter(domains::domain.eq(user_domain))
                .select((domains::pkid, domains::enabled))
                .first::<(i32, bool)>(&mut conn)
            {
                user.domain_id = Some(domain_id);
                user.domain_enabled = Some(domain_enabled);
            }
        }
    }

    // Find relays where the recipient domain doesn't exist or is disabled in the domains table
    let mut orphaned_relays: Vec<OrphanedRelay> = if relays_table_exists(pool) {
        relays::table
            .select((
                relays::pkid,
                relays::recipient,
                relays::status,
                relays::enabled,
            ))
            .order_by(relays::recipient.asc())
            .load::<(i32, String, String, bool)>(&mut conn)?
            .into_iter()
            .filter(|(_, recipient, _, _)| {
                // Extract the domain from the relay recipient address
                if let Some(at_pos) = recipient.rfind('@') {
                    let relay_domain = &recipient[at_pos + 1..];
                    // Check if this domain exists in our domains table
                    let domain_exists: Option<(String, bool)> = domains::table
                        .filter(domains::domain.eq(relay_domain))
                        .select((domains::domain, domains::enabled))
                        .first::<(String, bool)>(&mut conn)
                        .optional()
                        .unwrap_or(None);
                    // Consider orphaned if domain doesn't exist
                    domain_exists.is_none()
                } else {
                    false
                }
            })
            .map(|(id, recipient, status, enabled)| {
                let domain = recipient.split('@').nth(1).unwrap_or("").to_string();
                OrphanedRelay {
                    id,
                    recipient,
                    status,
                    domain,
                    domain_id: None,      // Will be populated later
                    domain_enabled: None, // Will be populated later
                    enabled,
                }
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    // Sort by domain first, then by recipient
    orphaned_relays.sort_by(|a, b| {
        a.domain
            .cmp(&b.domain)
            .then_with(|| a.recipient.cmp(&b.recipient))
    });

    // Populate domain IDs and enabled status for orphaned relays
    for relay in &mut orphaned_relays {
        if let Some(at_pos) = relay.recipient.rfind('@') {
            let relay_domain = &relay.recipient[at_pos + 1..];
            if let Ok((domain_id, domain_enabled)) = domains::table
                .filter(domains::domain.eq(relay_domain))
                .select((domains::pkid, domains::enabled))
                .first::<(i32, bool)>(&mut conn)
            {
                relay.domain_id = Some(domain_id);
                relay.domain_enabled = Some(domain_enabled);
            }
        }
    }

    // Find relocated where the old_address domain doesn't exist or is disabled in the domains table
    let mut orphaned_relocated: Vec<OrphanedRelocated> = if relocated_table_exists(pool) {
        relocated::table
            .select((
                relocated::pkid,
                relocated::old_address,
                relocated::new_address,
                relocated::enabled,
            ))
            .order_by(relocated::old_address.asc())
            .load::<(i32, String, String, bool)>(&mut conn)?
            .into_iter()
            .filter(|(_, old_address, _, _)| {
                // Extract the domain from the relocated old_address
                if let Some(at_pos) = old_address.rfind('@') {
                    let relocated_domain = &old_address[at_pos + 1..];
                    // Check if this domain exists in our domains table
                    let domain_exists: Option<(String, bool)> = domains::table
                        .filter(domains::domain.eq(relocated_domain))
                        .select((domains::domain, domains::enabled))
                        .first::<(String, bool)>(&mut conn)
                        .optional()
                        .unwrap_or(None);
                    // Consider orphaned if domain doesn't exist
                    domain_exists.is_none()
                } else {
                    false
                }
            })
            .map(|(id, old_address, new_address, enabled)| {
                let domain = old_address.split('@').nth(1).unwrap_or("").to_string();
                OrphanedRelocated {
                    id,
                    old_address,
                    new_address,
                    domain,
                    domain_id: None,      // Will be populated later
                    domain_enabled: None, // Will be populated later
                    enabled,
                }
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    // Sort by domain first, then by old_address
    orphaned_relocated.sort_by(|a, b| {
        a.domain
            .cmp(&b.domain)
            .then_with(|| a.old_address.cmp(&b.old_address))
    });

    // Populate domain IDs and enabled status for orphaned relocated
    for relocated in &mut orphaned_relocated {
        if let Some(at_pos) = relocated.old_address.rfind('@') {
            let relocated_domain = &relocated.old_address[at_pos + 1..];
            if let Ok((domain_id, domain_enabled)) = domains::table
                .filter(domains::domain.eq(relocated_domain))
                .select((domains::pkid, domains::enabled))
                .first::<(i32, bool)>(&mut conn)
            {
                relocated.domain_id = Some(domain_id);
                relocated.domain_enabled = Some(domain_enabled);
            }
        }
    }

    Ok(OrphanedAliasReport {
        orphaned_aliases,
        orphaned_users,
        users_without_aliases,
        orphaned_relays,
        orphaned_relocated,
    })
}

pub fn get_external_forwarders_report(pool: &DbPool) -> Result<ExternalForwarderReport, Error> {
    let mut conn = pool.get().unwrap();

    // Find aliases where the destination is an external email address (contains @ and doesn't match any domain in the domains table)
    let mut external_forwarders: Vec<ExternalForwarder> = aliases::table
        .filter(aliases::destination.like("%@%"))
        .select((
            aliases::pkid,
            aliases::mail,
            aliases::destination,
            aliases::enabled,
            aliases::created,
        ))
        .load::<(i32, String, String, bool, Option<NaiveDateTime>)>(&mut conn)?
        .into_iter()
        .filter(|(_, _, destination, _, _)| {
            // Extract the domain from the destination
            if let Some(at_pos) = destination.rfind('@') {
                let dest_domain = &destination[at_pos + 1..];
                // Check if this domain exists in our domains table
                let domain_exists: Option<String> = domains::table
                    .filter(domains::domain.eq(dest_domain))
                    .select(domains::domain)
                    .first::<String>(&mut conn)
                    .optional()
                    .unwrap_or(None);
                domain_exists.is_none()
            } else {
                false
            }
        })
        .map(|(id, mail, destination, enabled, _created)| {
            let domain = mail.split('@').nth(1).unwrap_or("").to_string();
            ExternalForwarder {
                id,
                mail,
                destination,
                domain,
                domain_id: None, // Will be populated later
                enabled,
            }
        })
        .collect::<Vec<_>>();

    // Sort by domain first, then by mail
    external_forwarders.sort_by(|a, b| a.domain.cmp(&b.domain).then_with(|| a.mail.cmp(&b.mail)));

    // Populate domain IDs for external forwarders
    for forwarder in &mut external_forwarders {
        if let Some(at_pos) = forwarder.mail.rfind('@') {
            let mail_domain = &forwarder.mail[at_pos + 1..];
            if let Ok(domain_id) = domains::table
                .filter(domains::domain.eq(mail_domain))
                .select(domains::pkid)
                .first::<i32>(&mut conn)
            {
                forwarder.domain_id = Some(domain_id);
            }
        }
    }

    Ok(ExternalForwarderReport {
        external_forwarders,
    })
}

pub fn get_missing_aliases_report(pool: &DbPool) -> Result<MissingAliasReport, Error> {
    let mut conn = pool.get().unwrap();

    // Get all domains
    let all_domains = domains::table
        .select(domains::domain)
        .load::<String>(&mut conn)?;

    let mut domains_missing_aliases = Vec::new();

    for domain in all_domains {
        // Check if domain has catch-all alias
        let catch_all_alias: Option<String> = aliases::table
            .filter(aliases::mail.eq(format!("@{domain}")))
            .select(aliases::mail)
            .first::<String>(&mut conn)
            .optional()?;

        let has_catch_all = catch_all_alias.is_some();

        // Get required aliases for this domain
        let required_aliases = get_required_aliases_for_domain(&mut conn, &domain)?;

        // Check which required aliases are missing
        let mut missing_required_aliases = Vec::new();
        for required_alias in required_aliases {
            let alias_exists: Option<String> = aliases::table
                .filter(aliases::mail.eq(format!("{required_alias}@{domain}")))
                .select(aliases::mail)
                .first::<String>(&mut conn)
                .optional()?;

            if alias_exists.is_none() {
                missing_required_aliases.push(required_alias);
            }
        }

        // Only include domains that are missing required aliases AND don't have a catch-all
        if !missing_required_aliases.is_empty() && !has_catch_all {
            domains_missing_aliases.push(DomainMissingAliases {
                domain,
                missing_required_aliases,
                has_catch_all,
                catch_all_alias,
            });
        }
    }

    Ok(MissingAliasReport {
        domains_missing_aliases,
    })
}

pub fn get_alias_cross_domain_report(
    pool: &DbPool,
    alias_name: &str,
) -> Result<AliasCrossDomainReport, Error> {
    let mut conn = pool.get().unwrap();

    // Find all occurrences of this alias across all domains
    let occurrences: Vec<AliasOccurrence> = aliases::table
        .filter(aliases::mail.like(format!("{alias_name}@%")))
        .select((
            aliases::pkid,
            aliases::mail,
            aliases::destination,
            aliases::enabled,
        ))
        .load::<(i32, String, String, bool)>(&mut conn)?
        .into_iter()
        .map(|(id, mail, destination, enabled)| {
            let domain = mail.split('@').nth(1).unwrap_or("").to_string();
            AliasOccurrence {
                id,
                domain,
                mail,
                destination,
                enabled,
            }
        })
        .collect();

    Ok(AliasCrossDomainReport {
        alias: alias_name.to_string(),
        occurrences,
    })
}

// Helper function to get required aliases for a domain
fn get_required_aliases_for_domain(
    _conn: &mut MysqlConnection,
    _domain: &str,
) -> Result<Vec<String>, Error> {
    // This would typically come from configuration, but for now we'll use a default list
    // In a real implementation, this would be configurable per domain
    Ok(vec![
        "postmaster".to_string(),
        "abuse".to_string(),
        "webmaster".to_string(),
        "admin".to_string(),
    ])
}

// Cross-database domain matrix report
pub async fn get_cross_database_domain_matrix_report(
    db_manager: &DatabaseManager,
    _current_db_id: Option<&str>,
) -> Result<CrossDatabaseDomainMatrixReport, Box<dyn std::error::Error>> {
    let configs = db_manager.get_configs();
    let mut all_domains = std::collections::HashSet::new();
    let mut domain_presence_map = std::collections::HashMap::new();

    // Collect all unique domains from all databases
    for config in configs {
        if let Some(pool) = db_manager.get_pool(&config.id).await {
            // Get domains from this database
            match get_domains(&pool) {
                Ok(domains) => {
                    for domain in domains {
                        all_domains.insert(domain.domain.clone());
                        domain_presence_map
                            .entry((domain.domain.clone(), config.id.clone()))
                            .or_insert_with(Vec::new)
                            .push(DomainPresence {
                                database_id: config.id.clone(),
                                database_label: config.label.clone(),
                                presence_type: DomainPresenceType::Primary,
                                enabled: domain.enabled,
                            });
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to get domains from database {}: {:?}", config.id, e);
                }
            }

            // Get backup domains from this database
            match get_backups(&pool) {
                Ok(backups) => {
                    for backup in backups {
                        all_domains.insert(backup.domain.clone());
                        domain_presence_map
                            .entry((backup.domain.clone(), config.id.clone()))
                            .or_insert_with(Vec::new)
                            .push(DomainPresence {
                                database_id: config.id.clone(),
                                database_label: config.label.clone(),
                                presence_type: DomainPresenceType::Backup,
                                enabled: backup.enabled,
                            });
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to get backups from database {}: {:?}", config.id, e);
                }
            }
        }
    }

    // Create database info list (each database will have 2 columns: primary and backup)
    let databases: Vec<DatabaseInfo> = configs
        .iter()
        .map(|config| DatabaseInfo {
            id: config.id.clone(),
            label: config.label.clone(),
            enabled: !config.features.disabled,
        })
        .collect();

    // Create domain rows with presence information for all databases
    let mut domain_rows = Vec::new();
    for domain in all_domains {
        let mut presence = Vec::new();
        let mut domain_id = 0;

        // Try to get the domain ID from the database where this domain actually exists
        // Check all databases to find where this domain exists (as primary or backup)
        let mut domain_database_id = String::new();
        for db_config in configs {
            if let Some(pool) = db_manager.get_pool(&db_config.id).await {
                // First try as primary domain
                if let Ok(domain_record) = get_domain_by_name(&pool, &domain) {
                    domain_id = *domain_record.id();
                    domain_database_id = db_config.id.clone();
                    break;
                }
                // If not found as primary domain, try as backup domain
                if let Ok(backup_record) = get_backup_by_name(&pool, &domain) {
                    domain_id = backup_record.pkid;
                    domain_database_id = db_config.id.clone();
                    break;
                }
            }
        }

        for db_config in configs {
            // Check for primary domain presence
            let primary_presence = domain_presence_map
                .get(&(domain.clone(), db_config.id.clone()))
                .and_then(|presences| {
                    presences
                        .iter()
                        .find(|p| p.presence_type == DomainPresenceType::Primary)
                })
                .cloned()
                .unwrap_or_else(|| DomainPresence {
                    database_id: db_config.id.clone(),
                    database_label: format!("{} (Primary)", db_config.label),
                    presence_type: DomainPresenceType::Missing,
                    enabled: false,
                });

            // Check for backup domain presence
            let backup_presence = domain_presence_map
                .get(&(domain.clone(), db_config.id.clone()))
                .and_then(|presences| {
                    presences
                        .iter()
                        .find(|p| p.presence_type == DomainPresenceType::Backup)
                })
                .cloned()
                .unwrap_or_else(|| DomainPresence {
                    database_id: db_config.id.clone(),
                    database_label: format!("{} (Backup)", db_config.label),
                    presence_type: DomainPresenceType::Missing,
                    enabled: false,
                });

            presence.push(primary_presence);
            presence.push(backup_presence);
        }

        domain_rows.push(CrossDatabaseDomainRow {
            domain,
            domain_id,
            domain_database_id,
            presence,
        });
    }

    // Sort domains alphabetically
    domain_rows.sort_by(|a, b| a.domain.cmp(&b.domain));

    Ok(CrossDatabaseDomainMatrixReport {
        databases,
        domains: domain_rows,
    })
}

// Cross-database User Distribution Report
pub async fn get_cross_database_user_distribution_report(
    db_manager: &DatabaseManager,
) -> Result<CrossDatabaseUserDistributionReport, Box<dyn std::error::Error>> {
    let configs = db_manager.get_configs();
    let mut all_users = std::collections::HashMap::new();
    let mut user_presence_map = std::collections::HashMap::new();

    // Collect all users from all databases
    for config in configs {
        if let Some(pool) = db_manager.get_pool(&config.id).await {
            match get_users(&pool) {
                Ok(users) => {
                    for user in users {
                        all_users.insert(user.id.clone(), user.name.clone());

                        // Get user's domain by checking aliases
                        let user_domain = get_user_domain(&pool, &user.id)
                            .unwrap_or(None)
                            .unwrap_or_default();

                        user_presence_map
                            .entry(user.id.clone())
                            .or_insert_with(Vec::new)
                            .push(UserPresence {
                                database_id: config.id.clone(),
                                database_label: config.label.clone(),
                                present: true,
                                enabled: user.enabled,
                                domain: user_domain,
                            });
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to get users from database {}: {:?}", config.id, e);
                }
            }
        }
    }

    // Create database info list
    let databases: Vec<DatabaseInfo> = configs
        .iter()
        .map(|config| DatabaseInfo {
            id: config.id.clone(),
            label: config.label.clone(),
            enabled: !config.features.disabled,
        })
        .collect();

    // Create user rows with presence information for all databases
    let mut user_rows = Vec::new();
    for (user_id, user_name) in all_users {
        let mut presence = Vec::new();

        for db_config in configs {
            let user_presence = user_presence_map
                .get(&user_id)
                .and_then(|presences| presences.iter().find(|p| p.database_id == db_config.id))
                .cloned()
                .unwrap_or_else(|| UserPresence {
                    database_id: db_config.id.clone(),
                    database_label: db_config.label.clone(),
                    present: false,
                    enabled: false,
                    domain: String::new(),
                });

            presence.push(user_presence);
        }

        user_rows.push(CrossDatabaseUserRow {
            user_id,
            user_name,
            presence,
        });
    }

    // Sort users by ID
    user_rows.sort_by(|a, b| a.user_id.cmp(&b.user_id));

    // Calculate summary statistics
    let total_users = user_rows.len() as i64;
    let users_in_multiple_dbs = user_rows
        .iter()
        .filter(|row| row.presence.iter().filter(|p| p.present).count() > 1)
        .count() as i64;
    let users_in_single_db = total_users - users_in_multiple_dbs;
    let enabled_users = user_rows
        .iter()
        .filter(|row| row.presence.iter().any(|p| p.present && p.enabled))
        .count() as i64;
    let disabled_users = total_users - enabled_users;

    let summary = UserDistributionSummary {
        total_users,
        users_in_multiple_dbs,
        users_in_single_db,
        enabled_users,
        disabled_users,
    };

    Ok(CrossDatabaseUserDistributionReport {
        databases,
        users: user_rows,
        summary,
    })
}

// Helper function to get user's domain
fn get_user_domain(pool: &DbPool, user_id: &str) -> Result<Option<String>, Error> {
    let mut conn = pool.get().unwrap();

    // Find aliases for this user to determine their domain
    let domain: Option<String> = aliases::table
        .filter(aliases::mail.like(format!("{user_id}@%")))
        .select(aliases::mail)
        .first::<String>(&mut conn)
        .optional()?
        .map(|mail| mail.split('@').nth(1).unwrap_or("").to_string());

    Ok(domain)
}

// Cross-database Feature Toggle Compliance Report
pub async fn get_cross_database_feature_toggle_report(
    db_manager: &DatabaseManager,
) -> Result<CrossDatabaseFeatureToggleReport, Box<dyn std::error::Error>> {
    let configs = db_manager.get_configs();
    let mut database_features = Vec::new();

    for config in configs {
        let features = DatabaseFeatures {
            read_only: config.features.read_only,
            no_new_users: config.features.no_new_users,
            no_new_domains: config.features.no_new_domains,
            no_password_updates: config.features.no_password_updates,
        };

        database_features.push(DatabaseFeatureInfo {
            id: config.id.clone(),
            label: config.label.clone(),
            enabled: !config.features.disabled,
            features,
        });
    }

    // Calculate compliance summary
    let total_databases = database_features.len() as i64;
    let databases_with_read_only = database_features
        .iter()
        .filter(|db| db.features.read_only)
        .count() as i64;
    let databases_with_no_new_users = database_features
        .iter()
        .filter(|db| db.features.no_new_users)
        .count() as i64;
    let databases_with_no_new_domains = database_features
        .iter()
        .filter(|db| db.features.no_new_domains)
        .count() as i64;
    let databases_with_no_password_updates = database_features
        .iter()
        .filter(|db| db.features.no_password_updates)
        .count() as i64;
    let fully_restricted_databases = database_features
        .iter()
        .filter(|db| {
            db.features.read_only
                && db.features.no_new_users
                && db.features.no_new_domains
                && db.features.no_password_updates
        })
        .count() as i64;

    let compliance_summary = FeatureComplianceSummary {
        total_databases,
        databases_with_read_only,
        databases_with_no_new_users,
        databases_with_no_new_domains,
        databases_with_no_password_updates,
        fully_restricted_databases,
    };

    Ok(CrossDatabaseFeatureToggleReport {
        databases: database_features,
        compliance_summary,
    })
}

// Cross-database Migration Status Report
pub async fn get_cross_database_migration_report(
    db_manager: &DatabaseManager,
) -> Result<CrossDatabaseMigrationReport, Box<dyn std::error::Error>> {
    let configs = db_manager.get_configs();
    let mut database_migrations = Vec::new();
    let mut latest_migration = None;

    for config in configs {
        let migration_status = if let Some(pool) = db_manager.get_pool(&config.id).await {
            // Try to check migration status by querying the schema_version table
            match check_migration_status(&pool).await {
                Ok(status) => status,
                Err(e) => {
                    tracing::warn!(
                        "Failed to check migration status for database {}: {:?}",
                        config.id,
                        e
                    );
                    MigrationStatus::Unknown
                }
            }
        } else {
            MigrationStatus::Unknown
        };

        // For now, we'll use placeholder values for migration details
        // In a real implementation, you'd query the actual migration tables
        let last_migration = "2025-07-08-111712_add_unique_constraint_to_aliases".to_string();
        let migration_count = 12; // This would be dynamic based on actual migrations

        if latest_migration.is_none() || &last_migration > latest_migration.as_ref().unwrap() {
            latest_migration = Some(last_migration.clone());
        }

        database_migrations.push(DatabaseMigrationInfo {
            id: config.id.clone(),
            label: config.label.clone(),
            enabled: !config.features.disabled,
            migration_status,
            last_migration,
            migration_count,
        });
    }

    // Calculate migration summary
    let total_databases = database_migrations.len() as i64;
    let up_to_date = database_migrations
        .iter()
        .filter(|db| db.migration_status == MigrationStatus::UpToDate)
        .count() as i64;
    let behind = database_migrations
        .iter()
        .filter(|db| db.migration_status == MigrationStatus::Behind)
        .count() as i64;
    let errors = database_migrations
        .iter()
        .filter(|db| db.migration_status == MigrationStatus::Error)
        .count() as i64;
    let unknown = database_migrations
        .iter()
        .filter(|db| db.migration_status == MigrationStatus::Unknown)
        .count() as i64;

    let migration_summary = MigrationSummary {
        total_databases,
        up_to_date,
        behind,
        errors,
        unknown,
        latest_migration: latest_migration.unwrap_or_default(),
    };

    Ok(CrossDatabaseMigrationReport {
        databases: database_migrations,
        migration_summary,
    })
}

// Helper function to check migration status
async fn check_migration_status(
    pool: &DbPool,
) -> Result<MigrationStatus, Box<dyn std::error::Error>> {
    // This is a simplified implementation
    // In a real scenario, you'd check against the actual migration system
    // For now, we'll assume all databases are up to date if they can connect
    let mut conn = pool.get().unwrap();

    // Try a simple query to check if the database is accessible
    match diesel::sql_query("SELECT 1").execute(&mut conn) {
        Ok(_) => Ok(MigrationStatus::UpToDate),
        Err(_) => Ok(MigrationStatus::Error),
    }
}

/// Get users using per-database field mapping with table-qualified field names
pub fn get_users_with_field_map(
    pool: &DbPool,
    db_config: &crate::config::DatabaseConfig,
) -> Result<Vec<User>, Error> {
    let mut conn = pool.get().unwrap();

    // Use table-qualified field mapping
    let user_id = db_config.field_for_table("users", "id");
    let enabled = db_config.field_for_table("users", "enabled");
    let crypt = db_config.field_for_table("users", "crypt");
    let name = db_config.field_for_table("users", "name");
    let maildir = db_config.field_for_table("users", "maildir");
    let home = db_config.field_for_table("users", "home");
    let uid = db_config.field_for_table("users", "uid");
    let gid = db_config.field_for_table("users", "gid");
    let created = db_config.field_for_table("users", "created");
    let modified = db_config.field_for_table("users", "modified");
    let change_password = db_config.field_for_table("users", "change_password");

    let sql = format!(
        "SELECT {user_id} as id, {enabled} as enabled, {crypt} as crypt, {name} as name, {maildir} as maildir, {home} as home, {uid} as uid, {gid} as gid, {created} as created, {modified} as modified, {change_password} as change_password FROM users"
    );

    sql_query(sql).load::<User>(&mut conn)
}

/// Helper function to build a SELECT query with field mapping for any table
pub fn build_field_mapped_query(
    table: &str,
    fields: &[(&str, &str)], // (logical_name, alias_name)
    db_config: &crate::config::DatabaseConfig,
) -> String {
    let mapped_fields: Vec<String> = fields
        .iter()
        .map(|(logical, alias)| {
            let mapped_field = db_config.field_for_table(table, logical);
            format!("{mapped_field} as {alias}")
        })
        .collect();

    format!("SELECT {} FROM {}", mapped_fields.join(", "), table)
}

/// Get domains using per-database field mapping with table-qualified field names
pub fn get_domains_with_field_map(
    pool: &DbPool,
    db_config: &crate::config::DatabaseConfig,
) -> Result<Vec<Domain>, Error> {
    let mut conn = pool.get().unwrap();

    let fields = [
        ("id", "pkid"),
        ("domain", "domain"),
        ("transport", "transport"),
        ("created", "created"),
        ("modified", "modified"),
        ("enabled", "enabled"),
    ];

    let sql = build_field_mapped_query("domains", &fields, db_config);
    sql_query(sql).load::<Domain>(&mut conn)
}

/// Get aliases using per-database field mapping with table-qualified field names
pub fn get_aliases_with_field_map(
    pool: &DbPool,
    db_config: &crate::config::DatabaseConfig,
) -> Result<Vec<Alias>, Error> {
    let mut conn = pool.get().unwrap();

    let fields = [
        ("id", "pkid"),
        ("mail", "mail"),
        ("destination", "destination"),
        ("created", "created"),
        ("modified", "modified"),
        ("enabled", "enabled"),
    ];

    let sql = build_field_mapped_query("aliases", &fields, db_config);
    sql_query(sql).load::<Alias>(&mut conn)
}

/// Get recent changes across all resource types
pub async fn get_recent_changes_report(
    pool: &DbPool,
    limit: Option<i64>,
) -> Result<RecentChangesReport, Box<dyn std::error::Error>> {
    let limit = limit.unwrap_or(50);
    let mut conn = pool.get().map_err(|e| {
        tracing::error!("Failed to get connection from pool: {:?}", e);
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(format!("Failed to get database connection: {e:?}")),
        )
    })?;

    let mut all_changes = Vec::new();

    // Get recent domains
    let recent_domains = domains::table
        .order(domains::modified.desc())
        .limit(limit)
        .load::<Domain>(&mut conn)?;

    for domain in recent_domains {
        all_changes.push(RecentChange {
            resource_type: "domain".to_string(),
            resource_id: domain.pkid.to_string(),
            resource_name: domain.domain.clone(),
            action: if domain.created == domain.modified {
                "created"
            } else {
                "updated"
            }
            .to_string(),
            timestamp: domain
                .modified
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            enabled: Some(domain.enabled),
        });
    }

    // Get recent users
    let recent_users = users::table
        .order(users::modified.desc())
        .limit(limit)
        .load::<User>(&mut conn)?;

    for user in recent_users {
        all_changes.push(RecentChange {
            resource_type: "user".to_string(),
            resource_id: user.id.clone(),
            resource_name: user.id.clone(),
            action: if user.created == user.modified {
                "created"
            } else {
                "updated"
            }
            .to_string(),
            timestamp: user
                .modified
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            enabled: Some(user.enabled),
        });
    }

    // Get recent aliases
    let recent_aliases = aliases::table
        .order(aliases::modified.desc())
        .limit(limit)
        .load::<Alias>(&mut conn)?;

    for alias in recent_aliases {
        all_changes.push(RecentChange {
            resource_type: "alias".to_string(),
            resource_id: alias.pkid.to_string(),
            resource_name: alias.mail.clone(),
            action: if alias.created == alias.modified {
                "created"
            } else {
                "updated"
            }
            .to_string(),
            timestamp: alias
                .modified
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            enabled: Some(alias.enabled),
        });
    }

    // Get recent backups
    let recent_backups = backups::table
        .order(backups::modified.desc())
        .limit(limit)
        .load::<Backup>(&mut conn)?;

    for backup in recent_backups {
        all_changes.push(RecentChange {
            resource_type: "backup".to_string(),
            resource_id: backup.pkid.to_string(),
            resource_name: backup.domain.clone(),
            action: if backup.created == backup.modified {
                "created"
            } else {
                "updated"
            }
            .to_string(),
            timestamp: backup
                .modified
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            enabled: Some(backup.enabled),
        });
    }

    // Get recent relays
    let recent_relays = relays::table
        .order(relays::modified.desc())
        .limit(limit)
        .load::<Relay>(&mut conn)?;

    for relay in recent_relays {
        all_changes.push(RecentChange {
            resource_type: "relay".to_string(),
            resource_id: relay.pkid.to_string(),
            resource_name: relay.recipient.clone(),
            action: if relay.created == relay.modified {
                "created"
            } else {
                "updated"
            }
            .to_string(),
            timestamp: relay
                .modified
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            enabled: Some(relay.enabled),
        });
    }

    // Get recent relocated
    let recent_relocated = relocated::table
        .order(relocated::modified.desc())
        .limit(limit)
        .load::<Relocated>(&mut conn)?;

    for relocated in recent_relocated {
        all_changes.push(RecentChange {
            resource_type: "relocated".to_string(),
            resource_id: relocated.pkid.to_string(),
            resource_name: format!("{} -> {}", relocated.old_address, relocated.new_address),
            action: if relocated.created == relocated.modified {
                "created"
            } else {
                "updated"
            }
            .to_string(),
            timestamp: relocated
                .modified
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            enabled: Some(relocated.enabled),
        });
    }

    // Get recent clients
    let recent_clients = clients::table
        .order(clients::updated_at.desc())
        .limit(limit)
        .load::<Client>(&mut conn)?;

    for client in recent_clients {
        all_changes.push(RecentChange {
            resource_type: "client".to_string(),
            resource_id: client.id.to_string(),
            resource_name: client.client.clone(),
            action: if client.created_at == client.updated_at {
                "created"
            } else {
                "updated"
            }
            .to_string(),
            timestamp: client
                .updated_at
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            enabled: Some(client.enabled),
        });
    }

    // Sort all changes by timestamp (most recent first)
    all_changes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Take only the requested limit
    let total_count = all_changes.len();
    all_changes.truncate(limit as usize);

    Ok(RecentChangesReport {
        changes: all_changes,
        total_count,
    })
}

/// Perform a comprehensive search across all tables
pub async fn search_all_tables(
    pool: &DbPool,
    query: &str,
    resource_types: Option<&[String]>,
    enabled_only: Option<bool>,
    limit: Option<i64>,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    let limit = limit.unwrap_or(100);
    let mut conn = pool.get().map_err(|e| {
        tracing::error!("Failed to get connection from pool: {:?}", e);
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(format!("Failed to get database connection: {e:?}")),
        )
    })?;

    let mut all_results = Vec::new();
    let search_pattern = format!("%{}%", query);

    // Search domains
    if resource_types.is_none()
        || resource_types
            .as_ref()
            .unwrap()
            .contains(&"domain".to_string())
    {
        let domains = domains::table
            .filter(
                domains::domain
                    .like(&search_pattern)
                    .or(domains::transport.like(&search_pattern)),
            )
            .load::<Domain>(&mut conn)?;

        for domain in domains {
            if enabled_only.is_some() && enabled_only.unwrap() && !domain.enabled {
                continue;
            }

            let mut match_fields = Vec::new();
            if domain.domain.contains(query) {
                match_fields.push("domain".to_string());
            }
            if domain
                .transport
                .as_ref()
                .map_or(false, |t| t.contains(query))
            {
                match_fields.push("transport".to_string());
            }

            all_results.push(SearchResult {
                resource_type: "domain".to_string(),
                resource_id: domain.pkid.to_string(),
                resource_name: domain.domain.clone(),
                resource_description: format!(
                    "Transport: {}",
                    domain.transport.as_deref().unwrap_or("N/A")
                ),
                enabled: Some(domain.enabled),
                created: domain.created,
                modified: domain.modified,
                match_fields,
            });
        }
    }

    // Search users
    if resource_types.is_none()
        || resource_types
            .as_ref()
            .unwrap()
            .contains(&"user".to_string())
    {
        let users = users::table
            .filter(
                users::id
                    .like(&search_pattern)
                    .or(users::name.like(&search_pattern))
                    .or(users::maildir.like(&search_pattern))
                    .or(users::home.like(&search_pattern)),
            )
            .load::<User>(&mut conn)?;

        for user in users {
            if enabled_only.is_some() && enabled_only.unwrap() && !user.enabled {
                continue;
            }

            let mut match_fields = Vec::new();
            if user.id.contains(query) {
                match_fields.push("id".to_string());
            }
            if user.name.contains(query) {
                match_fields.push("name".to_string());
            }
            if user.maildir.contains(query) {
                match_fields.push("maildir".to_string());
            }
            if user.home.contains(query) {
                match_fields.push("home".to_string());
            }

            all_results.push(SearchResult {
                resource_type: "user".to_string(),
                resource_id: user.id.clone(),
                resource_name: user.id.clone(),
                resource_description: format!("Name: {}, Maildir: {}", user.name, user.maildir),
                enabled: Some(user.enabled),
                created: user.created,
                modified: user.modified,
                match_fields,
            });
        }
    }

    // Search aliases
    if resource_types.is_none()
        || resource_types
            .as_ref()
            .unwrap()
            .contains(&"alias".to_string())
    {
        let aliases = aliases::table
            .filter(
                aliases::mail
                    .like(&search_pattern)
                    .or(aliases::destination.like(&search_pattern)),
            )
            .load::<Alias>(&mut conn)?;

        for alias in aliases {
            if enabled_only.is_some() && enabled_only.unwrap() && !alias.enabled {
                continue;
            }

            let mut match_fields = Vec::new();
            if alias.mail.contains(query) {
                match_fields.push("mail".to_string());
            }
            if alias.destination.contains(query) {
                match_fields.push("destination".to_string());
            }

            all_results.push(SearchResult {
                resource_type: "alias".to_string(),
                resource_id: alias.pkid.to_string(),
                resource_name: alias.mail.clone(),
                resource_description: format!("→ {}", alias.destination),
                enabled: Some(alias.enabled),
                created: alias.created,
                modified: alias.modified,
                match_fields,
            });
        }
    }

    // Search backups
    if resource_types.is_none()
        || resource_types
            .as_ref()
            .unwrap()
            .contains(&"backup".to_string())
    {
        let backups = backups::table
            .filter(
                backups::domain
                    .like(&search_pattern)
                    .or(backups::transport.like(&search_pattern)),
            )
            .load::<Backup>(&mut conn)?;

        for backup in backups {
            if enabled_only.is_some() && enabled_only.unwrap() && !backup.enabled {
                continue;
            }

            let mut match_fields = Vec::new();
            if backup.domain.contains(query) {
                match_fields.push("domain".to_string());
            }
            if backup
                .transport
                .as_ref()
                .map_or(false, |t| t.contains(query))
            {
                match_fields.push("transport".to_string());
            }

            all_results.push(SearchResult {
                resource_type: "backup".to_string(),
                resource_id: backup.pkid.to_string(),
                resource_name: backup.domain.clone(),
                resource_description: format!(
                    "Transport: {}",
                    backup.transport.as_deref().unwrap_or("N/A")
                ),
                enabled: Some(backup.enabled),
                created: backup.created,
                modified: backup.modified,
                match_fields,
            });
        }
    }

    // Search relays
    if resource_types.is_none()
        || resource_types
            .as_ref()
            .unwrap()
            .contains(&"relay".to_string())
    {
        let relays = relays::table
            .filter(
                relays::recipient
                    .like(&search_pattern)
                    .or(relays::status.like(&search_pattern)),
            )
            .load::<Relay>(&mut conn)?;

        for relay in relays {
            if enabled_only.is_some() && enabled_only.unwrap() && !relay.enabled {
                continue;
            }

            let mut match_fields = Vec::new();
            if relay.recipient.contains(query) {
                match_fields.push("recipient".to_string());
            }
            if relay.status.contains(query) {
                match_fields.push("status".to_string());
            }

            all_results.push(SearchResult {
                resource_type: "relay".to_string(),
                resource_id: relay.pkid.to_string(),
                resource_name: relay.recipient.clone(),
                resource_description: format!("Status: {}", relay.status),
                enabled: Some(relay.enabled),
                created: relay.created,
                modified: relay.modified,
                match_fields,
            });
        }
    }

    // Search relocated
    if resource_types.is_none()
        || resource_types
            .as_ref()
            .unwrap()
            .contains(&"relocated".to_string())
    {
        let relocated = relocated::table
            .filter(
                relocated::old_address
                    .like(&search_pattern)
                    .or(relocated::new_address.like(&search_pattern)),
            )
            .load::<Relocated>(&mut conn)?;

        for rel in relocated {
            if enabled_only.is_some() && enabled_only.unwrap() && !rel.enabled {
                continue;
            }

            let mut match_fields = Vec::new();
            if rel.old_address.contains(query) {
                match_fields.push("old_address".to_string());
            }
            if rel.new_address.contains(query) {
                match_fields.push("new_address".to_string());
            }

            all_results.push(SearchResult {
                resource_type: "relocated".to_string(),
                resource_id: rel.pkid.to_string(),
                resource_name: format!("{} → {}", rel.old_address, rel.new_address),
                resource_description: "Address relocation".to_string(),
                enabled: Some(rel.enabled),
                created: rel.created,
                modified: rel.modified,
                match_fields,
            });
        }
    }

    // Search clients
    if resource_types.is_none()
        || resource_types
            .as_ref()
            .unwrap()
            .contains(&"client".to_string())
    {
        let clients = clients::table
            .filter(
                clients::client
                    .like(&search_pattern)
                    .or(clients::status.like(&search_pattern)),
            )
            .load::<Client>(&mut conn)?;

        for client in clients {
            if enabled_only.is_some() && enabled_only.unwrap() && !client.enabled {
                continue;
            }

            let mut match_fields = Vec::new();
            if client.client.contains(query) {
                match_fields.push("client".to_string());
            }
            if client.status.contains(query) {
                match_fields.push("status".to_string());
            }

            all_results.push(SearchResult {
                resource_type: "client".to_string(),
                resource_id: client.id.to_string(),
                resource_name: client.client.clone(),
                resource_description: format!("Status: {}", client.status),
                enabled: Some(client.enabled),
                created: client.created_at,
                modified: client.updated_at,
                match_fields,
            });
        }
    }

    // Sort by relevance (more match fields = higher relevance)
    all_results.sort_by(|a, b| b.match_fields.len().cmp(&a.match_fields.len()));

    let total_count = all_results.len();
    all_results.truncate(limit as usize);

    let search_time = start_time.elapsed().as_millis() as u64;

    Ok(SearchResults {
        query: query.to_string(),
        results: all_results,
        total_count,
        search_time_ms: search_time,
    })
}

/// Get MX servers report for all domains
pub async fn get_mx_servers_report(
    pool: &DbPool,
    mail_servers: &[String],
    page: i64,
    per_page: i64,
    sort_by: &str,
    sort_order: &str,
    exclude_disabled: bool,
    exclude_subdomains: bool,
    filter_status: Option<MxStatus>,
) -> Result<crate::models::PaginatedResult<DomainMxStatus>, Box<dyn std::error::Error + Send + Sync>>
{
    use crate::schema::domains;
    use crate::services::dns_lookup::DnsLookupService;

    let mut conn = pool.get()?;

    // Build base query with filters
    let mut query = domains::table.into_boxed();

    if exclude_disabled {
        query = query.filter(domains::enabled.eq(true));
    }

    if exclude_subdomains {
        query = query.filter(domains::domain.not_like("%.%.%"));
    }

    // If we have a status filter, we need to fetch all domains to check their status
    // Otherwise, use efficient pagination
    let (domains_list, base_total_count) = if filter_status.is_some() {
        // Fetch all domains matching the basic filters
        let domains = match (sort_by, sort_order) {
            ("domain", "asc") => query
                .order(domains::domain.asc())
                .load::<Domain>(&mut conn)
                .map_err(|e| format!("Failed to load domains: {}", e))?,
            ("domain", "desc") => query
                .order(domains::domain.desc())
                .load::<Domain>(&mut conn)
                .map_err(|e| format!("Failed to load domains: {}", e))?,
            ("enabled", "asc") => query
                .order(domains::enabled.asc())
                .load::<Domain>(&mut conn)
                .map_err(|e| format!("Failed to load domains: {}", e))?,
            ("enabled", "desc") => query
                .order(domains::enabled.desc())
                .load::<Domain>(&mut conn)
                .map_err(|e| format!("Failed to load domains: {}", e))?,
            _ => query
                .order(domains::domain.asc())
                .load::<Domain>(&mut conn)
                .map_err(|e| format!("Failed to load domains: {}", e))?,
        };
        let count = domains.len() as i64;
        (domains, count)
    } else {
        // Get total count first for pagination
        let mut count_query = domains::table.into_boxed();
        if exclude_disabled {
            count_query = count_query.filter(domains::enabled.eq(true));
        }
        if exclude_subdomains {
            count_query = count_query.filter(domains::domain.not_like("%.%.%"));
        }
        let total = count_query
            .count()
            .get_result(&mut conn)
            .map_err(|e| format!("Failed to count domains: {}", e))?;

        // Use pagination for better performance when no status filter
        let offset = (page - 1) * per_page;

        let mut query_for_data = domains::table.into_boxed();

        if exclude_disabled {
            query_for_data = query_for_data.filter(domains::enabled.eq(true));
        }

        if exclude_subdomains {
            query_for_data = query_for_data.filter(domains::domain.not_like("%.%.%"));
        }

        let domains = match (sort_by, sort_order) {
            ("domain", "asc") => query_for_data
                .order(domains::domain.asc())
                .limit(per_page)
                .offset(offset)
                .load::<Domain>(&mut conn)
                .map_err(|e| format!("Failed to load domains: {}", e))?,
            ("domain", "desc") => query_for_data
                .order(domains::domain.desc())
                .limit(per_page)
                .offset(offset)
                .load::<Domain>(&mut conn)
                .map_err(|e| format!("Failed to load domains: {}", e))?,
            ("enabled", "asc") => query_for_data
                .order(domains::enabled.asc())
                .limit(per_page)
                .offset(offset)
                .load::<Domain>(&mut conn)
                .map_err(|e| format!("Failed to load domains: {}", e))?,
            ("enabled", "desc") => query_for_data
                .order(domains::enabled.desc())
                .limit(per_page)
                .offset(offset)
                .load::<Domain>(&mut conn)
                .map_err(|e| format!("Failed to load domains: {}", e))?,
            _ => query_for_data
                .order(domains::domain.asc())
                .limit(per_page)
                .offset(offset)
                .load::<Domain>(&mut conn)
                .map_err(|e| format!("Failed to load domains: {}", e))?,
        };
        (domains, total)
    };

    // Initialize DNS lookup service
    let dns_service = match DnsLookupService::new_system().await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize DNS lookup service: {:?}", e);
            return Err(format!("Failed to initialize DNS lookup service: {}", e).into());
        }
    };

    // Check which domains are backup domains
    use crate::schema::backups;
    let backup_domains: Vec<String> = backups::table
        .select(backups::domain)
        .load::<String>(&mut conn)
        .unwrap_or_default();

    let mut domains_status = Vec::new();

    for domain in &domains_list {
        let is_backup = backup_domains.contains(&domain.domain);
        // Lookup MX records for the domain
        let mx_records = match dns_service.lookup_mx(&domain.domain).await {
            Ok(records) => records
                .into_iter()
                .map(|r| r.exchange)
                .collect::<Vec<String>>(),
            Err(_) => {
                // DNS lookup failed
                domains_status.push(DomainMxStatus {
                    domain: domain.domain.clone(),
                    domain_id: domain.pkid,
                    enabled: domain.enabled,
                    is_backup,
                    mx_records: Vec::new(),
                    mx_status: MxStatus::Error,
                    missing_servers: Vec::new(),
                    unexpected_servers: Vec::new(),
                });
                continue;
            }
        };

        if mx_records.is_empty() {
            // No MX records found
            domains_status.push(DomainMxStatus {
                domain: domain.domain.clone(),
                domain_id: domain.pkid,
                enabled: domain.enabled,
                is_backup,
                mx_records: Vec::new(),
                mx_status: MxStatus::Empty,
                missing_servers: mail_servers.to_vec(),
                unexpected_servers: Vec::new(),
            });
            continue;
        }

        // Normalize MX records and mail servers by removing trailing dots
        let normalized_mx_records: Vec<String> = mx_records
            .iter()
            .map(|mx| mx.trim_end_matches('.').to_string())
            .collect();

        let normalized_mail_servers: Vec<String> = mail_servers
            .iter()
            .map(|server| server.trim_end_matches('.').to_string())
            .collect();

        // Check if ANY of the configured mail servers are found in MX records
        let mut missing_servers = Vec::new();
        let mut unexpected_servers = Vec::new();
        let mut is_compliant = false;

        // Check for missing mail servers (configured servers not in MX records)
        for mail_server in &normalized_mail_servers {
            if !normalized_mx_records
                .iter()
                .any(|mx| mx.ends_with(mail_server))
            {
                missing_servers.push(mail_server.clone());
            } else {
                // At least one configured server is found, so it's compliant
                is_compliant = true;
            }
        }

        // Check for unexpected servers (MX records not pointing to configured servers)
        for mx_record in &normalized_mx_records {
            if !normalized_mail_servers
                .iter()
                .any(|server| mx_record.ends_with(server))
            {
                unexpected_servers.push(mx_record.clone());
            }
        }

        let mx_status = if is_compliant {
            MxStatus::Compliant
        } else {
            MxStatus::NonCompliant
        };

        domains_status.push(DomainMxStatus {
            domain: domain.domain.clone(),
            domain_id: domain.pkid,
            enabled: domain.enabled,
            is_backup,
            mx_records: normalized_mx_records,
            mx_status,
            missing_servers,
            unexpected_servers,
        });
    }

    // Apply status filter if specified and calculate final count
    let (final_items, total_count) = if let Some(status_filter) = filter_status {
        // Filter by status
        domains_status.retain(|d| d.mx_status == status_filter);
        let filtered_count = domains_status.len() as i64;

        // Calculate pagination for filtered results
        let offset = ((page - 1) * per_page) as usize;
        let end = (offset + per_page as usize).min(domains_status.len());

        let paginated_items = if offset < domains_status.len() {
            domains_status[offset..end].to_vec()
        } else {
            Vec::new()
        };

        (paginated_items, filtered_count)
    } else {
        // No status filter, domains_status already contains only the current page
        (domains_status, base_total_count)
    };

    Ok(crate::models::PaginatedResult::new(
        final_items,
        total_count,
        page,
        per_page,
    ))
}
