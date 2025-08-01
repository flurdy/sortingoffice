//! Containerized UI Smoke Test for SortingOffice
//!
//! This module contains fully containerized end-to-end smoke tests that spin up
//! their own application and database containers, providing complete isolation.
//!
//! ## Usage
//!
//! ### Prerequisites
//! 1. Docker must be available and running
//! 2. No external application or database setup required
//!
//! ### Running the Containerized Smoke Test
//!
//! ```bash
//! # Run the containerized smoke test
//! cargo test ui_smoke_e2e_flow_testcontainers -- --nocapture
//!
//! # Or use the make target
//! make test-smoke-containerized
//! ```
//!
//! ### What the Containerized Smoke Test Does
//!
//! The containerized smoke test performs the following:
//! 1. **Database Setup**: Spins up a MySQL testcontainer
//! 2. **Application Setup**: Builds and starts the SortingOffice app container
//! 3. **Selenium Setup**: Starts a Selenium Chrome container
//! 4. **Full E2E Test**: Runs the complete workflow:
//!    - Authentication
//!    - Domain creation
//!    - Alias creation
//!    - User creation
//!    - Reports verification
//!    - Resource cleanup
//! 5. **Container Cleanup**: Automatically tears down all containers
//!
//! ### Debugging
//!
//! - All containers run in headless mode by default for CI compatibility
//! - Check the console output for detailed progress messages
//! - The test has built-in timeouts to prevent hanging
//! - All test data is generated randomly to avoid conflicts
//!
//! ### Notes
//!
//! - This test provides complete isolation but is slower than the regular smoke test
//! - All containers are automatically cleaned up when the test completes
//! - The test generates random data to avoid conflicts with existing data

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

/// Generate random string for test data
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

/// Authenticate with the application
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

/// Create a domain
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

/// Create an alias
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

/// Create a user
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

/// Check the reports page loads correctly
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

/// Delete a user
async fn delete_user(driver: &WebDriver, app_url: &str, user_email: &str) -> Result<()> {
    println!("[CONTAINERIZED SMOKE TEST] Deleting user: {user_email}");

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

    println!("[CONTAINERIZED SMOKE TEST] User deleted successfully");
    Ok(())
}

/// Delete an alias
async fn delete_alias(driver: &WebDriver, app_url: &str, alias_email: &str) -> Result<()> {
    println!("[CONTAINERIZED SMOKE TEST] Deleting alias: {alias_email}");

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

    println!("[CONTAINERIZED SMOKE TEST] Alias deleted successfully");
    Ok(())
}

/// Delete a domain
async fn delete_domain(driver: &WebDriver, app_url: &str, domain_name: &str) -> Result<()> {
    println!("[CONTAINERIZED SMOKE TEST] Deleting domain: {domain_name}");

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

    println!("[CONTAINERIZED SMOKE TEST] Domain deleted successfully");
    Ok(())
}

/// Clean up test resources
async fn cleanup_test_resources(
    driver: &WebDriver,
    app_url: &str,
    domain_name: &str,
    alias1domain: &str,
    alias2domain: &str,
    user_email: &str,
) -> Result<()> {
    println!("[CONTAINERIZED SMOKE TEST] Starting cleanup of test resources...");

    // Cleanup in reverse order: users -> aliases -> domains
    let cleanup_result = async {
        // Delete user first
        if let Err(e) = delete_user(driver, app_url, user_email).await {
            eprintln!("[CONTAINERIZED SMOKE TEST] Failed to delete user: {e:?}");
        }

        // Delete aliases
        if let Err(e) = delete_alias(driver, app_url, alias1domain).await {
            eprintln!("[CONTAINERIZED SMOKE TEST] Failed to delete alias1: {e:?}");
        }

        if let Err(e) = delete_alias(driver, app_url, alias2domain).await {
            eprintln!("[CONTAINERIZED SMOKE TEST] Failed to delete alias2: {e:?}");
        }

        // Delete domain last
        if let Err(e) = delete_domain(driver, app_url, domain_name).await {
            eprintln!("[CONTAINERIZED SMOKE TEST] Failed to delete domain: {e:?}");
        }

        println!("[CONTAINERIZED SMOKE TEST] Cleanup completed");
        Ok(())
    };

    // Run cleanup with timeout
    match timeout(Duration::from_secs(60), cleanup_result).await {
        Ok(result) => result,
        Err(_) => {
            eprintln!("[CONTAINERIZED SMOKE TEST] Cleanup timed out after 60 seconds");
            Err(anyhow::anyhow!("Cleanup timed out"))
        }
    }
}

/// Run smoke test with testcontainers support
pub async fn run_smoke_test_with_testcontainers() -> Result<()> {
    println!("[CONTAINERIZED SMOKE TEST] Starting smoke test with testcontainers...");

    // Setup test database using testcontainers
    let db_container = setup_test_db().await;
    let db_url = db_container.get_db_url();

    println!("[CONTAINERIZED SMOKE TEST] Test database ready: {db_url}");

    // Start Selenium container using the existing UI test function
    let (selenium_container, driver, selenium_port) = setup_selenium_container_and_driver().await?;
    println!("[CONTAINERIZED SMOKE TEST] Selenium container ready on port: {}", selenium_port);

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
        config_path.to_str().unwrap(),
        &container_name,
        extra_env,
    ).await?;
    
    let app_url = format!("http://{}:4000", app_ip);
    println!("[CONTAINERIZED SMOKE TEST] Application container ready at: {}", app_url);

    // Run the smoke test
    let result = run_containerized_smoke_test(&driver, &app_url).await;

    // Cleanup: drop containers and driver
    drop(driver);
    drop(app_container);
    drop(selenium_container);
    drop(db_container);

    result
}

/// Run the core smoke test logic
async fn run_containerized_smoke_test(driver: &WebDriver, app_url: &str) -> Result<()> {
    println!("[CONTAINERIZED SMOKE TEST] Starting core smoke test workflow...");

    // Run the test with proper error handling
    let result = timeout(Duration::from_secs(300), async {
        // Authenticate
        println!("[CONTAINERIZED SMOKE TEST] Authenticating...");
        authenticate_driver(driver, app_url).await?;
        println!("[CONTAINERIZED SMOKE TEST] Authentication successful");

        // Generate random test data
        let domain_name = format!("{}.test.com", rand_str()).to_lowercase();
        let alias1 = format!("alias1-{}", rand_str());
        let alias2 = format!("alias2-{}", rand_str());
        let user_name = format!("user-{}", rand_str());
        let user_maildir = format!("{}/user-{}/", domain_name, rand_str());
        let user_email = format!("{user_name}@{domain_name}");

        println!("[CONTAINERIZED SMOKE TEST] Test data generated: domain={domain_name}, user={user_email}");

        // 1. Create a new domain
        println!("[CONTAINERIZED SMOKE TEST] Creating domain...");
        create_domain(driver, app_url, &domain_name).await?;
        println!("[CONTAINERIZED SMOKE TEST] Domain created successfully");

        // 2. Create two aliases for the domain
        let alias1domain = format!("{alias1}@{domain_name}");
        let alias2domain = format!("{alias2}@{domain_name}");

        println!("[CONTAINERIZED SMOKE TEST] Creating first alias...");
        create_alias(driver, app_url, &alias1domain, &user_email).await?;
        println!("[CONTAINERIZED SMOKE TEST] First alias created successfully");

        println!("[CONTAINERIZED SMOKE TEST] Creating second alias...");
        create_alias(driver, app_url, &alias2domain, &user_email).await?;
        println!("[CONTAINERIZED SMOKE TEST] Second alias created successfully");

        // 3. Create a user for the domain
        println!("[CONTAINERIZED SMOKE TEST] Creating user...");
        create_user(
            driver,
            app_url,
            &user_email,
            &user_name,
            &user_maildir,
        )
        .await?;
        println!("[CONTAINERIZED SMOKE TEST] User created successfully");

        // 4. Check reports page
        println!("[CONTAINERIZED SMOKE TEST] Checking reports page...");
        check_reports_page(driver, app_url).await?;
        println!("[CONTAINERIZED SMOKE TEST] Reports page checked successfully");

        // 5. Cleanup test resources
        println!("[CONTAINERIZED SMOKE TEST] Starting cleanup...");
        cleanup_test_resources(
            driver,
            app_url,
            &domain_name,
            &alias1domain,
            &alias2domain,
            &user_email,
        )
        .await?;
        println!("[CONTAINERIZED SMOKE TEST] Cleanup completed successfully");

        println!("[CONTAINERIZED SMOKE TEST] All test steps completed successfully!");
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            println!("[CONTAINERIZED SMOKE TEST] ✅ Containerized smoke test completed successfully!");
            Ok(())
        }
        Ok(Err(e)) => {
            println!("[CONTAINERIZED SMOKE TEST] ❌ Containerized smoke test failed: {e}");
            Err(e)
        }
        Err(_) => {
            println!("[CONTAINERIZED SMOKE TEST] ❌ Containerized smoke test timed out after 300 seconds");
            Err(anyhow::anyhow!("Containerized smoke test timed out after 300 seconds"))
        }
    }
}

#[tokio::test]
#[ignore]
async fn ui_smoke_e2e_flow_testcontainers() -> Result<()> {
    // Run smoke test with testcontainers support
    run_smoke_test_with_testcontainers().await
}