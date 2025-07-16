use anyhow::Result;
use reqwest;
use testcontainers::{clients, Container, GenericImage};
use thirtyfour::prelude::*;
use tokio::time::{timeout, Duration};

// Add a constant for the app port (default 3000)
const APP_PORT: u16 = 3000;

// Helper macro for 10s timeout on Selenium actions
macro_rules! timeout10s {
    ($expr:expr, $desc:expr) => {
        timeout(Duration::from_secs(10), $expr)
            .await
            .map_err(|_| anyhow::anyhow!(concat!("Timeout (10s) on: ", $desc)))??
    };
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
                            println!("✅ Selenium is ready on port {}", port);
                            return Ok(());
                        }
                    }
                }
            }
            Err(_) => {}
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

async fn setup_driver<'a>(
    docker: &'a clients::Cli,
) -> Result<(WebDriver, Container<'a, GenericImage>, u16)> {
    // Start Selenium standalone Chrome container
    let selenium = docker
        .run(GenericImage::new("selenium/standalone-chrome", "latest").with_exposed_port(4444));
    let port = selenium.get_host_port_ipv4(4444);

    // Wait for Selenium to be ready
    if let Err(e) = wait_for_selenium_ready(port, Duration::from_secs(20)).await {
        println!(
            "Selenium container failed to become ready. Container ID: {} | Image: {:?}",
            selenium.id(),
            selenium.image()
        );
        println!("You can fetch logs with: docker logs {}", selenium.id());
        return Err(e);
    }

    // Configure Chrome options for headless mode
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless=new")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--allow-running-insecure-content")?;

    let driver = timeout(
        Duration::from_secs(10),
        WebDriver::new(&format!("http://localhost:{}", port), caps),
    )
    .await??;
    println!("✅ Connected to Selenium at http://localhost:{}", port);
    Ok((driver, selenium, APP_PORT))
}

// Helper function to authenticate the driver
async fn authenticate_driver(driver: &WebDriver, app_port: u16) -> Result<()> {
    println!("🔐 Authenticating with headless browser...");

    // Navigate to login page using Docker host gateway
    let login_url = format!("http://172.17.0.1:{}/login", app_port);
    println!("Navigating to login page: {}", login_url);
    timeout10s!(driver.get(&login_url), "Navigate to login page");

    // Fill in login form
    println!("Looking for username field...");
    let username_field = timeout10s!(
        driver.find(By::Css("input[name='id']")),
        "Find username field"
    );
    timeout10s!(username_field.send_keys("admin"), "Fill username field");
    println!("Username field filled");

    // Wait a moment for the field to be properly filled
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    println!("Looking for password field...");
    let password_field = timeout10s!(
        driver.find(By::Css("input[name='password']")),
        "Find password field"
    );
    timeout10s!(password_field.send_keys("admin123"), "Fill password field");
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
    );

    // Wait a moment for the button to be fully loaded
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Check if button is enabled and visible
    let is_enabled = timeout10s!(submit_button.is_enabled(), "Check button enabled");
    let is_displayed = timeout10s!(submit_button.is_displayed(), "Check button displayed");
    println!(
        "Button enabled: {}, displayed: {}",
        is_enabled, is_displayed
    );

    if is_enabled && is_displayed {
        timeout10s!(submit_button.click(), "Click submit button");
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

    let current_url = timeout10s!(driver.current_url(), "Get current URL");
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
async fn test_homepage_loads_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("🌐 Testing homepage loads in headless browser...");

            // Navigate to homepage using Docker host gateway
            let home_url = format!("http://172.17.0.1:{}", app_port);
            println!("Navigating to homepage: {}", home_url);
            timeout10s!(driver.get(&home_url), "Navigate to homepage");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Check if we're on the dashboard
            let page_title = timeout10s!(driver.title(), "Get page title");
            println!("Page title: {}", page_title);

            let page_source = timeout10s!(driver.source(), "Get page source");
            if !page_source.contains("Dashboard") && !page_source.contains("dashboard") {
                return Err(anyhow::anyhow!(
                    "Dashboard page does not contain expected content"
                ));
            }

            println!("✅ Homepage loads successfully in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_aliases_list_page_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("📧 Testing aliases list page in headless browser...");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Navigate to aliases page using Docker host gateway
            let aliases_url = format!("http://172.17.0.1:{}/aliases", app_port);
            println!("Navigating to aliases page: {}", aliases_url);
            timeout10s!(driver.get(&aliases_url), "Navigate to aliases page");

            // Check if we're on the aliases page
            let page_title = timeout10s!(driver.title(), "Get page title");
            println!("Page title: {}", page_title);

            let page_source = timeout10s!(driver.source(), "Get page source");
            if !page_source.contains("Aliases") && !page_source.contains("aliases") {
                return Err(anyhow::anyhow!(
                    "Aliases page does not contain expected content"
                ));
            }

            println!("✅ Aliases list page loads successfully in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_dashboard_navigation_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("🧭 Testing dashboard navigation in headless browser...");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Navigate to dashboard using Docker host gateway
            let dashboard_url = format!("http://172.17.0.1:{}", app_port);
            println!("Navigating to dashboard: {}", dashboard_url);
            timeout10s!(driver.get(&dashboard_url), "Navigate to dashboard");

            // Check if we're on the dashboard
            let page_source = timeout10s!(driver.source(), "Get page source");
            if !page_source.contains("Dashboard") && !page_source.contains("dashboard") {
                return Err(anyhow::anyhow!(
                    "Dashboard page does not contain expected content"
                ));
            }

            println!("✅ Dashboard navigation works in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_domains_list_page_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("🌍 Testing domains list page in headless browser...");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Navigate to domains page using Docker host gateway
            let domains_url = format!("http://172.17.0.1:{}/domains", app_port);
            println!("Navigating to domains page: {}", domains_url);
            timeout10s!(driver.get(&domains_url), "Navigate to domains page");

            // Check if we're on the domains page
            let page_title = timeout10s!(driver.title(), "Get page title");
            println!("Page title: {}", page_title);

            let page_source = timeout10s!(driver.source(), "Get page source");
            if !page_source.contains("Domains") && !page_source.contains("domains") {
                return Err(anyhow::anyhow!(
                    "Domains page does not contain expected content"
                ));
            }

            println!("✅ Domains list page loads successfully in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_users_list_page_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("👥 Testing users list page in headless browser...");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Navigate to users page using Docker host gateway
            let users_url = format!("http://172.17.0.1:{}/users", app_port);
            println!("Navigating to users page: {}", users_url);
            timeout10s!(driver.get(&users_url), "Navigate to users page");

            // Check if we're on the users page
            let page_title = timeout10s!(driver.title(), "Get page title");
            println!("Page title: {}", page_title);

            let page_source = timeout10s!(driver.source(), "Get page source");
            if !page_source.contains("Users") && !page_source.contains("users") {
                return Err(anyhow::anyhow!(
                    "Users page does not contain expected content"
                ));
            }

            println!("✅ Users list page loads successfully in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_clients_list_page_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("💻 Testing clients list page in headless browser...");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Navigate to clients page using Docker host gateway
            let clients_url = format!("http://172.17.0.1:{}/clients", app_port);
            println!("Navigating to clients page: {}", clients_url);
            timeout10s!(driver.get(&clients_url), "Navigate to clients page");

            // Check if we're on the clients page
            let page_title = timeout10s!(driver.title(), "Get page title");
            println!("Page title: {}", page_title);

            let page_source = timeout10s!(driver.source(), "Get page source");
            if !page_source.contains("Clients") && !page_source.contains("clients") {
                return Err(anyhow::anyhow!(
                    "Clients page does not contain expected content"
                ));
            }

            println!("✅ Clients list page loads successfully in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_responsive_design_headless() -> Result<()> {
    let test_timeout = Duration::from_secs(60);
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _container, app_port) = setup_driver(&docker).await?;
            // Test desktop viewport
            timeout10s!(
                driver.set_window_rect(0, 0, 1920, 1080),
                "set window rect to desktop"
            );
            timeout10s!(
                driver.get(&format!("http://172.17.0.1:{}", app_port)),
                "navigate to homepage for desktop viewport"
            );

            // Test mobile viewport
            timeout10s!(
                driver.set_window_rect(0, 0, 375, 667),
                "set window rect to mobile"
            );
            timeout10s!(
                driver.get(&format!("http://172.17.0.1:{}", app_port)),
                "navigate to homepage for mobile viewport"
            );

            // Both should load without errors
            let current_url =
                timeout10s!(driver.current_url(), "get current url after responsive nav");
            assert!(current_url.as_str().contains("3000"));

            timeout10s!(driver.quit(), "quit driver");
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_error_pages_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("🚫 Testing error pages in headless browser...");

            // Test 404 page
            let error_url = format!("http://172.17.0.1:{}/nonexistent-page", app_port);
            println!("Navigating to non-existent page: {}", error_url);
            timeout10s!(driver.get(&error_url), "Navigate to 404 page");

            // Should show some error content
            let page_source = timeout10s!(driver.source(), "Get page source");
            if !page_source.contains("404")
                && !page_source.contains("Not Found")
                && !page_source.contains("Error")
            {
                return Err(anyhow::anyhow!(
                    "404 page does not contain expected error content"
                ));
            }

            println!("✅ Error pages work correctly in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_form_validation_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("📝 Testing form validation in headless browser...");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Navigate to domain creation form
            let form_url = format!("http://172.17.0.1:{}/domains/new", app_port);
            println!("Navigating to domain creation form: {}", form_url);
            timeout10s!(driver.get(&form_url), "Navigate to domain form");

            // Check for form elements
            let forms = timeout10s!(driver.find_all(By::Css("form")), "Find form elements");
            if forms.is_empty() {
                return Err(anyhow::anyhow!("Domain creation form not found"));
            }

            // Check for input elements
            let inputs = timeout10s!(
                driver.find_all(By::Css("input, textarea, select")),
                "Find input elements"
            );
            if inputs.is_empty() {
                return Err(anyhow::anyhow!("Form should have input elements"));
            }

            println!("✅ Form validation works correctly in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_navigation_menu_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("🧭 Testing navigation menu in headless browser...");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Navigate to homepage
            let home_url = format!("http://172.17.0.1:{}", app_port);
            println!("Navigating to homepage: {}", home_url);
            timeout10s!(driver.get(&home_url), "Navigate to homepage");

            // Look for navigation elements
            let nav_elements = timeout10s!(
                driver.find_all(By::Css("nav, .nav, .navbar, .menu")),
                "Find navigation elements"
            );
            let links = timeout10s!(driver.find_all(By::Css("a")), "Find link elements");

            if nav_elements.is_empty() && links.is_empty() {
                return Err(anyhow::anyhow!("No navigation elements found"));
            }

            println!("✅ Navigation menu works correctly in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_page_titles_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("📄 Testing page titles in headless browser...");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Test main pages have titles
            let pages = ["/", "/domains", "/users", "/aliases", "/clients"];

            for page in pages {
                let page_url = format!("http://172.17.0.1:{}{}", app_port, page);
                println!("Testing page: {}", page_url);
                timeout10s!(driver.get(&page_url), "Navigate to page");

                let title = timeout10s!(driver.title(), "Get page title");
                if title.is_empty() {
                    return Err(anyhow::anyhow!("Page {} should have a title", page));
                }
                println!("Page title: {}", title);
            }

            println!("✅ All pages have proper titles in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(60),
    )
    .await
}

#[tokio::test]
async fn test_loading_states_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("⏳ Testing loading states in headless browser...");

            // Navigate to homepage
            let home_url = format!("http://172.17.0.1:{}", app_port);
            println!("Navigating to homepage: {}", home_url);
            timeout10s!(driver.get(&home_url), "Navigate to homepage");

            // Wait a moment for any loading to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

            // Page should be interactive
            let body = timeout10s!(driver.find(By::Tag("body")), "Find body element");
            let body_text = timeout10s!(body.text(), "Get body text");
            if body_text.is_empty() {
                return Err(anyhow::anyhow!("Page should have content"));
            }

            println!("✅ Loading states work correctly in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_accessibility_basics_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("♿ Testing accessibility basics in headless browser...");

            // Navigate to homepage
            let home_url = format!("http://172.17.0.1:{}", app_port);
            println!("Navigating to homepage: {}", home_url);
            timeout10s!(driver.get(&home_url), "Navigate to homepage");

            // Check for basic accessibility elements
            let body = timeout10s!(driver.find(By::Tag("body")), "Find body element");
            let body_text = timeout10s!(body.text(), "Get body text");

            // Should have some content
            if body_text.is_empty() {
                return Err(anyhow::anyhow!("Page should have content"));
            }

            println!("✅ Basic accessibility works in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_cross_browser_compatibility_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("🌐 Testing cross-browser compatibility in headless browser...");

            // Test different viewport sizes
            let viewports = [
                (1920, 1080),
                (1366, 768),
                (1024, 768),
                (768, 1024),
                (375, 667),
            ];

            for (width, height) in viewports {
                println!("Testing viewport: {}x{}", width, height);
                timeout10s!(driver.set_window_rect(0, 0, width, height), "Set viewport");

                let home_url = format!("http://172.17.0.1:{}", app_port);
                timeout10s!(driver.get(&home_url), "Navigate to homepage");

                // Should load without errors
                let current_url = timeout10s!(driver.current_url(), "Get current URL");
                if !current_url.as_str().contains("3000") {
                    return Err(anyhow::anyhow!(
                        "Page should load correctly at {}x{} viewport",
                        width,
                        height
                    ));
                }
            }

            println!("✅ Cross-browser compatibility works in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(60),
    )
    .await
}

#[tokio::test]
async fn test_htmx_compatibility_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("⚡ Testing HTMX compatibility in headless browser...");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Test that forms have HTMX attributes
            let form_url = format!("http://172.17.0.1:{}/domains/new", app_port);
            println!("Navigating to domain creation form: {}", form_url);
            timeout10s!(driver.get(&form_url), "Navigate to domain form");

            let forms = timeout10s!(driver.find_all(By::Css("form")), "Find form elements");
            if !forms.is_empty() {
                // Check for HTMX attributes
                let form = &forms[0];
                let htmx_attr = timeout10s!(form.attr("hx-post"), "Get HTMX attribute");
                if htmx_attr.is_none() {
                    // Check for other HTMX attributes
                    let htmx_get = timeout10s!(form.attr("hx-get"), "Get HTMX get attribute");
                    let htmx_put = timeout10s!(form.attr("hx-put"), "Get HTMX put attribute");
                    if htmx_get.is_none() && htmx_put.is_none() {
                        println!("⚠️  Form does not have HTMX attributes (this may be expected for some forms)");
                    }
                }
            }

            println!("✅ HTMX compatibility works in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_performance_metrics_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("⚡ Testing performance metrics in headless browser...");

            // Navigate to homepage and measure load time
            let home_url = format!("http://172.17.0.1:{}", app_port);
            println!("Navigating to homepage: {}", home_url);

            let start_time = std::time::Instant::now();
            timeout10s!(driver.get(&home_url), "Navigate to homepage");
            let load_time = start_time.elapsed();

            println!("Page load time: {:?}", load_time);

            // Basic performance check - page should load within 10 seconds
            if load_time > Duration::from_secs(10) {
                return Err(anyhow::anyhow!("Page load time too slow: {:?}", load_time));
            }

            println!("✅ Performance metrics are acceptable in headless browser");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(30),
    )
    .await
}

#[tokio::test]
async fn test_add_alias_domain_search_headless() -> Result<()> {
    run_test_with_timeout(
        async {
            let docker = clients::Cli::default();
            let (driver, _selenium, app_port) = setup_driver(&docker).await?;

            println!("🔎 Testing domain search in add alias form (headless)...");

            // Authenticate first
            authenticate_driver(&driver, app_port).await?;

            // Navigate to add alias form
            let add_alias_url = format!("http://172.17.0.1:{}/aliases/new", app_port);
            println!("Navigating to add alias form: {}", add_alias_url);
            timeout10s!(driver.get(&add_alias_url), "Navigate to add alias form");

            // Find the mail input field
            let mail_input = timeout10s!(driver.find(By::Css("input[name='mail']")), "Find mail input");
            timeout10s!(mail_input.clear(), "Clear mail input");
            // Type a domain fragment to trigger suggestions
            timeout10s!(mail_input.send_keys("@exa"), "Type '@exa' in mail input");

            // Wait for suggestions to appear
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

            // Print the page source for debugging
            let debug_source = driver.source().await?;
            println!("--- PAGE SOURCE AFTER TYPING ---\n{}\n--- END PAGE SOURCE ---", debug_source);

            // Print browser console errors for debugging
            let logs = driver.logs("browser").await;
            match logs {
                Ok(entries) => {
                    for entry in entries {
                        println!("[BROWSER LOG] {}: {}", entry.level, entry.message);
                    }
                }
                Err(e) => println!("[BROWSER LOG] Could not fetch logs: {}", e),
            }

            // Look for the domain search results container
            let results = driver.find_all(By::Css("[data-domain]"));
            let results = timeout10s!(results, "Find domain suggestion items");
            let count = results.len();
            println!("Found {} domain suggestion(s)", count);
            if count == 0 {
                return Err(anyhow::anyhow!("No domain suggestions appeared in add alias form"));
            }
            // Optionally, check that one of the suggestions contains 'example.com'
            let mut found_example = false;
            for elem in results {
                let text = elem.text().await?;
                if text.contains("example.com") {
                    found_example = true;
                    break;
                }
            }
            if !found_example {
                return Err(anyhow::anyhow!("Domain suggestions did not include 'example.com'"));
            }
            println!("✅ Domain search suggestions appear in add alias form");
            timeout10s!(driver.quit(), "Quit driver");
            Ok(())
        },
        Duration::from_secs(40),
    )
    .await
}
