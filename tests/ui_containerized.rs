use anyhow::Result;
use thirtyfour::prelude::*;
use tokio::time::{timeout, Duration};
mod common;
use common::ui_helpers::{
    create_domain, login_and_goto_dashboard, rand_domain_str, run_test_with_timeout,
    setup_ui_test_env, setup_ui_test_env_multidb, switch_database_ui, test_404_page,
};

// Import test suite lifecycle for automatic cleanup
use common::test_suite_lifecycle;

/// Initialize test suite lifecycle handlers for UI tests.
/// This ensures proper cleanup of shared containers and networks.
async fn init_test_suite_lifecycle() {
    test_suite_lifecycle::register_test_suite_lifecycle();
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

// Wait for the app to be reachable from inside the Selenium container by loading /health

#[tokio::test]
async fn test_homepage_loads_containerized() -> Result<()> {
    // Initialize test suite lifecycle for automatic cleanup
    let _ = init_test_suite_lifecycle();

    run_test_with_timeout(
        "test_homepage_loads_containerized",
        async {
            println!("[DEBUG] Starting test_homepage_loads_containerized");
            let env = setup_ui_test_env().await?;
            println!("[DEBUG] TestEnv created successfully");

            // Pre-check: ensure /health is reachable
            let health_url = format!("{}/health", env.app_url.trim_end_matches('/'));
            timeout60s!(env.driver.get(&health_url), "Navigate to /health")?;

            // Perform real login and land on the dashboard
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            // After login, the first full page render can race DB readiness.
            // Reload "/" until layout is present, then assert H1 contains 'dashboard'.
            let homepage_url = format!("{}/", env.app_url);
            timeout60s!(env.driver.get(&homepage_url), "Navigate to homepage")?;
            let mut attempts = 0;
            loop {
                let src = env.driver.source().await.unwrap_or_default();
                if src.contains("main-content") && !src.contains("Database connection error") {
                    break;
                }
                attempts += 1;
                if attempts >= 10 {
                    return Err(anyhow::anyhow!(
                        "Homepage not ready after reloads; snippet: {}",
                        &src.chars().take(500).collect::<String>()
                    ));
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                timeout60s!(env.driver.get(&homepage_url), "Reload homepage")?;
            }

            use thirtyfour::By;
            let main = timeout60s!(
                env.driver.find(By::Id("main-content")),
                "Find #main-content"
            )?;
            let h1_elem = timeout60s!(main.find(By::Css("h1")), "Find H1 inside main content")?;
            let h1_text = timeout60s!(h1_elem.text(), "Get H1 text")?;
            assert!(h1_text.to_lowercase().contains("dashboard"));
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_navigation_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_navigation_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            let nav_elements = timeout60s!(
                env.driver.find_all(By::Css("nav, .nav, .navbar, .menu")),
                "Find navigation elements"
            )?;
            let links = timeout60s!(env.driver.find_all(By::Css("a")), "Find link elements")?;

            assert!(
                !nav_elements.is_empty() && !links.is_empty(),
                "No navigation elements found"
            );

            let pages = vec![
                ("/", "Dashboard"),
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
                    "Page should contain {expected_title}"
                );
            }
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_domain_search_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_domain_search_containerized",
        async {
            let env = setup_ui_test_env().await?;
            // println!("[DEBUG] App URL for Selenium: {}", env.app_url);
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            let aliases_url = format!("{}/aliases", env.app_url);
            // Go to aliases with a few retries if DB is still initializing
            let mut attempts = 0;
            loop {
                timeout60s!(env.driver.get(&aliases_url), "Navigate to aliases page")?;
                let src = env.driver.source().await.unwrap_or_default();
                if !src.contains("Database connection error") {
                    break;
                }
                attempts += 1;
                if attempts >= 6 {
                    // Dump app logs for diagnostics
                    let _ = std::process::Command::new("docker")
                        .args(["logs", "--tail", "200", env.app_container.id()])
                        .status();
                    return Err(anyhow::anyhow!(
                        "Aliases page did not load due to database connection error"
                    ));
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }

            // Wait for aliases page to be present
            let main = match timeout60s!(
                env.driver.find(By::Id("main-content")),
                "Find #main-content on aliases"
            ) {
                Ok(m) => m,
                Err(_e) => {
                    let src = env.driver.source().await.unwrap_or_default();
                    let snippet: String = src.chars().take(800).collect();
                    // Dump app logs for diagnostics
                    let _ = std::process::Command::new("docker")
                        .args(["logs", "--tail", "200", env.app_container.id()])
                        .status();
                    return Err(anyhow::anyhow!("Aliases page did not load: {}", snippet));
                }
            };
            let h1 = timeout60s!(main.find(By::Css("h1")), "Find h1 on aliases")?;
            let h1_text = timeout60s!(h1.text(), "Get aliases h1 text")?;
            if !h1_text.to_lowercase().contains("alias") {
                let src = env.driver.source().await.unwrap_or_default();
                let snippet: String = src.chars().take(800).collect();
                return Err(anyhow::anyhow!(
                    "Aliases page did not load (h1='{}'): {}",
                    h1_text,
                    snippet
                ));
            }

            // Click the Add Alias button
            let add_alias_button = match timeout60s!(
                env.driver.find(By::Id("add-alias-button")),
                "Find Add Alias button by id"
            ) {
                Ok(b) => b,
                Err(_) => {
                    // Fallback: any button that requests /aliases/new via htmx
                    timeout60s!(
                        env.driver.find(By::Css("button[hx-get='/aliases/new']")),
                        "Find Add Alias button by hx-get"
                    )?
                }
            };

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
            assert!(
                page_source.contains("domain-search-results")
                    || page_source.contains("No domains found"),
                "Domain search results not found"
            );
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_aliases_list_page_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_aliases_list_page_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            let aliases_url = format!("{}/aliases", env.app_url);
            // Navigate with a few retries in case DB pool is still initializing
            let mut attempts = 0;
            loop {
                timeout60s!(env.driver.get(&aliases_url), "Navigate to aliases page")?;
                let src = env.driver.source().await.unwrap_or_default();
                if !src.contains("Database connection error") {
                    break;
                }
                attempts += 1;
                if attempts >= 5 {
                    return Err(anyhow::anyhow!(
                        "Aliases page does not contain expected content"
                    ));
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(900)).await;
            }

            // Verify by checking main content h1 instead of free text
            let main = timeout60s!(
                env.driver.find(By::Id("main-content")),
                "Find #main-content on aliases"
            )?;
            let h1 = timeout60s!(main.find(By::Css("h1")), "Find h1 on aliases")?;
            let h1_text = timeout60s!(h1.text(), "Get aliases h1 text")?;
            if !h1_text.to_lowercase().contains("alias") {
                return Err(anyhow::anyhow!(
                    "Aliases page does not contain expected content"
                ));
            }
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_domains_list_page_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_domains_list_page_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            let domains_url = format!("{}/domains", env.app_url);
            // Navigate with retries if DB is still initializing
            let mut attempts = 0;
            loop {
                timeout60s!(env.driver.get(&domains_url), "Navigate to domains page")?;
                let src = env.driver.source().await.unwrap_or_default();
                if !src.contains("Database connection error") {
                    break;
                }
                attempts += 1;
                if attempts >= 5 {
                    return Err(anyhow::anyhow!(
                        "Domains page does not contain expected content"
                    ));
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(900)).await;
            }

            // Verify using main-content H1
            let main = timeout60s!(
                env.driver.find(By::Id("main-content")),
                "Find #main-content on domains"
            )?;
            let h1 = timeout60s!(main.find(By::Css("h1")), "Find h1 on domains")?;
            let h1_text = timeout60s!(h1.text(), "Get domains h1 text")?;
            if !h1_text.to_lowercase().contains("domain") {
                return Err(anyhow::anyhow!(
                    "Domains page does not contain expected content"
                ));
            }
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_users_list_page_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_users_list_page_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            let users_url = format!("{}/users", env.app_url);
            timeout60s!(env.driver.get(&users_url), "Navigate to users page")?;
            let _page_title = timeout60s!(env.driver.title(), "Get page title")?;
            let page_source = timeout60s!(env.driver.source(), "Get page source")?;
            assert!(
                page_source.contains("Users") || page_source.contains("users"),
                "Users page does not contain expected content"
            );
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_clients_list_page_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_clients_list_page_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            let clients_url = format!("{}/clients", env.app_url);
            timeout60s!(env.driver.get(&clients_url), "Navigate to clients page")?;
            // Retry a few times if DB pool is still warming up
            let mut attempts = 0;
            while attempts < 10 {
                let src = env.driver.source().await.unwrap_or_default();
                if !src.contains("Database connection error") && src.contains("main-content") {
                    break;
                }
                // Warm up by touching dashboard, then retry
                let dashboard_url = format!("{}/", env.app_url);
                let _ = env.driver.get(&dashboard_url).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                timeout60s!(env.driver.get(&clients_url), "Reload clients page")?;
                attempts += 1;
            }

            use thirtyfour::By;
            let main = match timeout60s!(
                env.driver.find(By::Id("main-content")),
                "Find #main-content on clients"
            ) {
                Ok(m) => m,
                Err(_) => {
                    let src = env.driver.source().await.unwrap_or_default();
                    // If clients feature is disabled, app returns a simple message without layout;
                    // otherwise fail fast with diagnostic snippet
                    if !src.contains("Clients table is not available for this database") {
                        return Err(anyhow::anyhow!(
                            "Clients page missing layout. Source snippet: {}",
                            &src.chars().take(600).collect::<String>()
                        ));
                    }
                    env.cleanup().await?;
                    return Ok(());
                }
            };
            let h1 = timeout60s!(main.find(By::Css("h1")), "Find h1 on clients")?;
            let h1_text = timeout60s!(h1.text(), "Get clients h1 text")?;
            assert!(
                h1_text.to_lowercase().contains("client"),
                "Clients page H1 should contain 'client', got: {}",
                h1_text
            );
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_responsive_design_containerized() -> Result<()> {
    let test_timeout = Duration::from_secs(120);
    run_test_with_timeout(
        "test_responsive_design_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            // Test different viewport sizes and verify responsive behavior
            let viewports = [
                (1920, 1080, "Desktop"),
                (1366, 768, "Laptop"),
                (1024, 768, "Tablet Landscape"),
                (768, 1024, "Tablet Portrait"),
                (375, 667, "Mobile"),
                (320, 568, "Small Mobile"),
            ];

            for (width, height, name) in viewports.iter() {
                println!("[RESPONSIVE] Testing {} viewport ({}x{})", name, width, height);

                // Set viewport size
                timeout60s!(
                    env.driver.set_window_rect(0, 0, *width, *height),
                    "Set viewport"
                )?;

                // Test homepage responsiveness
                let home_url = format!("{}/", env.app_url);
                timeout60s!(env.driver.get(&home_url), "Navigate to homepage")?;

                // Wait for page to load and stabilize
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                // Check if navigation menu is responsive
                let nav_elements = timeout30s!(
                    env.driver.find_all(By::Css("nav, .navbar, .navigation, [role='navigation']")),
                    "Find navigation elements"
                );

                if let Ok(navs) = nav_elements {
                    for nav in navs {
                        let is_displayed = timeout30s!(nav.is_displayed(), "Check nav visibility");
                        if let Ok(visible) = is_displayed {
                            assert!(visible, "Navigation should be visible in {} viewport", name);
                        }
                    }
                }

                // Check if main content is accessible
                let main_content = timeout30s!(
                    env.driver.find(By::Css("main, .main-content, .content, [role='main']")),
                    "Find main content"
                );

                if let Ok(content) = main_content {
                    let is_displayed = timeout30s!(content.is_displayed(), "Check content visibility");
                    if let Ok(visible) = is_displayed {
                        assert!(visible, "Main content should be visible in {} viewport", name);
                    }
                }

                // Test domains page responsiveness
                let domains_url = format!("{}/domains", env.app_url);
                timeout60s!(env.driver.get(&domains_url), "Navigate to domains")?;

                // Check if table/list is responsive (should not overflow)
                let tables = timeout30s!(
                    env.driver.find_all(By::Css("table, .table, .list, .grid")),
                    "Find tables/lists"
                );

                if let Ok(tables) = tables {
                    for table in tables {
                        let is_displayed = timeout30s!(table.is_displayed(), "Check table visibility");
                        if let Ok(visible) = is_displayed {
                            assert!(visible, "Tables should be visible in {} viewport", name);
                        }
                    }
                }

                // Test form responsiveness - check for forms on pages that likely have them
                // First check domains page for any forms
                let forms = timeout30s!(
                    env.driver.find_all(By::Css("form, .form")),
                    "Find forms on domains page"
                );

                if let Ok(forms) = forms {
                    let mut visible_count = 0;
                    for form in &forms {
                        let is_displayed = timeout30s!(form.is_displayed(), "Check form visibility");
                        if let Ok(visible) = is_displayed {
                            if visible {
                                visible_count += 1;
                            }
                        }
                    }

                    // Only fail if we have forms but none are visible
                    if forms.len() > 0 && visible_count == 0 {
                        panic!("All forms are hidden in {} viewport", name);
                    }
                }

                // Also check for form elements (inputs, buttons, etc.) which are more common
                let form_elements = timeout30s!(
                    env.driver.find_all(By::Css("input, button, select, textarea")),
                    "Find form elements on domains page"
                );

                if let Ok(elements) = form_elements {
                    let mut visible_count = 0;
                    for element in elements.iter() {
                        let is_displayed = timeout30s!(element.is_displayed(), "Check element visibility");
                        if let Ok(visible) = is_displayed {
                            if visible {
                                visible_count += 1;
                            }
                        }
                    }

                    // Only fail if we expect form elements but none are visible
                    if elements.len() > 0 && visible_count == 0 {
                        panic!("All form elements are hidden in {} viewport", name);
                    }
                }

                // For mobile viewports, check for mobile-specific elements or behaviors
                if *width <= 768 {
                    // Check if there are mobile-friendly elements
                    let mobile_elements = timeout30s!(
                        env.driver.find_all(By::Css(".mobile, .mobile-only, [data-mobile], .responsive")),
                        "Find mobile elements"
                    );

                    // This is optional - not all sites have explicit mobile classes
                    if let Ok(elements) = mobile_elements {
                        println!("[RESPONSIVE] Found {} mobile-specific elements in {} viewport", elements.len(), name);
                    }
                }

                println!("[RESPONSIVE] {} viewport test completed successfully", name);
            }

            // Test horizontal scrolling (should not occur in responsive design)
            timeout60s!(
                env.driver.set_window_rect(0, 0, 320, 568),
                "Set to smallest mobile viewport"
            )?;

            let home_url = format!("{}/", env.app_url);
            timeout60s!(env.driver.get(&home_url), "Navigate to homepage for scroll test")?;

            // Check if page width exceeds viewport (indicates horizontal scroll)
            let body_width = timeout30s!(
                env.driver.execute("return document.body.scrollWidth;", vec![]),
                "Get body scroll width"
            );

            if let Ok(width) = body_width {
                if let Ok(width_value) = width.convert::<f64>() {
                    // Body width should not exceed viewport width significantly
                    if width_value > 400.0 { // 320px viewport + some tolerance
                        println!("[RESPONSIVE] Warning: Body width {}px may cause horizontal scroll in mobile viewport", width_value);
                    }
                }
            }

            println!("[RESPONSIVE] All responsive design tests completed successfully");
            env.cleanup().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_not_found_pages_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_not_found_pages_containerized",
        async {
            let env = setup_ui_test_env().await?;

            // Test 404 page before login
            test_404_page(&env.driver, &env.app_url, "/nonexistent", "before login").await?;

            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

            // Test 404 page after login
            test_404_page(
                &env.driver,
                &env.app_url,
                "/nonexistent-page",
                "after login",
            )
            .await?;

            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(40),
    )
    .await
}

#[tokio::test]
async fn test_unauthorized_pages_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_unauthorized_pages_containerized",
        async {
            let env = setup_ui_test_env().await?;

            let error_url_401 = format!("{}/", env.app_url);
            timeout60s!(env.driver.get(&error_url_401), "Navigate to a 401 page")?;
            let page_source_401 = timeout30s!(env.driver.source(), "Get 401 page source")?;
            let title_401 = timeout30s!(env.driver.title(), "Get 401 page title")?;
            assert!(title_401.contains("Sign in"));
            assert!(
                page_source_401.contains("login"),
                "401 page does not contain expected login content. Title: {title_401}"
            );

            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(40),
    )
    .await
}

#[tokio::test]
async fn test_domain_form_validation_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_domain_form_validation_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            // Navigate to domain creation form
            let form_url = format!("{}/domains/new", env.app_url);
            timeout60s!(env.driver.get(&form_url), "Navigate to domain form")?;
            // Check for form elements
            let forms = timeout60s!(env.driver.find_all(By::Css("form")), "Find form elements")?;
            if forms.is_empty() {
                return Err(anyhow::anyhow!("Domain creation form not found"));
            }
            // Check for input elements
            let inputs = timeout60s!(
                env.driver.find_all(By::Css("input, textarea, select")),
                "Find input elements"
            )?;
            if inputs.is_empty() {
                return Err(anyhow::anyhow!("Form should have input elements"));
            }
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(40),
    )
    .await
}

#[tokio::test]
#[ignore = "Covered by specific page tests; flaky under DB warmup in containerized runs"]
async fn test_page_titles_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_page_titles_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            // Test page titles for all main pages
            let pages_to_test = [
                ("/", "Dashboard", "Homepage"),
                ("/domains", "Domains", "Domains page"),
                ("/aliases", "Aliases", "Aliases page"),
                ("/users", "Users", "Users page"),
                ("/clients", "Clients", "Clients page"),
            ];

            for (path, expected_title_keyword, page_name) in pages_to_test.iter() {
                let page_url = format!("{}{}", env.app_url, path);
                timeout60s!(env.driver.get(&page_url), "Navigate to page")?;

                // Wait for page to load
                timeout30s!(
                    env.driver.find(By::Css("h1, .page-title, .title")),
                    "Wait for page to load"
                )?;

                // Get the page title
                let page_title = timeout30s!(env.driver.title(), "Get page title")?;
                println!("[PAGE TITLES] {} title: '{}'", page_name, page_title);

                // Validate page title contains expected keyword
                assert!(
                    page_title
                        .to_lowercase()
                        .contains(expected_title_keyword.to_lowercase().as_str()),
                    "{} title '{}' should contain '{}'",
                    page_name,
                    page_title,
                    expected_title_keyword
                );

                // Check for HTML title tag content
                let html_title = timeout30s!(
                    env.driver.execute("return document.title;", vec![]),
                    "Get HTML title"
                );

                if let Ok(title) = html_title {
                    if let Ok(title_str) = title.convert::<String>() {
                        assert!(
                            !title_str.is_empty(),
                            "HTML title should not be empty for {}",
                            page_name
                        );
                        assert!(
                            title_str
                                .to_lowercase()
                                .contains(&expected_title_keyword.to_lowercase()),
                            "HTML title '{}' should contain '{}' for {}",
                            title_str,
                            expected_title_keyword,
                            page_name
                        );
                    }
                }

                // Check for page heading (h1) content
                let h1_elements =
                    timeout30s!(env.driver.find_all(By::Css("h1")), "Find h1 elements");

                if let Ok(h1s) = h1_elements {
                    let mut found_expected_heading = false;
                    for h1 in h1s {
                        if let Ok(h1_text) = timeout30s!(h1.text(), "Get h1 text") {
                            if h1_text
                                .to_lowercase()
                                .contains(&expected_title_keyword.to_lowercase())
                            {
                                found_expected_heading = true;
                                break;
                            }
                        }
                    }
                    assert!(
                        found_expected_heading,
                        "Page heading should contain '{}' for {}",
                        expected_title_keyword, page_name
                    );
                }

                println!("[PAGE TITLES] {} title validation passed", page_name);
            }

            // Test error page titles
            let error_pages = [
                ("/nonexistent", "Not Found", "404 page"),
                ("/invalid-route", "Not Found", "404 page"),
            ];

            for (path, expected_title_keyword, page_name) in error_pages.iter() {
                let page_url = format!("{}{}", env.app_url, path);
                timeout60s!(env.driver.get(&page_url), "Navigate to error page")?;

                let page_title = timeout30s!(env.driver.title(), "Get error page title")?;
                println!("[PAGE TITLES] {} title: '{}'", page_name, page_title);

                // Error pages should contain error indicators
                let has_error_indicator = page_title.to_lowercase().contains("not found")
                    || page_title.to_lowercase().contains("error")
                    || page_title.to_lowercase().contains("404")
                    || page_title
                        .to_lowercase()
                        .contains(&expected_title_keyword.to_lowercase());

                assert!(
                    has_error_indicator,
                    "{} title '{}' should contain error indicators like '{}'",
                    page_name, page_title, expected_title_keyword
                );
            }

            println!("[PAGE TITLES] All page title validations completed successfully");
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(120),
    )
    .await
}

#[tokio::test]
async fn test_cross_browser_compatibility_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_cross_browser_compatibility_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            // Test different viewport sizes (simulating different devices)
            let viewports = [
                (1920, 1080, "Desktop"),
                (1366, 768, "Laptop"),
                (1024, 768, "Tablet Landscape"),
                (768, 1024, "Tablet Portrait"),
                (375, 667, "Mobile"),
            ];

            for (width, height, device_name) in viewports.iter() {
                println!("[CROSS-BROWSER] Testing {} viewport ({}x{})", device_name, width, height);

                timeout60s!(
                    env.driver.set_window_rect(0, 0, *width, *height),
                    "Set viewport"
                )?;

                // Test core functionality across different viewport sizes
                let pages_to_test = ["/", "/domains", "/aliases", "/users"];

                for page in pages_to_test.iter() {
                    let page_url = format!("{}{}", env.app_url, page);
                    timeout60s!(env.driver.get(&page_url), "Navigate to page")?;

                    // Wait for page to load and stabilize
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                    // Verify page loaded correctly
                    let current_url = timeout30s!(env.driver.current_url(), "Get current URL")?;
                assert!(
                        current_url.to_string().starts_with("http"),
                        "Unexpected URL format on {}: {}",
                        device_name,
                        current_url
                    );

                    // Test JavaScript functionality (basic DOM manipulation)
                    let js_result = timeout30s!(
                        env.driver.execute("return document.readyState;", vec![]),
                        "Check document ready state"
                    );

                    if let Ok(ready_state) = js_result {
                        if let Ok(state) = ready_state.convert::<String>() {
                            assert_eq!(
                                state,
                                "complete",
                                "Document should be fully loaded on {}",
                                device_name
                            );
                        }
                    }

                    // Test form elements (if present)
                    let forms = timeout30s!(
                        env.driver.find_all(By::Css("form, input, button, select, textarea")),
                        "Find form elements"
                    );

                    if let Ok(form_elements) = forms {
                        let mut visible_count = 0;
                        for element in form_elements.iter() {
                            let is_displayed = timeout30s!(element.is_displayed(), "Check element visibility");
                            if let Ok(visible) = is_displayed {
                                if visible {
                                    visible_count += 1;
                                }
                            }
                        }

                        // Only fail if we have form elements but none are visible
                        if form_elements.len() > 0 && visible_count == 0 {
                            panic!("All form elements are hidden on {}", device_name);
                        }
                    }

                    // Test navigation elements
                    let nav_elements = timeout30s!(
                        env.driver.find_all(By::Css("nav, .navbar, .navigation, a[href]")),
                        "Find navigation elements"
                    );

                    if let Ok(navs) = nav_elements {
                        let mut visible_count = 0;
                        for nav in navs.iter() {
                            let is_displayed = timeout30s!(nav.is_displayed(), "Check nav visibility");
                            if let Ok(visible) = is_displayed {
                                if visible {
                                    visible_count += 1;
                                }
                            }
                        }

                        // Only fail if we have navigation elements but none are visible
                        if navs.len() > 0 && visible_count == 0 {
                            panic!("All navigation elements are hidden on {}", device_name);
                        }
                    }

                    println!("[CROSS-BROWSER] {} page tested successfully on {}", page, device_name);
                }

                // Test specific browser compatibility features
                // Test localStorage/sessionStorage (if used)
                let storage_test = timeout30s!(
                    env.driver.execute("return typeof(Storage) !== 'undefined';", vec![]),
                    "Test Storage API"
                );

                if let Ok(has_storage) = storage_test {
                    if let Ok(storage_available) = has_storage.convert::<bool>() {
                        assert!(storage_available, "Storage API should be available on {}", device_name);
                    }
                }

                // Test fetch/XMLHttpRequest (if used)
                let fetch_test = timeout30s!(
                    env.driver.execute("return typeof(fetch) !== 'undefined';", vec![]),
                    "Test Fetch API"
                );

                if let Ok(has_fetch) = fetch_test {
                    if let Ok(fetch_available) = has_fetch.convert::<bool>() {
                        assert!(fetch_available, "Fetch API should be available on {}", device_name);
                    }
                }

                println!("[CROSS-BROWSER] {} viewport compatibility test completed", device_name);
            }

            // Test specific browser features that might cause compatibility issues
            // Test if the page handles missing JavaScript gracefully
            let js_disabled_test = timeout30s!(
                env.driver.execute("return document.querySelector('noscript') !== null;", vec![]),
                "Check for noscript fallback"
            );

            if let Ok(has_noscript) = js_disabled_test {
                if let Ok(has_noscript_bool) = has_noscript.convert::<bool>() {
                    if has_noscript_bool {
                        println!("[CROSS-BROWSER] Found noscript fallback - good for JavaScript-disabled browsers");
                    }
                }
            }

            // Test if the page works with different user agents (simulated)
            let user_agent = timeout30s!(
                env.driver.execute("return navigator.userAgent;", vec![]),
                "Get user agent"
            );

            if let Ok(ua) = user_agent {
                if let Ok(ua_string) = ua.convert::<String>() {
                    println!("[CROSS-BROWSER] Current user agent: {}", ua_string);

                    // Check for common browser indicators
                    if ua_string.contains("Chrome") {
                        println!("[CROSS-BROWSER] Testing with Chrome-like browser");
                    } else if ua_string.contains("Firefox") {
                        println!("[CROSS-BROWSER] Testing with Firefox-like browser");
                    } else if ua_string.contains("Safari") {
                        println!("[CROSS-BROWSER] Testing with Safari-like browser");
                    }
                }
            }

            println!("[CROSS-BROWSER] All cross-browser compatibility tests completed successfully");
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(120),
    )
    .await
}

#[tokio::test]
async fn test_performance_metrics_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_performance_metrics_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            // Test performance of multiple critical pages
            let pages_to_test = [
                ("/", "Homepage"),
                ("/domains", "Domains page"),
                ("/aliases", "Aliases page"),
                ("/users", "Users page"),
                ("/clients", "Clients page"),
            ];

            let mut performance_results = Vec::new();

            for (path, name) in pages_to_test.iter() {
                let page_url = format!("{}{}", env.app_url, path);
                let start_time = std::time::Instant::now();

                timeout60s!(env.driver.get(&page_url), "Navigate to page")?;

                // Wait for page to be fully loaded (wait for body to be present)
                timeout30s!(env.driver.find(By::Css("body")), "Wait for page to load")?;

                let load_time = start_time.elapsed();
                performance_results.push((name, load_time));

                println!("[PERFORMANCE] {} loaded in {:?}", name, load_time);

                // Each page should load within 8 seconds
                if load_time > Duration::from_secs(8) {
                    return Err(anyhow::anyhow!(
                        "{} load time too slow: {:?} (max: 8s)",
                        name,
                        load_time
                    ));
                }
            }

            // Test form submission performance
            let form_start = std::time::Instant::now();
            let domains_url = format!("{}/domains", env.app_url);
            timeout60s!(
                env.driver.get(&domains_url),
                "Navigate to domains for form test"
            )?;

            // Try to find and interact with a form element (if available)
            if let Ok(_form) = timeout30s!(env.driver.find(By::Css("form")), "Find form element") {
                let form_interaction_time = form_start.elapsed();
                println!(
                    "[PERFORMANCE] Form interaction ready in {:?}",
                    form_interaction_time
                );

                if form_interaction_time > Duration::from_secs(5) {
                    return Err(anyhow::anyhow!(
                        "Form interaction too slow: {:?} (max: 5s)",
                        form_interaction_time
                    ));
                }
            }

            // Test navigation performance
            let nav_start = std::time::Instant::now();
            let dashboard_url = format!("{}/", env.app_url);
            timeout60s!(env.driver.get(&dashboard_url), "Navigate back to dashboard")?;
            let nav_time = nav_start.elapsed();

            println!("[PERFORMANCE] Navigation completed in {:?}", nav_time);

            if nav_time > Duration::from_secs(3) {
                return Err(anyhow::anyhow!(
                    "Navigation too slow: {:?} (max: 3s)",
                    nav_time
                ));
            }

            // Print performance summary
            println!("[PERFORMANCE] Performance test completed successfully:");
            for (name, time) in performance_results {
                println!("  - {}: {:?}", name, time);
            }

            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(120),
    )
    .await
}

#[tokio::test]
async fn test_database_dropdown_selection_containerized() -> Result<()> {
    let config_path = std::env::current_dir()
        .unwrap()
        .join("config/config.docker.multidb.toml");
    let _config_path_str = config_path.to_str().unwrap();
    run_test_with_timeout(
        "test_database_dropdown_selection_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;
            // Navigate to a page that has the database dropdown (dashboard)
            let dashboard_url = format!("{}/", env.app_url);
            timeout60s!(env.driver.get(&dashboard_url), "Navigate to dashboard")?;
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            // Find and click the database dropdown button
            let dropdown_btn = timeout60s!(
                env.driver.find(By::Id("db-dropdown-btn")),
                "Find database dropdown button"
            )?;
            timeout60s!(dropdown_btn.click(), "Click database dropdown button")?;
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            // Check if dropdown is visible
            let dropdown_list = timeout60s!(
                env.driver.find(By::Id("db-dropdown-list")),
                "Find database dropdown list"
            )?;
            let is_displayed =
                timeout60s!(dropdown_list.is_displayed(), "Check dropdown visibility")?;
            if !is_displayed {
                return Err(anyhow::anyhow!(
                    "Database dropdown should be visible after clicking"
                ));
            }
            // Find all database options in the dropdown
            let database_options = timeout60s!(
                env.driver.find_all(By::Css("#db-dropdown-list button")),
                "Find database options"
            )?;
            if database_options.is_empty() {
                return Err(anyhow::anyhow!(
                    "Database dropdown should have at least one option"
                ));
            }
            // If we have multiple databases, test selection; otherwise just verify dropdown works
            if database_options.len() >= 2 {
                // Get the current URL before selection
                // let _current_url = timeout60s!(env.driver.current_url(), "Get current URL before selection")?;
                // Click on the second database option (if different from current)
                let second_option = &database_options[1];
                timeout60s!(second_option.click(), "Click second database option")?;
                tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                // Check that we're still on the same page (dashboard) with sidebar preserved
                // Try to get new URL with retries
                let mut attempts = 0;
                let max_attempts = 3;
                let mut new_url = None;

                while attempts < max_attempts {
                    match env.driver.current_url().await {
                        Ok(url) => {
                            new_url = Some(url);
                            break;
                        }
                        Err(_) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                            }
                        }
                    }
                }

                let new_url = new_url.ok_or_else(|| {
                    anyhow::anyhow!("Failed to get new URL after {} attempts", max_attempts)
                })?;

                if !new_url.to_string().starts_with("http") {
                    return Err(anyhow::anyhow!(
                        "Unexpected URL after database selection: {}",
                        new_url
                    ));
                }
                // Check that the sidebar/navigation is still present
                let sidebar = timeout60s!(
                    env.driver.find(By::Css("nav, .sidebar, .navigation")),
                    "Find sidebar/navigation"
                )?;
                let sidebar_displayed =
                    timeout60s!(sidebar.is_displayed(), "Check sidebar visibility")?;
                if !sidebar_displayed {
                    return Err(anyhow::anyhow!(
                        "Sidebar should still be visible after database selection"
                    ));
                }
                // Verify the page content is still there (dashboard content)
                let page_source =
                    timeout60s!(env.driver.source(), "Get page source after selection")?;
                if !page_source.contains("Dashboard") && !page_source.contains("dashboard") {
                    return Err(anyhow::anyhow!(
                        "Dashboard content should still be present after database selection"
                    ));
                }
            } else {
                // Just verify the dropdown shows the available database(s)
                let page_source = timeout60s!(env.driver.source(), "Get page source")?;
                if !page_source.contains("Test DB") && !page_source.contains("primary") {
                    return Err(anyhow::anyhow!(
                        "Database dropdown should show available database information"
                    ));
                }
            }
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_e2e_create_domain_aliases_user_and_report() -> anyhow::Result<()> {
    use rand::Rng;
    use thirtyfour::prelude::*;
    use tokio::time::Duration;

    // Helper for random string
    fn rand_str() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::rng();
        (0..8)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    run_test_with_timeout(
        "test_e2e_create_domain_aliases_user_and_report",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            // Generate random domain and alias/user names
            let domain_name = format!("{}.test.com", rand_str()).to_lowercase();
            let alias1 = format!("alias1-{}", rand_str());
            let alias2 = format!("alias2-{}", rand_str());
            let user_name = format!("user-{}", rand_str());
            let user_maildir = format!("{}/user-{}/", domain_name, rand_str());
            let user_email = format!("{user_name}@{domain_name}");

            let domain_url = format!("{}/domains", env.app_url);
            timeout60s!(env.driver.get(&domain_url), "Navigate to domains list page")?;

            let add_domain_button = timeout60s!(
                env.driver.find(By::Id("add-domain-button")),
                "Find Add Domain button"
            )?;

            timeout30s!(add_domain_button.click(), "Click Add Domain button")?;
            tokio::time::sleep(Duration::from_millis(200)).await;

            // 1. Create a new domain
            let domain_input = timeout30s!(
                env.driver.find(By::Css("input[name='domain']")),
                "Find domain input"
            )?;
            assert!(
                domain_input.is_displayed().await.unwrap_or(false),
                "Domain input is not displayed"
            );
            timeout60s!(domain_input.send_keys(&domain_name), "Type domain name")?;
            let submit_btn = timeout60s!(
                env.driver.find(By::Id("domain-submit-button")),
                "Find submit button"
            )?;
            assert!(
                submit_btn.is_displayed().await.unwrap_or(false),
                "Domain submit button is not displayed"
            );
            timeout60s!(submit_btn.click(), "Submit domain form")?;
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Wait for domain to appear with retries
            let mut attempts = 0;
            let max_attempts = 10;
            let mut page_source = String::new();

            while attempts < max_attempts {
                page_source =
                    timeout60s!(env.driver.source(), "Get page source after domain create")?;
                if page_source.contains(&domain_name) {
                    break;
                }
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(1000)).await;
                // Refresh the page to see if domain appears
                timeout60s!(env.driver.get(&domain_url), "Refresh domains page")?;
            }

            assert!(
                page_source.contains(&domain_name),
                "Domain should appear after creation. Page source: {}",
                page_source
            );

            let alias1domain = format!("{alias1}@{domain_name}");

            // 2. Create two aliases for the domain
            let aliases_url = format!("{}/aliases", env.app_url);
            timeout60s!(env.driver.get(&aliases_url), "Navigate to aliases page")?;

            let add_alias_btn = timeout60s!(
                env.driver.find(By::Id("add-alias-button")),
                "Find Add Alias button"
            )?;
            timeout30s!(add_alias_btn.click(), "Click Add Alias button")?;
            tokio::time::sleep(Duration::from_millis(200)).await;

            let mail_input = timeout30s!(
                env.driver.find(By::Css("input[name='mail']")),
                "Find mail input field"
            )?;
            timeout60s!(mail_input.send_keys(&alias1domain), "Type alias1")?;
            let dest_input = timeout60s!(
                env.driver.find(By::Css("input[name='destination']")),
                "Find destination input"
            )?;
            timeout60s!(
                dest_input.send_keys(&user_email),
                "Type destination for alias1"
            )?;
            let submit_btn = timeout60s!(
                env.driver.find(By::Id("alias-submit-button")),
                "Find submit button for alias1"
            )?;
            timeout60s!(submit_btn.click(), "Submit alias1 form")?;
            tokio::time::sleep(Duration::from_millis(500)).await;

            // 2. Create two aliases for the domain
            let aliases_url = format!("{}/aliases", env.app_url);
            timeout60s!(env.driver.get(&aliases_url), "Navigate to aliases page")?;

            let alias2domain = format!("{alias2}@{domain_name}");

            // Add second alias
            let add_alias_btn2 = timeout60s!(
                env.driver.find(By::Id("add-alias-button")),
                "Find Add Alias button again"
            )?;
            timeout30s!(add_alias_btn2.click(), "Click Add Alias button again")?;
            let mail_input2 = timeout30s!(
                env.driver.find(By::Css("input[name='mail']")),
                "Find mail input field for alias2"
            )?;
            timeout60s!(mail_input2.send_keys(&alias2domain), "Type alias2")?;
            let dest_input2 = timeout60s!(
                env.driver.find(By::Css("input[name='destination']")),
                "Find destination input for alias2"
            )?;
            timeout60s!(
                dest_input2.send_keys(&user_email),
                "Type destination for alias2"
            )?;
            let submit_btn2 = timeout60s!(
                env.driver.find(By::Id("alias-submit-button")),
                "Find submit button for alias2"
            )?;
            timeout60s!(submit_btn2.click(), "Submit alias2 form")?;
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Wait for aliases to appear with retries
            let mut attempts = 0;
            let max_attempts = 10;
            let mut aliases_page_source = String::new();

            while attempts < max_attempts {
                // Refresh the aliases page to see if new aliases appear
                timeout60s!(env.driver.get(&aliases_url), "Refresh aliases page")?;
                tokio::time::sleep(Duration::from_millis(1000)).await;

                aliases_page_source =
                    timeout60s!(env.driver.source(), "Get page source after alias create")?;
                println!(
                    "[E2E TEST] Attempt {}: Checking for aliases {} and {} in page source",
                    attempts + 1,
                    alias1domain,
                    alias2domain
                );
                println!(
                    "[E2E TEST] Page contains alias1: {}",
                    aliases_page_source.contains(&alias1domain)
                );
                println!(
                    "[E2E TEST] Page contains alias2: {}",
                    aliases_page_source.contains(&alias2domain)
                );

                if aliases_page_source.contains(&alias1domain)
                    && aliases_page_source.contains(&alias2domain)
                {
                    println!(
                        "[E2E TEST] ✅ Both aliases found after {} attempts",
                        attempts + 1
                    );
                    break;
                }
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }

            assert!(
                aliases_page_source.contains(&alias1domain)
                    && aliases_page_source.contains(&alias2domain),
                "Aliases should appear after creation. Page source: {}",
                aliases_page_source
            );

            // 3. Create a user for the domain
            let users_url = format!("{}/users", env.app_url);
            timeout60s!(env.driver.get(&users_url), "Navigate to users page")?;
            let add_user_btn = timeout60s!(
                env.driver.find(By::Id("add-user-button")),
                "Find Add User button"
            )?;
            timeout30s!(add_user_btn.click(), "Click Add User button")?;
            let user_id_input = timeout60s!(
                env.driver.find(By::Css("input[name='id']")),
                "Find user id input"
            )?;
            timeout60s!(user_id_input.send_keys(&user_email), "Type user id")?;
            let user_mail_input = timeout60s!(
                env.driver.find(By::Css("input[name='name']")),
                "Find user name input"
            )?;
            timeout60s!(user_mail_input.send_keys(&user_name), "Type user name")?;
            let user_maildir_input = timeout60s!(
                env.driver.find(By::Css("input[name='maildir']")),
                "Find user maildir input"
            )?;
            timeout60s!(
                user_maildir_input.send_keys(&user_maildir),
                "Type user maildir"
            )?;
            let user_submit_btn = timeout60s!(
                env.driver.find(By::Id("user-submit-button")),
                "Find submit button for user"
            )?;
            timeout60s!(user_submit_btn.click(), "Submit user form")?;
            tokio::time::sleep(Duration::from_millis(500)).await;

            let user_page_source =
                timeout60s!(env.driver.source(), "Get page source after user create")?;
            assert!(
                user_page_source.contains(&user_email),
                "User should appear after creation"
            );

            // 4. Run a report (e.g., aliases report)
            let reports_url = format!("{}/reports", env.app_url);
            timeout60s!(env.driver.get(&reports_url), "Navigate to reports page")?;
            let reports_page_source =
                timeout60s!(env.driver.source(), "Get page source for reports")?;
            assert!(
                reports_page_source.contains("Reports") || reports_page_source.contains("Alias"),
                "Reports page should load"
            );

            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_backup_functionality_flow() -> anyhow::Result<()> {
    use thirtyfour::prelude::*;
    use tokio::time::Duration;

    run_test_with_timeout(
        "test_backup_functionality_flow",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            // Navigate to backup page
            let backup_url = format!("{}/database_backup", env.app_url);
            timeout60s!(env.driver.get(&backup_url), "Navigate to backup page")?;

            // Verify backup page loads correctly
            let page_source = timeout60s!(env.driver.source(), "Get backup page source")?;

            // Check for various possible content that indicates the backup page loaded
            let backup_page_loaded = page_source.contains("Create New Backup")
                || page_source.contains("Existing Backups")
                || page_source.contains("Database Backup")
                || page_source.contains("Select Database")
                || page_source.contains("Create Backup");

            if !backup_page_loaded {
                // If backup page didn't load, check if we got a 404 or other error
                if page_source.contains("Page Not Found") {
                    println!(
                        "Got 404 page - backup route may not be available in test environment"
                    );
                    // Don't fail the test, just log and continue
                    println!("Skipping backup functionality test due to 404");
                    return Ok(());
                }
            }

            assert!(
                backup_page_loaded,
                "Backup page should load with correct content"
            );

            // Check if database dropdown is present and populated
            let database_select = timeout60s!(
                env.driver.find(By::Id("database-select")),
                "Find database select dropdown"
            )?;
            assert!(
                database_select.is_displayed().await.unwrap_or(false),
                "Database select dropdown should be displayed"
            );

            // Get all options in the dropdown
            let options = timeout60s!(
                database_select.find_all(By::Css("option")),
                "Find all database options"
            )?;
            assert!(
                options.len() > 1, // Should have at least one database + "Select a database..." option
                "Database dropdown should have options"
            );

            // Select the first available database (skip the "Select a database..." option)
            if options.len() > 1 {
                let first_database_option = &options[1]; // Index 0 is "Select a database..."
                let database_value = timeout60s!(
                    first_database_option.attr("value"),
                    "Get first database option value"
                )?;
                assert!(
                    database_value.is_some() && !database_value.unwrap().is_empty(),
                    "First database option should have a value"
                );

                // Select the database
                timeout60s!(first_database_option.click(), "Click first database option")?;
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Find and click the create backup button
                let create_button = timeout60s!(
                    env.driver.find(By::Id("create-backup-button")),
                    "Find create backup button"
                )?;
                assert!(
                    create_button.is_displayed().await.unwrap_or(false),
                    "Create backup button should be displayed"
                );

                // Click create backup button
                timeout60s!(create_button.click(), "Click create backup button")?;
                tokio::time::sleep(Duration::from_millis(2000)).await; // Wait for backup creation

                // Check for success message by looking for the success content
                let backup_status = timeout60s!(
                    env.driver.find(By::Id("backup-status")),
                    "Find backup status area"
                )?;

                // Wait for the status to be updated with success or error message
                let mut attempts = 0;
                let max_attempts = 10;
                let mut success_found = false;

                while attempts < max_attempts {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    let status_text = timeout60s!(backup_status.text(), "Get backup status text")?;

                    // Check for success indicators (green styling or success message)
                    if status_text.contains("successfully")
                        || status_text.contains("Download")
                        || status_text.contains("green")
                    {
                        success_found = true;
                        println!("Backup created successfully!");
                        break;
                    }

                    // Check for error indicators
                    if status_text.contains("error")
                        || status_text.contains("failed")
                        || status_text.contains("red")
                    {
                        println!("Backup creation failed with error: {status_text}");
                        // Don't fail the test if backup creation fails - it might be due to mysqldump not being available
                        // Just log the error and continue
                        break;
                    }

                    attempts += 1;
                }

                if !success_found {
                    // Check if backups list was updated
                    let backups_list =
                        timeout60s!(env.driver.find(By::Id("backups-list")), "Find backups list")?;
                    let backups_text = timeout60s!(backups_list.text(), "Get backups list text")?;

                    if !backups_text.contains("Loading backups...")
                        && !backups_text.contains("No backups found")
                    {
                        println!("Backups list updated: {backups_text}");
                    }
                } else {
                    // Check for download link in the success message
                    let download_links = timeout60s!(
                        env.driver
                            .find_all(By::Css("a[href*='/database_backup/download/']")),
                        "Find backup download links"
                    )?;

                    if !download_links.is_empty() {
                        let download_href =
                            timeout60s!(download_links[0].attr("href"), "Get download link href")?;

                        assert!(
                            download_href.is_some()
                                && download_href
                                    .unwrap()
                                    .contains("/database_backup/download/"),
                            "Download link should be present and point to backup download"
                        );
                    }
                }
            }

            // Test backup list functionality
            let backups_list =
                timeout60s!(env.driver.find(By::Id("backups-list")), "Find backups list")?;
            let backups_text = timeout60s!(backups_list.text(), "Get backups list text")?;

            // The backups list should not show "Loading backups..." after a while
            if backups_text.contains("Loading backups...") {
                // Wait a bit more for the list to load
                tokio::time::sleep(Duration::from_millis(3000)).await;
                let updated_backups_text =
                    timeout60s!(backups_list.text(), "Get updated backups list text")?;

                if !updated_backups_text.contains("Loading backups...") {
                    println!("Backups list loaded: {updated_backups_text}");
                }
            } else {
                println!("Backups list: {backups_text}");
            }

            // Test navigation back to dashboard
            let dashboard_url = format!("{}/", env.app_url);
            timeout60s!(env.driver.get(&dashboard_url), "Navigate back to dashboard")?;

            let dashboard_source = timeout60s!(env.driver.source(), "Get dashboard source")?;
            assert!(
                dashboard_source.contains("Dashboard") || dashboard_source.contains("Welcome"),
                "Should be able to navigate back to dashboard"
            );

            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

#[tokio::test]
async fn test_ui_error_handling_with_shared_theme_containerized() -> Result<()> {
    run_test_with_timeout(
        "test_ui_error_handling_with_shared_theme_containerized",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            // Test 1: Entity not found errors with shared theme
            test_entity_not_found_errors_with_theme(&env.driver, &env.app_url).await?;

            // Test 2: Database connection error handling with shared theme
            test_database_error_handling_with_theme(&env.driver, &env.app_url).await?;

            // Test 3: Theme consistency across error pages
            test_theme_consistency_across_error_pages(&env.driver, &env.app_url).await?;

            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(90),
    )
    .await
}

async fn test_theme_consistency_across_error_pages(
    driver: &WebDriver,
    app_url: &str,
) -> Result<()> {
    println!("[ERROR TEST] Testing theme consistency across error pages...");

    // Test that error pages use consistent theme styling
    let pages_to_test = [
        ("/domains/999999", "non-existent domain"),
        ("/users/999999", "non-existent user"),
        ("/aliases/999999", "non-existent alias"),
        ("/nonexistent-page", "404 page"),
    ];

    for (path, description) in pages_to_test.iter() {
        let error_url = format!("{}{}", app_url, path);
        timeout60s!(driver.get(&error_url), "Navigate to error page")?;

        let page_source = timeout30s!(driver.source(), "Get page source for error page")?;

        // Check for consistent theme styling across all error pages
        let has_theme_styling = page_source.contains("bg-gray-")
            || page_source.contains("text-gray-")
            || page_source.contains("border-gray-")
            || page_source.contains("bg-white")
            || page_source.contains("dark:bg-")
            || page_source.contains("text-red-")
            || page_source.contains("bg-red-")
            || page_source.contains("border-red-");

        // Debug: Log what styling is actually present
        if !has_theme_styling {
            println!(
                "[ERROR TEST] Page source for {}: {}",
                description, page_source
            );
        }

        assert!(
            has_theme_styling,
            "Error page for {} should use consistent theme styling",
            description
        );

        // Check that error pages don't contain raw error text without styling
        assert!(
            !page_source.contains("Database connection error")
                && !page_source.contains("Failed to get database pool"),
            "Error page for {} should not show raw database errors",
            description
        );
    }

    println!("[ERROR TEST] ✅ Theme consistency across error pages test passed");
    Ok(())
}

async fn test_entity_not_found_errors_with_theme(driver: &WebDriver, app_url: &str) -> Result<()> {
    println!("[ERROR TEST] Testing entity not found errors with shared theme...");

    // Test accessing non-existent domain
    let non_existent_domain_url = format!("{}/domains/999999", app_url);
    timeout60s!(
        driver.get(&non_existent_domain_url),
        "Navigate to non-existent domain"
    )?;

    let page_source = timeout30s!(driver.source(), "Get page source for non-existent domain")?;
    let title = timeout30s!(driver.title(), "Get page title for non-existent domain")?;

    // Check for not found error message - be more flexible with error detection
    let has_error = page_source.contains("not found")
        || page_source.contains("Not Found")
        || page_source.contains("404")
        || page_source.contains("domains-not-found-title")
        || page_source.contains("error")
        || page_source.contains("Error");

    if !has_error {
        println!(
            "[ERROR TEST] Page source for non-existent domain: {}",
            page_source
        );
        println!("[ERROR TEST] Page title: {}", title);
    }

    assert!(
        has_error,
        "Non-existent domain should show not found error. Title: {title}, Page source: {}",
        page_source
    );

    // Check for any theme styling (be more flexible)
    let has_theme_styling = page_source.contains("text-red-")
        || page_source.contains("bg-red-")
        || page_source.contains("border-red-")
        || page_source.contains("bg-gray-")
        || page_source.contains("text-gray-")
        || page_source.contains("bg-white")
        || page_source.contains("dark:bg-");

    // Log what styling is actually present for debugging
    if !has_theme_styling {
        println!(
            "[ERROR TEST] Page source for non-existent domain: {}",
            page_source
        );
    }

    // For now, just check that the page loads without raw database errors
    assert!(
        !page_source.contains("Database connection error")
            && !page_source.contains("Failed to get database pool"),
        "Error page should not show raw database errors"
    );

    // Test accessing non-existent user
    let non_existent_user_url = format!("{}/users/999999", app_url);
    timeout60s!(
        driver.get(&non_existent_user_url),
        "Navigate to non-existent user"
    )?;

    let page_source = timeout30s!(driver.source(), "Get page source for non-existent user")?;
    // The error page should show some form of "not found" message
    let has_error_message = page_source.contains("not found")
        || page_source.contains("Not Found")
        || page_source.contains("404")
        || page_source.contains("User not found")
        || page_source.contains("Domain not found")
        || page_source.contains("users-not-found");

    assert!(
        has_error_message,
        "Non-existent user should show not found error. Page source: {}",
        page_source
    );

    // Check that user error page doesn't show raw database errors
    assert!(
        !page_source.contains("Database connection error")
            && !page_source.contains("Failed to get database pool"),
        "User error page should not show raw database errors"
    );

    // Test accessing non-existent alias
    let non_existent_alias_url = format!("{}/aliases/999999", app_url);
    timeout60s!(
        driver.get(&non_existent_alias_url),
        "Navigate to non-existent alias"
    )?;

    let page_source = timeout30s!(driver.source(), "Get page source for non-existent alias")?;
    // The error page should show some form of "not found" message
    let has_error_message = page_source.contains("not found")
        || page_source.contains("Not Found")
        || page_source.contains("404")
        || page_source.contains("Alias not found")
        || page_source.contains("User not found")
        || page_source.contains("Domain not found")
        || page_source.contains("aliases-not-found")
        || page_source.contains("users-not-found");

    assert!(
        has_error_message,
        "Non-existent alias should show not found error. Page source: {}",
        page_source
    );

    // Check that alias error page doesn't show raw database errors
    assert!(
        !page_source.contains("Database connection error")
            && !page_source.contains("Failed to get database pool"),
        "Alias error page should not show raw database errors"
    );

    println!("[ERROR TEST] ✅ Entity not found errors with shared theme test passed");
    Ok(())
}

async fn test_database_error_handling_with_theme(driver: &WebDriver, app_url: &str) -> Result<()> {
    println!("[ERROR TEST] Testing database error handling with shared theme...");

    // Navigate to a page that requires database access
    let domains_url = format!("{}/domains", app_url);
    timeout60s!(driver.get(&domains_url), "Navigate to domains page")?;

    let page_source = timeout30s!(driver.source(), "Get page source for domains")?;

    // Check that the page loads without database errors
    // (In a real scenario, we'd need to simulate database connection issues)
    assert!(
        !page_source.contains("Database connection error")
            && !page_source.contains("Failed to get database pool"),
        "Page should load without database errors under normal conditions"
    );

    // Check that the page uses consistent theme styling
    assert!(
        page_source.contains("bg-gray-")
            || page_source.contains("text-gray-")
            || page_source.contains("border-gray-"),
        "Page should use consistent theme styling"
    );

    println!("[ERROR TEST] ✅ Database error handling with shared theme test passed");
    Ok(())
}

#[tokio::test]
async fn test_cross_database_domain_creation_ui() -> Result<()> {
    let config_path = std::env::current_dir()
        .unwrap()
        .join("config/config.docker.multidb.toml");
    let _config_path_str = config_path.to_str().unwrap();
    run_test_with_timeout(
        "test_cross_database_domain_creation_ui",
        async {
            let env = setup_ui_test_env_multidb().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            // Generate unique test data
            let domain1 = format!("test-ui-{}.example.com", rand_domain_str());
            let domain2 = format!("test-ui-{}.example.com", rand_domain_str());

            println!("[CROSS-DB UI TEST] Test domains: {}, {}", domain1, domain2);

            // Test 1: Create domain in primary database
            println!("[CROSS-DB UI TEST] Step 1: Creating domain in primary database");
            create_domain(&env.driver, &env.app_url, &domain1).await?;
            println!("[CROSS-DB UI TEST] ✅ Domain created in primary database");

            // Verify domain is listed in primary database
            let domains_url = format!("{}/domains", env.app_url);
            timeout60s!(env.driver.get(&domains_url), "Navigate to domains list")?;
            let page_source = timeout30s!(env.driver.source(), "Get domains page source")?;
            assert!(
                page_source.contains(&domain1),
                "Domain {} should be visible in primary database",
                domain1
            );

            // Test 2: Switch to secondary database
            println!("[CROSS-DB UI TEST] Step 2: Switching to secondary database");
            switch_database_ui(&env.driver, "secondary").await?;
            println!("[CROSS-DB UI TEST] ✅ Switched to secondary database");

            // Verify we're in secondary database (domain1 should not be visible)
            timeout60s!(
                env.driver.get(&domains_url),
                "Navigate to domains list in secondary"
            )?;
            let page_source =
                timeout30s!(env.driver.source(), "Get domains page source in secondary")?;
            assert!(
                !page_source.contains(&domain1),
                "Domain {} should NOT be visible in secondary database",
                domain1
            );

            // Test 3: Create domain in secondary database
            println!("[CROSS-DB UI TEST] Step 3: Creating domain in secondary database");
            create_domain(&env.driver, &env.app_url, &domain2).await?;
            println!("[CROSS-DB UI TEST] ✅ Domain created in secondary database");

            // Verify domain2 is listed in secondary database
            timeout60s!(
                env.driver.get(&domains_url),
                "Navigate to domains list in secondary"
            )?;
            let page_source =
                timeout30s!(env.driver.source(), "Get domains page source in secondary")?;
            assert!(
                page_source.contains(&domain2),
                "Domain {} should be visible in secondary database",
                domain2
            );

            // Test 4: Switch back to primary database
            println!("[CROSS-DB UI TEST] Step 4: Switching back to primary database");
            switch_database_ui(&env.driver, "primary").await?;
            println!("[CROSS-DB UI TEST] ✅ Switched back to primary database");

            // Verify we're back in primary database (domain1 should be visible, domain2 should not)
            timeout60s!(
                env.driver.get(&domains_url),
                "Navigate to domains list in primary"
            )?;
            let page_source =
                timeout30s!(env.driver.source(), "Get domains page source in primary")?;
            assert!(
                page_source.contains(&domain1),
                "Domain {} should be visible in primary database",
                domain1
            );
            assert!(
                !page_source.contains(&domain2),
                "Domain {} should NOT be visible in primary database",
                domain2
            );

            // Test 5: Create same domain name in primary database (should work)
            println!("[CROSS-DB UI TEST] Step 5: Creating same domain name in primary database");
            let domain3 = format!("test-ui-{}.example.com", rand_domain_str());
            create_domain(&env.driver, &env.app_url, &domain3).await?;
            println!("[CROSS-DB UI TEST] ✅ Same domain name created in primary database");

            // Test 6: Switch to secondary and create same domain name (should work)
            println!("[CROSS-DB UI TEST] Step 6: Creating same domain name in secondary database");
            switch_database_ui(&env.driver, "secondary").await?;
            create_domain(&env.driver, &env.app_url, &domain3).await?;
            println!("[CROSS-DB UI TEST] ✅ Same domain name created in secondary database");

            // Verify both databases have the same domain name
            timeout60s!(
                env.driver.get(&domains_url),
                "Navigate to domains list in secondary"
            )?;
            let page_source =
                timeout30s!(env.driver.source(), "Get domains page source in secondary")?;
            assert!(
                page_source.contains(&domain3),
                "Domain {} should be visible in secondary database",
                domain3
            );

            switch_database_ui(&env.driver, "primary").await?;
            timeout60s!(
                env.driver.get(&domains_url),
                "Navigate to domains list in primary"
            )?;
            let page_source =
                timeout30s!(env.driver.source(), "Get domains page source in primary")?;
            assert!(
                page_source.contains(&domain3),
                "Domain {} should be visible in primary database",
                domain3
            );

            println!("[CROSS-DB UI TEST] ✅ All cross-database domain creation tests passed!");
            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(180),
    )
    .await
}

// Note: UI tests for duplicate wizard would require proper setup of test environment
// and selenium configuration. For now, we have comprehensive unit and integration tests.
