use crate::{
    db,
    handlers::{language::get_user_locale, utils::get_current_db_pool},
    models::{
        Alias, DeletedResourceCount, DisabledResourceCount, Domain, Relay, Relocated,
        RemoveDomainForm, RemoveDomainSession, RemoveWizardStep, User,
    },
    AppState, DbPool,
};
use axum::{
    extract::{Form, State},
    http::HeaderMap,
    response::{Html, Redirect},
};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{error, info};

#[derive(Deserialize)]
pub struct DomainSearchQuery {
    pub domain: Option<String>,
    pub limit: Option<i64>,
}

// Simple session storage using static HashMap
lazy_static! {
    static ref REMOVE_WIZARD_SESSIONS: Mutex<HashMap<String, RemoveDomainSession>> =
        Mutex::new(HashMap::new());
}

/// Helper functions for session management
fn get_session() -> Option<RemoveDomainSession> {
    REMOVE_WIZARD_SESSIONS
        .lock()
        .unwrap()
        .get("default")
        .cloned()
}

fn save_session(session: RemoveDomainSession) {
    REMOVE_WIZARD_SESSIONS
        .lock()
        .unwrap()
        .insert("default".to_string(), session);
}

fn clear_session() {
    REMOVE_WIZARD_SESSIONS.lock().unwrap().remove("default");
}

/// Remove wizard index page
pub async fn index(State(_state): State<AppState>, _headers: HeaderMap) -> Redirect {
    // Clear any existing session when starting a new wizard
    clear_session();

    // Redirect directly to domain selection step
    Redirect::to("/remove-wizard/domain-selection")
}

/// Step 1: Domain selection
pub async fn domain_selection(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);

    // Render domain selection page
    crate::handlers::rendering::render_remove_domain_selection_page(&state, &locale, &headers, "")
        .await
}

/// Submit domain selection
pub async fn submit_domain_selection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RemoveDomainForm>,
) -> Result<Html<String>, Redirect> {
    let locale = get_user_locale(&headers);

    // Validate input
    if form.domain_name.is_empty() {
        return Ok(
            crate::handlers::rendering::render_remove_domain_selection_page(
                &state,
                &locale,
                &headers,
                "Please select a domain",
            )
            .await,
        );
    }

    let pool = match get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Ok(
                crate::handlers::rendering::render_remove_domain_selection_page(
                    &state,
                    &locale,
                    &headers,
                    "Database connection error",
                )
                .await,
            );
        }
    };

    // Check if domain exists - check both primary and backup domains
    let (domain, is_backup) = match db::get_domain_by_name(&pool, &form.domain_name) {
        Ok(domain) => (domain, false),
        Err(_) => {
            // Not a primary domain, check if it's a backup domain
            match db::get_backup_by_name(&pool, &form.domain_name) {
                Ok(backup) => (
                    Domain {
                        pkid: backup.pkid,
                        domain: backup.domain,
                        transport: backup.transport,
                        enabled: backup.enabled,
                        created: backup.created,
                        modified: backup.modified,
                    },
                    true,
                ),
                Err(_) => {
                    return Ok(
                        crate::handlers::rendering::render_remove_domain_selection_page(
                            &state,
                            &locale,
                            &headers,
                            "Domain not found",
                        )
                        .await,
                    );
                }
            }
        }
    };

    // Find all affected resources
    let affected_aliases = get_aliases_for_domain(&pool, &domain.domain).await;
    let affected_users = get_users_for_domain(&pool, &domain.domain).await;
    let affected_relays = get_relays_for_domain(&pool, &domain.domain).await;
    let affected_relocated = get_relocated_for_domain(&pool, &domain.domain).await;

    // Get aliases with domain in destination field, but exclude those that will be deleted
    let all_orphaned_aliases = get_aliases_with_domain_in_destination(&pool, &domain.domain).await;
    let affected_alias_ids: std::collections::HashSet<i32> =
        affected_aliases.iter().map(|a| a.pkid).collect();
    let orphaned_aliases: Vec<crate::models::Alias> = all_orphaned_aliases
        .into_iter()
        .filter(|alias| !affected_alias_ids.contains(&alias.pkid))
        .collect();

    // Get cross-database domain information (excluding current database)
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    let cross_db_domains = get_cross_db_domain_list(&state, &domain.domain, &current_db_id).await;

    info!(
        "Found {} aliases, {} users, {} relays, {} relocated for domain {}",
        affected_aliases.len(),
        affected_users.len(),
        affected_relays.len(),
        affected_relocated.len(),
        domain.domain
    );

    // Initialize session
    let session = RemoveDomainSession {
        step: RemoveWizardStep::ReviewAffected,
        domain: Some(domain),
        is_backup,
        affected_aliases,
        affected_users,
        affected_relays,
        affected_relocated,
        orphaned_aliases,
        disabled_count: DisabledResourceCount::default(),
        deleted_count: DeletedResourceCount::default(),
        cross_db_domains,
    };

    save_session(session.clone());

    // Redirect to review affected resources
    Ok(
        crate::handlers::rendering::render_remove_domain_review_affected_page(
            &session, &state, &locale, &headers,
        )
        .await,
    )
}

/// Helper function to get aliases for a domain
async fn get_aliases_for_domain(pool: &DbPool, domain: &str) -> Vec<Alias> {
    use crate::schema::aliases;
    use diesel::prelude::*;

    let pattern = format!("%@{}", domain);
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {:?}", e);
            return vec![];
        }
    };

    aliases::table
        .filter(aliases::mail.like(&pattern))
        .order_by(aliases::mail.asc())
        .load::<Alias>(&mut conn)
        .unwrap_or_default()
}

/// Helper function to get users for a domain
async fn get_users_for_domain(pool: &DbPool, domain: &str) -> Vec<User> {
    use crate::schema::users;
    use diesel::prelude::*;

    let pattern = format!("%@{}", domain);
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {:?}", e);
            return vec![];
        }
    };

    users::table
        .filter(users::id.like(&pattern))
        .order_by(users::id.asc())
        .load::<User>(&mut conn)
        .unwrap_or_default()
}

/// Helper function to get relays for a domain
async fn get_relays_for_domain(pool: &DbPool, domain: &str) -> Vec<Relay> {
    if !db::relays_table_exists(pool) {
        return vec![];
    }

    use crate::schema::relays;
    use diesel::prelude::*;

    let pattern = format!("%@{}", domain);
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {:?}", e);
            return vec![];
        }
    };

    relays::table
        .filter(relays::recipient.like(&pattern))
        .order_by(relays::recipient.asc())
        .load::<Relay>(&mut conn)
        .unwrap_or_default()
}

/// Helper function to get relocated for a domain
async fn get_relocated_for_domain(pool: &DbPool, domain: &str) -> Vec<Relocated> {
    if !db::relocated_table_exists(pool) {
        return vec![];
    }

    use crate::schema::relocated;
    use diesel::prelude::*;

    let pattern = format!("%@{}", domain);
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {:?}", e);
            return vec![];
        }
    };

    relocated::table
        .filter(relocated::old_address.like(&pattern))
        .order_by(relocated::old_address.asc())
        .load::<Relocated>(&mut conn)
        .unwrap_or_default()
}

/// Helper function to get aliases that have this domain in their destination field
async fn get_aliases_with_domain_in_destination(pool: &DbPool, domain: &str) -> Vec<Alias> {
    use crate::schema::aliases;
    use diesel::prelude::*;

    let pattern = format!("%@{}%", domain);
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {:?}", e);
            return vec![];
        }
    };

    aliases::table
        .filter(aliases::destination.like(&pattern))
        .order_by(aliases::mail.asc())
        .load::<Alias>(&mut conn)
        .unwrap_or_default()
}

/// Helper function to get list of other DBs where this domain exists (excluding current DB)
async fn get_cross_db_domain_list(
    state: &AppState,
    domain_name: &str,
    current_db_id: &str,
) -> Vec<String> {
    let configs = state.db_manager.get_configs();
    let mut db_list = Vec::new();

    for config in configs {
        // Skip the current database
        if config.id == current_db_id {
            continue;
        }

        if let Some(pool) = state.db_manager.get_pool(&config.id).await {
            // Check primary domains
            if db::get_domain_by_name(&pool, domain_name).is_ok() {
                db_list.push(format!("{} (primary)", config.label));
                continue;
            }
            // Check backup domains
            if db::get_backup_by_name(&pool, domain_name).is_ok() {
                db_list.push(format!("{} (backup)", config.label));
            }
        }
    }

    db_list
}

/// Step 2: Review affected resources
pub async fn review_affected(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);

    let session = match get_session() {
        Some(session) => session,
        None => {
            return Html("No session found. Please start over.".to_string());
        }
    };

    crate::handlers::rendering::render_remove_domain_review_affected_page(
        &session, &state, &locale, &headers,
    )
    .await
}

/// Step 3: Disable resources
pub async fn disable_resources(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RemoveDomainForm>,
) -> Result<Html<String>, Redirect> {
    if !form.confirmed {
        // User didn't confirm, go back to review
        return Ok(review_affected(State(state), headers).await);
    }

    let mut session = match get_session() {
        Some(session) => session,
        None => {
            return Ok(Html("No session found. Please start over.".to_string()));
        }
    };

    let pool = match get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Ok(Html("Database connection error".to_string()));
        }
    };

    // Check if everything is already disabled
    let all_disabled = session.domain.as_ref().map_or(false, |d| !d.enabled)
        && session.affected_aliases.iter().all(|a| !a.enabled)
        && session.affected_users.iter().all(|u| !u.enabled)
        && session.affected_relays.iter().all(|r| !r.enabled)
        && session.affected_relocated.iter().all(|r| !r.enabled);

    if all_disabled {
        info!("All resources already disabled, skipping to delete confirmation");
        session.step = RemoveWizardStep::ConfirmDelete;
        save_session(session.clone());
        return Ok(
            crate::handlers::rendering::render_remove_domain_confirm_delete_page(
                &session,
                &state,
                &get_user_locale(&headers),
                &headers,
            )
            .await,
        );
    }

    // Execute disable operations
    let mut disabled_count = DisabledResourceCount::default();

    // Disable domain/backup
    if let Some(domain) = &session.domain {
        if domain.enabled {
            if session.is_backup {
                match db::toggle_backup_enabled(&pool, domain.pkid) {
                    Ok(_) => {
                        disabled_count.domain = 1;
                        info!("Disabled backup domain: {}", domain.domain);
                    }
                    Err(e) => {
                        error!("Failed to disable backup domain: {:?}", e);
                    }
                }
            } else {
                match db::toggle_domain_enabled(&pool, domain.pkid) {
                    Ok(_) => {
                        disabled_count.domain = 1;
                        info!("Disabled domain: {}", domain.domain);
                    }
                    Err(e) => {
                        error!("Failed to disable domain: {:?}", e);
                    }
                }
            }
        }
    }

    // Disable aliases
    for alias in &session.affected_aliases {
        if alias.enabled {
            match db::toggle_alias_enabled(&pool, alias.pkid) {
                Ok(_) => {
                    disabled_count.aliases += 1;
                }
                Err(e) => {
                    error!("Failed to disable alias {}: {:?}", alias.mail, e);
                }
            }
        }
    }
    info!("Disabled {} aliases", disabled_count.aliases);

    // Disable users
    for user in &session.affected_users {
        if user.enabled {
            match db::toggle_user_enabled(&pool, user.id.clone()) {
                Ok(_) => {
                    disabled_count.users += 1;
                }
                Err(e) => {
                    error!("Failed to disable user {}: {:?}", user.id, e);
                }
            }
        }
    }
    info!("Disabled {} users", disabled_count.users);

    // Disable relays
    for relay in &session.affected_relays {
        if relay.enabled {
            match db::toggle_relay_enabled(&pool, relay.pkid) {
                Ok(_) => {
                    disabled_count.relays += 1;
                }
                Err(e) => {
                    error!("Failed to disable relay {}: {:?}", relay.recipient, e);
                }
            }
        }
    }
    info!("Disabled {} relays", disabled_count.relays);

    // Disable relocated
    for relocated in &session.affected_relocated {
        if relocated.enabled {
            match db::toggle_relocated_enabled(&pool, relocated.pkid) {
                Ok(_) => {
                    disabled_count.relocated += 1;
                }
                Err(e) => {
                    error!(
                        "Failed to disable relocated {}: {:?}",
                        relocated.old_address, e
                    );
                }
            }
        }
    }
    info!("Disabled {} relocated", disabled_count.relocated);

    // Update session
    session.step = RemoveWizardStep::ReviewDisabled;
    session.disabled_count = disabled_count;
    save_session(session.clone());

    // Render review disabled page
    let locale = get_user_locale(&headers);
    Ok(
        crate::handlers::rendering::render_remove_domain_review_disabled_page(
            &session, &state, &locale, &headers,
        )
        .await,
    )
}

/// Step 4: Review disabled resources
pub async fn review_disabled(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);

    let session = match get_session() {
        Some(session) => session,
        None => {
            return Html("No session found. Please start over.".to_string());
        }
    };

    crate::handlers::rendering::render_remove_domain_review_disabled_page(
        &session, &state, &locale, &headers,
    )
    .await
}

/// Step 5: Confirm deletion
pub async fn confirm_delete(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = get_user_locale(&headers);

    let session = match get_session() {
        Some(session) => session,
        None => {
            return Html("No session found. Please start over.".to_string());
        }
    };

    crate::handlers::rendering::render_remove_domain_confirm_delete_page(
        &session, &state, &locale, &headers,
    )
    .await
}

/// Step 6: Execute deletion
pub async fn execute_deletion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RemoveDomainForm>,
) -> Result<Html<String>, Redirect> {
    if !form.confirmed {
        // User didn't confirm, go back to confirm delete page
        return Ok(confirm_delete(State(state), headers).await);
    }

    let mut session = match get_session() {
        Some(session) => session,
        None => {
            return Ok(Html("No session found. Please start over.".to_string()));
        }
    };

    let pool = match get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Ok(Html("Database connection error".to_string()));
        }
    };

    let mut deleted_count = DeletedResourceCount::default();

    // Delete relocated entries
    for relocated in &session.affected_relocated {
        match db::delete_relocated(&pool, relocated.pkid) {
            Ok(_) => {
                deleted_count.relocated += 1;
            }
            Err(e) => {
                error!(
                    "Failed to delete relocated {}: {:?}",
                    relocated.old_address, e
                );
            }
        }
    }
    info!("Deleted {} relocated entries", deleted_count.relocated);

    // Delete relays
    for relay in &session.affected_relays {
        match db::delete_relay(&pool, relay.pkid) {
            Ok(_) => {
                deleted_count.relays += 1;
            }
            Err(e) => {
                error!("Failed to delete relay {}: {:?}", relay.recipient, e);
            }
        }
    }
    info!("Deleted {} relays", deleted_count.relays);

    // Delete users
    for user in &session.affected_users {
        match db::delete_user(&pool, user.id.clone()) {
            Ok(_) => {
                deleted_count.users += 1;
            }
            Err(e) => {
                error!("Failed to delete user {}: {:?}", user.id, e);
            }
        }
    }
    info!("Deleted {} users", deleted_count.users);

    // Delete aliases
    for alias in &session.affected_aliases {
        match db::delete_alias(&pool, alias.pkid) {
            Ok(_) => {
                deleted_count.aliases += 1;
            }
            Err(e) => {
                error!("Failed to delete alias {}: {:?}", alias.mail, e);
            }
        }
    }
    info!("Deleted {} aliases", deleted_count.aliases);

    // Delete domain/backup (last)
    if let Some(domain) = &session.domain {
        if session.is_backup {
            match db::delete_backup(&pool, domain.pkid) {
                Ok(_) => {
                    deleted_count.domain = 1;
                    info!("Deleted backup domain: {}", domain.domain);
                }
                Err(e) => {
                    error!("Failed to delete backup domain: {:?}", e);
                }
            }
        } else {
            match db::delete_domain(&pool, domain.pkid) {
                Ok(_) => {
                    deleted_count.domain = 1;
                    info!("Deleted domain: {}", domain.domain);
                }
                Err(e) => {
                    error!("Failed to delete domain: {:?}", e);
                }
            }
        }
    }

    // Update session
    session.step = RemoveWizardStep::Complete;
    session.deleted_count = deleted_count;
    save_session(session.clone());

    // Render completion page
    let locale = get_user_locale(&headers);
    Ok(
        crate::handlers::rendering::render_remove_domain_complete_page(
            &session, &state, &locale, &headers,
        )
        .await,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        DeletedResourceCount, DisabledResourceCount, RemoveDomainSession, RemoveWizardStep,
    };

    #[test]
    fn test_disabled_resource_count_default() {
        let count = DisabledResourceCount::default();
        assert_eq!(count.domain, 0);
        assert_eq!(count.aliases, 0);
        assert_eq!(count.users, 0);
        assert_eq!(count.relays, 0);
        assert_eq!(count.relocated, 0);
    }

    #[test]
    fn test_deleted_resource_count_default() {
        let count = DeletedResourceCount::default();
        assert_eq!(count.domain, 0);
        assert_eq!(count.aliases, 0);
        assert_eq!(count.users, 0);
        assert_eq!(count.relays, 0);
        assert_eq!(count.relocated, 0);
    }

    #[test]
    fn test_remove_wizard_step_equality() {
        assert_eq!(
            RemoveWizardStep::DomainSelection,
            RemoveWizardStep::DomainSelection
        );
        assert_ne!(
            RemoveWizardStep::DomainSelection,
            RemoveWizardStep::ReviewAffected
        );
    }

    #[test]
    fn test_session_storage() {
        // Clear any existing session
        clear_session();

        // Verify no session exists
        assert!(get_session().is_none());

        // Create a test session
        let session = RemoveDomainSession {
            step: RemoveWizardStep::DomainSelection,
            domain: None,
            is_backup: false,
            affected_aliases: vec![],
            affected_users: vec![],
            affected_relays: vec![],
            affected_relocated: vec![],
            orphaned_aliases: vec![],
            disabled_count: DisabledResourceCount::default(),
            deleted_count: DeletedResourceCount::default(),
            cross_db_domains: vec![],
        };

        // Save session
        save_session(session.clone());

        // Verify session can be retrieved
        let retrieved = get_session();
        assert!(retrieved.is_some());
        let retrieved_session = retrieved.unwrap();
        assert_eq!(retrieved_session.step, RemoveWizardStep::DomainSelection);
        assert!(!retrieved_session.is_backup);

        // Clear session
        clear_session();
        assert!(get_session().is_none());
    }

    #[test]
    fn test_orphaned_alias_filtering() {
        // Test the logic of filtering orphaned aliases
        let affected_alias_ids: std::collections::HashSet<i32> =
            vec![1, 2, 3].into_iter().collect();

        // Simulate aliases with domain in destination
        let all_orphaned = vec![
            // This one will be deleted (id=1)
            1,  // This one won't be deleted (id=10)
            10, // This one will be deleted (id=2)
            2,  // This one won't be deleted (id=20)
            20,
        ];

        let truly_orphaned: Vec<i32> = all_orphaned
            .into_iter()
            .filter(|id| !affected_alias_ids.contains(id))
            .collect();

        assert_eq!(truly_orphaned.len(), 2);
        assert!(truly_orphaned.contains(&10));
        assert!(truly_orphaned.contains(&20));
        assert!(!truly_orphaned.contains(&1));
        assert!(!truly_orphaned.contains(&2));
    }

    #[test]
    fn test_cross_db_filtering_logic() {
        // Test the logic of filtering current database from cross-db list
        let current_db_id = "primary";
        let all_db_ids = vec!["primary", "secondary", "tertiary"];

        let other_dbs: Vec<&str> = all_db_ids
            .into_iter()
            .filter(|id| *id != current_db_id)
            .collect();

        assert_eq!(other_dbs.len(), 2);
        assert!(!other_dbs.contains(&"primary"));
        assert!(other_dbs.contains(&"secondary"));
        assert!(other_dbs.contains(&"tertiary"));
    }
}
