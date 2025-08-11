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
use sortingoffice::{config_utils::ConfigUtils, test_helpers::testcontainers_setup::setup_test_db};
use std::time::Duration;
use testcontainers::core::Mount;
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;
use testcontainers::ImageExt;
use thirtyfour::prelude::*;
use tokio::time::timeout;

#[macro_use]
mod common;
use common::ui_helpers::*;
use common::testcontainer_helpers::*;

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
        let config = ConfigUtils::get_smoke_test_config();
        Self {
            app_url: config
                .app_url
                .unwrap_or_else(|| "http://host.docker.internal:3000".to_string()),
            headless: config.headless,
            timeout_seconds: config.timeout_seconds as u64,
            enable_vnc: config.enable_vnc,
        }
    }
}

/// Run the smoke test with the given configuration
pub async fn run_smoke_test_with_config(config: SmokeTestConfig) -> Result<()> {
    println!("[SMOKE TEST] Starting smoke test with config: {config:?}");

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
        .with_env_var("SE_NODE_HOST", "localhost")
        .with_mount(Mount::bind_mount("/dev/shm", "/dev/shm"));

    let selenium = AsyncRunner::start(selenium_image).await?;
    let selenium_port = selenium.get_host_port_ipv4(4444).await?;

    println!("[SMOKE TEST] Selenium container started");
    println!("[SMOKE TEST] Selenium URL: http://localhost:{selenium_port}");
    println!(
        "[SMOKE TEST] Selenium VNC URL: vnc://localhost:{}",
        selenium.get_host_port_ipv4(5900).await?
    );

    // Wait for Selenium to be ready
    println!("[SMOKE TEST] Waiting for Selenium to be ready...");
    timeout90s!(
        wait_for_selenium_ready(selenium_port, Duration::from_secs(90)),
        "Wait for selenium ready"
    )?;
    println!("[SMOKE TEST] ✅ Selenium is ready and responding");

    // Set up WebDriver capabilities
    println!("[SMOKE TEST] Setting up WebDriver capabilities...");
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

    println!("[SMOKE TEST] Chrome configured with minimal settings to avoid conflicts");

    println!("[SMOKE TEST] Connecting to WebDriver at http://localhost:{selenium_port}");
    let driver = timeout(
        Duration::from_secs(20),
        WebDriver::new(&format!("http://localhost:{selenium_port}"), caps),
    )
    .await??;
    println!("[SMOKE TEST] ✅ WebDriver connected successfully");

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
    // println!("[SMOKE TEST] Starting workflow...");

    // Step 1: Authenticate
    // println!("[SMOKE TEST] Step 1: Authenticating...");
    authenticate_driver(driver, app_url).await?;
    println!("[SMOKE TEST] ✅ Authentication successful");

    // Step 2: Create test data
    let domain_name = format!("test-{}.example.com", rand_domain_str());
    let alias1domain = format!("alias1@{domain_name}");
    let alias2domain = format!("alias2@{domain_name}");
    let user_email = format!("user@{domain_name}");
    let user_name = "Test User";
    let user_maildir = "testdir";

    println!("[SMOKE TEST] Test data prepared:");
    println!("[SMOKE TEST]   Domain: {domain_name}");
    println!("[SMOKE TEST]   Alias 1: {alias1domain}");
    println!("[SMOKE TEST]   Alias 2: {alias2domain}");
    println!("[SMOKE TEST]   User: {user_email}");

    // Step 3: Create domain
    println!("[SMOKE TEST] Step 2: Creating domain: {domain_name}");
    create_domain(driver, app_url, &domain_name).await?;
    println!("[SMOKE TEST] ✅ Domain created successfully");

    // Step 4: Create aliases
    println!("[SMOKE TEST] Step 3: Creating aliases...");
    create_alias(driver, app_url, &alias1domain, "user1@example.com").await?;
    create_alias(driver, app_url, &alias2domain, "user2@example.com").await?;
    println!("[SMOKE TEST] ✅ Aliases created successfully");

    // Step 5: Create user
    println!("[SMOKE TEST] Step 4: Creating user: {user_email}");
    create_user(driver, app_url, &user_email, user_name, user_maildir).await?;
    println!("[SMOKE TEST] ✅ User created successfully");

    // Step 6: Check reports page
    println!("[SMOKE TEST] Step 5: Checking reports page...");
    check_reports_page(driver, app_url).await?;
    println!("[SMOKE TEST] ✅ Reports page loaded successfully");

    // Step 6: Cleaning up test resources...
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

/// Find the application URL to use for testing
/// Either uses SMOKE_TEST_APP_URL environment variable or tries to find localhost:3000
async fn find_app_url() -> anyhow::Result<String> {
    // Check if SMOKE_TEST_APP_URL is set - if so, use it directly
    if let Ok(app_url) = std::env::var("SMOKE_TEST_APP_URL") {
        println!("[SMOKE TEST] Using provided app URL: {app_url}");
        return Ok(app_url);
    }

    // If no URL provided, try to use localhost:3000 with retries
    println!("[SMOKE TEST] No SMOKE_TEST_APP_URL provided, trying localhost:3000...");

    // Try to connect to localhost:3000 with retries (app might be restarting)
    let localhost_url = "http://localhost:3000";
    let client = reqwest::Client::new();
    let timeout = std::time::Duration::from_secs(30);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        match client.get(localhost_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                println!("[SMOKE TEST] Found running application at {localhost_url}");

                // For Selenium container to reach host localhost, we need to use the host's bridge IP
                // On Linux, host.docker.internal doesn't work, so we need to get the host's IP
                let host_ip = std::env::var("HOST_IP").unwrap_or_else(|_| {
                    // Try to get the host's actual IP address, not the gateway
                    let output = std::process::Command::new("ip")
                        .args(["route", "get", "8.8.8.8"])
                        .output();

                    if let Ok(output) = output {
                        if let Ok(stdout) = String::from_utf8(output.stdout) {
                            if let Some(line) = stdout.lines().next() {
                                // The source IP is the 7th field in the output
                                if let Some(src_ip) = line.split_whitespace().nth(6) {
                                    return src_ip.to_string();
                                }
                            }
                        }
                    }

                    // Fallback: try to get the IP of the default interface
                    let output = std::process::Command::new("ip")
                        .args(["route", "show", "default"])
                        .output();

                    if let Ok(output) = output {
                        if let Ok(stdout) = String::from_utf8(output.stdout) {
                            if let Some(line) = stdout.lines().next() {
                                if let Some(dev) = line.split_whitespace().nth(4) {
                                    // Get the IP of this interface
                                    let if_output = std::process::Command::new("ip")
                                        .args(["addr", "show", dev])
                                        .output();

                                    if let Ok(if_output) = if_output {
                                        if let Ok(if_stdout) = String::from_utf8(if_output.stdout) {
                                            for line in if_stdout.lines() {
                                                if line.contains("inet ")
                                                    && !line.contains("127.0.0.1")
                                                {
                                                    if let Some(ip) = line.split_whitespace().nth(1)
                                                    {
                                                        if let Some(ip_only) = ip.split('/').next()
                                                        {
                                                            return ip_only.to_string();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Final fallback to common Docker bridge IP
                    "172.17.0.1".to_string()
                });

                let app_url_for_selenium = format!("http://{host_ip}:3000");
                println!(
                    "[SMOKE TEST] Using {app_url_for_selenium} for Selenium container to reach host application"
                );

                return Ok(app_url_for_selenium);
            }
            _ => {
                // App not ready yet, wait a bit and retry
                let elapsed = start.elapsed();
                let remaining = timeout - elapsed;
                println!(
                    "[SMOKE TEST] Application not ready at {localhost_url} (elapsed: {elapsed:?}, remaining: {remaining:?})"
                );

                if remaining.as_secs() > 0 {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                } else {
                    break;
                }
            }
        }
    }

    println!("[SMOKE TEST] No application found at localhost:3000 after 30 seconds");
    println!(
        "[SMOKE TEST] Please start the application or set SMOKE_TEST_APP_URL environment variable"
    );
    Err(anyhow::anyhow!(
        "No application found at localhost:3000 and no SMOKE_TEST_APP_URL provided"
    ))
}

/// Main smoke test function (environment-based)
#[tokio::test]
#[ignore]
async fn ui_smoke_e2e_flow() -> Result<()> {
    println!("[SMOKE TEST] Starting environment-based smoke test...");

    // Find the application URL to test
    let app_url = find_app_url().await?;

    // Set up Selenium container and driver using the new helper
    let (selenium_container, driver, _selenium_port) =
        setup_selenium_with_default_args().await?;

    // Run the actual test workflow
    let test_result = run_smoke_test_workflow(&driver, &app_url).await;

    // Clean up using the new helper
    cleanup_selenium_test_env(driver, selenium_container, None).await?;

    test_result
}

/// Testcontainers-based smoke test (separate function to avoid --ignored conflicts)
#[tokio::test]
async fn ui_smoke_containerized_e2e_flow() -> Result<()> {
    println!("[SMOKE TEST] Starting testcontainers smoke test...");

    // Set up test database
    let test_db = setup_test_db().await;
    let db_url = test_db.get_db_url();
    println!("[SMOKE TEST] Test database ready: {db_url}");

    // Start the application container using the shared network approach
    let config_path = std::env::current_dir()?
        .join("config")
        .join("config.docker.toml");
    let config_path_str = config_path.to_str().unwrap().to_string();
    let extra_env: Vec<(&str, &str)> = vec![("TESTING", "true"), ("RUST_ENV", "test")];

    // Use the shared network approach like ui_containerized tests
    let (app_container, _) =
        setup_app_on_shared_network(&test_db.get_schema(), None, &config_path_str, &extra_env)
            .await?;

    // Start selenium on shared network with extra args
    let extra_args = vec![
        "--disable-http2".to_string(),
        "--disable-quic".to_string(),
        "--proxy-server=direct://".to_string(),
        "--proxy-bypass-list=*".to_string(),
        "--lang=en".to_string(),
    ];
    let (selenium_container, driver, _selenium_port) =
        setup_selenium_with_custom_args(extra_args).await?;

    // Determine app container IP on the shared bridge network
    let app_ip = get_container_ip_on_network(&app_container.id(), "sortingoffice-e2e").await?;
    let app_url = format!("http://{app_ip}:3000");

    // Wait for the app to be reachable from inside the Selenium container
    let health_url = format!("{}/health", app_url.trim_end_matches('/'));
    println!("[DEBUG] Health check URL: {health_url}");

    // Wait for app to be ready
    let start = std::time::Instant::now();
    let max_wait = Duration::from_secs(30);
    loop {
        match driver.get(&health_url).await {
            Ok(_) => break,
            Err(_) => {
                if start.elapsed() >= max_wait {
                    return Err(anyhow::anyhow!(
                        "Timed out waiting for app from Selenium at {}",
                        health_url
                    ));
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    println!("[SMOKE TEST] App container ready at: {app_url}");
    println!("[SMOKE TEST] Using database URL: {db_url}");
    println!(
        "[SMOKE TEST] Config file mounted from: {}",
        config_path.to_str().unwrap()
    );

    // Debug: Check container logs to see what config is being loaded
    if let Ok(logs) = std::process::Command::new("docker")
        .args(["logs", app_container.id()])
        .output()
    {
        println!(
            "[SMOKE TEST] Container logs: {}",
            String::from_utf8_lossy(&logs.stdout)
        );
    }

    // Run the actual test
    let test_result = run_smoke_test_workflow(&driver, &app_url).await;

    // Clean up containers using the new helper
    cleanup_selenium_test_env(driver, selenium_container, Some(app_container)).await?;

    test_result
}

/// Demo test using the new testcontainer helpers
#[tokio::test]
#[ignore]
async fn ui_smoke_demo_with_testcontainer_helpers() -> Result<()> {
    println!("[DEMO] Starting smoke test with new testcontainer helpers...");

    // Find the application URL to test
    let app_url = find_app_url().await?;

    // Set up Selenium container using the new helper with default args
    let (selenium_container, driver, selenium_port) = 
        setup_selenium_with_default_args().await?;
    
    println!("[DEMO] Selenium container started on port: {}", selenium_port);

    // Run a simple test workflow
    let test_result = run_smoke_test_workflow(&driver, &app_url).await;

    // Clean up using the new helper
    cleanup_selenium_test_env(driver, selenium_container, None).await?;
    
    println!("[DEMO] Cleanup completed successfully");

    test_result
}
