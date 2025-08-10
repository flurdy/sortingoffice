use std::sync::Once;
use tokio::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};

static SUITE_CLEANUP_INIT: Once = Once::new();
static SUITE_CLEANUP_REGISTERED: OnceCell<bool> = OnceCell::const_new();
static CLEANUP_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Register test suite lifecycle handlers.
/// This should be called early in test execution to ensure proper cleanup.
pub async fn register_test_suite_lifecycle() {
    if SUITE_CLEANUP_REGISTERED.get().is_some() {
        return; // Already registered
    }
    
    SUITE_CLEANUP_INIT.call_once(|| {
        // Set up panic hook to ensure cleanup on panic
        std::panic::set_hook(Box::new(|panic_info| {
            eprintln!("[SUITE CLEANUP] Test panic detected, attempting cleanup...");
            eprintln!("[SUITE CLEANUP] Panic info: {:?}", panic_info);
            
            // Only attempt cleanup if not already in progress
            if !CLEANUP_IN_PROGRESS.load(Ordering::SeqCst) {
                CLEANUP_IN_PROGRESS.store(true, Ordering::SeqCst);
                
                // Spawn a cleanup task in a new runtime to avoid deadlocks
                let rt = tokio::runtime::Runtime::new();
                if let Ok(rt) = rt {
                    let _ = rt.block_on(async {
                        if let Err(e) = cleanup_test_suite_resources().await {
                            eprintln!("[SUITE CLEANUP] Error during panic cleanup: {}", e);
                        }
                    });
                }
            }
        }));
        
        // Set up ctrl-c handler for graceful shutdown
        ctrlc::set_handler(|| {
            eprintln!("[SUITE CLEANUP] Ctrl-C received, cleaning up test suite resources...");
            
            if !CLEANUP_IN_PROGRESS.load(Ordering::SeqCst) {
                CLEANUP_IN_PROGRESS.store(true, Ordering::SeqCst);
                
                let rt = tokio::runtime::Runtime::new();
                if let Ok(rt) = rt {
                    let _ = rt.block_on(async {
                        if let Err(e) = cleanup_test_suite_resources().await {
                            eprintln!("[SUITE CLEANUP] Error during ctrl-c cleanup: {}", e);
                        }
                    });
                }
            }
            
            std::process::exit(0);
        }).expect("Failed to set ctrl-c handler");
        
        // Note: Process exit hooks are not available in current Rust versions
        // We rely on the panic hook and ctrl-c handler for cleanup scenarios
    });
    
    // Mark as registered
    let _ = SUITE_CLEANUP_REGISTERED.set(true);
}

/// Clean up all test suite resources including shared containers and networks.
/// This should be called at the end of test suites or when the process exits.
pub async fn cleanup_test_suite_resources() -> anyhow::Result<()> {
    // Prevent multiple simultaneous cleanup attempts
    if CLEANUP_IN_PROGRESS.load(Ordering::SeqCst) {
        println!("[SUITE CLEANUP] Cleanup already in progress, skipping...");
        return Ok(());
    }
    
    CLEANUP_IN_PROGRESS.store(true, Ordering::SeqCst);
    
    println!("[SUITE CLEANUP] Starting cleanup of test suite resources...");
    
    // Use the comprehensive cleanup function from testcontainers_setup
    sortingoffice::test_helpers::testcontainers_setup::cleanup_all_test_resources().await;
    
    // Clean up any remaining test schemas
    cleanup_test_schemas().await?;
    
    println!("[SUITE CLEANUP] Test suite resource cleanup completed");
    
    // Reset cleanup flag
    CLEANUP_IN_PROGRESS.store(false, Ordering::SeqCst);
    Ok(())
}

/// Clean up orphaned containers that might be left behind by the test suite.
async fn cleanup_orphaned_suite_containers() -> anyhow::Result<()> {
    use std::process::Command;
    
    println!("[SUITE CLEANUP] Cleaning up orphaned test containers...");
    
    // Find and remove orphaned MySQL test containers
    let mysql_output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.ID}} {{.Image}} {{.Names}}"])
        .output()?;
    
    if mysql_output.status.success() {
        let output_str = String::from_utf8_lossy(&mysql_output.stdout);
        let mysql_containers: Vec<&str> = output_str
            .lines()
            .filter(|line| {
                line.contains(" mysql") && 
                !line.contains("sortingoffice") &&
                (line.contains("test") || line.contains("mysql"))
            })
            .map(|line| line.split_whitespace().next().unwrap_or(""))
            .filter(|id| !id.is_empty())
            .collect();
        
        for container_id in mysql_containers {
            println!("[SUITE CLEANUP] Removing orphaned MySQL container: {}", container_id);
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
                line.contains(" selenium") && 
                !line.contains("sortingoffice") &&
                (line.contains("test") || line.contains("selenium"))
            })
            .map(|line| line.split_whitespace().next().unwrap_or(""))
            .filter(|id| !id.is_empty())
            .collect();
        
        for container_id in selenium_containers {
            println!("[SUITE CLEANUP] Removing orphaned Selenium container: {}", container_id);
            let _ = Command::new("docker")
                .args(["rm", "-f", container_id])
                .output();
        }
    }
    
    // Find and remove orphaned app test containers
    let app_output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.ID}} {{.Image}} {{.Names}}"])
        .output()?;
    
    if app_output.status.success() {
        let output_str = String::from_utf8_lossy(&app_output.stdout);
        let app_containers: Vec<&str> = output_str
            .lines()
            .filter(|line| {
                line.contains(" sortingoffice") && 
                line.contains("test")
            })
            .map(|line| line.split_whitespace().next().unwrap_or(""))
            .filter(|id| !id.is_empty())
            .collect();
        
        for container_id in app_containers {
            println!("[SUITE CLEANUP] Removing orphaned app container: {}", container_id);
            let _ = Command::new("docker")
                .args(["rm", "-f", container_id])
                .output();
        }
    }
    
    Ok(())
}

/// Clean up test schemas that might be left behind in the shared MySQL container.
async fn cleanup_test_schemas() -> anyhow::Result<()> {
    println!("[SUITE CLEANUP] Cleaning up test schemas...");
    
    // This would require connecting to the MySQL container to drop test schemas
    // For now, we'll rely on the container cleanup to handle this
    // In the future, we could implement schema cleanup here
    
    Ok(())
}

/// Macro to automatically register test suite lifecycle when a test starts.
/// Use this at the beginning of test functions that need suite-level cleanup.
#[macro_export]
macro_rules! test_with_suite_lifecycle {
    ($test_fn:expr) => {
        #[tokio::test]
        async fn test_with_suite_lifecycle() {
            // Register test suite lifecycle handlers
            test_suite_lifecycle::register_test_suite_lifecycle().await;
            
            // Run the test
            let result = $test_fn().await;
            
            // Return the result
            result
        }
    };
}

/// Macro to automatically register test suite lifecycle for integration tests.
/// This ensures cleanup happens when integration test suites finish.
#[macro_export]
macro_rules! integration_test_with_suite_lifecycle {
    ($test_fn:expr) => {
        #[tokio::test]
        async fn integration_test_with_suite_lifecycle() {
            // Register test suite lifecycle handlers
            test_suite_lifecycle::register_test_suite_lifecycle().await;
            
            // Run the test
            let result = $test_fn().await;
            
            // Return the result
            result
        }
    };
}

/// Macro to automatically register test suite lifecycle for UI tests.
/// This ensures cleanup happens when UI test suites finish.
#[macro_export]
macro_rules! ui_test_with_suite_lifecycle {
    ($test_fn:expr) => {
        #[tokio::test]
        async fn ui_test_with_suite_lifecycle() {
            // Register test suite lifecycle handlers
            test_suite_lifecycle::register_test_suite_lifecycle().await;
            
            // Run the test
            let result = $test_fn().await;
            
            // Return the result
            result
        }
    };
}

/// Function to be called at the end of test suites to ensure cleanup.
/// This should be called in the main test runner or after all tests complete.
pub async fn finalize_test_suite() -> anyhow::Result<()> {
    println!("[SUITE CLEANUP] Finalizing test suite...");
    
    // Clean up all test suite resources
    cleanup_test_suite_resources().await?;
    
    println!("[SUITE CLEANUP] Test suite finalized successfully");
    Ok(())
}
