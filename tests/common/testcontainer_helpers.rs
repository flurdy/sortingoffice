//! Testcontainer helper functions for setting up and managing test containers
//!
//! This module provides a unified API for setting up common test containers like:
//! - Selenium containers with Chrome for UI testing
//! - Application containers for the sortingoffice app
//! - Consistent cleanup utilities
//!
//! # Examples
//!
//! ```rust
//! // Set up Selenium with default configuration
//! let (selenium_container, driver, port) = setup_selenium_with_default_args().await?;
//!
//! // Set up Selenium with custom Chrome arguments
//! let extra_args = vec!["--window-size=1920,1080".to_string()];
//! let (selenium_container, driver, port) = setup_selenium_with_custom_args(extra_args).await?;
//!
//! // Clean up everything together
//! cleanup_selenium_test_env(driver, selenium_container, None).await?;
//! ```

use anyhow::Result;
use std::collections::HashMap;
use testcontainers::core::Mount;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use thirtyfour::prelude::*;

/// Configuration for setting up a Selenium container
#[derive(Debug, Clone)]
pub struct SeleniumConfig {
    pub max_sessions: u8,
    pub session_timeout: u16,
    pub extra_chrome_args: Vec<String>,
    pub enable_vnc: bool,
    pub network: Option<String>,
}

impl Default for SeleniumConfig {
    fn default() -> Self {
        Self {
            max_sessions: 1,
            session_timeout: 300,
            extra_chrome_args: vec![
                "--no-sandbox".to_string(),
                "--disable-dev-shm-usage".to_string(),
                "--disable-gpu".to_string(),
                "--disable-web-security".to_string(),
                "--allow-running-insecure-content".to_string(),
                "--disable-features=VizDisplayCompositor".to_string(),
                "--lang=en".to_string(),
            ],
            enable_vnc: false,
            network: None,
        }
    }
}

/// Configuration for setting up an application container
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub image_name: String,
    pub image_tag: String,
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub extra_env: HashMap<String, String>,
    pub network: Option<String>,
    pub host_port: Option<u16>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            image_name: "sortingoffice".to_string(),
            image_tag: "latest".to_string(),
            port: 3000,
            host: "0.0.0.0".to_string(),
            database_url: String::new(),
            extra_env: HashMap::new(),
            network: None,
            host_port: None,
        }
    }
}

/// Result type for Selenium container setup
pub type SeleniumResult = Result<(ContainerAsync<GenericImage>, WebDriver, u16)>;

/// Result type for App container setup  
pub type AppResult = Result<(ContainerAsync<GenericImage>, String, u16)>;

/// Helper to find a free port on the system
pub fn find_free_port() -> u16 {
    use std::net::TcpListener;
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to find free port")
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

/// Set up a Selenium container with configurable options
pub async fn setup_selenium_container(config: SeleniumConfig) -> SeleniumResult {
    let _selenium_port = find_free_port();
    
    let mut selenium_image = GenericImage::new("selenium/standalone-chrome", "latest")
        .with_env_var("SE_NODE_MAX_SESSIONS", &config.max_sessions.to_string())
        .with_env_var("SE_NODE_OVERRIDE_MAX_SESSIONS", "true")
        .with_env_var("SE_NODE_SESSION_TIMEOUT", &config.session_timeout.to_string())
        .with_env_var("SE_START_XVFB", "false")

        .with_mount(Mount::bind_mount("/dev/shm", "/dev/shm"));

    // Add VNC if enabled
    if config.enable_vnc {
        selenium_image = selenium_image
            .with_env_var("SE_START_VNC", "true");
    }

    // Add custom Chrome arguments
    if !config.extra_chrome_args.is_empty() {
        let chrome_opts = config.extra_chrome_args.join(" ");
        selenium_image = selenium_image.with_env_var("SE_NODE_CHROME_OPTIONS", &chrome_opts);
    }

    // Start the container
    let selenium_container = AsyncRunner::start(selenium_image).await?;
    let selenium_host_port = selenium_container
        .get_host_port_ipv4(4444)
        .await?;

    println!("[SELENIUM] Container started on port {}", selenium_host_port);

    // Set up WebDriver capabilities
    let mut caps = DesiredCapabilities::chrome();
    
    // Add all the Chrome arguments
    for arg in &config.extra_chrome_args {
        caps.add_arg(arg)?;
    }

    // Connect to WebDriver
    let selenium_url = format!("http://127.0.0.1:{}", selenium_host_port);
    println!("[SELENIUM] Connecting to WebDriver at {}", selenium_url);
    
    let driver = WebDriver::new(&selenium_url, caps).await?;
    
    Ok((selenium_container, driver, selenium_host_port))
}

/// Set up an application container with configurable options
pub async fn setup_app_container(config: AppConfig) -> AppResult {
    let _host_port = config.host_port.unwrap_or_else(find_free_port);
    
    let mut app_image = GenericImage::new(&config.image_name, &config.image_tag)
        .with_env_var("DATABASE_URL", &config.database_url)
        .with_env_var("PORT", &config.port.to_string())
        .with_env_var("HOST", &config.host)
;

    // Add extra environment variables
    for (key, value) in &config.extra_env {
        app_image = app_image.with_env_var(key, value);
    }

    // Start the container  
    let app_container = AsyncRunner::start(app_image).await?;
    let app_host_port = app_container
        .get_host_port_ipv4(config.port)
        .await?;

    // Get container bridge IP for inter-container communication
    let bridge_ip = app_container.get_bridge_ip_address().await?;
    
    println!("[APP] Container started on host port {} (bridge IP: {})", app_host_port, bridge_ip);

    Ok((app_container, bridge_ip.to_string(), app_host_port))
}

/// Convenience function for setting up Selenium with default Chrome args
pub async fn setup_selenium_with_default_args() -> SeleniumResult {
    setup_selenium_container(SeleniumConfig::default()).await
}

/// Convenience function for setting up Selenium with custom Chrome args
pub async fn setup_selenium_with_custom_args(extra_args: Vec<String>) -> SeleniumResult {
    let mut config = SeleniumConfig::default();
    config.extra_chrome_args.extend(extra_args);
    setup_selenium_container(config).await
}

/// Convenience function for setting up the sortingoffice app container
pub async fn setup_sortingoffice_app(database_url: &str, extra_env: HashMap<String, String>) -> AppResult {
    let mut config = AppConfig::default();
    config.database_url = database_url.to_string();
    config.extra_env = extra_env;
    setup_app_container(config).await
}

/// Clean up multiple containers safely
pub async fn cleanup_containers(containers: Vec<ContainerAsync<GenericImage>>) -> Result<()> {
    for container in containers {
        if let Err(e) = container.stop().await {
            eprintln!("[CLEANUP] Warning: Failed to stop container: {}", e);
        }
    }
    Ok(())
}

/// Clean up WebDriver and containers together  
pub async fn cleanup_selenium_test_env(
    driver: WebDriver,
    selenium_container: ContainerAsync<GenericImage>,
    app_container: Option<ContainerAsync<GenericImage>>,
) -> Result<()> {
    // Clean up driver first
    if let Err(e) = driver.quit().await {
        eprintln!("[CLEANUP] Warning: Failed to quit WebDriver: {}", e);
    }

    // Clean up containers
    let mut containers = vec![selenium_container];
    if let Some(app) = app_container {
        containers.push(app);
    }
    
    cleanup_containers(containers).await
}
