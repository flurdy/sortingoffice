use crate::DbPool;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::Once;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::mysql::Mysql;
use tokio::sync::OnceCell;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

static INIT: Once = Once::new();
static SHARED_CONTAINER: OnceCell<ContainerAsync<Mysql>> = OnceCell::const_new();
static SHARED_PORT: OnceCell<u16> = OnceCell::const_new();

pub struct TestContainer {
    pub pool: DbPool,
    pub schema: String,
    pub port: u16,
    pub bridge_ip: String,
}

impl TestContainer {
    pub fn get_pool(&self) -> &DbPool {
        &self.pool
    }
    pub fn get_schema(&self) -> &str {
        &self.schema
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
}

impl Default for TestContainer {
    fn default() -> Self {
        panic!("TestContainer::default() is not supported. Use setup_test_db().await instead.")
    }
}

impl TestContainer {
    pub fn get_bridge_ip(&self) -> &str {
        &self.bridge_ip
    }
}

pub async fn get_shared_mysql_port() -> u16 {
    let container = SHARED_CONTAINER
        .get_or_init(|| async {
            AsyncRunner::start(Mysql::default())
                .await
                .expect("Failed to start MySQL container")
        })
        .await;
    let port = container.get_host_port_ipv4(3306).await.expect("get port");
    SHARED_PORT.get_or_init(|| async { port }).await;
    port
}

pub async fn setup_test_db() -> TestContainer {
    INIT.call_once(|| {
        std::env::set_var("RUST_LOG", "error,testcontainers=error,bollard=error");
        let _ = tracing_subscriber::fmt::try_init();
    });

    // Start or get the shared MySQL container and port
    let port = get_shared_mysql_port().await;
    let host = "127.0.0.1";

    // Create a unique schema/database for this test
    let schema = unique_test_id();
    let admin_url = format!("mysql://root@{}:{}/mysql", host, port);
    let test_url = format!("mysql://root@{}:{}/{}", host, port, schema);

    // Create the schema
    {
        let manager = ConnectionManager::<MysqlConnection>::new(&admin_url);
        let pool = Pool::builder()
            .max_size(2)
            .min_idle(Some(1))
            .build(manager)
            .expect("Failed to create admin pool");
        let mut conn = pool.get().expect("Failed to get admin connection");
        diesel::sql_query(format!("CREATE DATABASE IF NOT EXISTS `{}`", schema))
            .execute(&mut conn)
            .expect("Failed to create test schema");
    }

    // Create connection pool for the test schema
    let manager = ConnectionManager::<MysqlConnection>::new(&test_url);
    let pool = Pool::builder()
        .max_size(5)
        .min_idle(Some(1))
        .build(manager)
        .expect("Failed to create pool");

    // Run migrations on the new schema
    let mut conn = pool.get().expect("Failed to get connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    // Get the MySQL container's bridge IP
    let container = SHARED_CONTAINER
        .get()
        .expect("Shared container not initialized");
    let bridge_ip = {
        use std::process::Command;
        let output = Command::new("docker")
            .args([
                "inspect",
                "-f",
                "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}",
                &container.id(),
            ])
            .output()
            .expect("Failed to get MySQL container bridge IP");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };

    TestContainer {
        pool,
        schema,
        port,
        bridge_ip,
    }
}

pub fn cleanup_test_db(_container: &TestContainer) {
    // Optionally drop the schema after the test (not implemented for now)
}

pub fn unique_test_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test_{}", timestamp)
}
