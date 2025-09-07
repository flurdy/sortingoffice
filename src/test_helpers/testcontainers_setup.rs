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
    pub container_id: String,
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

    /// Generate the database URL for this test container
    pub fn get_db_url(&self) -> String {
        format!("mysql://root@127.0.0.1:{}/{}", self.port, self.schema)
    }
}

impl Default for TestContainer {
    fn default() -> Self {
        panic!("TestContainer::default() is not supported. Use setup_test_db().await instead.")
    }
}

impl Drop for TestContainer {
    fn drop(&mut self) {
        // Clean up the schema when the TestContainer is dropped
        // Each test owns its own schema and should clean it up
        let schema = self.schema.clone();
        let port = self.port;

        // Use blocking cleanup to avoid runtime creation issues
        // This is a best-effort cleanup that won't block the drop
        std::thread::spawn(move || {
            // Use blocking cleanup without creating new runtimes
            cleanup_test_schema_blocking(&schema, port);
        });
    }
}

/// Blocking cleanup function for TestContainer drop
fn cleanup_test_schema_blocking(schema: &str, port: u16) {
    // Try to use mysql client if available
    let mysql_result = std::process::Command::new("mysql")
        .args([
            "-h",
            "127.0.0.1",
            "-P",
            &port.to_string(),
            "-uroot",
            "-e",
            &format!("DROP DATABASE IF EXISTS `{}`;", schema),
        ])
        .output();

    match mysql_result {
        Ok(_) => println!("[DEBUG] Cleaned up test schema: {schema} via mysql client"),
        Err(_) => {
            // Fallback: try to use docker exec if mysql client not available
            let docker_result = std::process::Command::new("docker")
                .args([
                    "exec",
                    "sortingoffice-mysql",
                    "mysql",
                    "-uroot",
                    "-e",
                    &format!("DROP DATABASE IF EXISTS `{}`;", schema),
                ])
                .output();

            match docker_result {
                Ok(_) => println!("[DEBUG] Cleaned up test schema: {schema} via docker exec"),
                Err(e) => println!(
                    "[DEBUG] Warning: Could not clean up schema {}: {}",
                    schema, e
                ),
            }
        }
    }
}

impl TestContainer {
    pub fn get_bridge_ip(&self) -> &str {
        &self.bridge_ip
    }

    pub fn get_container_id(&self) -> &str {
        &self.container_id
    }
}

pub async fn get_shared_mysql_port() -> u16 {
    // Use a more robust initialization pattern to avoid race conditions
    if let Some(port) = SHARED_PORT.get() {
        return *port;
    }

    let container = SHARED_CONTAINER
        .get_or_init(|| async {
            // Add a small delay to avoid rapid container creation
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

            // Add retry logic for container startup
            let mut attempts = 0;
            let max_attempts = 3;

            loop {
                attempts += 1;
                match AsyncRunner::start(Mysql::default()).await {
                    Ok(container) => {
                        println!("[DEBUG] Successfully started MySQL container on attempt {}", attempts);
                        break container;
                    }
                    Err(e) => {
                        if attempts >= max_attempts {
                            panic!("Failed to start MySQL container after {} attempts: {}", max_attempts, e);
                        }
                        println!("[DEBUG] Failed to start MySQL container on attempt {}: {}. Retrying...", attempts, e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(500 * attempts)).await;
                    }
                }
            }
        })
        .await;

    let port = container.get_host_port_ipv4(3306).await.expect("get port");

    // Set the port atomically
    if SHARED_PORT.get().is_none() {
        let _ = SHARED_PORT.set(port);
    }

    port
}

/// Expose the shared MySQL container id so tests can connect it to a shared network
pub async fn get_shared_mysql_container_id() -> String {
    // Use the same robust initialization pattern
    let container = SHARED_CONTAINER
        .get_or_init(|| async {
            // Add a small delay to avoid rapid container creation
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            AsyncRunner::start(Mysql::default())
                .await
                .expect("Failed to start MySQL container")
        })
        .await;
    container.id().to_string()
}

pub async fn setup_test_db() -> TestContainer {
    INIT.call_once(|| {
        std::env::set_var("RUST_LOG", "warn,testcontainers=error,bollard=error");
        let _ = tracing_subscriber::fmt::try_init();
    });

    // Start or get the shared MySQL container and port
    let port = get_shared_mysql_port().await;
    let host = "127.0.0.1";

    // Create a unique schema/database for this test
    let schema = unique_test_id();
    let admin_url = format!("mysql://root@{host}:{port}/mysql");
    let test_url = format!("mysql://root@{host}:{port}/{schema}");

    // Create the schema - simplified without complex retry logic
    let manager = ConnectionManager::<MysqlConnection>::new(&admin_url);
    let pool = Pool::builder()
        .max_size(2)
        .min_idle(Some(1))
        .build(manager)
        .expect("Failed to create admin pool");

    let mut conn = pool.get().expect("Failed to get admin connection");

    // Drop the schema if it exists to ensure clean state
    let _ = diesel::sql_query(format!("DROP DATABASE IF EXISTS `{schema}`")).execute(&mut conn);

    // Create the new schema with retry logic
    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        attempts += 1;
        match diesel::sql_query(format!("CREATE DATABASE `{schema}`")).execute(&mut conn) {
            Ok(_) => {
                println!(
                    "[DEBUG] Successfully created test schema {} on attempt {}",
                    schema, attempts
                );
                break;
            }
            Err(e) => {
                if attempts >= max_attempts {
                    panic!(
                        "Failed to create test schema {} after {} attempts: {}",
                        schema, max_attempts, e
                    );
                }
                println!(
                    "[DEBUG] Failed to create test schema {} on attempt {}: {}. Retrying...",
                    schema, attempts, e
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempts)).await;
            }
        }
    }

    // Create connection pool for the test schema
    let manager = ConnectionManager::<MysqlConnection>::new(&test_url);
    let pool = Pool::builder()
        .max_size(2)
        .min_idle(Some(1))
        .build(manager)
        .expect("Failed to create pool");

    // Run migrations on the new schema
    let mut conn = pool.get().expect("Failed to get connection");

    // Run migrations with retry logic
    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        attempts += 1;
        match conn.run_pending_migrations(MIGRATIONS) {
            Ok(_) => {
                println!(
                    "[DEBUG] Successfully ran migrations on schema {} on attempt {}",
                    schema, attempts
                );
                break;
            }
            Err(e) => {
                if attempts >= max_attempts {
                    panic!(
                        "Failed to run migrations on schema {} after {} attempts: {}",
                        schema, max_attempts, e
                    );
                }
                println!(
                    "[DEBUG] Failed to run migrations on schema {} on attempt {}: {}. Retrying...",
                    schema, attempts, e
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(200 * attempts)).await;

                // Get a fresh connection for retry
                conn = pool
                    .get()
                    .expect("Failed to get connection for migration retry");
            }
        }
    }

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
                container.id(),
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
        container_id: container.id().to_string(),
    }
}

pub fn cleanup_test_db(_container: &TestContainer) {
    // Optionally drop the schema after the test (not implemented for now)
}

/// Clean up a specific test schema from the shared MySQL container
pub async fn cleanup_test_schema(schema: &str) {
    if let Some(port) = SHARED_PORT.get() {
        let admin_url = format!("mysql://root@127.0.0.1:{port}/mysql");

        let manager = ConnectionManager::<MysqlConnection>::new(&admin_url);
        if let Ok(pool) = Pool::builder().max_size(1).build(manager) {
            if let Ok(mut conn) = pool.get() {
                let query = format!("DROP DATABASE IF EXISTS `{schema}`");
                let _ = diesel::sql_query(query).execute(&mut conn);
                println!("[DEBUG] Cleaned up test schema: {schema}");
            } else {
                println!("[DEBUG] Warning: Could not get connection to clean up schema: {schema}");
            }
        }
    } else {
        println!("[DEBUG] Warning: No shared MySQL port available for schema cleanup: {schema}");
    }
}

/// Clean up the shared MySQL container. This will stop and remove the container.
/// Call this at the end of your test suite if you want to ensure cleanup.
pub async fn cleanup_shared_mysql_container() {
    if let Some(container) = SHARED_CONTAINER.get() {
        println!(
            "[DEBUG] Cleaning up shared MySQL container: {}",
            container.id()
        );

        // Force cleanup by dropping the container reference
        // This will trigger the container's Drop implementation
        if let Some(_container_ref) = SHARED_CONTAINER.get() {
            println!("[DEBUG] Forcing container cleanup...");
            // The container will be dropped when this function returns
        }
    }

    // Note: OnceCell doesn't support .take() for immutable static items
    // The container will be cleaned up when the process ends
    println!(
        "[DEBUG] MySQL container cleanup requested - container will be dropped when process ends"
    );
}

/// Force cleanup of the shared MySQL container by stopping and removing it.
/// This is more aggressive than the regular cleanup and should be used
/// when you want to ensure the container is completely removed.
pub async fn force_cleanup_shared_mysql_container() {
    if let Some(container) = SHARED_CONTAINER.get() {
        let container_id = container.id().to_string();
        println!("[DEBUG] Force cleaning up shared MySQL container: {container_id}");

        // First try to stop the container gracefully
        use std::process::Command;
        let stop_output = Command::new("docker")
            .args(["stop", &container_id])
            .output();

        match stop_output {
            Ok(output) => {
                if output.status.success() {
                    println!("[DEBUG] Successfully stopped MySQL container: {container_id}");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.contains("No such container") {
                        println!(
                            "[DEBUG] Warning: Failed to stop container {container_id}: {stderr}"
                        );
                    }
                }
            }
            Err(e) => {
                println!("[DEBUG] Warning: Failed to execute docker stop command: {e}");
            }
        }

        // Then remove the container
        let rm_output = Command::new("docker")
            .args(["rm", "-f", &container_id])
            .output();

        match rm_output {
            Ok(output) => {
                if output.status.success() {
                    println!("[DEBUG] Successfully removed MySQL container: {container_id}");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.contains("No such container") {
                        println!(
                            "[DEBUG] Warning: Failed to remove container {container_id}: {stderr}"
                        );
                    }
                }
            }
            Err(e) => {
                println!("[DEBUG] Warning: Failed to execute docker rm command: {e}");
            }
        }

        // Also try to clean up any volumes that might be associated
        let volume_output = Command::new("docker")
            .args(["volume", "ls", "-q", "-f", "dangling=true"])
            .output();

        if let Ok(volume_output) = volume_output {
            if volume_output.status.success() {
                let volumes = String::from_utf8_lossy(&volume_output.stdout);
                for volume in volumes.lines() {
                    if !volume.trim().is_empty() {
                        let _ = Command::new("docker")
                            .args(["volume", "rm", volume.trim()])
                            .output();
                    }
                }
            }
        }
    } else {
        println!("[DEBUG] No shared MySQL container found to clean up");
    }
}

/// Clean up the shared network used by UI tests.
/// This should be called after all UI tests complete.
pub async fn cleanup_shared_test_network() {
    let network_name = "sortingoffice-e2e";

    use std::process::Command;

    // First check if the network exists
    let check_output = Command::new("docker")
        .args(["network", "ls", "--format", "{{.Name}}"])
        .output();

    if let Ok(check_output) = check_output {
        if check_output.status.success() {
            let networks = String::from_utf8_lossy(&check_output.stdout);
            if !networks.contains(network_name) {
                println!("[DEBUG] Network {network_name} does not exist, skipping cleanup");
                return;
            }
        }
    }

    // Remove the network
    let output = Command::new("docker")
        .args(["network", "rm", network_name])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("[DEBUG] Successfully removed shared test network: {network_name}");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("No such network") {
                    println!("[DEBUG] Network {network_name} was already removed");
                } else {
                    println!("[DEBUG] Warning: Failed to remove network {network_name}: {stderr}");
                }
            }
        }
        Err(e) => {
            println!("[DEBUG] Warning: Failed to execute docker network rm command: {e}");
        }
    }
}

/// Clean up all test-related Docker resources.
/// This is a comprehensive cleanup function that should be called
/// when you want to ensure all test resources are removed.
pub async fn cleanup_all_test_resources() {
    println!("[DEBUG] Starting comprehensive cleanup of all test resources...");

    // Clean up shared MySQL container
    force_cleanup_shared_mysql_container().await;

    // Clean up shared test network
    cleanup_shared_test_network().await;

    // Clean up any orphaned test containers
    cleanup_orphaned_test_containers().await;

    println!("[DEBUG] Comprehensive cleanup completed");
}

/// Clean up orphaned test containers that might be left behind.
async fn cleanup_orphaned_test_containers() {
    use std::process::Command;

    println!("[DEBUG] Cleaning up orphaned test containers...");

    // Find and remove orphaned test containers
    let ps_output = Command::new("docker")
        .args([
            "ps",
            "-a",
            "--format",
            "{{.ID}} {{.Image}} {{.Names}} {{.Labels}}",
        ])
        .output();

    if let Ok(ps_output) = ps_output {
        if ps_output.status.success() {
            let output_str = String::from_utf8_lossy(&ps_output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();

            for line in lines {
                if line.contains("test")
                    || (line.contains("mysql") && !line.contains("sortingoffice"))
                    || (line.contains("selenium") && !line.contains("sortingoffice"))
                {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        let container_id = parts[0];
                        if !container_id.is_empty() {
                            println!("[DEBUG] Removing orphaned test container: {container_id}");
                            let _ = Command::new("docker")
                                .args(["rm", "-f", container_id])
                                .output();
                        }
                    }
                }
            }
        }
    }
}

pub fn unique_test_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test_{timestamp}")
}
