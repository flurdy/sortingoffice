use anyhow::Result;
use reqwest;
use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;
use std::env;
use std::net::TcpListener;
use std::process::Command;

use testcontainers::core::Mount;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers::GenericImage;
use testcontainers::ImageExt;
use thirtyfour::prelude::*;
use tokio::time::{timeout, Duration};

fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .unwrap()
        .port()
}

// Helper macro for 10s timeout on Selenium actions
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

async fn wait_for_selenium_ready(port: u16, max_wait: Duration) -> Result<()> {
    let _client = reqwest::Client::new();
    let url = format!("http://localhost:{}/status", port);
    let start = std::time::Instant::now();
    while start.elapsed() < max_wait {
        match reqwest::get(&url).await {
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

// Helper function to authenticate the driver
async fn authenticate_driver(driver: &WebDriver, base_url: &str) -> Result<()> {
    // println!("🔐 Authenticating with headless browser...");

    // Navigate to login page using the provided base_url
    let login_url = format!("{}/login", base_url.trim_end_matches('/'));
    // println!("Navigating to login page: {}", login_url);
    timeout60s!(driver.get(&login_url), "Navigate to login page")?;

    // Fill in login form
    // println!("Looking for username field...");
    let username_field = timeout60s!(
        driver.find(By::Css("input[name='id']")),
        "Find username field"
    )?;
    timeout60s!(username_field.send_keys("admin"), "Fill username field")?;
    // println!("Username field filled");

    // Wait a moment for the field to be properly filled
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // println!("Looking for password field...");
    let password_field = timeout60s!(
        driver.find(By::Css("input[name='password']")),
        "Find password field"
    )?;
    timeout60s!(password_field.send_keys("admin123"), "Fill password field")?;
    // println!("Password field filled");

    // Wait a moment for the field to be properly filled
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Submit the form
    // println!("Looking for submit button...");
    let submit_button = timeout60s!(
        driver.find(By::XPath(
            "//button[@type='submit' and contains(text(), 'Sign in')]"
        )),
        "Find submit button"
    )?;

    // Wait a moment for the button to be fully loaded
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Check if button is enabled and visible
    let is_enabled = timeout60s!(submit_button.is_enabled(), "Check button enabled")?;
    let is_displayed = timeout60s!(submit_button.is_displayed(), "Check button displayed")?;
    // println!(
    //     "Button enabled: {}, displayed: {}",
    //     is_enabled, is_displayed
    // );

    if is_enabled && is_displayed {
        timeout60s!(submit_button.click(), "Click submit button")?;
        // println!("Form submitted");
    } else {
        return Err(anyhow::anyhow!(
            "Submit button is not clickable: enabled={}, displayed={}",
            is_enabled,
            is_displayed
        ));
    }

    // Wait for redirect and check if we're authenticated
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let current_url = timeout60s!(driver.current_url(), "Get current URL")?;
    // println!("Current URL after login: {}", current_url);

    if current_url.as_str().contains("/login") {
        return Err(anyhow::anyhow!(
            "Still on login page after authentication attempt"
        ));
    }

    println!("✅ Authentication successful!");
    Ok(())
}

// Helper function to run a test with timeout
async fn run_test_with_timeout<F, T>(test_fn: F, timeout_duration: Duration) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    timeout(timeout_duration, test_fn)
        .await
        .map_err(|_| anyhow::anyhow!("Test timed out after {:?}", timeout_duration))?
}

/// Helper to get the bridge IP address of a running container by its ID
async fn get_container_bridge_ip(container_id: &str) -> anyhow::Result<String> {
    let output = Command::new("docker")
        .args([
            "inspect",
            "-f",
            "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}",
            container_id,
        ])
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to inspect container IP: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if ip.is_empty() {
        return Err(anyhow::anyhow!(
            "No IP address found for container {}",
            container_id
        ));
    }
    Ok(ip)
}

/// Centralized helper to start the app container with all required env/config
async fn setup_app_container(
    db_url: &str,
    host_port: u16,
    _admin_username: &str,
    _admin_password_hash: &str,
    config_path: &str,
    container_name: &str,
) -> anyhow::Result<(ContainerAsync<GenericImage>, String /* bridge IP */)> {
    // println!("[DEBUG] setup_app_container called with:");
    // println!("  db_url: {}", db_url);
    // println!("  host_port: {}", host_port);
    // println!("  admin_username: {}", admin_username);
    // println!("  admin_password_hash: {}", admin_password_hash);
    // println!("  config_path: {}", config_path);
    // println!("  container_name: {}", container_name);
    // println!("[DEBUG] Starting app container with image: sortingoffice:latest");
    let app_image = GenericImage::new("sortingoffice", "latest")
        .with_env_var("DATABASE_URL", db_url)
        .with_env_var("PORT", "4000")
        // .with_env_var("ADMIN_USERNAME", admin_username)
        // .with_env_var("ADMIN_PASSWORD_HASH", admin_password_hash)
        .with_mapped_port(host_port, 4000.into())
        .with_container_name(container_name)
        .with_mount(Mount::bind_mount(config_path, "/app/config/config.toml"));
    // println!("[DEBUG] Attempting to start app container...");
    let app_container = match AsyncRunner::start(app_image).await {
        Ok(c) => {
            // println!("[DEBUG] App container started: id={}", c.id());
            c
        }
        Err(e) => {
            println!("[ERROR] Failed to start app container: {:?}", e);
            return Err(e.into());
        }
    };
    let app_id = app_container.id();
    let app_ip = get_container_bridge_ip(&app_id).await?;
    // println!("[DEBUG] App container bridge IP: {}", app_ip);
    // Wait for the app to be healthy before returning
    let health_url = format!("http://{}:4000/health", app_ip);
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(30);
    loop {
        match client.get(&health_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                // println!("[DEBUG] App /health is healthy!");
                break;
            }
            Ok(resp) => {
                println!("[DEBUG] /health status: {}", resp.status());
            }
            Err(e) => {
                println!("[DEBUG] /health error: {}", e);
            }
        }
        if start.elapsed() > timeout {
            println!(
                "[ERROR] App /health did not become healthy in time: {}",
                health_url
            );
            return Err(anyhow::anyhow!("App /health not healthy after 30s"));
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Ok((app_container, app_ip))
}

/// Centralized helper to start Selenium container and return (container, WebDriver, port)
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
        WebDriver::new(&format!("http://localhost:{}", selenium_port), caps),
    )
    .await??;
    Ok((selenium, driver, selenium_port))
}

/// Helper to login and land on the dashboard
async fn login_and_goto_dashboard(driver: &WebDriver, app_url: &str) -> Result<()> {
    // Go to login page (or homepage, which should redirect to login if not authenticated)
    let login_url = format!("{}/login", app_url.trim_end_matches('/'));
    // println!("[DEBUG] Navigating to login: {}", login_url);
    timeout60s!(driver.get(&login_url), "Navigate to login page")?;
    authenticate_driver(driver, app_url).await?;
    // After login, go to dashboard/homepage
    let app_url = format!("{}/", app_url.trim_end_matches('/'));
    // println!("[DEBUG] Navigating to dashboard: {}", app_url);
    timeout60s!(driver.get(app_url), "Navigate to dashboard after login")?;
    Ok(())
}

struct TestEnv {
    app_container: ContainerAsync<GenericImage>,
    selenium_container: ContainerAsync<GenericImage>,
    driver: WebDriver,
    app_url: String,
}

async fn setup_ui_test_env() -> anyhow::Result<TestEnv> {
    let port = find_free_port();
    let test_db = setup_test_db().await;
    let db_url = format!(
        "mysql://root@{}:3306/{}",
        test_db.get_bridge_ip(),
        test_db.schema
    );
    let config_path = env::current_dir()
        .unwrap()
        .join("config/config.docker.toml");
    let config_path_str = config_path.to_str().unwrap();
    let unique_app_name = format!("app-{}", port);
    let admin_username = "admin";
    let admin_password_hash = "$2a$12$o8thacsiGCRhN1JN8xnW6e0KqNb7KrSgM67xxa62RKoAC9fOPf.aO";
    let (app_container, app_ip) = setup_app_container(
        &db_url,
        port,
        admin_username,
        admin_password_hash,
        config_path_str,
        &unique_app_name,
    )
    .await?;
    let (selenium_container, driver, _selenium_port) =
        setup_selenium_container_and_driver().await?;
    let app_url = format!("http://{}:4000", app_ip);
    Ok(TestEnv {
        app_container,
        selenium_container,
        driver,
        app_url,
    })
}

#[tokio::test]
async fn test_homepage_loads_containerized() -> Result<()> {
    run_test_with_timeout(
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            let _page_title = timeout60s!(env.driver.title(), "Get page title")?;
            let page_source = timeout60s!(env.driver.source(), "Get page source")?;
            assert!(page_source.contains("Dashboard"));
            assert!(page_source.contains("Quick Actions"));
            drop(env.app_container);
            drop(env.selenium_container);
            Ok(())
        },
        Duration::from_secs(40),
    )
    .await
}

#[tokio::test]
async fn test_domain_search_containerized() -> Result<()> {
    run_test_with_timeout(
        async {
            let env = setup_ui_test_env().await?;
            // println!("[DEBUG] App URL for Selenium: {}", env.app_url);
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            let aliases_url = format!("{}/aliases", env.app_url);
            // println!("[DEBUG] Navigating to: {}", aliases_url);
            timeout60s!(env.driver.get(&aliases_url), "Navigate to aliases page")?;
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Click the Add Alias button
            let add_alias_button = timeout60s!(
                env.driver.find(By::Id("add-alias-button")),
                "Find Add Alias button"
            )?;

            timeout30s!(add_alias_button.click(), "Click Add Alias button")?;
            // tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Try to find the mail input, but fail gracefully with a concise message
            let mail_input = match timeout30s!(
                env.driver.find(By::Css("input[name='mail']")),
                "Find mail input field"
            ) {
                Ok(input) => input,
                Err(_) => {
                    eprintln!(
                        "Test failed: Could not find mail input field after clicking Add Alias."
                    );
                    return Err(anyhow::anyhow!(
                        "Could not find mail input field after clicking Add Alias."
                    ));
                }
            };
            timeout60s!(mail_input.send_keys("@exa"), "Type @exa in mail field")?;
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            let page_source = timeout90s!(env.driver.source(), "Get page source")?;
            if page_source.contains("domain-search-results")
                || page_source.contains("No domains found")
            {
                // ok
            }
            drop(env.app_container);
            drop(env.selenium_container);
            Ok(())
        },
        Duration::from_secs(40),
    )
    .await
}

#[tokio::test]
async fn test_navigation_containerized() -> Result<()> {
    run_test_with_timeout(
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            let pages = vec![
                ("/domains", "Domains"),
                ("/users", "Users"),
                ("/aliases", "Aliases"),
                ("/stats", "Statistics"),
            ];
            for (path, expected_title) in pages {
                let url = format!("{}{}", env.app_url, path);
                timeout60s!(env.driver.get(&url), "Navigate to page")?;
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                let page_source = timeout60s!(env.driver.source(), "Get page source")?;
                assert!(
                    page_source.contains(expected_title),
                    "Page should contain {}",
                    expected_title
                );
            }
            drop(env.app_container);
            drop(env.selenium_container);
            Ok(())
        },
        Duration::from_secs(40),
    )
    .await
}

#[tokio::test]
#[ignore]
async fn test_minimal_async_testcontainers() {
    use testcontainers::{runners::AsyncRunner, GenericImage};
    let image = GenericImage::new("hello-world", "latest");
    let _container = AsyncRunner::start(image)
        .await
        .expect("Failed to start hello-world container");
    // If this compiles and runs, async API is available
}

#[tokio::test]
#[ignore]
async fn test_selenium_container_starts_and_status() -> Result<()> {
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
    // println!("[DEBUG] Selenium mapped port: {}", selenium_port);
    // Wait a bit for Selenium to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    // Try to fetch /status
    let url = format!("http://localhost:{}/status", selenium_port);
    let resp = reqwest::get(&url).await;
    match resp {
        Ok(r) => {
            // println!("[DEBUG] /status response: {}", r.status());
            let _text = r
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read body>".to_string());
            // println!("[DEBUG] /status body: {}", text);
        }
        Err(e) => {
            println!("[DEBUG] Error fetching /status: {}", e);
        }
    }
    // println!("[DEBUG] Sleeping for 30 seconds. Inspect the container now if needed.");
    // tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_minimal_webdriver_session() -> Result<()> {
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
    // println!("[DEBUG] Selenium mapped port: {}", selenium_port);
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
    // println!("[DEBUG] Attempting to create minimal WebDriver session...");
    let driver_result = timeout(
        Duration::from_secs(20),
        WebDriver::new(&format!("http://localhost:{}", selenium_port), caps),
    )
    .await;
    match driver_result {
        Ok(Ok(_driver)) => {
            // println!("[DEBUG] Minimal WebDriver session created successfully.");
            Ok(())
        }
        Ok(Err(e)) => {
            println!("[DEBUG] Minimal WebDriver::new error: {:#?}", e);
            Err(e.into())
        }
        Err(e) => {
            println!(
                "[DEBUG] Timeout waiting for minimal WebDriver::new: {:#?}",
                e
            );
            Err(anyhow::anyhow!(
                "Timeout waiting for minimal WebDriver::new: {:#?}",
                e
            ))
        }
    }
}

// #[tokio::test]
// async fn test_mysql_container_cleanup() -> Result<()> {
//     // This test demonstrates the cleanup functionality
//     // In a real test suite, you might call this at the end of all tests

//     // First, ensure a MySQL container is started
//     let _test_db = setup_test_db().await;
//     println!("[DEBUG] MySQL container started for cleanup test");

//     // Now clean it up
//     cleanup_shared_mysql_container().await;
//     println!("[DEBUG] MySQL container cleanup completed");

//     Ok(())
// }
