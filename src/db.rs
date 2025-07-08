use crate::models::*;
use crate::schema::*;
use crate::DbPool;
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error;

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

pub fn get_user(pool: &DbPool, user_id: i32) -> Result<User, Error> {
    let mut conn = pool.get().unwrap();
    users::table
        .find(user_id)
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
        .order(users::pkid.desc())
        .select(User::as_select())
        .first::<User>(&mut conn)
}

pub fn update_user(pool: &DbPool, user_id: i32, user_data: UserForm) -> Result<User, Error> {
    let mut conn = pool.get().unwrap();

    if !user_data.password.is_empty() {
        let hashed_password = bcrypt::hash(user_data.password.as_bytes(), bcrypt::DEFAULT_COST)
            .map_err(|e| {
                Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::Unknown,
                    Box::new(e.to_string()),
                )
            })?;

        diesel::update(users::table.find(user_id))
            .set((
                users::id.eq(user_data.id),
                users::name.eq(user_data.name),
                users::enabled.eq(user_data.enabled),
                users::modified.eq(Utc::now().naive_utc()),
                users::crypt.eq(hashed_password),
            ))
            .execute(&mut conn)?;
    } else {
        diesel::update(users::table.find(user_id))
            .set((
                users::id.eq(user_data.id),
                users::name.eq(user_data.name),
                users::enabled.eq(user_data.enabled),
                users::modified.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut conn)?;
    }

    get_user(pool, user_id)
}

pub fn delete_user(pool: &DbPool, user_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    diesel::delete(users::table.find(user_id)).execute(&mut conn)
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

pub fn toggle_user_enabled(pool: &DbPool, user_id: i32) -> Result<User, Error> {
    let mut conn = pool.get().unwrap();

    // First get the current user to check its enabled status
    let current_user = get_user(pool, user_id)?;
    let new_enabled_status = !current_user.enabled;

    diesel::update(users::table.find(user_id))
        .set((
            users::enabled.eq(new_enabled_status),
            users::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;

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
    let mut conn = pool.get().unwrap();

    let total_domains: i64 = domains::table.count().get_result(&mut conn)?;
    let total_users: i64 = users::table.count().get_result(&mut conn)?;
    let total_aliases: i64 = aliases::table.count().get_result(&mut conn)?;
    let total_backups: i64 = backups::table.count().get_result(&mut conn)?;
    let total_relays: i64 = relays::table.count().get_result(&mut conn)?;
    let total_relocated: i64 = relocated::table.count().get_result(&mut conn)?;
    let total_clients: i64 = clients::table.count().get_result(&mut conn)?;

    let total_quota: i64 = 0; // Quota field removed from users table

    Ok(SystemStats {
        total_domains,
        total_users,
        total_aliases,
        total_backups,
        total_relays,
        total_relocated,
        total_clients,
        total_quota,
        used_quota: 0, // This would need to be calculated from actual disk usage
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

pub fn update_relay(
    pool: &DbPool,
    relay_id: i32,
    relay_data: RelayForm,
) -> Result<Relay, Error> {
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
            .filter(aliases::mail.like(format!("%@{}", domain)))
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
            .map(|alias| {
                alias.mail.split('@').next().unwrap_or("").to_string()
            })
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
pub fn get_domain_alias_report(pool: &DbPool, domain_name: &str) -> Result<DomainAliasReport, Error> {
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
        .filter(aliases::mail.eq(format!("@{}", domain_name)))
        .filter(aliases::enabled.eq(true))
        .select(Alias::as_select())
        .first::<Alias>(&mut conn)
        .optional()?;

    // Get all aliases for this domain
    let domain_aliases = aliases::table
        .filter(aliases::mail.like(format!("%@{}", domain_name)))
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
        .map(|alias| {
            alias.mail.split('@').next().unwrap_or("").to_string()
        })
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
pub fn create_domain_aliases(pool: &DbPool, domain: &str, aliases: Vec<(String, String)>) -> Result<Vec<Alias>, Error> {
    let mut conn = pool.get().unwrap();
    let now = Utc::now().naive_utc();
    let mut created_aliases = Vec::new();

    for (local_part, destination) in aliases {
        let mail = if local_part == "@" {
            format!("@{}", domain)
        } else {
            format!("{}@{}", local_part, domain)
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
        .filter(aliases::mail.like(format!("%@{}", domain_name)))
        .select(Alias::as_select())
        .order(aliases::mail.asc())
        .load::<Alias>(&mut conn)
}
