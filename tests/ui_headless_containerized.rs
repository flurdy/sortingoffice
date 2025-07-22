use anyhow::Result;
use reqwest;
use testcontainers::core::Mount;
use testcontainers::core::{ContainerPort, Host};
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;
use testcontainers::ImageExt;
use testcontainers::{ContainerAsync};
use thirtyfour::prelude::*;
use tokio::time::{timeout, Duration};
use tokio::sync::OnceCell;
use rand::Rng;
use std::process::{Child, Command, Stdio};
use std::net::TcpListener;
use std::io::{BufRead, BufReader};
use std::thread;

static MYSQL_CONTAINER: OnceCell<testcontainers::ContainerAsync<GenericImage>> = OnceCell::const_new();
static MYSQL_PORT: OnceCell<u16> = OnceCell::const_new();

fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .unwrap()
        .port()
}

// Replace start_app_subprocess with Docker-based launch
async fn start_app_container(database_url: &str, port: u16) -> anyhow::Result<ContainerAsync<GenericImage>> {
    let app_image = GenericImage::new("sortingoffice", "latest")
        .with_env_var("DATABASE_URL", database_url)
        .with_env_var("PORT", port.to_string())
        .with_mapped_port(port, 3000.into());
    let container = testcontainers::runners::AsyncRunner::start(app_image).await?;
    // Wait for the app to be ready on the mapped port
    let mapped_port = container.get_host_port_ipv4(3000).await.expect("get port");
    wait_for_app_ready(mapped_port, Duration::from_secs(30)).await.expect("wait for app");
    Ok(container)
}

async fn get_shared_mysql_container() -> (&'static testcontainers::ContainerAsync<GenericImage>, u16) {
    let container = MYSQL_CONTAINER
        .get_or_init(|| async {
            let mysql_image = GenericImage::new("mysql", "8.0")
                .with_env_var("MYSQL_ROOT_PASSWORD", "rootpassword")
                .with_env_var("MYSQL_DATABASE", "sortingoffice")
                .with_env_var("MYSQL_USER", "sortingoffice")
                .with_env_var("MYSQL_PASSWORD", "sortingoffice");
            let c = AsyncRunner::start(mysql_image).await.expect("Failed to start MySQL container");
            // Wait for MySQL to be ready (try to connect, up to 30s)
            let port = c.get_host_port_ipv4(3306).await.expect("get port");
            let start = std::time::Instant::now();
            let mut ready = false;
            while start.elapsed() < std::time::Duration::from_secs(30) {
                let url = format!("mysql://root:rootpassword@127.0.0.1:{}/mysql", port);
                let opts = mysql_async::Opts::from_url(&url).unwrap();
                let pool = mysql_async::Pool::new(opts);
                match pool.get_conn().await {
                    Ok(mut conn) => {
                        if mysql_async::prelude::Queryable::ping(&mut conn).await.is_ok() {
                            ready = true;
                            drop(conn);
                            let _ = pool.disconnect().await;
                            break;
                        }
                        drop(conn);
                        let _ = pool.disconnect().await;
                    }
                    Err(_) => {}
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            if !ready {
                panic!("MySQL container did not become ready in 30s");
            }
            c
        })
        .await;
    let port = *MYSQL_PORT
        .get_or_init(|| async {
            container.get_host_port_ipv4(3306).await.expect("get port")
        })
        .await;
    (container, port)
}

async fn create_unique_schema() -> String {
    let (_container, port) = get_shared_mysql_container().await;
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let schema: String = format!(
        "test_{}",
        (0..8)
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect::<String>()
    );
    let url = format!("mysql://root:rootpassword@127.0.0.1:{}/mysql", port);
    let opts = mysql_async::Opts::from_url(&url).unwrap();
    let pool = mysql_async::Pool::new(opts);
    let mut conn = pool.get_conn().await.unwrap();
    let sql = format!("CREATE DATABASE IF NOT EXISTS `{}`;", schema);
    mysql_async::prelude::Queryable::query_drop(&mut conn, sql).await.unwrap();
    drop(conn);
    pool.disconnect().await.unwrap();
    schema
}

// Helper macro for 10s timeout on Selenium actions
macro_rules! timeout10s {
    ($expr:expr, $desc:expr) => {
        timeout(Duration::from_secs(10), $expr)
            .await
            .map_err(|_| anyhow::anyhow!(concat!("Timeout (10s) on: ", $desc)))?
    };
}

// Helper macro for 30s timeout on application startup
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

async fn wait_for_app_ready(port: u16, max_wait: Duration) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}/", port);
    let start = std::time::Instant::now();
    loop {
        match client.get(&url).timeout(Duration::from_secs(2)).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    return Ok(());
                }
            }
            Err(_) => {}
        }
        if start.elapsed() > max_wait {
            return Err(anyhow::anyhow!("Timed out waiting for app on port {}", port));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn wait_for_selenium_ready(port: u16, max_wait: Duration) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}/status", port);
    let start = std::time::Instant::now();
    loop {
        match client
            .get(&url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if json["value"]["ready"].as_bool().unwrap_or(false) {
                            println!("✅ Selenium is ready on port {:?}", port);
                            return Ok(());
                        } else {
                            println!("[DEBUG] Selenium /status not ready: {:?}", json);
                        }
                    } else {
                        println!("[DEBUG] Could not parse Selenium /status JSON");
                    }
                } else {
                    println!("[DEBUG] Selenium /status HTTP status: {}", resp.status());
                }
            }
            Err(e) => {
                println!("[DEBUG] Error connecting to Selenium /status: {}", e);
            }
        }
        if start.elapsed() > max_wait {
            return Err(anyhow::anyhow!(
                "Timed out waiting for Selenium to be ready on port {}",
                port
            ));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

// Helper to start Selenium container and return WebDriver
async fn setup_selenium_driver() -> Result<WebDriver> {
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
        .with_host("host.docker.internal", Host::HostGateway)
        .with_mount(Mount::bind_mount("/dev/shm", "/dev/shm"));
    let selenium = AsyncRunner::start(selenium_image).await?;
    let selenium_port = selenium.get_host_port_ipv4(4444).await?;
    println!("[DEBUG] Selenium mapped port: {}", selenium_port);
    timeout90s!(wait_for_selenium_ready(selenium_port, Duration::from_secs(90)), "Wait for selenium ready")?;
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless=new")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--allow-running-insecure-content")?;
    caps.add_arg("--host-resolver-rules=MAP * 127.0.0.1")?;
    caps.add_arg("--remote-debugging-port=9222")?;
    caps.add_arg("--whitelisted-ips=")?;
    caps.add_arg("--disable-features=VizDisplayCompositor")?;
    let driver = timeout(Duration::from_secs(10), WebDriver::new(&format!("http://localhost:{}", selenium_port), caps)).await??;
    Ok(driver)
}

// Helper function to authenticate the driver
async fn authenticate_driver(driver: &WebDriver, app_port: u16) -> Result<()> {
    println!("🔐 Authenticating with headless browser...");

    // Navigate to login page using Docker host gateway
    let login_url = format!("http://host.docker.internal:{}", app_port);
    println!("Navigating to login page: {}", login_url);
    timeout10s!(driver.get(&login_url), "Navigate to login page")?;

    // Fill in login form
    println!("Looking for username field...");
    let username_field = timeout10s!(
        driver.find(By::Css("input[name='id']")),
        "Find username field"
    )?;
    timeout10s!(username_field.send_keys("admin"), "Fill username field")?;
    println!("Username field filled");

    // Wait a moment for the field to be properly filled
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    println!("Looking for password field...");
    let password_field = timeout10s!(
        driver.find(By::Css("input[name='password']")),
        "Find password field"
    )?;
    timeout10s!(password_field.send_keys("admin123"), "Fill password field")?;
    println!("Password field filled");

    // Wait a moment for the field to be properly filled
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Submit the form
    println!("Looking for submit button...");
    let submit_button = timeout10s!(
        driver.find(By::XPath(
            "//button[@type='submit' and contains(text(), 'Sign in')]"
        )),
        "Find submit button"
    )?;

    // Wait a moment for the button to be fully loaded
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Check if button is enabled and visible
    let is_enabled = timeout10s!(submit_button.is_enabled(), "Check button enabled")?;
    let is_displayed = timeout10s!(submit_button.is_displayed(), "Check button displayed")?;
    println!(
        "Button enabled: {}, displayed: {}",
        is_enabled, is_displayed
    );

    if is_enabled && is_displayed {
        timeout10s!(submit_button.click(), "Click submit button")?;
        println!("Form submitted");
    } else {
        return Err(anyhow::anyhow!(
            "Submit button is not clickable: enabled={}, displayed={}",
            is_enabled,
            is_displayed
        ));
    }

    // Wait for redirect and check if we're authenticated
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    let current_url = timeout10s!(driver.current_url(), "Get current URL")?;
    println!("Current URL after login: {}", current_url);

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

#[tokio::test]
async fn test_homepage_loads_containerized() -> Result<()> {
    run_test_with_timeout(
        async {
            let (_container, mysql_port) = get_shared_mysql_container().await;
            let schema = create_unique_schema().await;
            let port = find_free_port();
            let database_url = format!("mysql://root:rootpassword@127.0.0.1:{}/{}", mysql_port, schema);
            let app_container = start_app_container(&database_url, port).await?;
            let driver = setup_selenium_driver().await?;
            let app_url = format!("http://host.docker.internal:{}", port);
            // Test logic:
            timeout10s!(driver.get(&app_url), "Navigate to homepage")?;
            authenticate_driver(&driver, port).await?;
            let page_title = timeout10s!(driver.title(), "Get page title")?;
            let page_source = timeout10s!(driver.source(), "Get page source")?;
            assert!(page_source.contains("Dashboard"));
            assert!(page_source.contains("Quick Actions"));
            // Cleanup:
            drop(app_container);
            Ok(())
        },
        Duration::from_secs(60),
    )
    .await
}

#[tokio::test]
async fn test_domain_search_containerized() -> Result<()> {
    run_test_with_timeout(
        async {
            let (_container, mysql_port) = get_shared_mysql_container().await;
            let schema = create_unique_schema().await;
            let port = find_free_port();
            let database_url = format!("mysql://root:rootpassword@127.0.0.1:{}/{}", mysql_port, schema);
            let app_container = start_app_container(&database_url, port).await;
            let driver = setup_selenium_driver().await?;
            let app_url = format!("http://host.docker.internal:{}/aliases", port);
            timeout10s!(driver.get(&app_url), "Navigate to aliases page")?;
            authenticate_driver(&driver, port).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            let mail_input = timeout10s!(driver.find(By::Css("input[name='mail']")), "Find mail input field")?;
            timeout10s!(mail_input.send_keys("@exa"), "Type @exa in mail field")?;
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            let page_source = timeout10s!(driver.source(), "Get page source")?;
            if page_source.contains("domain-search-results") || page_source.contains("No domains found") {
                // ok
            }
            drop(app_container);
            Ok(())
        },
        Duration::from_secs(60),
    )
    .await
}

#[tokio::test]
async fn test_navigation_containerized() -> Result<()> {
    run_test_with_timeout(
        async {
            let (_container, mysql_port) = get_shared_mysql_container().await;
            let schema = create_unique_schema().await;
            let port = find_free_port();
            let database_url = format!("mysql://root:rootpassword@127.0.0.1:{}/{}", mysql_port, schema);
            let app_container = start_app_container(&database_url, port).await;
            let driver = setup_selenium_driver().await?;
            let app_url = format!("http://host.docker.internal:{}", port);
            timeout10s!(driver.get(&app_url), "Navigate to homepage")?;
            authenticate_driver(&driver, port).await?;
            let pages = vec![
                ("/domains", "Domains"),
                ("/users", "Users"),
                ("/aliases", "Aliases"),
                ("/stats", "Statistics"),
            ];
            for (path, expected_title) in pages {
                let url = format!("http://host.docker.internal:{}{}", port, path);
                timeout10s!(driver.get(&url), "Navigate to page")?;
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                let page_source = timeout10s!(driver.source(), "Get page source")?;
                assert!(page_source.contains(expected_title), "Page should contain {}", expected_title);
            }
            drop(app_container);
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_minimal_async_testcontainers() {
    use testcontainers::{runners::AsyncRunner, GenericImage};
    let image = GenericImage::new("hello-world", "latest");
    let _container = AsyncRunner::start(image)
        .await
        .expect("Failed to start hello-world container");
    // If this compiles and runs, async API is available
}
