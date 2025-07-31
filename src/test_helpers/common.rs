use crate::schema::{
    aliases::dsl::aliases, backups::dsl::backups, domains::dsl::domains, users::dsl::users,
    relays::dsl::relays, relocated::dsl::relocated, clients::dsl::clients,
};
use crate::DbPool;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::Once;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

static INIT: Once = Once::new();
// WARNING: The following uses a shared reference to mutable static. This is a known limitation for test pool setup in this test harness.
// See: https://doc.rust-lang.org/nightly/edition-guide/rust-2024/static-mut-references.html
#[allow(static_mut_refs)]
static mut TEST_POOL: Option<DbPool> = None;

pub fn setup_test_db() -> DbPool {
    unsafe {
        #[allow(static_mut_refs)]
        if TEST_POOL.is_none() {
            INIT.call_once(|| {
                std::env::set_var("RUST_LOG", "debug");
                tracing_subscriber::fmt::init();
            });

            // For now, keep using MySQL but with a test database
            let database_url = std::env::var("TEST_DATABASE_URL")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .unwrap_or_else(|_| {
                    "mysql://root:password@localhost/sortingoffice_test".to_string()
                });

            let manager = ConnectionManager::<MysqlConnection>::new(database_url);
            let pool = Pool::builder()
                .max_size(5) // Limit pool size for tests
                .min_idle(Some(1))
                .build(manager)
                .expect("Failed to create pool");

            // Run migrations
            let mut conn = pool.get().expect("Failed to get connection");
            conn.run_pending_migrations(MIGRATIONS)
                .expect("Failed to run migrations");

            TEST_POOL = Some(pool);
        }

        #[allow(static_mut_refs)]
        TEST_POOL.as_ref().unwrap().clone()
    }
}

pub fn cleanup_test_db(pool: &DbPool) {
    // Try to get a connection, but don't panic if we can't
    if let Ok(mut conn) = pool.get() {
        // Clean up test data in reverse dependency order
        diesel::delete(aliases).execute(&mut conn).ok();
        diesel::delete(users).execute(&mut conn).ok();
        diesel::delete(backups).execute(&mut conn).ok();
        diesel::delete(domains).execute(&mut conn).ok();
        diesel::delete(relays).execute(&mut conn).ok();
        diesel::delete(relocated).execute(&mut conn).ok();
        diesel::delete(clients).execute(&mut conn).ok();
    }
}

/// Enhanced cleanup with specific table targeting
pub fn cleanup_specific_tables(pool: &DbPool, table_names: &[&str]) {
    if let Ok(mut conn) = pool.get() {
        for table_name in table_names {
            match *table_name {
                "aliases" => diesel::delete(aliases).execute(&mut conn).ok(),
                "users" => diesel::delete(users).execute(&mut conn).ok(),
                "backups" => diesel::delete(backups).execute(&mut conn).ok(),
                "domains" => diesel::delete(domains).execute(&mut conn).ok(),
                "relays" => diesel::delete(relays).execute(&mut conn).ok(),
                "relocated" => diesel::delete(relocated).execute(&mut conn).ok(),
                "clients" => diesel::delete(clients).execute(&mut conn).ok(),
                _ => None,
            };
        }
    }
}

/// Cleanup with data verification
pub fn cleanup_with_verification(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(mut conn) = pool.get() {
        // Clean up in reverse dependency order
        let alias_count = diesel::delete(aliases).execute(&mut conn)?;
        let user_count = diesel::delete(users).execute(&mut conn)?;
        let backup_count = diesel::delete(backups).execute(&mut conn)?;
        let domain_count = diesel::delete(domains).execute(&mut conn)?;
        let relay_count = diesel::delete(relays).execute(&mut conn)?;
        let relocated_count = diesel::delete(relocated).execute(&mut conn)?;
        let client_count = diesel::delete(clients).execute(&mut conn)?;

        // Log cleanup results for debugging
        tracing::debug!(
            "Test cleanup completed: {} aliases, {} users, {} backups, {} domains, {} relays, {} relocated, {} clients",
            alias_count, user_count, backup_count, domain_count, relay_count, relocated_count, client_count
        );
    }
    Ok(())
}

/// Test data management utilities
pub struct TestDataManager;

impl TestDataManager {
    /// Create test data and return cleanup function
    pub fn create_test_data_with_cleanup<F, T>(pool: &DbPool, creator: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnOnce(&DbPool) -> Result<T, Box<dyn std::error::Error>>,
    {
        // Create the test data
        let result = creator(pool)?;
        
        // Return the result (cleanup will be handled by caller)
        Ok(result)
    }

    /// Create multiple test datasets
    pub fn create_multiple_datasets<F, T>(
        pool: &DbPool, 
        count: usize, 
        creator: F
    ) -> Result<Vec<T>, Box<dyn std::error::Error>>
    where
        F: Fn(&DbPool, usize) -> Result<T, Box<dyn std::error::Error>>,
    {
        let mut datasets = Vec::new();
        for i in 0..count {
            datasets.push(creator(pool, i)?);
        }
        Ok(datasets)
    }

    /// Create test data with automatic cleanup
    pub fn with_test_data<F, T>(pool: &DbPool, creator: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnOnce(&DbPool) -> Result<T, Box<dyn std::error::Error>>,
    {
        // Clean up before creating test data
        cleanup_test_db(pool);
        
        // Create the test data
        let result = creator(pool)?;
        
        // Clean up after creating test data
        cleanup_test_db(pool);
        
        Ok(result)
    }
}

pub fn unique_test_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test-{timestamp}")
}

/// Generate a unique test ID with prefix
pub fn unique_test_id_with_prefix(prefix: &str) -> String {
    format!("{}-{}", prefix, unique_test_id())
}

/// Generate a unique test ID with timestamp
pub fn unique_test_id_with_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("test-{}", timestamp)
}

/// Generate a unique test ID with random component
pub fn unique_test_id_with_random() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_num: u32 = rng.random();
    format!("test-{}-{}", unique_test_id(), random_num)
}
