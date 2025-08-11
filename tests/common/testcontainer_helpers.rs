//! Testcontainer helper functions for setting up and managing test containers
//!
//! This module provides a unified API for setting up Selenium containers with optimized
//! startup times and parallel operations where possible.
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
//! // Set up Selenium on a shared network
//! let (selenium_container, driver, port) = setup_selenium_on_shared_network(extra_args, "network-name").await?;
//!
//! // Clean up everything together
//! cleanup_selenium_test_env(driver, selenium_container, None).await?;
//! ```

use anyhow::Result;
use std::time::Duration;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use thirtyfour::prelude::*;

/// Configuration for Selenium container optimization
#[derive(Debug, Clone)]
struct SeleniumOptimizationConfig {
    /// Timeout for container startup (shorter in CI)
    startup_timeout: Duration,
    /// Health check interval (more frequent in CI)
    health_check_interval: Duration,
}

impl Default for SeleniumOptimizationConfig {
    fn default() -> Self {
        let is_ci = std::env::var("CI").unwrap_or_default() == "true";
        Self {
            startup_timeout: if is_ci {
                Duration::from_secs(30) // Faster fail in CI
            } else {
                Duration::from_secs(60) // More patient locally
            },
            health_check_interval: if is_ci {
                Duration::from_millis(500) // More frequent in CI
            } else {
                Duration::from_secs(2) // Less aggressive locally
            },
        }
    }
}

/// Core Selenium container setup with optimized configuration
async fn setup_selenium_core(
    extra_args: Vec<String>,
    network: Option<&str>,
) -> Result<(ContainerAsync<GenericImage>, u16)> {
    let config = SeleniumOptimizationConfig::default();

    // Build optimized Selenium image
    let mut selenium_image = GenericImage::new("selenium/standalone-chrome", "latest")
        .with_env_var("SE_NODE_MAX_SESSIONS", "1")
        .with_env_var("SE_NODE_OVERRIDE_MAX_SESSIONS", "true")
        .with_env_var("SE_NODE_SESSION_TIMEOUT", "300")
        .with_env_var("SE_START_XVFB", "false")
        .with_env_var("SE_SCREEN_WIDTH", "1920")
        .with_env_var("SE_SCREEN_HEIGHT", "1080")
        .with_env_var("SE_SCREEN_DEPTH", "24")
        .with_env_var("SE_SCREEN_DPI", "96")
        .with_env_var("SE_SCREEN_RESOLUTION", "1920x1080x24")
        .with_env_var("SE_VNC_NO_PASSWORD", "1");

    // Add network configuration if specified
    if let Some(net) = network {
        selenium_image = selenium_image.with_network(net);
    }

    // Add extra Chrome arguments efficiently
    if !extra_args.is_empty() {
        selenium_image = selenium_image.with_env_var("SE_NODE_CHROME_ARGS", extra_args.join(" "));
    }

    // Start container and get port in parallel
    let selenium_container = AsyncRunner::start(selenium_image).await?;
    let selenium_port = selenium_container.get_host_port_ipv4(4444).await?;

    // Optimized health check with configurable timeouts
    let selenium_url = format!("http://localhost:{selenium_port}");
    let start = std::time::Instant::now();

    while start.elapsed() < config.startup_timeout {
        match reqwest::get(&selenium_url).await {
            Ok(resp) if resp.status().is_success() => break,
            _ => {
                tokio::time::sleep(config.health_check_interval).await;
            }
        }
    }

    if start.elapsed() >= config.startup_timeout {
        return Err(anyhow::anyhow!(
            "Selenium container failed to start within {:?}",
            config.startup_timeout
        ));
    }

    Ok((selenium_container, selenium_port))
}

/// Set up WebDriver with optimized capabilities
async fn setup_webdriver_core(selenium_url: &str, extra_args: &[String]) -> Result<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();

    // Essential Chrome arguments for stability
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;

    // Add extra arguments efficiently
    for arg in extra_args {
        caps.add_arg(arg)?;
    }

    // Connect with timeout
    let driver =
        tokio::time::timeout(Duration::from_secs(10), WebDriver::new(selenium_url, caps)).await??;

    Ok(driver)
}

/// Set up a Selenium container with default configuration (optimized)
pub async fn setup_selenium_with_default_args(
) -> Result<(ContainerAsync<GenericImage>, WebDriver, u16)> {
    let (selenium_container, selenium_port) = setup_selenium_core(vec![], None).await?;
    let selenium_url = format!("http://localhost:{selenium_port}");

    let driver = setup_webdriver_core(&selenium_url, &[]).await?;

    Ok((selenium_container, driver, selenium_port))
}

/// Set up a Selenium container with custom Chrome arguments (optimized)
pub async fn setup_selenium_with_custom_args(
    extra_args: Vec<String>,
) -> Result<(ContainerAsync<GenericImage>, WebDriver, u16)> {
    let (selenium_container, selenium_port) = setup_selenium_core(extra_args.clone(), None).await?;
    let selenium_url = format!("http://localhost:{selenium_port}");

    let driver = setup_webdriver_core(&selenium_url, &extra_args).await?;

    Ok((selenium_container, driver, selenium_port))
}

/// Set up a Selenium container on a shared network (optimized)
pub async fn setup_selenium_on_shared_network(
    extra_args: Vec<String>,
    network: &str,
) -> Result<(ContainerAsync<GenericImage>, WebDriver, u16)> {
    let (selenium_container, selenium_port) =
        setup_selenium_core(extra_args.clone(), Some(network)).await?;
    let selenium_url = format!("http://localhost:{selenium_port}");

    let driver = setup_webdriver_core(&selenium_url, &extra_args).await?;

    Ok((selenium_container, driver, selenium_port))
}

/// Parallel container cleanup for better performance
pub async fn cleanup_containers(containers: Vec<ContainerAsync<GenericImage>>) -> Result<()> {
    if containers.is_empty() {
        return Ok(());
    }

    // Use join_all for parallel cleanup
    let cleanup_futures: Vec<_> = containers
        .into_iter()
        .map(|container| async move {
            if let Err(e) = container.stop().await {
                eprintln!("[CLEANUP] Warning: Failed to stop container: {}", e);
            }
        })
        .collect();

    futures_util::future::join_all(cleanup_futures).await;
    Ok(())
}

/// Clean up WebDriver and containers together (optimized)
pub async fn cleanup_selenium_test_env(
    driver: WebDriver,
    selenium_container: ContainerAsync<GenericImage>,
    app_container: Option<ContainerAsync<GenericImage>>,
) -> Result<()> {
    // Clean up driver first (this can be slow)
    let driver_cleanup = async {
        if let Err(e) = driver.quit().await {
            eprintln!("[CLEANUP] Warning: Failed to quit WebDriver: {}", e);
        }
    };

    // Prepare containers for cleanup
    let mut containers = vec![selenium_container];
    if let Some(app) = app_container {
        containers.push(app);
    }

    // Run driver cleanup and container cleanup in parallel
    let (_, _) = tokio::join!(driver_cleanup, cleanup_containers(containers));

    Ok(())
}
