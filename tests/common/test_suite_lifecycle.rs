use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};

static SUITE_LIFECYCLE_INIT: Once = Once::new();
static SUITE_LIFECYCLE_REGISTERED: AtomicBool = AtomicBool::new(false);

/// Register test suite lifecycle handlers for panic and ctrl-c cleanup.
/// This should be called early in test execution to ensure proper cleanup.
/// This function is idempotent - it will only register handlers once.
pub fn register_test_suite_lifecycle() {
    // Only register once across all tests
    if SUITE_LIFECYCLE_REGISTERED.load(Ordering::Acquire) {
        return;
    }

    SUITE_LIFECYCLE_INIT.call_once(|| {
        // Set up panic hook to ensure cleanup on panic
        std::panic::set_hook(Box::new(|panic_info| {
            eprintln!("[SUITE CLEANUP] Panic info: {:?}", panic_info);
            // Use blocking cleanup to avoid runtime issues
            cleanup_suite_on_panic_blocking();
        }));

        // Set up ctrl-c handler for graceful shutdown
        if let Err(e) = ctrlc::set_handler(|| {
            eprintln!("[SUITE CLEANUP] Ctrl-C received, cleaning up test suite resources...");
            // Use blocking cleanup to avoid runtime issues
            cleanup_suite_on_panic_blocking();
            std::process::exit(0);
        }) {
            eprintln!("[SUITE CLEANUP] Warning: Failed to set ctrl-c handler: {}", e);
            // Don't fail the test if ctrl-c handler setup fails
        }

        // Mark as registered
        SUITE_LIFECYCLE_REGISTERED.store(true, Ordering::Release);
    });
}

/// Blocking cleanup function for suite panic and ctrl-c handlers
fn cleanup_suite_on_panic_blocking() {
    // Targeted cleanup - only remove resources specifically related to this app's tests
    // This is much safer than the broad prune commands

    // Only remove test containers with specific naming patterns
    let _ = std::process::Command::new("docker")
        .args(["ps", "-a", "--format", "{{.ID}} {{.Names}} {{.Image}}"])
        .output()
        .and_then(|output| {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Only target containers that are clearly test-related and belong to this app's tests
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
                    .filter(|line| {
                        line.contains("test") || line.contains("sortingoffice-e2e")
                    })
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
                    .filter(|line| {
                        line.contains("test") || line.contains("sortingoffice")
                    })
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

/// Clean up all test suite resources including shared containers and networks.
/// This should be called at the end of test suites or when the process exits.
pub async fn cleanup_test_suite_resources() -> anyhow::Result<()> {
    println!("[SUITE CLEANUP] Starting cleanup of test suite resources...");

    // Use the comprehensive cleanup function from testcontainers_setup
    sortingoffice::test_helpers::testcontainers_setup::cleanup_all_test_resources().await;

    // Clean up any remaining test schemas
    cleanup_test_schemas().await?;

    println!("[SUITE CLEANUP] Test suite resource cleanup completed");
    Ok(())
}

/// Clean up orphaned containers that might be left behind by the test suite.
async fn cleanup_orphaned_suite_containers() -> anyhow::Result<()> {
    println!("[SUITE CLEANUP] Cleaning up orphaned test containers...");

    // Use shell-based cleanup for now
    cleanup_orphaned_suite_containers_shell().await?;

    Ok(())
}

/// Fallback shell-based cleanup for orphaned suite containers
async fn cleanup_orphaned_suite_containers_shell() -> anyhow::Result<()> {
    use std::process::Command;

    println!("[SUITE CLEANUP] Running shell-based fallback cleanup...");

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
                "[SUITE CLEANUP] Removing orphaned MySQL container: {}",
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
                "[SUITE CLEANUP] Removing orphaned Selenium container: {}",
                container_id
            );
            let _ = Command::new("docker")
                .args(["rm", "-f", container_id])
                .output();
        }
    }

    Ok(())
}

/// Clean up test schemas that might be left behind.
async fn cleanup_test_schemas() -> anyhow::Result<()> {
    use std::process::Command;

    println!("[SUITE CLEANUP] Cleaning up test schemas...");

    // Find and remove test schemas
    let schema_output = Command::new("docker")
        .args([
            "exec",
            "sortingoffice-mysql",
            "mysql",
            "-uroot",
            "-e",
            "SHOW DATABASES LIKE 'test_%';",
        ])
        .output();

    if let Ok(output) = schema_output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let schemas: Vec<&str> = output_str
                .lines()
                .filter(|line| line.starts_with("test_"))
                .collect();

            for schema in schemas {
                println!("[SUITE CLEANUP] Removing test schema: {}", schema);
                let _ = Command::new("docker")
                    .args([
                        "exec",
                        "sortingoffice-mysql",
                        "mysql",
                        "-uroot",
                        "-e",
                        &format!("DROP DATABASE IF EXISTS `{}`;", schema),
                    ])
                    .output();
            }
        }
    }

    Ok(())
}

/// Finalize the test suite and clean up all resources.
/// This should be called at the end of test suites.
pub async fn finalize_test_suite() -> anyhow::Result<()> {
    println!("[SUITE CLEANUP] Finalizing test suite...");

    // Register lifecycle handlers if not already done
    register_test_suite_lifecycle();

    // Clean up all test suite resources
    cleanup_test_suite_resources().await?;

    println!("[SUITE CLEANUP] Test suite finalized successfully");
    Ok(())
}
