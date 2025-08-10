//! Shared UI test helper functions
//!
//! This module contains helper functions that are shared between ui_smoke.rs and ui_containerized.rs
//! to eliminate code duplication.

use anyhow::Result;
use diesel::RunQueryDsl;
use diesel_migrations::MigrationHarness;
use rand::RngCore;
use sortingoffice::test_helpers::testcontainers_setup::TestContainer;
use sortingoffice::test_helpers::testcontainers_setup::MIGRATIONS;
use std::net::TcpListener;
use std::process::Command;
use testcontainers::core::Mount;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers::GenericImage;
use testcontainers::ImageExt;
use testcontainers_modules::mysql::Mysql;
use thirtyfour::prelude::*;
use tokio::time::{timeout, Duration};

/// Find a free port for the application
#[allow(dead_code)]
pub fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .unwrap()
        .port()
}

// Helper macros for timeouts
macro_rules! timeout30s {
    ($expr:expr, $desc:expr) => {
        timeout(Duration::from_secs(30), $expr)
            .await
            .map_err(|_| anyhow::anyhow!(concat!("Timeout (30s) on: ", $desc)))?
    };
}

macro_rules! timeout60s {
    ($expr:expr, $desc:expr) => {
        timeout(Duration::from_secs(60), $expr)
            .await
            .map_err(|_| anyhow::anyhow!(concat!("Timeout (60s) on: ", $desc)))?
    };
}

macro_rules! timeout90s {
    ($expr:expr, $desc:expr) => {
        timeout(Duration::from_secs(90), $expr)
            .await
            .map_err(|_| anyhow::anyhow!(concat!("Timeout (90s) on: ", $desc)))?
    };
}

/// Wait for selenium to be ready
pub async fn wait_for_selenium_ready(port: u16, max_wait: Duration) -> Result<()> {
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

/// Setup selenium container and driver
#[allow(dead_code)]
pub async fn setup_selenium_container_and_driver(
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

    // println!("[UI HELPERS] Selenium container started");
    println!("[UI HELPERS] Selenium URL: http://localhost:{selenium_port}");
    // println!(
    //     "[UI HELPERS] Selenium VNC URL: vnc://localhost:{}",
    //     selenium.get_host_port_ipv4(5900).await?
    // );

    // println!("[UI HELPERS] Waiting for Selenium to be ready...");
    timeout90s!(
        wait_for_selenium_ready(selenium_port, Duration::from_secs(90)),
        "Wait for selenium ready"
    )?;
    println!("[UI HELPERS] ✅ Selenium is ready and responding");

    // println!("[UI HELPERS] Setting up WebDriver capabilities...");
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
    // Add SSL/TLS configuration to handle protocol errors
    caps.add_arg("--ignore-ssl-errors")?;
    caps.add_arg("--ignore-certificate-errors")?;
    caps.add_arg("--allow-insecure-localhost")?;
    caps.add_arg("--disable-extensions")?;
    caps.add_arg("--disable-plugins")?;

    // println!("[UI HELPERS] Chrome configured with minimal settings to avoid conflicts");

    // println!(
    //     "[UI HELPERS] Connecting to WebDriver at http://localhost:{}",
    //     selenium_port
    // );
    let driver = timeout(
        Duration::from_secs(20),
        WebDriver::new(&format!("http://localhost:{selenium_port}"), caps),
    )
    .await??;
    println!("[UI HELPERS] ✅ WebDriver connected successfully");
    Ok((selenium, driver, selenium_port))
}

/// Authenticate driver with admin credentials
#[allow(dead_code)]
pub async fn authenticate_driver(driver: &WebDriver, base_url: &str) -> Result<()> {
    let logout_url = format!("{}/logout", base_url.trim_end_matches('/'));
    let login_url = format!("{}/login", base_url.trim_end_matches('/'));

    println!("[AUTH] Starting authentication process...");
    
    // First logout to ensure clean state
    println!("[AUTH] Logging out first...");
    timeout60s!(driver.get(&logout_url), "Navigate to logout page")?;
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Navigate to login page
    println!("[AUTH] Navigating to login page...");
    timeout60s!(driver.get(&login_url), "Navigate to login page")?;
    // If DB is warming up, the login route may render a plain error page.
    // Reload until the login form is present.
    let mut attempts: usize = 0;
    println!("[AUTH] Waiting for login form to be ready...");
    loop {
        // Try to locate the username input; if found, proceed.
        if let Ok(_) = driver.find(By::Css("input[name='id']")).await {
            println!("[AUTH] Login form found, proceeding with authentication...");
            break;
        }
        // Check page source for transient DB error and retry if seen
        let src = driver.source().await.unwrap_or_default();
        if !src.contains("Database connection error") && src.contains("<form") {
            // Form likely present but element not yet queried; small settle delay
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        } else {
            attempts += 1;
            println!("[AUTH] Login form not ready, attempt {}/15. Page contains form: {}, DB error: {}", 
                attempts, src.contains("<form"), src.contains("Database connection error"));
            if attempts >= 15 {
                let snippet: String = src.chars().take(600).collect();
                return Err(anyhow::anyhow!(
                    "Login page not ready after retries. Snippet: {}",
                    snippet
                ));
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
            timeout60s!(driver.get(&login_url), "Reload login page")?;
        }
    }

    // Find and fill username field
    println!("[AUTH] Filling username field...");
    let username_field = timeout60s!(
        driver.find(By::Css("input[name='id']")),
        "Find username field"
    )?;
    timeout60s!(username_field.send_keys("admin"), "Fill username field")?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Find and fill password field
    println!("[AUTH] Filling password field...");
    let password_field = timeout60s!(
        driver.find(By::Css("input[name='password']")),
        "Find password field"
    )?;
    timeout60s!(password_field.send_keys("admin123"), "Fill password field")?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Find and click submit button
    println!("[AUTH] Finding submit button...");
    let submit_button = timeout60s!(
        driver.find(By::XPath(
            "//button[@type='submit' and contains(text(), 'Sign in')]"
        )),
        "Find submit button"
    )?;

    // Check if button is clickable
    let is_enabled = timeout60s!(submit_button.is_enabled(), "Check button enabled")?;
    let is_displayed = timeout60s!(submit_button.is_displayed(), "Check button displayed")?;

    println!("[AUTH] Submit button - enabled: {}, displayed: {}", is_enabled, is_displayed);

    if is_enabled && is_displayed {
        println!("[AUTH] Clicking submit button...");
        timeout60s!(submit_button.click(), "Click submit button")?;
    } else {
        return Err(anyhow::anyhow!(
            "Submit button is not clickable: enabled={}, displayed={}",
            is_enabled,
            is_displayed
        ));
    }

    // Wait for redirect and verify authentication
    println!("[AUTH] Waiting for redirect after login...");
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    
    // Wait for redirect with retries
    let mut redirect_attempts = 0;
    let max_redirect_attempts = 10;
    let mut current_url = None;
    
    while redirect_attempts < max_redirect_attempts {
        match driver.current_url().await {
            Ok(url) => {
                let url_str = url.as_str().to_string();
                current_url = Some(url);
                if !url_str.contains("/login") {
                    println!("[AUTH] Successfully redirected away from login page");
                    break;
                }
            }
            Err(e) => {
                println!("[AUTH] Error getting current URL (attempt {}/{}): {}", 
                    redirect_attempts + 1, max_redirect_attempts, e);
            }
        }
        
        redirect_attempts += 1;
        if redirect_attempts < max_redirect_attempts {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        }
    }
    
    let current_url = current_url.ok_or_else(|| {
        anyhow::anyhow!("Failed to get current URL after {} attempts", max_redirect_attempts)
    })?;

    println!("[AUTH] Current URL after login: {}", current_url);

    if current_url.as_str().contains("/login") {
        return Err(anyhow::anyhow!(
            "Still on login page after authentication attempt"
        ));
    }

    println!("[AUTH] Authentication successful!");
    Ok(())
}

/// Setup app container
#[allow(dead_code)]
pub async fn setup_app_container(
    db_url: &str,
    host_port: Option<u16>,
    config_path: &str,
    container_name: &str,
    extra_env: &[(&str, &str)],
) -> anyhow::Result<(
    ContainerAsync<GenericImage>,
    String, /* bridge IP */
    u16,
)> {
    let app_port = if host_port.is_some() {
        host_port.unwrap()
    } else {
        find_free_port()
    };
    let mut app_image = GenericImage::new("sortingoffice", "latest")
        .with_env_var("DATABASE_URL", db_url)
        .with_env_var("PORT", "3000")
        .with_env_var("HOST", "0.0.0.0") // Bind to all interfaces so other containers can reach it
        .with_env_var("CONFIG_PATH", "/app/config/config.toml")
        .with_mapped_port(app_port, 3000.into())
        .with_container_name(container_name)
        .with_mount(Mount::bind_mount(config_path, "/app/config/config.toml"));
    for (key, value) in extra_env {
        app_image = app_image.with_env_var(*key, *value);
    }

    let app_container = match AsyncRunner::start(app_image).await {
        Ok(c) => {
            println!("[DEBUG] App container started successfully");
            c
        }
        Err(e) => {
            println!("[ERROR] Failed to start app container: {e:?}");
            return Err(e.into());
        }
    };

    let app_id = app_container.id();
    let app_ip = get_container_bridge_ip(app_id).await?;
    let health_url = format!("http://{app_ip}:3000/health");

    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(30);

    loop {
        match client.get(&health_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                println!("[DEBUG] Health check successful: {}", resp.status());
                break;
            }
            Ok(resp) => {
                println!("[DEBUG] Health check failed with status: {}", resp.status());
            }
            Err(e) => {
                println!("[DEBUG] Health check error: {}", e);
            }
        }
        if start.elapsed() > timeout {
            return Err(anyhow::anyhow!("App /health not healthy after 30s"));
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Ok((app_container, app_ip, app_port))
}

/// Get container bridge IP
pub async fn get_container_bridge_ip(container_id: &str) -> anyhow::Result<String> {
    let output = std::process::Command::new("docker")
        .args([
            "inspect",
            "-f",
            "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}",
            container_id,
        ])
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

/// Generate a domain-safe random string (lowercase letters and numbers only)
#[allow(dead_code)]
pub fn rand_domain_str() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rngs::ThreadRng::default();
    (0..8)
        .map(|_| {
            let idx = (rng.next_u32() as usize) % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}

/// Create a new domain
#[allow(dead_code)]
pub async fn create_domain(driver: &WebDriver, app_url: &str, domain_name: &str) -> Result<()> {
    let domain_url = format!("{app_url}/domains");
    timeout60s!(driver.get(&domain_url), "Navigate to domains list page")?;

    let add_domain_button = timeout60s!(
        driver.find(By::Id("add-domain-button")),
        "Find Add Domain button"
    )?;
    timeout30s!(add_domain_button.click(), "Click Add Domain button")?;

    // Wait for HTMX request to complete and form to be loaded
    // println!("[CREATE] Waiting for HTMX form to load...");
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Wait for the form to appear in the main content
    let _form = timeout60s!(
        driver.find(By::Css("form")),
        "Wait for form to be loaded by HTMX"
    )?;
    // println!("[CREATE] Form found and loaded");

    // Wait a bit more for any animations or JavaScript to complete
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Debug: Check what page we're on
    let _current_url = timeout60s!(
        driver.current_url(),
        "Get current URL after clicking add button"
    )?;
    // println!("[CREATE] Current URL after clicking add domain button: {}", current_url);

    // Debug: Get page title and h1 to understand what page we're on
    // let page_title = timeout60s!(driver.title(), "Get page title")?;
    // println!("[CREATE] Page title: {}", page_title);

    // Try to find h1 element
    let h1_element = timeout60s!(driver.find(By::Css("#main-content h1")), "Find h1 element")?;
    let h1_text = timeout60s!(h1_element.text(), "Get h1 text")?;
    // println!("[CREATE] Page h1: {}", h1_text);
    assert!(
        h1_text.contains("Add Domain"),
        "Page title should contain 'Add Domain'"
    );

    // let main_content = timeout60s!(
    //     driver.find(By::Id("main-content")),
    //     "Find main content area"
    // )?;
    // let main_content_text = timeout60s!(main_content.text(), "Get main content text")?;
    // println!("[CREATE] Main content text: {}", main_content_text);

    // Debug: Check if we can find the form elements
    // let domain_input = timeout60s!(
    //     driver.find(By::Css("input[name='domain']")),
    //     "Find domain input"
    // )?;

    // Debug: Get more details about the input element
    // let input_id = timeout60s!(domain_input.get_attribute("id"), "Get domain input id")?;
    // let input_type = timeout60s!(domain_input.get_attribute("type"), "Get domain input type")?;
    // let input_class = timeout60s!(domain_input.get_attribute("class"), "Get domain input class")?;
    // let input_style = timeout60s!(domain_input.get_attribute("style"), "Get domain input style")?;
    // let input_disabled = timeout60s!(domain_input.get_attribute("disabled"), "Get domain input disabled")?;
    // let input_readonly = timeout60s!(domain_input.get_attribute("readonly"), "Get domain input readonly")?;

    // println!("[CREATE] Domain input details:");
    // println!("[CREATE]   ID: {:?}", input_id);
    // println!("[CREATE]   Type: {:?}", input_type);
    // println!("[CREATE]   Class: {:?}", input_class);
    // println!("[CREATE]   Style: {:?}", input_style);
    // println!("[CREATE]   Disabled: {:?}", input_disabled);
    // println!("[CREATE]   Readonly: {:?}", input_readonly);

    // Debug: Check if the element is displayed and enabled
    // let is_displayed = timeout60s!(domain_input.is_displayed(), "Check if domain input is displayed")?;
    // let is_enabled = timeout60s!(domain_input.is_enabled(), "Check if domain input is enabled")?;
    // println!("[CREATE] Domain input - displayed: {}, enabled: {}", is_displayed, is_enabled);

    // Debug: Check if there are any overlays or modals
    // let page_source = timeout60s!(driver.source(), "Get page source for debugging")?;
    // println!("[CREATE] Page source contains 'modal': {}", page_source.contains("modal"));
    // println!("[CREATE] Page source contains 'overlay': {}", page_source.contains("overlay"));
    // println!("[CREATE] Page source contains 'dialog': {}", page_source.contains("dialog"));
    // println!("[CREATE] Page source contains 'form': {}", page_source.contains("form"));
    // println!("[CREATE] Page source contains 'input': {}", page_source.contains("input"));

    // if !is_displayed || !is_enabled {
    //     return Err(anyhow::anyhow!(
    //         "Domain input is not interactable - displayed: {}, enabled: {}",
    //         is_displayed, is_enabled
    //     ));
    // }

    // println!("[CREATE] Attempting to send keys to domain input");

    // Try using ID selector instead of CSS
    let domain_input = timeout60s!(
        driver.find(By::Css("input[name='domain']")),
        "Find domain input by Name"
    )?;

    // // Try clearing the field fi;rst
    // println!("[CREATE] Clearing domain input field");
    // timeout30s!(domain_input_by_id.clear(), "Clear domain input")?;
    // tokio::time::sleep(Duration::from_millis(500)).await

    // Try clicking the input field first to focus it
    // println!("[CREATE] Clicking domain input to focus it");
    // timeout30s!(domain_input.click(), "Click domain input to focus")?;
    // tokio::time::sleep(Duration::from_millis(500)).await;

    // Now try send_keys after focusing
    // println!("[CREATE] Sending keys to focused domain input");
    timeout30s!(domain_input.send_keys(domain_name), "Type domain name")?;

    let transport_input = timeout60s!(
        driver.find(By::Css("input[name='transport']")),
        "Find transport input"
    )?;
    timeout30s!(transport_input.send_keys("virtual"), "Type transport")?;

    let submit_button = timeout60s!(
        driver.find(By::Id("domain-submit-button")),
        "Find submit button"
    )?;
    timeout30s!(submit_button.click(), "Click submit button")?;

    // Wait for form submission and check for validation errors
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Check if we're still on the form page (indicating validation errors)
    let current_url = timeout60s!(
        driver.current_url(),
        "Get current URL after domain creation"
    )?;
    // println!("[CREATE] Current URL after domain creation: {}", current_url);

    let main_content = timeout60s!(
        driver.find(By::Id("main-content")),
        "Find main content area"
    )?;
    let main_content_text = timeout60s!(main_content.text(), "Get main content text")?;
    // println!("[CREATE] 1. Main content text: {}", main_content_text);

    // Check for validation error messages on the page
    // let page_source = timeout60s!(driver.source(), "Get page source")?;

    // Look for common validation error indicators
    let validation_indicators = [
        "validation-error",
        // "error",
        // "invalid",
        // "required",
        "Domain cannot contain uppercase letters",
        "Domain can only contain lowercase letters",
        "Domain cannot be empty",
        "Domain cannot start with dot or hyphen",
        "Domain cannot end with dot or hyphen",
        "Domain cannot have consecutive dots or hyphens",
    ];

    let mut found_errors = Vec::new();
    for indicator in &validation_indicators {
        if main_content_text.contains(indicator) {
            found_errors.push(*indicator);
        }
    }

    if !found_errors.is_empty() {
        println!("[CREATE] Validation errors detected: {found_errors:?}");
        println!(
            "[CREATE] Page source contains validation errors - domain creation may have failed"
        );
        return Err(anyhow::anyhow!(
            "Domain creation failed due to validation errors: {:?}",
            found_errors
        ));
    }

    // Check if we're still on the form page (should have redirected on success)
    if current_url.as_str().contains("/domains/new")
        || current_url.as_str().contains("/domains/create")
    {
        println!("[CREATE] Still on form page after submission - domain creation may have failed");
        return Err(anyhow::anyhow!(
            "Domain creation failed - still on form page after submission"
        ));
    }

    // Navigate back to domains list to verify creation
    timeout60s!(driver.get(&domain_url), "Navigate back to domains list")?;

    // Check if the domain appears in the list using pagination
    let domain_found = check_item_in_paginated_list(
        driver,
        app_url,
        "/domains",
        domain_name,
        10, // max pages to check
    )
    .await?;

    if domain_found {
        // println!("[CREATE] Domain {} successfully created and visible in list", domain_name);
        Ok(())
    } else {
        println!(
            "[CREATE] Domain {domain_name} not found in paginated list - creation may have failed"
        );
        Err(anyhow::anyhow!(
            "Domain {} not found in paginated list - creation may have failed",
            domain_name
        ))
    }
}

/// Create an alias
#[allow(dead_code)]
pub async fn create_alias(
    driver: &WebDriver,
    app_url: &str,
    alias_email: &str,
    destination: &str,
) -> Result<()> {
    let aliases_url = format!("{app_url}/aliases");
    timeout60s!(driver.get(&aliases_url), "Navigate to aliases list page")?;

    let add_alias_button = timeout60s!(
        driver.find(By::Id("add-alias-button")),
        "Find Add Alias button"
    )?;
    timeout30s!(add_alias_button.click(), "Click Add Alias button")?;

    // Wait for HTMX request to complete and form to be loaded
    // println!("[CREATE] Waiting for HTMX form to load...");
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Wait for the form to appear in the main content
    let _form = timeout60s!(
        driver.find(By::Css("form")),
        "Wait for form to be loaded by HTMX"
    )?;
    // println!("[CREATE] Form found and loaded");

    // Wait a bit more for any animations or JavaScript to complete
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let _current_url = timeout60s!(
        driver.current_url(),
        "Get current URL before alias creation"
    )?;
    // println!("[CREATE] 0. Current URL before alias creation: {}", current_url);

    let alias_input = timeout30s!(
        driver.find(By::Css("input[name='mail']")),
        "Find alias input"
    )?;
    timeout30s!(alias_input.send_keys(alias_email), "Type alias email")?;

    let destination_input = timeout60s!(
        driver.find(By::Css("input[name='destination']")),
        "Find destination input"
    )?;
    timeout30s!(destination_input.send_keys(destination), "Type destination")?;

    let submit_button = timeout30s!(
        driver.find(By::Id("alias-submit-button")),
        "Find submit button"
    )?;
    timeout30s!(submit_button.click(), "Click submit button")?;

    // Wait for form submission and check for validation errors
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Check if we're still on the form page (indicating validation errors)
    let current_url = timeout60s!(driver.current_url(), "Get current URL after alias creation")?;
    // println!("[CREATE] 1. Current URL after alias creation: {}", current_url);

    // Check for validation error messages on the page
    let page_source = timeout60s!(driver.source(), "Get page source")?;

    // Look for common validation error indicators
    let validation_indicators = [
        "validation-error",
        // "error",
        // "invalid",
        // "required",
        "Domain cannot contain uppercase letters",
        "Domain can only contain lowercase letters",
        "Local part contains invalid characters",
        "Alias mail must contain @",
        "Invalid domain",
        "Domain cannot be empty",
    ];

    let mut found_errors = Vec::new();
    for indicator in &validation_indicators {
        if page_source.contains(indicator) {
            found_errors.push(*indicator);
        }
    }

    if !found_errors.is_empty() {
        println!("[CREATE] Validation errors detected: {found_errors:?}");
        println!(
            "[CREATE] Page source contains validation errors - alias creation may have failed"
        );
        return Err(anyhow::anyhow!(
            "Alias creation failed due to validation errors: {:?}",
            found_errors
        ));
    }

    // Check if we're still on the form page (should have redirected on success)
    if current_url.as_str().contains("/aliases/new")
        || current_url.as_str().contains("/aliases/create")
    {
        println!("[CREATE] Still on form page after submission - alias creation may have failed");
        return Err(anyhow::anyhow!(
            "Alias creation failed - still on form page after submission"
        ));
    }

    // Navigate back to aliases list to verify creation
    timeout60s!(driver.get(&aliases_url), "Navigate back to aliases list")?;

    // Check if the alias appears in the list using pagination
    let alias_found = check_item_in_paginated_list(
        driver,
        app_url,
        "/aliases",
        alias_email,
        10, // max pages to check
    )
    .await?;

    if alias_found {
        // println!("[CREATE] Alias {} successfully created and visible in aliases list", alias_email);
        Ok(())
    } else {
        println!(
            "[CREATE] Alias {alias_email} not found in paginated list - checking domain show page"
        );

        // Try to find the alias in the domain show page instead
        let domain_name = alias_email.split('@').nth(1).unwrap_or("");
        let domain_show_url = format!("{app_url}/domains");
        timeout60s!(driver.get(&domain_show_url), "Navigate to domains list")?;

        // Find and click the domain view link
        let domain_view_link = timeout60s!(
            driver.find(By::XPath(format!(
                "//tr[contains(., '{domain_name}')]//a[contains(@href, '/domains/')]"
            ))),
            "Find domain view link"
        )?;
        timeout30s!(domain_view_link.click(), "Click domain view link")?;

        // Check if alias is visible in domain show page
        let domain_main_content = timeout60s!(
            driver.find(By::Id("main-content")),
            "Find main content area in domain show page"
        )?;
        let domain_main_content_text =
            timeout60s!(domain_main_content.text(), "Get domain main content text")?;

        // println!("[CREATE] 2. Current URL after alias creation: {}", current_url);
        // println!("[CREATE] Main content: {}", domain_main_content_text);

        if domain_main_content_text.contains(alias_email) {
            // println!("[CREATE] Alias {} successfully created and visible in domain show page", alias_email);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Alias {} not found in domain show page either - creation may have failed",
                alias_email
            ))
        }
    }
}

/// Create a user
#[allow(dead_code)]
pub async fn create_user(
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

    // Wait for HTMX request to complete and form to be loaded
    // println!("[CREATE] Waiting for HTMX form to load...");
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Wait for the form to appear in the main content
    let _form = timeout60s!(
        driver.find(By::Css("form")),
        "Wait for form to be loaded by HTMX"
    )?;
    // println!("[CREATE] Form found and loaded");

    // Wait a bit more for any animations or JavaScript to complete
    tokio::time::sleep(Duration::from_millis(1000)).await;

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

    // Add password field - required by validation
    let user_password_input = timeout60s!(
        driver.find(By::Css("input[name='password']")),
        "Find user password input"
    )?;
    timeout60s!(
        user_password_input.send_keys("testpassword123"),
        "Type user password"
    )?;

    // Add home field - required by validation
    let user_home_input = timeout60s!(
        driver.find(By::Css("input[name='home']")),
        "Find user home input"
    )?;
    timeout60s!(
        user_home_input.send_keys("/var/spool/mail/virtual"),
        "Type user home directory"
    )?;

    let _current_url = timeout60s!(driver.current_url(), "Get current URL after user create")?;
    // println!("[CREATE] 1. Current URL after user creation: {}", current_url);

    let user_submit_btn = timeout60s!(
        driver.find(By::Id("user-submit-button")),
        "Find submit button for user"
    )?;
    timeout60s!(user_submit_btn.click(), "Submit user form")?;
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // After form submission, we should be redirected to the users list page
    // Verify the user appears in the list
    let _current_url = timeout60s!(driver.current_url(), "Get current URL after user create")?;
    // println!("[CREATE] 2. Current URL after user creation: {}", current_url);

    // If we're not on the users list page, navigate there
    if !_current_url.as_str().ends_with("/users") {
        timeout60s!(driver.get(&users_url), "Navigate back to users list page")?;
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    let _current_url = timeout60s!(driver.current_url(), "Get current URL after user create")?;
    // println!("[CREATE] 3. Current URL after user creation: {}", current_url);

    // Verify the user appears in the list using pagination
    let user_found = check_item_in_paginated_list(
        driver, app_url, "/users", user_email, 10, // max pages to check
    )
    .await?;

    if user_found {
        // println!(
        //     "[CREATE] User {} successfully created and visible in list",
        //     user_email
        // );
        Ok(())
    } else {
        println!(
            "[CREATE] User {user_email} not found in paginated list - creation may have failed"
        );
        Err(anyhow::anyhow!(
            "User {} not found in paginated list - creation may have failed",
            user_email
        ))
    }
}

/// Check reports page
#[allow(dead_code)]
pub async fn check_reports_page(driver: &WebDriver, app_url: &str) -> Result<()> {
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
#[allow(dead_code)]
pub async fn delete_user(driver: &WebDriver, app_url: &str, user_email: &str) -> Result<()> {
    // First navigate to users list page
    let users_url = format!("{app_url}/users");
    timeout60s!(driver.get(&users_url), "Navigate to users list page")?;

    // Check if we were redirected to login page (authentication expired)
    let current_url = timeout60s!(driver.current_url(), "Get current URL")?;
    if current_url.as_str().contains("/login") {
        println!("[CLEANUP] Authentication expired, re-authenticating...");
        authenticate_driver(driver, app_url).await?;
        // Navigate back to users page after re-authentication
        timeout60s!(
            driver.get(&users_url),
            "Navigate to users list page after re-auth"
        )?;
    }

    // Check if the user exists in the list using pagination
    let user_found = check_item_in_paginated_list(
        driver, app_url, "/users", user_email, 10, // max pages to check
    )
    .await?;

    if !user_found {
        return Err(anyhow::anyhow!(
            "User {} not found in paginated users list - cannot delete user",
            user_email
        ));
    }

    // Now navigate to the specific page where the user was found and click its "View" link
    // We need to find the user again on the current page to get its view link
    let view_link = timeout60s!(
        driver.find(By::XPath(format!(
            "//tr[contains(., '{user_email}')]//a[contains(@href, '/users/')]"
        ))),
        "Find user view link"
    )?;
    timeout30s!(view_link.click(), "Click user view link")?;

    // Now on the user show page, find and click delete button by ID
    let delete_button = timeout60s!(
        driver.find(By::Id("delete-user-button")),
        "Find delete button"
    )?;
    timeout30s!(delete_button.click(), "Click delete button")?;

    // Handle the JavaScript alert dialog that appears
    tokio::time::sleep(Duration::from_millis(500)).await;
    // Accept the alert to confirm deletion
    driver.accept_alert().await?;

    // Switch back to the main content
    driver.enter_default_frame().await?;

    // println!("[CLEANUP] Successfully deleted user: {}", user_email);
    Ok(())
}

/// Delete an alias
#[allow(dead_code)]
pub async fn delete_alias(driver: &WebDriver, app_url: &str, alias_email: &str) -> Result<()> {
    // First navigate to domains list page
    let domains_url = format!("{app_url}/domains");
    timeout60s!(driver.get(&domains_url), "Navigate to domains list page")?;

    // Check if we were redirected to login page (authentication expired)
    let current_url = timeout60s!(driver.current_url(), "Get current URL")?;
    if current_url.as_str().contains("/login") {
        println!("[CLEANUP] Authentication expired, re-authenticating...");
        authenticate_driver(driver, app_url).await?;
        // Navigate back to domains page after re-authentication
        timeout60s!(
            driver.get(&domains_url),
            "Navigate to domains list page after re-auth"
        )?;
    }

    // Extract domain name from alias email
    let domain_name = alias_email.split('@').nth(1).unwrap_or("");
    // println!("[CLEANUP] Looking for domain '{}' to find alias '{}'", domain_name, alias_email);

    // Use pagination function to find the domain in the domains list
    let domain_found = check_item_in_paginated_list(
        driver,
        app_url,
        "/domains",
        domain_name,
        10, // max pages to check
    )
    .await?;

    if !domain_found {
        return Err(anyhow::anyhow!(
            "Domain {} not found in paginated domains list - cannot delete alias {}",
            domain_name,
            alias_email
        ));
    }

    // Now navigate to the specific page where the domain was found and click its "View" link
    // We need to find the domain again on the current page to get its view link
    let domain_view_link = timeout60s!(
        driver.find(By::XPath(format!(
            "//tr[contains(., '{domain_name}')]//a[contains(@href, '/domains/')]"
        ))),
        "Find domain view link"
    )?;
    timeout30s!(domain_view_link.click(), "Click domain view link")?;

    // Now we should be on the domain show page, check if the alias exists
    let main_content = timeout60s!(
        driver.find(By::Id("main-content")),
        "Find main content area"
    )?;
    let main_content_text = timeout60s!(main_content.text(), "Get main content text")?;

    if !main_content_text.contains(alias_email) {
        return Err(anyhow::anyhow!(
            "Alias {} not found in domain show page - this indicates a problem with the creation process",
            alias_email
        ));
    }

    // Find the specific alias row and click its "View" link in the domain show page
    let view_link = timeout60s!(
        driver.find(By::XPath(format!(
            "//tr[contains(., '{alias_email}')]//a[contains(@href, '/aliases/')]"
        ))),
        "Find alias view link in domain show page"
    )?;
    timeout30s!(view_link.click(), "Click alias view link")?;

    // Now on the alias show page, find and click delete button by ID
    let delete_button = timeout60s!(
        driver.find(By::Id("delete-alias-button")),
        "Find delete button"
    )?;
    timeout30s!(delete_button.click(), "Click delete button")?;

    // Handle the JavaScript alert dialog that appears
    tokio::time::sleep(Duration::from_millis(500)).await;
    // Accept the alert to confirm deletion
    driver.accept_alert().await?;

    // Switch back to the main content
    driver.enter_default_frame().await?;

    // println!("[CLEANUP] Successfully deleted alias: {}", alias_email);
    Ok(())
}

/// Delete a domain
#[allow(dead_code)]
pub async fn delete_domain(driver: &WebDriver, app_url: &str, domain_name: &str) -> Result<()> {
    // First navigate to domains list page
    let domains_url = format!("{app_url}/domains");
    timeout60s!(driver.get(&domains_url), "Navigate to domains list page")?;

    // Check if we were redirected to login page (authentication expired)
    let current_url = timeout60s!(driver.current_url(), "Get current URL")?;
    if current_url.as_str().contains("/login") {
        println!("[CLEANUP] Authentication expired, re-authenticating...");
        authenticate_driver(driver, app_url).await?;
        // Navigate back to domains page after re-authentication
        timeout60s!(
            driver.get(&domains_url),
            "Navigate to domains list page after re-auth"
        )?;
    }

    // Check if the domain exists in the list using pagination
    let domain_found = check_item_in_paginated_list(
        driver,
        app_url,
        "/domains",
        domain_name,
        10, // max pages to check
    )
    .await?;

    if !domain_found {
        return Err(anyhow::anyhow!(
            "Domain {} not found in paginated domains list - cannot delete domain",
            domain_name
        ));
    }

    // Now navigate to the specific page where the domain was found and click its "View" link
    // We need to find the domain again on the current page to get its view link
    let view_link = timeout60s!(
        driver.find(By::XPath(format!(
            "//tr[contains(., '{domain_name}')]//a[contains(@href, '/domains/')]"
        ))),
        "Find domain view link"
    )?;
    timeout30s!(view_link.click(), "Click domain view link")?;

    // Now on the domain show page, find and click delete button by ID
    let delete_button = timeout60s!(
        driver.find(By::Id("delete-domain-button")),
        "Find delete button"
    )?;
    timeout30s!(delete_button.click(), "Click delete button")?;

    // Handle the JavaScript alert dialog that appears
    tokio::time::sleep(Duration::from_millis(500)).await;
    // Accept the alert to confirm deletion
    driver.accept_alert().await?;

    // Switch back to the main content
    driver.enter_default_frame().await?;

    // println!("[CLEANUP] Successfully deleted domain: {}", domain_name);
    Ok(())
}

/// Cleanup test resources
#[allow(dead_code)]
pub async fn cleanup_test_resources(
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
        // println!("[CLEANUP] Attempting to delete user: {}", user_email);
        delete_user(driver, app_url, user_email).await?;
        println!("[CLEANUP] Successfully deleted user: {user_email}");

        // Delete aliases
        // println!("[CLEANUP] Attempting to delete alias1: {}", alias1domain);
        delete_alias(driver, app_url, alias1domain).await?;
        println!("[CLEANUP] Successfully deleted alias1: {alias1domain}");

        // println!("[CLEANUP] Attempting to delete alias2: {}", alias2domain);
        delete_alias(driver, app_url, alias2domain).await?;
        println!("[CLEANUP] Successfully deleted alias2: {alias2domain}");

        // Delete domain last
        // println!("[CLEANUP] Attempting to delete domain: {}", domain_name);
        delete_domain(driver, app_url, domain_name).await?;
        println!("[CLEANUP] Successfully deleted domain: {domain_name}");

        println!("[SMOKE TEST] Cleanup completed successfully");
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

/// Check if an item exists in a paginated list by iterating through pages
#[allow(dead_code)]
pub async fn check_item_in_paginated_list(
    driver: &WebDriver,
    app_url: &str,
    list_path: &str,
    item_name: &str,
    max_pages: usize,
) -> Result<bool> {
    // println!("[CHECK] Searching for '{}' in paginated list at {}", item_name, list_path);

    for page_num in 1..=max_pages {
        let page_url = if page_num == 1 {
            format!("{app_url}{list_path}")
        } else {
            format!("{app_url}{list_path}?page={page_num}&per_page=25")
        };

        // println!("[CHECK] Checking page {} at URL: {}", page_num, page_url);
        timeout60s!(driver.get(&page_url), "Navigate to paginated list page")?;

        // Wait a moment for the page to load
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Get the main content
        let main_content = timeout60s!(
            driver.find(By::Id("main-content")),
            "Find main content area"
        )?;
        let main_content_text = timeout60s!(main_content.text(), "Get main content text")?;

        // Debug: Show a snippet of the content
        let _content_preview = if main_content_text.len() > 200 {
            format!("{}...", &main_content_text[..200])
        } else {
            main_content_text.clone()
        };
        // println!("[CHECK] Page {} content preview: {}", page_num, content_preview);

        // Check if the item is on this page
        if main_content_text.contains(item_name) {
            println!("[CHECK] Found '{item_name}' on page {page_num}");
            return Ok(true);
        }

        // Check if there are more pages (look for "Next" link)
        let has_next = main_content_text.contains("Next")
            || main_content_text.contains("next")
            || main_content_text.contains(">");

        if !has_next {
            println!("[CHECK] No more pages to search");
            break;
        }
    }

    println!("[CHECK] Item '{item_name}' not found after checking {max_pages} pages");
    Ok(false)
}

/// Create a custom Docker network for test containers
pub async fn create_test_network(network_name: &str) -> anyhow::Result<()> {
    println!("[NETWORK] Creating custom Docker network: {}", network_name);

    // Check if network already exists
    let check_output = Command::new("docker")
        .args(["network", "ls", "--format", "{{.Name}}"])
        .output()?;

    let networks = String::from_utf8_lossy(&check_output.stdout);
    if networks.lines().any(|line| line == network_name) {
        println!("[NETWORK] Network {} already exists", network_name);
        return Ok(());
    }

    // Create the network with a specific subnet to allow static IP assignment
    let output = Command::new("docker")
        .args([
            "network",
            "create",
            "--subnet",
            "10.20.0.0/16",
            network_name,
        ])
        .output()?;

    if output.status.success() {
        println!(
            "[NETWORK] ✅ Successfully created network: {}",
            network_name
        );
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!(
            "Failed to create network {}: {}",
            network_name,
            error
        ))
    }
}

/// Clean up a custom Docker network
pub async fn cleanup_test_network(network_name: &str) -> anyhow::Result<()> {
    println!("[NETWORK] Cleaning up network: {}", network_name);

    let output = Command::new("docker")
        .args(["network", "rm", network_name])
        .output()?;

    if output.status.success() {
        println!(
            "[NETWORK] ✅ Successfully removed network: {}",
            network_name
        );
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        println!(
            "[NETWORK] Warning: Failed to remove network {}: {}",
            network_name, error
        );
    }

    Ok(())
}

/// Setup MySQL container on a custom network
pub async fn setup_test_db_with_network(network_name: &str) -> anyhow::Result<TestContainer> {
    println!(
        "[DB] Setting up MySQL container on network: {}",
        network_name
    );

    // Create the network first
    create_test_network(network_name).await?;

    // Start MySQL container on the custom network with a static IP
    let mysql_image = Mysql::default();
    let mysql_container = AsyncRunner::start(mysql_image).await?;

    // Get container ID for later use
    let container_id = mysql_container.id();
    println!("[DB] MySQL container ID: {}", container_id);

    // Connect to the custom network with a static IP
    let output = Command::new("docker")
        .args([
            "network",
            "connect",
            "--ip",
            "10.20.0.10",
            network_name,
            container_id,
        ])
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to connect MySQL container to network: {}",
            error
        ));
    }

    println!(
        "[DB] ✅ MySQL container connected to network: {} with static IP",
        network_name
    );

    // Use the static IP we assigned
    let bridge_ip = "10.20.0.10";
    println!("[DB] MySQL container IP on network: {}", bridge_ip);

    // Create a unique schema for this test
    let schema = format!("test_{}", rand::random::<u64>());

    // Create the schema
    let test_url = format!("mysql://root@{}:3306/{}", bridge_ip, schema);

    // Wait for MySQL to be ready
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Create the schema
    let output = Command::new("docker")
        .args([
            "exec",
            container_id,
            "mysql",
            "-uroot",
            "-e",
            &format!("CREATE DATABASE IF NOT EXISTS `{}`", schema),
        ])
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to create database schema: {}",
            error
        ));
    }

    println!("[DB] ✅ Created database schema: {}", schema);

    // Create connection pool for the test schema
    let manager = diesel::r2d2::ConnectionManager::<diesel::mysql::MysqlConnection>::new(&test_url);
    let pool = diesel::r2d2::Pool::builder()
        .max_size(5)
        .min_idle(Some(1))
        .build(manager)
        .expect("Failed to create connection pool");

    // Run migrations on the new schema
    let mut conn = pool.get().expect("Failed to get connection");
    if let Err(e) = MigrationHarness::run_pending_migrations(&mut conn, MIGRATIONS) {
        return Err(anyhow::anyhow!(e.to_string()));
    }
    println!("[DB] ✅ Successfully ran database migrations");

    // Seed the database immediately after creation, before the container goes out of scope
    println!("[DB] Seeding database immediately after creation");

    // Copy the seed data file into the container
    let copy_status = std::process::Command::new("docker")
        .args([
            "cp",
            "seed_data/all.sql",
            &format!("{}:/tmp/seed_data.sql", container_id),
        ])
        .status();

    match copy_status {
        Ok(exit_status) => {
            if !exit_status.success() {
                println!("[DB] Warning: Failed to copy seed data file to container");
            } else {
                println!("[DB] Successfully copied seed data file to container");

                // Now execute the seed data file
                let seed_status = std::process::Command::new("docker")
                    .args([
                        "exec",
                        container_id,
                        "mysql",
                        "-uroot",
                        &schema,
                        "-e",
                        "source /tmp/seed_data.sql",
                    ])
                    .status();

                match seed_status {
                    Ok(exit_status) => {
                        if !exit_status.success() {
                            println!(
                                "[DB] Warning: Seeding DB failed with status: {exit_status:?}"
                            );
                        } else {
                            println!("[DB] Successfully seeded database: {}", schema);
                        }
                    }
                    Err(e) => {
                        println!("[DB] Error running seeding command: {}", e);
                        println!("[DB] Skipping seeding due to connection issues");
                    }
                }
            }
        }
        Err(e) => {
            println!("[DB] Error copying seed data file: {}", e);
            println!("[DB] Skipping seeding due to file copy issues");
        }
    }

    // Create connection pool for the test schema
    let manager = diesel::r2d2::ConnectionManager::<diesel::mysql::MysqlConnection>::new(&test_url);
    let pool = diesel::r2d2::Pool::builder()
        .max_size(5)
        .min_idle(Some(1))
        .build(manager)
        .expect("Failed to create connection pool");

    println!("[DB] Storing container ID: {} for later use", container_id);
    Ok(TestContainer {
        pool,
        schema,
        port: 3306,
        bridge_ip: bridge_ip.to_string(),
        container_id: container_id.to_string(),
    })
}

/// Add shared-network helpers and per-schema helpers

pub async fn ensure_shared_network() -> anyhow::Result<String> {
    let network_name = "sortingoffice-e2e";
    // Create network if missing (user-defined bridge gives DNS between containers)
    let check = Command::new("docker")
        .args(["network", "ls", "--format", "{{.Name}}"])
        .output()?;
    let exists = String::from_utf8_lossy(&check.stdout)
        .lines()
        .any(|n| n == network_name);
    if !exists {
        let out = Command::new("docker")
            .args(["network", "create", network_name])
            .output()?;
        if !out.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create shared network: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        println!("[NETWORK] ✅ Created shared network: {}", network_name);
    }
    Ok(network_name.to_string())
}

pub async fn connect_container_to_network(
    container_id: &str,
    network: &str,
    alias: &str,
) -> anyhow::Result<()> {
    // If already connected, docker returns error; ignore if already connected
    let out = Command::new("docker")
        .args([
            "network",
            "connect",
            "--alias",
            alias,
            network,
            container_id,
        ])
        .output()?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        if !err.contains("already exists") {
            return Err(anyhow::anyhow!(
                "Failed to connect {} to {}: {}",
                container_id,
                network,
                err
            ));
        }
    }
    Ok(())
}

pub async fn get_container_ip_on_network(
    container_id: &str,
    network: &str,
) -> anyhow::Result<String> {
    let output = Command::new("docker")
        .args([
            "inspect",
            "-f",
            &format!(
                "{{{{ (index .NetworkSettings.Networks \"{}\").IPAddress }}}}",
                network
            ),
            container_id,
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run docker inspect: {}", e))?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "docker inspect failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if ip.is_empty() {
        return Err(anyhow::anyhow!("No IP for network {}", network));
    }
    Ok(ip)
}

// Per-schema helpers on the shared MySQL container
pub async fn create_schema(schema: &str) -> anyhow::Result<()> {
    let port = sortingoffice::test_helpers::testcontainers_setup::get_shared_mysql_port().await;
    let admin_url = format!("mysql://root@127.0.0.1:{}/mysql", port);
    let manager =
        diesel::r2d2::ConnectionManager::<diesel::mysql::MysqlConnection>::new(&admin_url);
    let pool = diesel::r2d2::Pool::builder()
        .max_size(3)
        .min_idle(Some(1))
        .build(manager)?;
    let mut conn = pool.get()?;
    diesel::sql_query(format!("CREATE DATABASE IF NOT EXISTS `{}`", schema)).execute(&mut conn)?;
    Ok(())
}

pub async fn run_migrations_for_schema(schema: &str) -> anyhow::Result<()> {
    let port = sortingoffice::test_helpers::testcontainers_setup::get_shared_mysql_port().await;
    let url = format!("mysql://root@127.0.0.1:{}/{}", port, schema);
    let manager = diesel::r2d2::ConnectionManager::<diesel::mysql::MysqlConnection>::new(&url);
    let pool = diesel::r2d2::Pool::builder()
        .max_size(5)
        .min_idle(Some(1))
        .build(manager)?;
    let mut conn = pool.get()?;
    match MigrationHarness::run_pending_migrations(&mut conn, MIGRATIONS) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!(e.to_string())),
    }
}

pub async fn seed_schema(schema: &str) -> anyhow::Result<()> {
    use sortingoffice::test_helpers::testcontainers_setup::get_shared_mysql_container_id;
    let container_id = get_shared_mysql_container_id().await;
    let copy = Command::new("docker")
        .args([
            "cp",
            "seed_data/all.sql",
            &format!("{}:/tmp/seed.sql", container_id),
        ])
        .status()?;
    if !copy.success() {
        return Err(anyhow::anyhow!("Failed to copy seed file"));
    }
    let exec = Command::new("docker")
        .args([
            "exec",
            &container_id,
            "mysql",
            "-uroot",
            schema,
            "-e",
            "source /tmp/seed.sql",
        ])
        .status()?;
    if !exec.success() {
        return Err(anyhow::anyhow!("Failed to seed schema"));
    }
    Ok(())
}

// Simplified app container: attach to shared network with name 'app', connect to DB by service name 'db'
pub async fn setup_app_on_shared_network(
    schema: &str,
    host_port: Option<u16>,
    config_path: &str,
    extra_env: &[(&str, &str)],
) -> anyhow::Result<(ContainerAsync<GenericImage>, u16)> {
    let network = ensure_shared_network().await?;

    // Ensure shared DB exists and is connected to network as alias 'db'
    let db_container_id =
        sortingoffice::test_helpers::testcontainers_setup::get_shared_mysql_container_id().await;
    connect_container_to_network(&db_container_id, &network, "db").await?;

    let db_url = format!("mysql://root@db:3306/{}", schema);
    let app_port = host_port.unwrap_or(find_free_port());

    let mut app_image = GenericImage::new("sortingoffice", "latest")
        .with_env_var("DATABASE_URL", &db_url)
        .with_env_var("PORT", "3000")
        .with_env_var("HOST", "0.0.0.0")
        .with_env_var("BASE_URL", "http://app:3000")
        .with_env_var("DEFAULT_LOCALE", "en")
        .with_env_var("SESSION_SECRET", "testsessionsecret")
        .with_env_var("COOKIE_SECRET", "testcookiesecret")
        .with_env_var("CSRF_DISABLED", "true")
        .with_env_var("CONFIG_PATH", "/app/config/config.toml")
        .with_mapped_port(app_port, 3000.into())
        .with_network(&network)
        .with_mount(Mount::bind_mount(config_path, "/app/config/config.toml"));

    for (k, v) in extra_env {
        app_image = app_image.with_env_var(*k, *v);
    }

    let app = AsyncRunner::start(app_image).await?;

    // Debug: show network containers and aliases
    // Removed noisy network logging to reduce test output noise
    // if let Ok(out) = Command::new("docker")
    //     .args([
    //         "network",
    //         "inspect",
    //         &network,
    //         "--format",
    //         "{{json .Containers}}",
    //     ])
    //     .output()
    // {
    //     println!(
    //         "[NETWORK] containers on {}: {}",
    //         &network,
    //         String::from_utf8_lossy(&out.stdout)
    //     );
    // }

    // Health check via mapped host port (simpler and avoids intra-container IP)
    let health_url = format!("http://127.0.0.1:{}/health", app_port);
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(60);
    loop {
        if let Ok(resp) = client.get(&health_url).send().await {
            if resp.status().is_success() {
                break;
            }
        }
        if start.elapsed() > timeout {
            return Err(anyhow::anyhow!("App /health not healthy after 60s"));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Determine app container IP on the shared bridge network
    let _app_ip = get_container_ip_on_network(&app.id(), &network).await?;
    // Removed noisy debug logging
    // println!("[DEBUG] App container IP: {}", app_ip);

    Ok((app, app_port))
}

pub async fn setup_selenium_on_shared_network(
) -> anyhow::Result<(ContainerAsync<GenericImage>, WebDriver, u16)> {
    let network = ensure_shared_network().await?;

    let selenium_image = GenericImage::new("selenium/standalone-chrome", "latest")
        .with_env_var("SE_NODE_MAX_SESSIONS", "1")
        .with_env_var("SE_NODE_OVERRIDE_MAX_SESSIONS", "true")
        .with_mount(Mount::bind_mount("/dev/shm", "/dev/shm"))
        .with_network(&network);

    let selenium = AsyncRunner::start(selenium_image).await?;
    let selenium_port = selenium.get_host_port_ipv4(4444).await?;
    println!("[UI HELPERS] Selenium URL: http://localhost:{selenium_port}");

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

    let driver = timeout(
        Duration::from_secs(20),
        WebDriver::new(&format!("http://localhost:{selenium_port}"), caps),
    )
    .await??;
    Ok((selenium, driver, selenium_port))
}

pub async fn setup_selenium_on_shared_network_with_args(
    extra_chrome_args: &[String],
) -> anyhow::Result<(ContainerAsync<GenericImage>, WebDriver, u16)> {
    let network = ensure_shared_network().await?;

    let selenium_image = GenericImage::new("selenium/standalone-chrome", "latest")
        .with_env_var("SE_NODE_MAX_SESSIONS", "1")
        .with_env_var("SE_NODE_OVERRIDE_MAX_SESSIONS", "true")
        .with_mount(Mount::bind_mount("/dev/shm", "/dev/shm"))
        .with_network(&network);

    let selenium = AsyncRunner::start(selenium_image).await?;
    let selenium_port = selenium.get_host_port_ipv4(4444).await?;
    println!("[UI HELPERS] Selenium URL: http://localhost:{selenium_port}");

    timeout90s!(
        wait_for_selenium_ready(selenium_port, Duration::from_secs(90)),
        "Wait for selenium ready"
    )?;

    let mut caps = DesiredCapabilities::chrome();
    // Base args
    caps.add_arg("--headless=new")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;
    // Relax network/security for CI
    caps.add_arg("--allow-insecure-localhost")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--disable-http2")?;
    caps.add_arg("--disable-quic")?;
    caps.add_arg("--test-type")?;
    for a in extra_chrome_args {
        caps.add_arg(a)?;
    }

    let driver = timeout(
        Duration::from_secs(20),
        WebDriver::new(&format!("http://localhost:{selenium_port}"), caps),
    )
    .await??;
    Ok((selenium, driver, selenium_port))
}

pub async fn setup_selenium_host() -> anyhow::Result<(ContainerAsync<GenericImage>, WebDriver)> {
    let selenium_image = GenericImage::new("selenium/standalone-chrome", "latest")
        .with_env_var("SE_NODE_MAX_SESSIONS", "1")
        .with_env_var("SE_NODE_OVERRIDE_MAX_SESSIONS", "true")
        .with_mount(Mount::bind_mount("/dev/shm", "/dev/shm"))
        .with_network("host");

    let selenium = AsyncRunner::start(selenium_image).await?;

    // With host network, WebDriver listens on localhost:4444
    let wd_url = "http://localhost:4444";
    timeout90s!(
        wait_for_selenium_ready(4444, Duration::from_secs(90)),
        "Wait for selenium ready (host)"
    )?;

    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless=new")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;

    let driver = timeout(Duration::from_secs(20), WebDriver::new(wd_url, caps)).await??;
    Ok((selenium, driver))
}

/// Wait until a page is ready by ensuring either the layout is present
/// (identified by #main-content) or that an allowed plain-message appears.
/// Returns true if layout found, false if allowed plain-message detected.
pub async fn ensure_page_ready(
    driver: &WebDriver,
    page_url: &str,
    max_attempts: usize,
    allow_plain_message_contains: Option<&str>,
) -> anyhow::Result<bool> {
    use thirtyfour::By;
    let mut attempt: usize = 0;
    loop {
        timeout(Duration::from_secs(30), driver.get(page_url)).await??;
        let src = driver.source().await.unwrap_or_default();

        // Success cases
        if let Ok(_) = driver.find(By::Id("main-content")).await {
            return Ok(true);
        }
        if let Some(needle) = allow_plain_message_contains {
            if src.contains(needle) {
                return Ok(false);
            }
        }

        // Backoff and retry
        let sleep_ms = std::cmp::min(250 * (1 << attempt), 3000);
        tokio::time::sleep(Duration::from_millis(sleep_ms as u64)).await;
        attempt += 1;
        if attempt >= max_attempts {
            let snippet: String = src.chars().take(600).collect();
            return Err(anyhow::anyhow!(
                "Page not ready after {} attempts at {}. Snippet: {}",
                max_attempts,
                page_url,
                snippet
            ));
        }
    }
}
