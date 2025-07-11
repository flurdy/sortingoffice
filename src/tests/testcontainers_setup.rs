use crate::DbPool;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::Once;
use testcontainers::{clients::Cli, Container};
use testcontainers_modules::mysql::Mysql;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

static INIT: Once = Once::new();

pub struct TestContainer {
    pub pool: DbPool,
    _docker: Box<Cli>,
    _container: Container<'static, Mysql>,
}

impl TestContainer {
    pub fn new() -> Self {
        INIT.call_once(|| {
            std::env::set_var("RUST_LOG", "debug");
            let _ = tracing_subscriber::fmt::try_init();
        });

        // Start testcontainers client (not static)
        let docker = Box::new(Cli::default());
        // SAFETY: We must extend the container's lifetime to 'static for the struct, so we leak the container only (not the client)
        let mysql_container: Container<'static, Mysql> = unsafe {
            std::mem::transmute::<Container<'_, Mysql>, Container<'static, Mysql>>(
                docker.run(Mysql::default()),
            )
        };

        // Get connection details
        let host = "127.0.0.1";
        let port = mysql_container.get_host_port_ipv4(3306);

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
            _docker: docker,
            _container: mysql_container,
        }
    }

    pub fn get_pool(&self) -> &DbPool {
        &self.pool
    }

    pub fn get_mysql_port(&self) -> u16 {
        self._container.get_host_port_ipv4(3306)
    }
}

impl Default for TestContainer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn setup_test_db() -> TestContainer {
    TestContainer::new()
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
