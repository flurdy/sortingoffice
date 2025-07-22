use crate::DbPool;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::Once;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::mysql::Mysql;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

static INIT: Once = Once::new();

pub struct TestContainer {
    pub pool: DbPool,
    _container: ContainerAsync<Mysql>,
}

impl TestContainer {
    pub async fn new() -> Self {
        INIT.call_once(|| {
            std::env::set_var("RUST_LOG", "debug");
            let _ = tracing_subscriber::fmt::try_init();
        });

        // Start MySQL container using AsyncRunner
        let mysql_container = AsyncRunner::start(Mysql::default())
            .await
            .expect("Failed to start MySQL container");

        // Get connection details
        let host = "127.0.0.1";
        let port = mysql_container
            .get_host_port_ipv4(3306)
            .await
            .expect("get port");

        // Create database URL
        let database_url = format!("mysql://root@{}:{}/mysql", host, port);

        // Create connection pool
        let manager = ConnectionManager::<MysqlConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(5)
            .min_idle(Some(1))
            .build(manager)
            .expect("Failed to create pool");

        // Run migrations
        let mut conn = pool.get().expect("Failed to get connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        TestContainer {
            pool,
            _container: mysql_container,
        }
    }

    pub fn get_pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn get_mysql_port(&self) -> u16 {
        self._container
            .get_host_port_ipv4(3306)
            .await
            .expect("get port")
    }
}

impl Default for TestContainer {
    fn default() -> Self {
        // This is a fallback for when async is not available
        // In practice, we should use setup_test_db().await
        panic!("TestContainer::default() is not supported. Use setup_test_db().await instead.")
    }
}

pub async fn setup_test_db() -> TestContainer {
    TestContainer::new().await
}

pub fn cleanup_test_db(_container: &TestContainer) {
    // The container will be automatically cleaned up when it goes out of scope
}

pub fn unique_test_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test-{}", timestamp)
}
