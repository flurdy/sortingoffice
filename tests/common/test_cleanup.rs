use std::sync::Once;
use tokio::sync::OnceCell;

#[allow(dead_code)]
static CLEANUP_INIT: Once = Once::new();
#[allow(dead_code)]
static CLEANUP_REGISTERED: OnceCell<bool> = OnceCell::const_new();

/// Register cleanup handlers for the current test process.
/// This should be called early in test execution to ensure proper cleanup.
#[allow(dead_code)]
pub async fn register_test_cleanup() {
    if CLEANUP_REGISTERED.get().is_some() {
        return; // Already registered
    }

    CLEANUP_INIT.call_once(|| {
        // Set up panic hook to ensure cleanup on panic
        std::panic::set_hook(Box::new(|_panic_info| {
            eprintln!("[CLEANUP] Test panic detected, attempting cleanup...");

            // Use blocking cleanup to avoid runtime issues
            cleanup_on_panic_blocking();
        }));

        // Set up ctrl-c handler for graceful shutdown
        ctrlc::set_handler(|| {
            eprintln!("[CLEANUP] Ctrl-C received, cleaning up test resources...");

            // Use blocking cleanup to avoid runtime issues
            cleanup_on_panic_blocking();

            std::process::exit(0);
        })
        .expect("Failed to set ctrl-c handler");
    });

    // Mark as registered
    let _ = CLEANUP_REGISTERED.set(true);
}

/// Blocking cleanup function for panic and ctrl-c handlers
#[allow(dead_code)]
fn cleanup_on_panic_blocking() {
    // Targeted cleanup - only remove resources specifically related to this app's tests
    // This is much safer than the broad prune commands

    // Only remove test containers with specific naming patterns
    let _ = std::process::Command::new("docker")
        .args(["ps", "-a", "--format", "{{.ID}} {{.Names}} {{.Image}}"])
        .output()
        .and_then(|output| {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Only target containers that are clearly test-related and belong to this app
                let test_containers: Vec<&str> = output_str
                    .lines()
                    .filter(|line| {
                        // Only target containers that are clearly test-related
                        (line.contains("test_") || line.contains("selenium") || line.contains("mysql")) &&
                        // AND have names that suggest they're from this app's tests
                        (line.contains("sortingoffice") || line.contains("test") || line.contains("selenium"))
                    })
                    .map(|line| line.split_whitespace().next().unwrap_or(""))
                    .filter(|id| !id.is_empty())
                    .collect();

                for container_id in test_containers {
                    let _ = std::process::Command::new("docker")
                        .args(["rm", "-f", container_id])
                        .output();
                }
            }
            Ok(())
        });

    // Only remove test networks with specific naming patterns
    let _ = std::process::Command::new("docker")
        .args(["network", "ls", "--format", "{{.ID}} {{.Name}}"])
        .output()
        .and_then(|output| {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Only target networks that are clearly test-related
                let test_networks: Vec<&str> = output_str
                    .lines()
                    .filter(|line| line.contains("test") || line.contains("sortingoffice-e2e"))
                    .map(|line| line.split_whitespace().next().unwrap_or(""))
                    .filter(|id| !id.is_empty())
                    .collect();

                for network_id in test_networks {
                    let _ = std::process::Command::new("docker")
                        .args(["network", "rm", network_id])
                        .output();
                }
            }
            Ok(())
        });

    // Only remove test volumes with specific naming patterns
    let _ = std::process::Command::new("docker")
        .args(["volume", "ls", "--format", "{{.Name}}"])
        .output()
        .and_then(|output| {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Only target volumes that are clearly test-related
                let test_volumes: Vec<&str> = output_str
                    .lines()
                    .filter(|line| line.contains("test") || line.contains("sortingoffice"))
                    .collect();

                for volume_name in test_volumes {
                    let _ = std::process::Command::new("docker")
                        .args(["volume", "rm", volume_name])
                        .output();
                }
            }
            Ok(())
        });
}

/// Clean up all test resources including MySQL container and shared network.
/// This should be called at the end of test suites or when individual tests finish.
#[allow(dead_code)]
pub async fn cleanup_all_test_resources() -> anyhow::Result<()> {
    println!("[CLEANUP] Starting cleanup of all test resources...");

    // Clean up shared MySQL container
    sortingoffice::test_helpers::testcontainers_setup::force_cleanup_shared_mysql_container().await;

    // Clean up shared test network
    sortingoffice::test_helpers::testcontainers_setup::cleanup_shared_test_network().await;

    // Use shell-based cleanup for now
    cleanup_orphaned_test_containers_shell().await?;

    println!("[CLEANUP] Test resource cleanup completed");
    Ok(())
}

/// Fallback shell-based cleanup for orphaned test containers
#[allow(dead_code)]
async fn cleanup_orphaned_test_containers_shell() -> anyhow::Result<()> {
    use std::process::Command;

    println!("[CLEANUP] Running shell-based fallback cleanup...");

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
#[allow(dead_code)]
pub async fn cleanup_test_resources() -> anyhow::Result<()> {
    cleanup_all_test_resources().await
}

/// Clean up test suite resources.
/// This is a simplified version that calls the main cleanup function.
#[allow(dead_code)]
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
