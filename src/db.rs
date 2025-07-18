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
use tokio::sync::RwLock;

/// Manages multiple database connections
#[derive(Clone)]
pub struct DatabaseManager {
    pools: Arc<RwLock<HashMap<String, DbPool>>>,
    configs: Vec<DatabaseConfig>,
    default_db: String,
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
            let pool = r2d2::Pool::builder()
                .build(manager)
                .map_err(|e| format!("Failed to create pool for {}: {}", config.id, e))?;

            pools.insert(config.id.clone(), pool);
        }

        Ok(DatabaseManager {
            pools: Arc::new(RwLock::new(pools)),
            configs,
            default_db,
        })
    }

    /// Get a database pool by ID
    pub async fn get_pool(&self, db_id: &str) -> Option<DbPool> {
        let pools = self.pools.read().await;
        pools.get(db_id).cloned()
    }

    /// Get the default database pool
    pub async fn get_default_pool(&self) -> Option<DbPool> {
        self.get_pool(&self.default_db).await
    }

    /// Get all available database configurations
    pub fn get_configs(&self) -> &[DatabaseConfig] {
        &self.configs
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

    /// Run migrations on all configured databases
    pub async fn run_migrations_on_all_databases(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

        let pools = self.pools.read().await;

        for config in &self.configs {
            if let Some(pool) = pools.get(&config.id) {
                tracing::info!("Running migrations on database: {}", config.id);

                match pool.get() {
                    Ok(mut conn) => match conn.run_pending_migrations(MIGRATIONS) {
                        Ok(_) => tracing::info!(
                            "✅ Migrations completed successfully for database: {}",
                            config.id
                        ),
                        Err(e) => {
                            tracing::error!(
                                "❌ Failed to run migrations on database {}: {}",
                                config.id,
                                e
                            );
                            return Err(format!(
                                "Failed to run migrations on database {}: {}",
                                config.id, e
                            )
                            .into());
                        }
                    },
                    Err(e) => {
                        tracing::error!(
                            "❌ Failed to get connection for database {}: {}",
                            config.id,
                            e
                        );
                        return Err(format!(
                            "Failed to get connection for database {}: {}",
                            config.id, e
                        )
                        .into());
                    }
                }
            } else {
                tracing::warn!("⚠️  No pool found for database: {}", config.id);
            }
        }

        tracing::info!("✅ Migrations completed on all databases");
        Ok(())
    }

    /// Run migrations on a specific database
    pub async fn run_migrations_on_database(
        &self,
        db_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

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
    let mut conn = pool.get().unwrap();
    domains::table
        .find(domain_id)
        .select(Domain::as_select())
        .first::<Domain>(&mut conn)
}

pub fn get_domain_by_name(pool: &DbPool, domain_name: &str) -> Result<Domain, Error> {
    let mut conn = pool.get().unwrap();
    domains::table
        .filter(domains::domain.eq(domain_name))
        .select(Domain::as_select())
        .first::<Domain>(&mut conn)
}

pub fn create_domain(pool: &DbPool, new_domain: NewDomain) -> Result<Domain, Error> {
    let mut conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();

    diesel::insert_into(domains::table)
        .values((
            domains::domain.eq(new_domain.domain),
            domains::transport.eq(new_domain.transport.clone()),
            domains::enabled.eq(new_domain.enabled),
            domains::created.eq(now),
            domains::modified.eq(now),
        ))
        .execute(&mut conn)?;

    domains::table
        .order(domains::pkid.desc())
        .select(Domain::as_select())
        .first::<Domain>(&mut conn)
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
            domains::transport.eq(domain_data.transport.clone()),
            domains::enabled.eq(domain_data.enabled),
            domains::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    get_domain(pool, domain_id)
}

pub fn delete_domain(pool: &DbPool, domain_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    diesel::delete(domains::table.find(domain_id)).execute(&mut conn)
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
            users::created.eq(now),
            users::modified.eq(now),
        ))
        .execute(&mut conn)?;

    users::table
        .order(users::id.desc())
        .select(User::as_select())
        .first::<User>(&mut conn)
}

pub fn update_user(pool: &DbPool, user_id: String, user_data: UserForm) -> Result<User, Error> {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().unwrap();

    // First get the current user to preserve unchanged fields
    let _current_user = get_user(pool, user_id.clone())?;

    // Update the user - include id if it's different from the current one
    if user_data.id != user_id {
        diesel::update(users.filter(id.eq(user_id.clone())))
            .set((
                id.eq(user_data.id.clone()),
                name.eq(user_data.name),
                enabled.eq(user_data.enabled),
                change_password.eq(user_data.change_password),
            ))
            .execute(&mut conn)?;
    } else {
        diesel::update(users.filter(id.eq(user_id.clone())))
            .set((
                name.eq(user_data.name),
                enabled.eq(user_data.enabled),
                change_password.eq(user_data.change_password),
            ))
            .execute(&mut conn)?;
    }

    // Return the updated user using the new ID if it changed
    let final_user_id = if user_data.id != user_id {
        user_data.id
    } else {
        user_id
    };
    get_user(pool, final_user_id)
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
        .set(crypt.eq(hashed_password))
        .execute(&mut conn)?;

    Ok(())
}

pub fn delete_user(pool: &DbPool, user_id: String) -> Result<usize, Error> {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().unwrap();

    diesel::delete(users.filter(id.eq(user_id))).execute(&mut conn)
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

    aliases::table
        .order(aliases::pkid.desc())
        .select(Alias::as_select())
        .first::<Alias>(&mut conn)
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
    diesel::delete(aliases::table.find(alias_id)).execute(&mut conn)
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

    get_domain(pool, domain_id)
}

pub fn toggle_user_enabled(pool: &DbPool, user_id: String) -> Result<User, Error> {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().unwrap();

    // Get current user
    let current_user = get_user(pool, user_id.clone())?;

    // Toggle the enabled status
    diesel::update(users.filter(id.eq(user_id.clone())))
        .set(enabled.eq(!current_user.enabled))
        .execute(&mut conn)?;

    // Return the updated user
    get_user(pool, user_id)
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

    get_alias(pool, alias_id)
}

// Statistics functions
pub fn get_system_stats(pool: &DbPool) -> Result<SystemStats, Error> {
    use chrono::Duration;
    use chrono::Utc;
    let mut conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();
    let week_ago = now - Duration::days(7);

    // Domains
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
        .filter(domains::created.ge(week_ago))
        .count()
        .get_result(&mut conn)?;

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
        .filter(users::created.ge(week_ago))
        .count()
        .get_result(&mut conn)?;

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
        .filter(aliases::created.ge(week_ago))
        .count()
        .get_result(&mut conn)?;

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
        .filter(backups::created.ge(week_ago))
        .count()
        .get_result(&mut conn)?;

    // Relays
    let total_relays: i64 = relays::table.count().get_result(&mut conn)?;
    let enabled_relays: i64 = relays::table
        .filter(relays::enabled.eq(true))
        .count()
        .get_result(&mut conn)?;
    let disabled_relays: i64 = relays::table
        .filter(relays::enabled.eq(false))
        .count()
        .get_result(&mut conn)?;
    let recent_relays: i64 = relays::table
        .filter(relays::created.ge(week_ago))
        .count()
        .get_result(&mut conn)?;

    // Relocated
    let total_relocated: i64 = relocated::table.count().get_result(&mut conn)?;
    let enabled_relocated: i64 = relocated::table
        .filter(relocated::enabled.eq(true))
        .count()
        .get_result(&mut conn)?;
    let disabled_relocated: i64 = relocated::table
        .filter(relocated::enabled.eq(false))
        .count()
        .get_result(&mut conn)?;
    let recent_relocated: i64 = relocated::table
        .filter(relocated::created.ge(week_ago))
        .count()
        .get_result(&mut conn)?;

    // Clients
    let total_clients: i64 = clients::table.count().get_result(&mut conn)?;
    let enabled_clients: i64 = clients::table
        .filter(clients::enabled.eq(true))
        .count()
        .get_result(&mut conn)?;
    let disabled_clients: i64 = clients::table
        .filter(clients::enabled.eq(false))
        .count()
        .get_result(&mut conn)?;
    let recent_clients: i64 = clients::table
        .filter(clients::created_at.ge(week_ago))
        .count()
        .get_result(&mut conn)?;

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

        let total_quota: i64 = 0; // Quota field removed from users table

        stats.push(DomainStats {
            domain: domain.domain,
            user_count,
            alias_count,
            total_quota,
            used_quota: 0, // This would need to be calculated from actual disk usage
        });
    }

    Ok(stats)
}

// Backup functions
pub fn get_backups(pool: &DbPool) -> Result<Vec<Backup>, Error> {
    let mut conn = pool.get().unwrap();
    backups::table
        .select(Backup::as_select())
        .order(backups::domain.asc())
        .load::<Backup>(&mut conn)
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
            backups::transport.eq(new_backup.transport.clone()),
            backups::enabled.eq(new_backup.enabled),
            backups::created.eq(now),
            backups::modified.eq(now),
        ))
        .execute(&mut conn)?;

    backups::table
        .order(backups::pkid.desc())
        .select(Backup::as_select())
        .first::<Backup>(&mut conn)
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
            backups::transport.eq(backup_data.transport.clone()),
            backups::enabled.eq(backup_data.enabled),
            backups::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

    get_backup(pool, backup_id)
}

pub fn delete_backup(pool: &DbPool, backup_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    diesel::delete(backups::table.find(backup_id)).execute(&mut conn)
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

    get_backup(pool, backup_id)
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

// Enhanced alias report functions
pub fn get_alias_report(pool: &DbPool) -> Result<AliasReport, Error> {
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
    let mut domains_with_catch_all = Vec::new();
    let mut domains_without_catch_all = Vec::new();

    for domain in domains {
        // Check if this domain has a catch-all alias
        let catch_all_alias = aliases::table
            .filter(aliases::mail.eq(format!("@{}", domain.domain)))
            .filter(aliases::enabled.eq(true))
            .select(Alias::as_select())
            .first::<Alias>(&mut conn)
            .optional()?;

        // Get all aliases for this domain
        let domain_aliases = aliases::table
            .filter(aliases::mail.like(format!("%@{}", domain.domain)))
            .filter(aliases::enabled.eq(true))
            .select(Alias::as_select())
            .load::<Alias>(&mut conn)?;

        // Convert to RequiredAlias format
        let required_aliases: Vec<RequiredAlias> = domain_aliases
            .iter()
            .map(|alias| RequiredAlias {
                mail: alias.mail.clone(),
                destination: alias.destination.clone(),
                enabled: alias.enabled,
            })
            .collect();

        // Get required aliases for this specific domain
        let domain_required_aliases = config.get_required_aliases_for_domain(&domain.domain);
        let domain_common_aliases = config.get_common_aliases_for_domain(&domain.domain);
        let _domain_all_aliases = config.get_all_aliases_for_domain(&domain.domain);

        // Find missing required aliases
        let existing_aliases: std::collections::HashSet<String> = domain_aliases
            .iter()
            .map(|alias| alias.mail.split('@').next().unwrap_or("").to_string())
            .collect();

        let missing_required_aliases: Vec<String> = domain_required_aliases
            .iter()
            .filter(|required| !existing_aliases.contains(*required))
            .cloned()
            .collect();

        let missing_common_aliases: Vec<String> = domain_common_aliases
            .iter()
            .filter(|common| !existing_aliases.contains(*common))
            .cloned()
            .collect();

        let domain_report = DomainAliasReport {
            domain: domain.domain,
            has_catch_all: catch_all_alias.is_some(),
            catch_all_alias: catch_all_alias.as_ref().map(|ca| ca.mail.clone()),
            catch_all_destination: catch_all_alias.as_ref().map(|ca| ca.destination.clone()),
            required_aliases,
            missing_required_aliases,
            missing_common_aliases,
        };

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

    // Check if this domain has a catch-all alias
    let catch_all_alias = aliases::table
        .filter(aliases::mail.eq(format!("@{domain_name}")))
        .filter(aliases::enabled.eq(true))
        .select(Alias::as_select())
        .first::<Alias>(&mut conn)
        .optional()?;

    // Get all aliases for this domain
    let domain_aliases = aliases::table
        .filter(aliases::mail.like(format!("%@{domain_name}")))
        .filter(aliases::enabled.eq(true))
        .select(Alias::as_select())
        .load::<Alias>(&mut conn)?;

    // Convert to RequiredAlias format and sort by mail
    let mut required_aliases: Vec<RequiredAlias> = domain_aliases
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

    // Find missing required aliases
    let existing_aliases: std::collections::HashSet<String> = domain_aliases
        .iter()
        .map(|alias| alias.mail.split('@').next().unwrap_or("").to_string())
        .collect();

    let mut missing_required_aliases: Vec<String> = domain_required_aliases
        .iter()
        .filter(|required| !existing_aliases.contains(*required))
        .cloned()
        .collect();
    missing_required_aliases.sort();

    let mut missing_common_aliases: Vec<String> = domain_common_aliases
        .iter()
        .filter(|common| !existing_aliases.contains(*common))
        .cloned()
        .collect();
    missing_common_aliases.sort();

    Ok(DomainAliasReport {
        domain: domain_name.to_string(),
        has_catch_all: catch_all_alias.is_some(),
        catch_all_alias: catch_all_alias.as_ref().map(|ca| ca.mail.clone()),
        catch_all_destination: catch_all_alias.as_ref().map(|ca| ca.destination.clone()),
        required_aliases,
        missing_required_aliases,
        missing_common_aliases,
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

pub fn search_aliases(pool: &DbPool, query: &str, limit: i64) -> Result<Vec<Alias>, Error> {
    let mut conn = pool.get().unwrap();
    let search_pattern = format!("%{}%", query);

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
    let search_pattern = format!("{}%@%", query);

    aliases::table
        .filter(aliases::mail.like(&search_pattern))
        .select(Alias::as_select())
        .order(aliases::mail.asc())
        .limit(limit)
        .load::<Alias>(&mut conn)
}

pub fn search_domains(pool: &DbPool, query: &str, limit: i64) -> Result<Vec<Domain>, Error> {
    let mut conn = pool.get().unwrap();
    let search_pattern = format!("%{}%", query);

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
) -> Result<PaginatedResult<Domain>, Error> {
    let mut conn = pool.get().map_err(|e| {
        tracing::error!("Failed to get connection from pool: {:?}", e);
        Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    let offset = (page - 1) * per_page;

    // Get total count
    let total_count: i64 = domains::table.count().get_result(&mut conn)?;

    // Get paginated results
    let domains = domains::table
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
) -> Result<PaginatedResult<Alias>, Error> {
    let mut conn = pool.get().unwrap();

    let offset = (page - 1) * per_page;

    // Get total count
    let total_count: i64 = aliases::table.count().get_result(&mut conn)?;

    // Get paginated results
    let aliases = aliases::table
        .select(Alias::as_select())
        .order(aliases::mail.asc())
        .limit(per_page)
        .offset(offset)
        .load::<Alias>(&mut conn)?;

    Ok(PaginatedResult::new(aliases, total_count, page, per_page))
}

pub fn get_users_paginated(
    pool: &DbPool,
    page: i64,
    per_page: i64,
) -> Result<PaginatedResult<User>, Error> {
    let mut conn = pool.get().unwrap();

    let offset = (page - 1) * per_page;

    // Get total count
    let total_count: i64 = users::table.count().get_result(&mut conn)?;

    // Get paginated results
    let users = users::table
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
) -> Result<PaginatedResult<Client>, Error> {
    let mut conn = pool.get().unwrap();

    let offset = (page - 1) * per_page;

    // Get total count
    let total_count: i64 = clients::table.count().get_result(&mut conn)?;

    // Get paginated results
    let clients = clients::table
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
) -> Result<PaginatedResult<Relay>, Error> {
    let mut conn = pool.get().unwrap();

    let offset = (page - 1) * per_page;

    // Get total count
    let total_count: i64 = relays::table.count().get_result(&mut conn)?;

    // Get paginated results
    let relays = relays::table
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
) -> Result<PaginatedResult<Relocated>, Error> {
    let mut conn = pool.get().unwrap();

    let offset = (page - 1) * per_page;

    // Get total count
    let total_count: i64 = relocated::table.count().get_result(&mut conn)?;

    // Get paginated results
    let relocated = relocated::table
        .select(Relocated::as_select())
        .order(relocated::old_address.asc())
        .limit(per_page)
        .offset(offset)
        .load::<Relocated>(&mut conn)?;

    Ok(PaginatedResult::new(relocated, total_count, page, per_page))
}

// Additional report functions
pub fn get_orphaned_aliases_report(pool: &DbPool) -> Result<OrphanedAliasReport, Error> {
    let mut conn = pool.get().unwrap();

    // Find aliases where the mail domain doesn't exist in the domains table
    let orphaned_aliases: Vec<OrphanedAlias> = aliases::table
        .select((
            aliases::mail,
            aliases::destination,
            aliases::enabled,
            aliases::created,
        ))
        .load::<(String, String, bool, NaiveDateTime)>(&mut conn)?
        .into_iter()
        .filter(|(mail, _, _, _)| {
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
                domain_exists.map_or(true, |(_, enabled)| !enabled)
            } else {
                false
            }
        })
        .map(|(mail, destination, enabled, created)| {
            let domain = mail.split('@').nth(1).unwrap_or("").to_string();
            OrphanedAlias {
                mail,
                destination,
                domain,
                enabled,
                created,
            }
        })
        .collect();

    // Find users where the domain doesn't exist or is disabled in the domains table
    let orphaned_users: Vec<OrphanedUser> = users::table
        .select((users::id, users::name, users::enabled, users::created))
        .load::<(String, String, bool, NaiveDateTime)>(&mut conn)?
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
                domain_exists.map_or(true, |(_, enabled)| !enabled)
            } else {
                false
            }
        })
        .map(|(id, name, enabled, created)| {
            let domain = id.split('@').nth(1).unwrap_or("").to_string();
            OrphanedUser {
                id,
                name,
                domain,
                enabled,
                created,
            }
        })
        .collect();

    // Find users who don't have a corresponding alias
    let users_without_aliases: Vec<UserWithoutAlias> = users::table
        .select((users::id, users::name, users::enabled, users::created))
        .load::<(String, String, bool, NaiveDateTime)>(&mut conn)?
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
        .map(|(id, name, enabled, created)| {
            let domain = id.split('@').nth(1).unwrap_or("").to_string();
            UserWithoutAlias {
                id,
                name,
                domain,
                enabled,
                created,
            }
        })
        .collect();

    Ok(OrphanedAliasReport {
        orphaned_aliases,
        orphaned_users,
        users_without_aliases,
    })
}

pub fn get_external_forwarders_report(pool: &DbPool) -> Result<ExternalForwarderReport, Error> {
    let mut conn = pool.get().unwrap();

    // Find aliases where the destination is an external email address (contains @ and doesn't match any domain in the domains table)
    let external_forwarders: Vec<ExternalForwarder> = aliases::table
        .filter(aliases::destination.like("%@%"))
        .select((
            aliases::mail,
            aliases::destination,
            aliases::enabled,
            aliases::created,
        ))
        .load::<(String, String, bool, NaiveDateTime)>(&mut conn)?
        .into_iter()
        .filter(|(_, destination, _, _)| {
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
        .map(|(mail, destination, enabled, created)| {
            let domain = mail.split('@').nth(1).unwrap_or("").to_string();
            ExternalForwarder {
                mail,
                destination,
                domain,
                enabled,
                created,
            }
        })
        .collect();

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
            .filter(aliases::mail.eq(format!("@{}", domain)))
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
                .filter(aliases::mail.eq(format!("{}@{}", required_alias, domain)))
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
        .filter(aliases::mail.like(format!("{}@%", alias_name)))
        .select((aliases::mail, aliases::destination, aliases::enabled))
        .load::<(String, String, bool)>(&mut conn)?
        .into_iter()
        .map(|(mail, destination, enabled)| {
            let domain = mail.split('@').nth(1).unwrap_or("").to_string();
            AliasOccurrence {
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

        domain_rows.push(CrossDatabaseDomainRow { domain, presence });
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
        .filter(aliases::mail.like(format!("{}@%", user_id)))
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
        "SELECT {user_id} as id, {enabled} as enabled, {crypt} as crypt, {name} as name, {maildir} as maildir, {home} as home, {uid} as uid, {gid} as gid, {created} as created, {modified} as modified, {change_password} as change_password FROM users",
        user_id = user_id,
        enabled = enabled,
        crypt = crypt,
        name = name,
        maildir = maildir,
        home = home,
        uid = uid,
        gid = gid,
        created = created,
        modified = modified,
        change_password = change_password
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
            format!("{} as {}", mapped_field, alias)
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
