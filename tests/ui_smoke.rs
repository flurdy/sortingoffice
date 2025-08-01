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
use rand::RngCore;
use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers::GenericImage;
use testcontainers::ImageExt;
use testcontainers::core::Mount;
use thirtyfour::prelude::*;
use std::time::Duration;
use std::process::Command;
use tokio::time::timeout;

// Helper macro for timeouts
macro_rules! timeout60s {
    ($expr:expr, $desc:expr) => {
        timeout(Duration::from_secs(60), $expr)
            .await
            .map_err(|_| anyhow::anyhow!(concat!("Timeout (60s) on: ", $desc)))?
    };
}

macro_rules! timeout30s {
    ($expr:expr, $desc:expr) => {
        timeout(Duration::from_secs(30), $expr)
            .await
            .map_err(|_| anyhow::anyhow!(concat!("Timeout (30s) on: ", $desc)))?
    };
}

macro_rules! timeout90s {
    ($expr:expr, $desc:expr) => {
        timeout(Duration::from_secs(90), $expr)
            .await
            .map_err(|_| anyhow::anyhow!(concat!("Timeout (90s) on: ", $desc)))?
    };
}

/// Find a free port for the application
fn find_free_port() -> u16 {
    use std::net::TcpListener;
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Get container bridge IP
async fn get_container_bridge_ip(container_id: &str) -> anyhow::Result<String> {
    let output = Command::new("docker")
        .args(["inspect", "-f", "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}", container_id])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to get container IP: {}", e))?;

    let ip = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse container IP: {}", e))?
        .trim()
        .to_string();

    if ip.is_empty() {
        return Err(anyhow::anyhow!(
            "No IP address found for container {}",
            container_id
        ));
    }
    Ok(ip)
}

/// Wait for selenium to be ready
async fn wait_for_selenium_ready(port: u16, max_wait: Duration) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{port}/status");
    let start = std::time::Instant::now();
    
    while start.elapsed() < max_wait {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            _ => {
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
    
    Err(anyhow::anyhow!(
        "Timed out waiting for Selenium on port {}",
        port
    ))
}

/// Setup app container
async fn setup_app_container(
    db_url: &str,
    host_port: u16,
    _admin_username: &str,
    _admin_password_hash: &str,
    config_path: &str,
    container_name: &str,
    extra_env: &[(&str, &str)],
) -> anyhow::Result<(ContainerAsync<GenericImage>, String /* bridge IP */)> {
    let mut app_image = GenericImage::new("sortingoffice", "latest")
        .with_env_var("DATABASE_URL", db_url)
        .with_env_var("PORT", "4000")
        .with_mapped_port(host_port, 4000.into())
        .with_container_name(container_name)
        .with_mount(Mount::bind_mount(config_path, "/app/config/config.toml"));
    for (key, value) in extra_env {
        app_image = app_image.with_env_var(*key, *value);
    }
    let app_container = match AsyncRunner::start(app_image).await {
        Ok(c) => c,
        Err(e) => {
            println!("[ERROR] Failed to start app container: {e:?}");
            return Err(e.into());
        }
    };
    let app_id = app_container.id();
    let app_ip = get_container_bridge_ip(app_id).await?;
    let health_url = format!("http://{app_ip}:4000/health");
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(30);
    loop {
        match client.get(&health_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                break;
            }
            Ok(_) | Err(_) => {}
        }
        if start.elapsed() > timeout {
            return Err(anyhow::anyhow!("App /health not healthy after 30s"));
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Ok((app_container, app_ip))
}

/// Setup selenium container and driver
async fn setup_selenium_container_and_driver(
) -> anyhow::Result<(ContainerAsync<GenericImage>, WebDriver, u16)> {
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
        .with_mount(Mount::bind_mount("/dev/shm", "/dev/shm"));
    let selenium = AsyncRunner::start(selenium_image).await?;
    let selenium_port = selenium.get_host_port_ipv4(4444).await?;
    timeout90s!(
        wait_for_selenium_ready(selenium_port, Duration::from_secs(90)),
        "Wait for selenium ready"
    )?;
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless=new")?;
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
    Ok((selenium, driver, selenium_port))
}

/// Configuration for smoke test execution
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
            headless: std::env::var("SMOKE_TEST_HEADLESS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            timeout_seconds: std::env::var("SMOKE_TEST_TIMEOUT")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            enable_vnc: std::env::var("SMOKE_TEST_VNC")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        }
    }
}

/// Wrapper function for running smoke tests with configurable parameters
pub async fn run_smoke_test_with_config(config: SmokeTestConfig) -> Result<()> {
    println!("[SMOKE TEST] Starting smoke test with configuration:");
    println!("  App URL: {}", config.app_url);
    println!("  Headless: {}", config.headless);
    println!("  Timeout: {}s", config.timeout_seconds);
    println!("  VNC: {}", config.enable_vnc);

    // Setup Chrome capabilities
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;

    if config.headless {
        caps.add_arg("--headless")?;
    } else {
        caps.add_arg("--window-size=1200,900")?;
    }

    // Create WebDriver with timeout
    let driver = timeout(
        Duration::from_secs(30),
        WebDriver::new("http://localhost:4444", caps),
    )
    .await??;

    println!("[SMOKE TEST] WebDriver created successfully");

    // Run the test with proper error handling
    let result = timeout(Duration::from_secs(config.timeout_seconds), async {
        // Authenticate
        println!("[SMOKE TEST] Authenticating...");
        authenticate_driver(&driver, &config.app_url).await?;
        println!("[SMOKE TEST] Authentication successful");

        // Generate random test data
        let domain_name = format!("{}.test.com", rand_str()).to_lowercase();
        let alias1 = format!("alias1-{}", rand_str());
        let alias2 = format!("alias2-{}", rand_str());
        let user_name = format!("user-{}", rand_str());
        let user_maildir = format!("{}/user-{}/", domain_name, rand_str());
        let user_email = format!("{user_name}@{domain_name}");

        println!("[SMOKE TEST] Test data generated: domain={domain_name}, user={user_email}");

        // 1. Create a new domain
        println!("[SMOKE TEST] Creating domain...");
        create_domain(&driver, &config.app_url, &domain_name).await?;
        println!("[SMOKE TEST] Domain created successfully");

        // 2. Create two aliases for the domain
        let alias1domain = format!("{alias1}@{domain_name}");
        let alias2domain = format!("{alias2}@{domain_name}");

        println!("[SMOKE TEST] Creating first alias...");
        create_alias(&driver, &config.app_url, &alias1domain, &user_email).await?;
        println!("[SMOKE TEST] First alias created successfully");

        println!("[SMOKE TEST] Creating second alias...");
        create_alias(&driver, &config.app_url, &alias2domain, &user_email).await?;
        println!("[SMOKE TEST] Second alias created successfully");

        // 3. Create a user for the domain
        println!("[SMOKE TEST] Creating user...");
        create_user(
            &driver,
            &config.app_url,
            &user_email,
            &user_name,
            &user_maildir,
        )
        .await?;
        println!("[SMOKE TEST] User created successfully");

        // 4. Check reports page
        println!("[SMOKE TEST] Checking reports page...");
        check_reports_page(&driver, &config.app_url).await?;
        println!("[SMOKE TEST] Reports page checked successfully");

        // 5. Cleanup test resources
        println!("[SMOKE TEST] Starting cleanup...");
        cleanup_test_resources(
            &driver,
            &config.app_url,
            &domain_name,
            &alias1domain,
            &alias2domain,
            &user_email,
        )
        .await?;
        println!("[SMOKE TEST] Cleanup completed successfully");

        println!("[SMOKE TEST] All test steps completed successfully!");
        Ok(())
    })
    .await;

    // Cleanup: Always try to quit the driver
    println!("[SMOKE TEST] Cleaning up WebDriver...");
    match timeout(Duration::from_secs(10), driver.quit()).await {
        Ok(Ok(_)) => {
            println!("[SMOKE TEST] WebDriver quit successfully");
        }
        _ => {
            println!("[SMOKE TEST] WebDriver quit failed or timed out");
        }
    }

    match result {
        Ok(Ok(())) => {
            println!("[SMOKE TEST] ✅ Smoke test completed successfully!");
            Ok(())
        }
        Ok(Err(e)) => {
            println!("[SMOKE TEST] ❌ Smoke test failed: {e}");
            Err(e)
        }
        Err(_) => {
            println!(
                "[SMOKE TEST] ❌ Smoke test timed out after {} seconds",
                config.timeout_seconds
            );
            Err(anyhow::anyhow!(
                "Smoke test timed out after {} seconds",
                config.timeout_seconds
            ))
        }
    }
}

/// Run smoke test with testcontainers support
pub async fn run_smoke_test_with_testcontainers() -> Result<()> {
    println!("[SMOKE TEST] Starting smoke test with testcontainers...");

    // Setup test database using testcontainers
    let db_container = setup_test_db().await;
    let db_url = db_container.get_db_url();

    println!("[SMOKE TEST] Test database ready: {db_url}");

    // Start Selenium container using the existing UI test function
    let (selenium_container, driver, selenium_port) = setup_selenium_container_and_driver().await?;
    println!("[SMOKE TEST] Selenium container ready on port: {}", selenium_port);

    // Start the application container using the existing UI test function
    let config_path = std::env::current_dir()?.join("config").join("config.toml");
    let container_name = format!("smoke-test-app-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis());
    let extra_env = &[
        ("PRIMARY_DB_URL", db_url.as_str()),
        ("BACKUP1_DB_URL", db_url.as_str()),
        ("BACKUP2_DB_URL", db_url.as_str()),
    ];
    
    let (app_container, app_ip) = setup_app_container(
        &db_url,
        find_free_port(),
        "admin",
        "$2a$12$o8thacsiGCRhN1JN8xnW6e0KqNb7KrSgM67xxa62RKoAC9fOPf.aO",
        config_path.to_str().unwrap(),
        &container_name,
        extra_env,
    ).await?;
    
    let app_url = format!("http://{}:4000", app_ip);
    println!("[SMOKE TEST] Application container ready at: {}", app_url);

    let config = SmokeTestConfig {
        app_url,
        headless: true, // Headless for CI
        timeout_seconds: 300,
        enable_vnc: false,
    };

    // Run the smoke test
    let result = run_smoke_test_with_config(config).await;

    // Cleanup: drop containers and driver
    drop(driver);
    drop(app_container);
    drop(selenium_container);
    drop(db_container);

    result
}

fn rand_str() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rngs::ThreadRng::default();
    (0..8)
        .map(|_| {
            let idx = (rng.next_u32() as usize) % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}

async fn authenticate_driver(driver: &WebDriver, base_url: &str) -> Result<()> {
    let logout_url = format!("{}/logout", base_url.trim_end_matches('/'));
    let login_url = format!("{}/login", base_url.trim_end_matches('/'));

    // First logout to ensure clean state
    timeout60s!(driver.get(&logout_url), "Navigate to logout page")?;
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Navigate to login page
    timeout60s!(driver.get(&login_url), "Navigate to login page")?;

    // Find and fill username field
    let username_field = timeout60s!(
        driver.find(By::Css("input[name='id']")),
        "Find username field"
    )?;
    timeout60s!(username_field.send_keys("admin"), "Fill username field")?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Find and fill password field
    let password_field = timeout60s!(
        driver.find(By::Css("input[name='password']")),
        "Find password field"
    )?;
    timeout60s!(password_field.send_keys("admin123"), "Fill password field")?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Find and click submit button
    let submit_button = timeout60s!(
        driver.find(By::XPath(
            "//button[@type='submit' and contains(text(), 'Sign in')]"
        )),
        "Find submit button"
    )?;

    // Check if button is clickable
    let is_enabled = timeout60s!(submit_button.is_enabled(), "Check button enabled")?;
    let is_displayed = timeout60s!(submit_button.is_displayed(), "Check button displayed")?;

    if is_enabled && is_displayed {
        timeout60s!(submit_button.click(), "Click submit button")?;
    } else {
        return Err(anyhow::anyhow!(
            "Submit button is not clickable: enabled={}, displayed={}",
            is_enabled,
            is_displayed
        ));
    }

    // Wait for redirect and verify authentication
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    let current_url = timeout60s!(driver.current_url(), "Get current URL")?;

    if current_url.as_str().contains("/login") {
        return Err(anyhow::anyhow!(
            "Still on login page after authentication attempt"
        ));
    }

    Ok(())
}

async fn create_domain(driver: &WebDriver, app_url: &str, domain_name: &str) -> Result<()> {
    let domain_url = format!("{app_url}/domains");
    timeout60s!(driver.get(&domain_url), "Navigate to domains list page")?;

    let add_domain_button = timeout60s!(
        driver.find(By::Id("add-domain-button")),
        "Find Add Domain button"
    )?;
    timeout30s!(add_domain_button.click(), "Click Add Domain button")?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let domain_input = timeout30s!(
        driver.find(By::Css("input[name='domain']")),
        "Find domain input"
    )?;
    assert!(
        domain_input.is_displayed().await.unwrap_or(false),
        "Domain input is not displayed"
    );
    timeout60s!(domain_input.send_keys(domain_name), "Type domain name")?;

    let submit_btn = timeout60s!(
        driver.find(By::Id("domain-submit-button")),
        "Find submit button"
    )?;
    assert!(
        submit_btn.is_displayed().await.unwrap_or(false),
        "Domain submit button is not displayed"
    );
    timeout60s!(submit_btn.click(), "Submit domain form")?;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let page_source = timeout60s!(driver.source(), "Get page source after domain create")?;
    assert!(
        page_source.contains(domain_name),
        "Domain should appear after creation"
    );

    Ok(())
}

async fn create_alias(
    driver: &WebDriver,
    app_url: &str,
    alias_email: &str,
    destination: &str,
) -> Result<()> {
    let aliases_url = format!("{app_url}/aliases");
    timeout60s!(driver.get(&aliases_url), "Navigate to aliases page")?;

    let add_alias_btn = timeout60s!(
        driver.find(By::Id("add-alias-button")),
        "Find Add Alias button"
    )?;
    timeout30s!(add_alias_btn.click(), "Click Add Alias button")?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mail_input = timeout30s!(
        driver.find(By::Css("input[name='mail']")),
        "Find mail input field"
    )?;
    timeout60s!(mail_input.send_keys(alias_email), "Type alias email")?;

    let dest_input = timeout60s!(
        driver.find(By::Css("input[name='destination']")),
        "Find destination input"
    )?;
    timeout60s!(dest_input.send_keys(destination), "Type destination")?;

    let submit_btn = timeout60s!(
        driver.find(By::Id("alias-submit-button")),
        "Find submit button"
    )?;
    timeout60s!(submit_btn.click(), "Submit alias form")?;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let page_source = timeout60s!(driver.source(), "Get page source after alias create")?;
    assert!(
        page_source.contains(alias_email),
        "Alias should appear after creation"
    );

    Ok(())
}

async fn create_user(
    driver: &WebDriver,
    app_url: &str,
    user_email: &str,
    user_name: &str,
    user_maildir: &str,
) -> Result<()> {
    let users_url = format!("{app_url}/users");
    timeout60s!(driver.get(&users_url), "Navigate to users page")?;

    let add_user_btn = timeout60s!(
        driver.find(By::Id("add-user-button")),
        "Find Add User button"
    )?;
    timeout30s!(add_user_btn.click(), "Click Add User button")?;

    let user_id_input = timeout60s!(
        driver.find(By::Css("input[name='id']")),
        "Find user id input"
    )?;
    timeout60s!(user_id_input.send_keys(user_email), "Type user id")?;

    let user_name_input = timeout60s!(
        driver.find(By::Css("input[name='name']")),
        "Find user name input"
    )?;
    timeout60s!(user_name_input.send_keys(user_name), "Type user name")?;

    let user_maildir_input = timeout60s!(
        driver.find(By::Css("input[name='maildir']")),
        "Find user maildir input"
    )?;
    timeout60s!(
        user_maildir_input.send_keys(user_maildir),
        "Type user maildir"
    )?;

    let user_submit_btn = timeout60s!(
        driver.find(By::Id("user-submit-button")),
        "Find submit button for user"
    )?;
    timeout60s!(user_submit_btn.click(), "Submit user form")?;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let page_source = timeout60s!(driver.source(), "Get page source after user create")?;
    assert!(
        page_source.contains(user_email),
        "User should appear after creation"
    );

    Ok(())
}

async fn check_reports_page(driver: &WebDriver, app_url: &str) -> Result<()> {
    let reports_url = format!("{app_url}/reports");
    timeout60s!(driver.get(&reports_url), "Navigate to reports page")?;

    let reports_page_source = timeout60s!(driver.source(), "Get page source for reports")?;
    assert!(
        reports_page_source.contains("Reports") || reports_page_source.contains("Alias"),
        "Reports page should load"
    );

    Ok(())
}

async fn delete_user(driver: &WebDriver, app_url: &str, user_email: &str) -> Result<()> {
    println!("[SMOKE TEST] Deleting user: {user_email}");

    // Navigate to users page
    let users_url = format!("{}/users", app_url.trim_end_matches('/'));
    timeout60s!(driver.get(&users_url), "Navigate to users page")?;

    // Find and click the delete button for the user
    let delete_button = timeout30s!(
        driver.find(By::XPath(format!(
            "//tr[contains(., '{user_email}')]//button[contains(@class, 'delete')]"
        ))),
        "Find delete button for user"
    )?;
    timeout30s!(delete_button.click(), "Click delete button")?;

    // Confirm deletion in modal
    let confirm_button = timeout30s!(
        driver.find(By::Css("button.btn-danger")),
        "Find confirm delete button"
    )?;
    timeout30s!(confirm_button.click(), "Click confirm delete")?;

    // Wait for deletion to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    println!("[SMOKE TEST] User deleted successfully");
    Ok(())
}

async fn delete_alias(driver: &WebDriver, app_url: &str, alias_email: &str) -> Result<()> {
    println!("[SMOKE TEST] Deleting alias: {alias_email}");

    // Navigate to aliases page
    let aliases_url = format!("{}/aliases", app_url.trim_end_matches('/'));
    timeout60s!(driver.get(&aliases_url), "Navigate to aliases page")?;

    // Find and click the delete button for the alias
    let delete_button = timeout30s!(
        driver.find(By::XPath(format!(
            "//tr[contains(., '{alias_email}')]//button[contains(@class, 'delete')]"
        ))),
        "Find delete button for alias"
    )?;
    timeout30s!(delete_button.click(), "Click delete button")?;

    // Confirm deletion in modal
    let confirm_button = timeout30s!(
        driver.find(By::Css("button.btn-danger")),
        "Find confirm delete button"
    )?;
    timeout30s!(confirm_button.click(), "Click confirm delete")?;

    // Wait for deletion to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    println!("[SMOKE TEST] Alias deleted successfully");
    Ok(())
}

async fn delete_domain(driver: &WebDriver, app_url: &str, domain_name: &str) -> Result<()> {
    println!("[SMOKE TEST] Deleting domain: {domain_name}");

    // Navigate to domains page
    let domains_url = format!("{}/domains", app_url.trim_end_matches('/'));
    timeout60s!(driver.get(&domains_url), "Navigate to domains page")?;

    // Find and click the delete button for the domain
    let delete_button = timeout30s!(
        driver.find(By::XPath(format!(
            "//tr[contains(., '{domain_name}')]//button[contains(@class, 'delete')]"
        ))),
        "Find delete button for domain"
    )?;
    timeout30s!(delete_button.click(), "Click delete button")?;

    // Confirm deletion in modal
    let confirm_button = timeout30s!(
        driver.find(By::Css("button.btn-danger")),
        "Find confirm delete button"
    )?;
    timeout30s!(confirm_button.click(), "Click confirm delete")?;

    // Wait for deletion to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    println!("[SMOKE TEST] Domain deleted successfully");
    Ok(())
}

async fn cleanup_test_resources(
    driver: &WebDriver,
    app_url: &str,
    domain_name: &str,
    alias1domain: &str,
    alias2domain: &str,
    user_email: &str,
) -> Result<()> {
    println!("[SMOKE TEST] Starting cleanup of test resources...");

    // Cleanup in reverse order: users -> aliases -> domains
    let cleanup_result = async {
        // Delete user first
        if let Err(e) = delete_user(driver, app_url, user_email).await {
            eprintln!("[SMOKE TEST] Failed to delete user: {e:?}");
        }

        // Delete aliases
        if let Err(e) = delete_alias(driver, app_url, alias1domain).await {
            eprintln!("[SMOKE TEST] Failed to delete alias1: {e:?}");
        }

        if let Err(e) = delete_alias(driver, app_url, alias2domain).await {
            eprintln!("[SMOKE TEST] Failed to delete alias2: {e:?}");
        }

        // Delete domain last
        if let Err(e) = delete_domain(driver, app_url, domain_name).await {
            eprintln!("[SMOKE TEST] Failed to delete domain: {e:?}");
        }

        println!("[SMOKE TEST] Cleanup completed");
        Ok(())
    };

    // Run cleanup with timeout
    match timeout(Duration::from_secs(60), cleanup_result).await {
        Ok(result) => result,
        Err(_) => {
            eprintln!("[SMOKE TEST] Cleanup timed out after 60 seconds");
            Err(anyhow::anyhow!("Cleanup timed out"))
        }
    }
}

#[tokio::test]
#[ignore]
async fn ui_smoke_e2e_flow() -> Result<()> {
    // Use the new wrapper with default configuration
    let config = SmokeTestConfig::default();
    run_smoke_test_with_config(config).await
}

#[tokio::test]
#[ignore]
async fn ui_smoke_e2e_flow_testcontainers() -> Result<()> {
    // Run smoke test with testcontainers support
    run_smoke_test_with_testcontainers().await
}
