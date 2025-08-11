use std::sync::Once;
use tokio::sync::OnceCell;

static CLEANUP_INIT: Once = Once::new();
static CLEANUP_REGISTERED: OnceCell<bool> = OnceCell::const_new();

/// Register cleanup handlers for the current test process.
/// This should be called early in test execution to ensure proper cleanup.
pub async fn register_test_cleanup() {
    if CLEANUP_REGISTERED.get().is_some() {
        return; // Already registered
    }

    CLEANUP_INIT.call_once(|| {
        // Set up panic hook to ensure cleanup on panic
        std::panic::set_hook(Box::new(|_panic_info| {
            eprintln!("[CLEANUP] Test panic detected, attempting cleanup...");

            // Spawn a cleanup task in a new runtime to avoid deadlocks
            let rt = tokio::runtime::Runtime::new();
            if let Ok(rt) = rt {
                let _ = rt.block_on(async {
                    if let Err(e) = cleanup_all_test_resources().await {
                        eprintln!("[CLEANUP] Error during panic cleanup: {}", e);
                    }
                });
            }
        }));

        // Set up ctrl-c handler for graceful shutdown
        ctrlc::set_handler(|| {
            eprintln!("[CLEANUP] Ctrl-C received, cleaning up test resources...");

            let rt = tokio::runtime::Runtime::new();
            if let Ok(rt) = rt {
                let _ = rt.block_on(async {
                    if let Err(e) = cleanup_all_test_resources().await {
                        eprintln!("[CLEANUP] Error during ctrl-c cleanup: {}", e);
                    }
                });
            }

            std::process::exit(0);
        })
        .expect("Failed to set ctrl-c handler");
    });

    // Mark as registered
    let _ = CLEANUP_REGISTERED.set(true);
}

/// Clean up all test resources including MySQL container and shared network.
/// This should be called at the end of test suites or when individual tests finish.
pub async fn cleanup_all_test_resources() -> anyhow::Result<()> {
    println!("[CLEANUP] Starting cleanup of all test resources...");

    // Clean up shared MySQL container
    sortingoffice::test_helpers::testcontainers_setup::force_cleanup_shared_mysql_container().await;

    // Clean up shared test network
    sortingoffice::test_helpers::testcontainers_setup::cleanup_shared_test_network().await;

    // Clean up any orphaned test containers
    cleanup_orphaned_test_containers().await?;

    println!("[CLEANUP] Test resource cleanup completed");
    Ok(())
}

/// Clean up orphaned test containers that might be left behind.
async fn cleanup_orphaned_test_containers() -> anyhow::Result<()> {
    use std::process::Command;

    // Find and remove orphaned MySQL test containers
    let mysql_output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.ID}} {{.Image}} {{.Names}}"])
        .output()?;

    if mysql_output.status.success() {
        let output_str = String::from_utf8_lossy(&mysql_output.stdout);
        let mysql_containers: Vec<&str> = output_str
            .lines()
            .filter(|line| {
                line.contains(" mysql")
                    && !line.contains("sortingoffice")
                    && (line.contains("test") || line.contains("mysql"))
            })
            .map(|line| line.split_whitespace().next().unwrap_or(""))
            .filter(|id| !id.is_empty())
            .collect();

        for container_id in mysql_containers {
            println!(
                "[CLEANUP] Removing orphaned MySQL container: {}",
                container_id
            );
            let _ = Command::new("docker")
                .args(["rm", "-f", container_id])
                .output();
        }
    }

    // Find and remove orphaned Selenium test containers
    let selenium_output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.ID}} {{.Image}} {{.Names}}"])
        .output()?;

    if selenium_output.status.success() {
        let output_str = String::from_utf8_lossy(&selenium_output.stdout);
        let selenium_containers: Vec<&str> = output_str
            .lines()
            .filter(|line| {
                line.contains("selenium") && (line.contains("test") || line.contains("selenium"))
            })
            .map(|line| line.split_whitespace().next().unwrap_or(""))
            .filter(|id| !id.is_empty())
            .collect();

        for container_id in selenium_containers {
            println!(
                "[CLEANUP] Removing orphaned Selenium container: {}",
                container_id
            );
            let _ = Command::new("docker")
                .args(["rm", "-f", container_id])
                .output();
        }
    }

    Ok(())
}

/// Clean up test resources for a specific test.
/// This is a simplified version that calls the main cleanup function.
pub async fn cleanup_test_resources() -> anyhow::Result<()> {
    cleanup_all_test_resources().await
}

/// Clean up test suite resources.
/// This is a simplified version that calls the main cleanup function.
pub async fn cleanup_test_suite() -> anyhow::Result<()> {
    cleanup_all_test_resources().await
}

/// Macro to automatically register cleanup when a test starts.
/// Use this at the beginning of test functions that need cleanup.
#[macro_export]
macro_rules! test_with_cleanup {
    ($test_fn:expr) => {
        #[tokio::test]
        async fn test_with_cleanup() {
            // Register cleanup handlers
            test_cleanup::register_test_cleanup().await;

            // Run the test
            let result = $test_fn().await;

            // Clean up test resources
            if let Err(ref e) = result {
                eprintln!("[CLEANUP] Test failed, cleaning up resources...");
                let _ = test_cleanup::cleanup_test_resources().await;
            }

            // Return the result
            result
        }
    };
}
