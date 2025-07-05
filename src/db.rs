use crate::models::*;
use crate::schema::*;
use crate::DbPool;
use diesel::prelude::*;
use diesel::result::Error;
use chrono::Utc;
use bigdecimal::{BigDecimal, ToPrimitive};

pub fn get_domains(pool: &DbPool) -> Result<Vec<Domain>, Error> {
    let mut conn = pool.get().unwrap();
    domains::table
        .select(Domain::as_select())
        .order(domains::domain.asc())
        .load::<Domain>(&mut conn)
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
    diesel::insert_into(domains::table)
        .values(&new_domain)
        .execute(&mut conn)?;
    
    domains::table
        .order(domains::id.desc())
        .select(Domain::as_select())
        .first::<Domain>(&mut conn)
}

pub fn update_domain(pool: &DbPool, domain_id: i32, domain_data: DomainForm) -> Result<Domain, Error> {
    let mut conn = pool.get().unwrap();
    diesel::update(domains::table.find(domain_id))
        .set((
            domains::domain.eq(domain_data.domain),
            domains::description.eq(domain_data.description),
            domains::aliases.eq(domain_data.aliases),
            domains::mailboxes.eq(domain_data.mailboxes),
            domains::maxquota.eq(domain_data.maxquota),
            domains::quota.eq(domain_data.quota),
            domains::transport.eq(domain_data.transport),
            domains::backupmx.eq(domain_data.backupmx),
            domains::active.eq(domain_data.active),
            domains::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;
    
    get_domain(pool, domain_id)
}

pub fn delete_domain(pool: &DbPool, domain_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    diesel::delete(domains::table.find(domain_id))
        .execute(&mut conn)
}

pub fn get_users(pool: &DbPool) -> Result<Vec<User>, Error> {
    let mut conn = pool.get().unwrap();
    users::table
        .select(User::as_select())
        .order(users::username.asc())
        .load::<User>(&mut conn)
}

pub fn get_user(pool: &DbPool, user_id: i32) -> Result<User, Error> {
    let mut conn = pool.get().unwrap();
    users::table
        .find(user_id)
        .select(User::as_select())
        .first::<User>(&mut conn)
}

pub fn get_user_by_username(pool: &DbPool, uname: &str) -> Result<User, Error> {
    let mut conn = pool.get().unwrap();
    users::table
        .filter(users::username.eq(uname))
        .select(User::as_select())
        .first::<User>(&mut conn)
}

pub fn create_user(pool: &DbPool, user_data: UserForm) -> Result<User, Error> {
    let mut conn = pool.get().unwrap();
    
    // Hash the password
    let hashed_password = bcrypt::hash(user_data.password.as_bytes(), bcrypt::DEFAULT_COST)
        .map_err(|e| Error::DatabaseError(diesel::result::DatabaseErrorKind::Unknown, Box::new(e.to_string())))?;
    
    let maildir = format!("{}/", user_data.username);
    
    let new_user = NewUser {
        username: user_data.username,
        password: hashed_password,
        name: user_data.name,
        maildir,
        quota: user_data.quota,
        domain: user_data.domain,
        active: user_data.active,
    };
    
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)?;
    
    users::table
        .order(users::id.desc())
        .select(User::as_select())
        .first::<User>(&mut conn)
}

pub fn update_user(pool: &DbPool, user_id: i32, user_data: UserForm) -> Result<User, Error> {
    let mut conn = pool.get().unwrap();
    
    if !user_data.password.is_empty() {
        let hashed_password = bcrypt::hash(user_data.password.as_bytes(), bcrypt::DEFAULT_COST)
            .map_err(|e| Error::DatabaseError(diesel::result::DatabaseErrorKind::Unknown, Box::new(e.to_string())))?;
        
        diesel::update(users::table.find(user_id))
            .set((
                users::username.eq(user_data.username),
                users::name.eq(user_data.name),
                users::domain.eq(user_data.domain),
                users::quota.eq(user_data.quota),
                users::active.eq(user_data.active),
                users::modified.eq(Utc::now().naive_utc()),
                users::password.eq(hashed_password),
            ))
            .execute(&mut conn)?;
    } else {
        diesel::update(users::table.find(user_id))
            .set((
                users::username.eq(user_data.username),
                users::name.eq(user_data.name),
                users::domain.eq(user_data.domain),
                users::quota.eq(user_data.quota),
                users::active.eq(user_data.active),
                users::modified.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut conn)?;
    }
    
    get_user(pool, user_id)
}

pub fn delete_user(pool: &DbPool, user_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    diesel::delete(users::table.find(user_id))
        .execute(&mut conn)
}

pub fn get_aliases(pool: &DbPool) -> Result<Vec<Alias>, Error> {
    let mut conn = pool.get().unwrap();
    aliases::table
        .select(Alias::as_select())
        .order(aliases::address.asc())
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
    
    let new_alias = NewAlias {
        address: alias_data.address,
        goto: alias_data.goto,
        domain: alias_data.domain,
        active: alias_data.active,
    };
    
    diesel::insert_into(aliases::table)
        .values(&new_alias)
        .execute(&mut conn)?;
    
    aliases::table
        .order(aliases::id.desc())
        .select(Alias::as_select())
        .first::<Alias>(&mut conn)
}

pub fn update_alias(pool: &DbPool, alias_id: i32, alias_data: AliasForm) -> Result<Alias, Error> {
    let mut conn = pool.get().unwrap();
    diesel::update(aliases::table.find(alias_id))
        .set((
            aliases::address.eq(alias_data.address),
            aliases::goto.eq(alias_data.goto),
            aliases::domain.eq(alias_data.domain),
            aliases::active.eq(alias_data.active),
            aliases::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;
    
    get_alias(pool, alias_id)
}

pub fn delete_alias(pool: &DbPool, alias_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    diesel::delete(aliases::table.find(alias_id))
        .execute(&mut conn)
}

pub fn get_mailboxes(pool: &DbPool) -> Result<Vec<Mailbox>, Error> {
    let mut conn = pool.get().unwrap();
    mailboxes::table
        .select(Mailbox::as_select())
        .order(mailboxes::username.asc())
        .load::<Mailbox>(&mut conn)
}

pub fn get_mailbox(pool: &DbPool, mailbox_id: i32) -> Result<Mailbox, Error> {
    let mut conn = pool.get().unwrap();
    mailboxes::table
        .find(mailbox_id)
        .select(Mailbox::as_select())
        .first::<Mailbox>(&mut conn)
}

pub fn create_mailbox(pool: &DbPool, mailbox_data: MailboxForm) -> Result<Mailbox, Error> {
    let mut conn = pool.get().unwrap();
    
    // Hash the password
    let hashed_password = bcrypt::hash(mailbox_data.password.as_bytes(), bcrypt::DEFAULT_COST)
        .map_err(|e| Error::DatabaseError(diesel::result::DatabaseErrorKind::Unknown, Box::new(e.to_string())))?;
    
    let maildir = format!("{}/", mailbox_data.username);
    
    let new_mailbox = NewMailbox {
        username: mailbox_data.username,
        password: hashed_password,
        name: mailbox_data.name,
        maildir,
        quota: mailbox_data.quota,
        domain: mailbox_data.domain,
        active: mailbox_data.active,
    };
    
    diesel::insert_into(mailboxes::table)
        .values(&new_mailbox)
        .execute(&mut conn)?;
    
    mailboxes::table
        .order(mailboxes::id.desc())
        .select(Mailbox::as_select())
        .first::<Mailbox>(&mut conn)
}

pub fn update_mailbox(pool: &DbPool, mailbox_id: i32, mailbox_data: MailboxForm) -> Result<Mailbox, Error> {
    let mut conn = pool.get().unwrap();
    
    if !mailbox_data.password.is_empty() {
        let hashed_password = bcrypt::hash(mailbox_data.password.as_bytes(), bcrypt::DEFAULT_COST)
            .map_err(|e| Error::DatabaseError(diesel::result::DatabaseErrorKind::Unknown, Box::new(e.to_string())))?;

        diesel::update(mailboxes::table.find(mailbox_id))
            .set((
                mailboxes::username.eq(mailbox_data.username),
                mailboxes::name.eq(mailbox_data.name),
                mailboxes::domain.eq(mailbox_data.domain),
                mailboxes::quota.eq(mailbox_data.quota),
                mailboxes::active.eq(mailbox_data.active),
                mailboxes::modified.eq(Utc::now().naive_utc()),
                mailboxes::password.eq(hashed_password),
            ))
            .execute(&mut conn)?;
    } else {
        diesel::update(mailboxes::table.find(mailbox_id))
            .set((
                mailboxes::username.eq(mailbox_data.username),
                mailboxes::name.eq(mailbox_data.name),
                mailboxes::domain.eq(mailbox_data.domain),
                mailboxes::quota.eq(mailbox_data.quota),
                mailboxes::active.eq(mailbox_data.active),
                mailboxes::modified.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut conn)?;
    }
    
    get_mailbox(pool, mailbox_id)
}

pub fn delete_mailbox(pool: &DbPool, mailbox_id: i32) -> Result<usize, Error> {
    let mut conn = pool.get().unwrap();
    diesel::delete(mailboxes::table.find(mailbox_id))
        .execute(&mut conn)
}

// Toggle functions for enable/disable functionality
pub fn toggle_domain_active(pool: &DbPool, domain_id: i32) -> Result<Domain, Error> {
    let mut conn = pool.get().unwrap();
    
    // First get the current domain to check its active status
    let current_domain = get_domain(pool, domain_id)?;
    let new_active_status = !current_domain.active;
    
    diesel::update(domains::table.find(domain_id))
        .set((
            domains::active.eq(new_active_status),
            domains::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;
    
    get_domain(pool, domain_id)
}

pub fn toggle_user_active(pool: &DbPool, user_id: i32) -> Result<User, Error> {
    let mut conn = pool.get().unwrap();
    
    // First get the current user to check its active status
    let current_user = get_user(pool, user_id)?;
    let new_active_status = !current_user.active;
    
    diesel::update(users::table.find(user_id))
        .set((
            users::active.eq(new_active_status),
            users::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;
    
    get_user(pool, user_id)
}

pub fn toggle_alias_active(pool: &DbPool, alias_id: i32) -> Result<Alias, Error> {
    let mut conn = pool.get().unwrap();
    
    // First get the current alias to check its active status
    let current_alias = get_alias(pool, alias_id)?;
    let new_active_status = !current_alias.active;
    
    diesel::update(aliases::table.find(alias_id))
        .set((
            aliases::active.eq(new_active_status),
            aliases::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;
    
    get_alias(pool, alias_id)
}

pub fn toggle_mailbox_active(pool: &DbPool, mailbox_id: i32) -> Result<Mailbox, Error> {
    let mut conn = pool.get().unwrap();
    
    // First get the current mailbox to check its active status
    let current_mailbox = get_mailbox(pool, mailbox_id)?;
    let new_active_status = !current_mailbox.active;
    
    diesel::update(mailboxes::table.find(mailbox_id))
        .set((
            mailboxes::active.eq(new_active_status),
            mailboxes::modified.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)?;
    
    get_mailbox(pool, mailbox_id)
}

// Statistics functions
pub fn get_system_stats(pool: &DbPool) -> Result<SystemStats, Error> {
    let mut conn = pool.get().unwrap();
    
    let total_domains: i64 = domains::table.count().get_result(&mut conn)?;
    let total_users: i64 = users::table.count().get_result(&mut conn)?;
    let total_aliases: i64 = aliases::table.count().get_result(&mut conn)?;
    let total_mailboxes: i64 = mailboxes::table.count().get_result(&mut conn)?;
    
    let total_quota: i64 = users::table
        .select(diesel::dsl::sum(users::quota))
        .get_result::<Option<BigDecimal>>(&mut conn)?
        .and_then(|d| d.to_i64())
        .unwrap_or(0);
    
    Ok(SystemStats {
        total_domains,
        total_users,
        total_aliases,
        total_mailboxes,
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
        let user_count: i64 = users::table
            .filter(users::domain.eq(&domain.domain))
            .count()
            .get_result(&mut conn)?;
            
        let alias_count: i64 = aliases::table
            .filter(aliases::domain.eq(&domain.domain))
            .count()
            .get_result(&mut conn)?;
            
        let mailbox_count: i64 = mailboxes::table
            .filter(mailboxes::domain.eq(&domain.domain))
            .count()
            .get_result(&mut conn)?;
            
        let total_quota: i64 = users::table
            .filter(users::domain.eq(&domain.domain))
            .select(diesel::dsl::sum(users::quota))
            .get_result::<Option<BigDecimal>>(&mut conn)?
            .and_then(|d| d.to_i64())
            .unwrap_or(0);
        
        stats.push(DomainStats {
            domain: domain.domain,
            user_count,
            alias_count,
            mailbox_count,
            total_quota,
            used_quota: 0, // This would need to be calculated from actual disk usage
        });
    }
    
    Ok(stats)
} 
