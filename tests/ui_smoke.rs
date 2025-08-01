//! UI Smoke Test for SortingOffice
//!
//! This module contains end-to-end smoke tests that can be run against a running
//! SortingOffice application to verify the complete user workflow.
//!
//! ## Usage
//!
//! ### Prerequisites
//! 1. Start the Selenium container: `make selenium-up` or `docker compose --profile test up -d selenium`
//! 2. Start the SortingOffice application: `cargo run` or `make up`
//! 3. Ensure the application is accessible at the expected URL (default: http://host.docker.internal:3000)
//!
//! ### Running the Smoke Test
//!
//! ```bash
//! # Run with default URL (http://host.docker.internal:3000)
//! cargo test ui_smoke_e2e_flow -- --nocapture
//!
//! # Run with custom URL
//! SMOKE_TEST_APP_URL=http://localhost:3000 cargo test ui_smoke_e2e_flow -- --nocapture
//!
//! # Run the VNC test for debugging (shows browser for 30 seconds)
//! cargo test minimal_vnc_browser_test -- --nocapture
//! ```
//!
//! ### What the Smoke Test Does
//!
//! The smoke test performs the following workflow:
//! 1. **Authentication**: Logs in with admin credentials
//! 2. **Domain Creation**: Creates a new test domain with random name
//! 3. **Alias Creation**: Creates two aliases for the domain
//! 4. **User Creation**: Creates a user account for the domain
//! 5. **Reports Check**: Verifies the reports page loads correctly
//!
//! ### Debugging
//!
//! - The test uses a visible browser (not headless) so you can watch the automation
//! - Use VNC viewer to connect to localhost:5900 to see the browser
//! - Check the console output for detailed progress messages
//! - The test has a 5-minute total timeout to prevent hanging
//!
//! ### Environment Variables
//!
//! - `SMOKE_TEST_APP_URL`: URL of the running SortingOffice application (default: http://host.docker.internal:3000)
//!
//! ### Notes
//!
//! - This test is marked with `#[ignore]` by default to prevent accidental execution in CI
//! - The test generates random data to avoid conflicts with existing data
//! - All test data is cleaned up automatically when the test completes

use anyhow::Result;
use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;
use std::time::Duration;
use testcontainers::core::Mount;
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;
use testcontainers::ImageExt;
use thirtyfour::prelude::*;
use tokio::time::timeout;

#[macro_use]
mod ui_helpers;
use ui_helpers::*;

/// Configuration for smoke test execution
#[derive(Debug, Clone)]
pub struct SmokeTestConfig {
    /// URL of the application to test
    pub app_url: String,
    /// Whether to run in headless mode (for CI)
    pub headless: bool,
    /// Timeout for the entire test (in seconds)
    pub timeout_seconds: u64,
    /// Whether to enable VNC for debugging
    pub enable_vnc: bool,
}

impl Default for SmokeTestConfig {
    fn default() -> Self {
        Self {
            app_url: std::env::var("SMOKE_TEST_APP_URL")
                .unwrap_or_else(|_| "http://host.docker.internal:3000".to_string()),
            headless: false,
            timeout_seconds: 300, // 5 minutes
            enable_vnc: true,
        }
    }
}

/// Run the smoke test with the given configuration
pub async fn run_smoke_test_with_config(config: SmokeTestConfig) -> Result<()> {
    println!("[SMOKE TEST] Starting smoke test with config: {:?}", config);

    // Set up Selenium with VNC for debugging
    let selenium_image = GenericImage::new("selenium/standalone-chrome", "latest")
        .with_env_var("SE_NODE_MAX_SESSIONS", "1")
        .with_env_var("SE_NODE_OVERRIDE_MAX_SESSIONS", "true")
        .with_env_var("SE_NODE_SESSION_TIMEOUT", "300")
        .with_env_var("SE_START_XVFB", "false")
        .with_env_var("SE_SCREEN_WIDTH", "1920")
        .with_env_var("SE_SCREEN_HEIGHT", "1080")
        .with_env_var("SE_SCREEN_DEPTH", "24")
        .with_env_var("SE_SCREEN_DPI", "96")
        .with_env_var("SE_SCREEN_RESOLUTION", "1920x1080x24")
        .with_env_var("SE_VNC_NO_PASSWORD", "1")
        .with_env_var("SE_NODE_GRID_URL", "http://localhost:4444")
        .with_env_var("SE_NODE_HOST", "localhost")
        .with_env_var("SE_EVENT_BUS_HOST", "localhost")
        .with_env_var("SE_EVENT_BUS_PUBLISH_PORT", "4442")
        .with_env_var("SE_EVENT_BUS_SUBSCRIBE_PORT", "4443")
        .with_env_var("SE_NODE_GRID_URL", "http://localhost:4444")
        .with_mount(Mount::bind_mount("/dev/shm", "/dev/shm"));

    let selenium = AsyncRunner::start(selenium_image).await?;
    let selenium_port = selenium.get_host_port_ipv4(4444).await?;

    // Wait for Selenium to be ready
    timeout90s!(
        wait_for_selenium_ready(selenium_port, Duration::from_secs(90)),
        "Wait for selenium ready"
    )?;

    // Set up WebDriver capabilities
    let mut caps = DesiredCapabilities::chrome();
    if config.headless {
        caps.add_arg("--headless=new")?;
    }
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--allow-running-insecure-content")?;
    caps.add_arg("--remote-debugging-port=9222")?;
    caps.add_arg("--whitelisted-ips=")?;
    caps.add_arg("--disable-features=VizDisplayCompositor")?;

    let driver = timeout(
        Duration::from_secs(20),
        WebDriver::new(&format!("http://localhost:{selenium_port}"), caps),
    )
    .await??;

    // Run the actual test with timeout
    let test_result = timeout(
        Duration::from_secs(config.timeout_seconds),
        run_smoke_test_workflow(&driver, &config.app_url),
    )
    .await;

    // Clean up
    let _ = driver.quit().await;
    let _ = selenium.stop().await;

    match test_result {
        Ok(result) => result,
        Err(_) => Err(anyhow::anyhow!(
            "Smoke test timed out after {} seconds",
            config.timeout_seconds
        )),
    }
}

/// Run the smoke test workflow
async fn run_smoke_test_workflow(driver: &WebDriver, app_url: &str) -> Result<()> {
    println!("[SMOKE TEST] Starting workflow...");

    // Step 1: Authenticate
    println!("[SMOKE TEST] Step 1: Authenticating...");
    authenticate_driver(driver, app_url).await?;
    println!("[SMOKE TEST] ✅ Authentication successful");

    // Step 2: Create test data
    let domain_name = format!("test-{}.example.com", rand_str());
    let alias1domain = format!("alias1@{}", domain_name);
    let alias2domain = format!("alias2@{}", domain_name);
    let user_email = format!("user@{}", domain_name);
    let user_name = "Test User";
    let user_maildir = "testdir";

    // Step 3: Create domain
    println!("[SMOKE TEST] Step 2: Creating domain: {}", domain_name);
    create_domain(driver, app_url, &domain_name).await?;
    println!("[SMOKE TEST] ✅ Domain created successfully");

    // Step 4: Create aliases
    println!("[SMOKE TEST] Step 3: Creating aliases...");
    create_alias(driver, app_url, &alias1domain, "user1@example.com").await?;
    create_alias(driver, app_url, &alias2domain, "user2@example.com").await?;
    println!("[SMOKE TEST] ✅ Aliases created successfully");

    // Step 5: Create user
    println!("[SMOKE TEST] Step 4: Creating user: {}", user_email);
    create_user(driver, app_url, &user_email, user_name, user_maildir).await?;
    println!("[SMOKE TEST] ✅ User created successfully");

    // Step 6: Check reports page
    println!("[SMOKE TEST] Step 5: Checking reports page...");
    check_reports_page(driver, app_url).await?;
    println!("[SMOKE TEST] ✅ Reports page loaded successfully");

    // Step 7: Cleanup test resources
    println!("[SMOKE TEST] Step 6: Cleaning up test resources...");
    cleanup_test_resources(
        driver,
        app_url,
        &domain_name,
        &alias1domain,
        &alias2domain,
        &user_email,
    )
    .await?;
    println!("[SMOKE TEST] ✅ Cleanup completed");

    println!("[SMOKE TEST] 🎉 All smoke test steps completed successfully!");
    Ok(())
}

/// Main smoke test function (environment-based)
#[tokio::test]
#[ignore]
async fn ui_smoke_e2e_flow() -> Result<()> {
    let config = SmokeTestConfig::default();
    run_smoke_test_with_config(config).await
}

/// Testcontainers-based smoke test (separate function to avoid --ignored conflicts)
#[tokio::test]
async fn ui_smoke_containerized_e2e_flow() -> Result<()> {
    run_smoke_test_with_testcontainers().await
}

/// Run smoke test with testcontainers (database + app + selenium)
pub async fn run_smoke_test_with_testcontainers() -> Result<()> {
    println!("[SMOKE TEST] Starting testcontainers smoke test...");

    // Set up test database
    let test_db = setup_test_db().await;
    let db_url = test_db.get_db_url();
    println!("[SMOKE TEST] Test database ready: {}", db_url);

    // Start the application container using the existing UI test function
    let config_path = std::env::current_dir()?
        .join("config")
        .join("config.docker.toml");
    let container_name = format!(
        "smoke-test-app-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Convert the database URL to use host.docker.internal for container networking
    let db_url_for_container = db_url.replace("127.0.0.1", "host.docker.internal");
    let extra_env = &[("DATABASE_URL", db_url_for_container.as_str())];

    let (app_container, app_ip) = setup_app_container(
        &db_url_for_container,
        find_free_port(),
        "admin",
        "$2y$10$92IXUNpkjO0rOQ5byMi.Ye4oKoEa3Ro9llC/.og/at2.uheWG/igi", // password
        config_path.to_str().unwrap(),
        &container_name,
        extra_env,
    )
    .await?;

    let app_url = format!("http://{}:4000", app_ip);
    println!("[SMOKE TEST] App container ready at: {}", app_url);

    // Set up Selenium container and driver
    let (selenium_container, driver, _selenium_port) =
        setup_selenium_container_and_driver().await?;

    // Run the actual test
    let test_result = run_smoke_test_workflow(&driver, &app_url).await;

    // Clean up containers
    let _ = driver.quit().await;
    let _ = selenium_container.stop().await;
    let _ = app_container.stop().await;

    test_result
}
