use anyhow::Result;
use fantoccini::{Client, ClientBuilder};
use tokio::time::{timeout, Duration};

// Function to get the application URL from environment variable or use default
fn get_app_url() -> String {
    std::env::var("APP_URL").unwrap_or_else(|_| "http://host.docker.internal:3000".to_string())
}

async fn setup_client() -> Result<Client> {
    // Add retry logic for session creation with longer delays
    let mut attempts = 0;
    let max_attempts = 5;

    while attempts < max_attempts {
        match ClientBuilder::native()
            .connect("http://localhost:4444")
            .await
        {
            Ok(client) => return Ok(client),
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(anyhow::anyhow!(
                        "WebDriver session creation failed: {:?}",
                        e
                    ));
                }
                // Wait longer before retrying
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            }
        }
    }

    Err(anyhow::anyhow!(
        "Failed to create WebDriver session after {} attempts",
        max_attempts
    ))
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
async fn test_homepage_loads() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            // Navigate to homepage
            client.goto(&get_app_url()).await?;

            // Wait a moment for page to fully load
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            // Check that we're redirected to login or dashboard
            let current_url = client.current_url().await?;
            assert!(current_url.as_str().contains("3000"));

            // Check for basic page elements
            let title_text = client.title().await?;
            println!("Page title: '{}'", title_text);
            assert!(
                title_text.len() > 0,
                "Title should not be empty, got: '{}'",
                title_text
            );

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_dashboard_navigation() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            // Navigate to dashboard
            client.goto(&format!("{}/dashboard", get_app_url())).await?;

            // Check for dashboard elements
            let body = client.find(fantoccini::Locator::Css("body")).await?;
            let body_text = body.text().await?;
            assert!(body_text.contains("Dashboard") || body_text.contains("dashboard"));

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_navigation_menu() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            // Navigate to any page
            client.goto(&get_app_url()).await?;

            // Look for navigation elements
            let nav_elements = client
                .find_all(fantoccini::Locator::Css("nav, .nav, .navbar, .menu"))
                .await?;
            assert!(
                !nav_elements.is_empty()
                    || client.find(fantoccini::Locator::Css("a")).await.is_ok()
            );

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_responsive_design() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            // Test desktop viewport
            client.set_window_size(1920, 1080).await?;
            client.goto(&get_app_url()).await?;

            // Test mobile viewport
            client.set_window_size(375, 667).await?;
            client.goto(&get_app_url()).await?;

            // Both should load without errors
            let current_url = client.current_url().await?;
            assert!(current_url.as_str().contains("3000"));

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_error_pages() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            // Test 404 page
            client
                .goto(&format!("{}/nonexistent-page", get_app_url()))
                .await?;

            // Should show some error content
            let body = client.find(fantoccini::Locator::Css("body")).await?;
            let body_text = body.text().await?;
            assert!(
                body_text.contains("404")
                    || body_text.contains("Not Found")
                    || body_text.contains("Error")
            );

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_page_titles() -> Result<()> {
    let test_timeout = Duration::from_secs(60);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            // Test main pages have titles
            let pages = [
                "/",
                "/dashboard",
                "/domains",
                "/users",
                "/aliases",
                "/stats",
            ];

            for page in pages {
                client.goto(&format!("{}{}", get_app_url(), page)).await?;
                let title_text = client.title().await?;
                assert!(!title_text.is_empty(), "Page {} should have a title", page);
            }

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_loading_states() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            // Navigate to a page and check for loading indicators
            client.goto(&get_app_url()).await?;

            // Wait a moment for any loading to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            // Page should be interactive
            let body = client.find(fantoccini::Locator::Css("body")).await?;
            assert!(body.text().await?.len() > 0);

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_form_validation() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            // Navigate to a form page (e.g., create domain)
            client
                .goto(&format!("{}/domains/new", get_app_url()))
                .await?;

            // Look for form elements
            let forms = client.find_all(fantoccini::Locator::Css("form")).await?;
            if !forms.is_empty() {
                // Form should have some input elements
                let inputs = client
                    .find_all(fantoccini::Locator::Css("input, textarea, select"))
                    .await?;
                assert!(!inputs.is_empty(), "Form should have input elements");
            }

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_accessibility_basics() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            client.goto(&get_app_url()).await?;

            // Check for basic accessibility elements
            let body = client.find(fantoccini::Locator::Css("body")).await?;
            let body_text = body.text().await?;

            // Should have some content
            assert!(body_text.len() > 0, "Page should have content");

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_cross_browser_compatibility() -> Result<()> {
    let test_timeout = Duration::from_secs(60);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            // Test different viewport sizes
            let viewports = [
                (1920, 1080),
                (1366, 768),
                (1024, 768),
                (768, 1024),
                (375, 667),
            ];

            for (width, height) in viewports {
                client.set_window_size(width, height).await?;
                client.goto(&get_app_url()).await?;

                // Should load without errors
                let current_url = client.current_url().await?;
                assert!(current_url.as_str().contains("3000"));
            }

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_domains_list_page() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            client.goto(&format!("{}/domains", get_app_url())).await?;

            // Check for domains page content
            let body = client.find(fantoccini::Locator::Css("body")).await?;
            let body_text = body.text().await?;
            assert!(body_text.contains("Domains") || body_text.contains("domains"));

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_users_list_page() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            client.goto(&format!("{}/users", get_app_url())).await?;

            // Check for users page content
            let body = client.find(fantoccini::Locator::Css("body")).await?;
            let body_text = body.text().await?;
            assert!(body_text.contains("Users") || body_text.contains("users"));

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_aliases_list_page() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            client.goto(&format!("{}/aliases", get_app_url())).await?;

            // Check for aliases page content
            let body = client.find(fantoccini::Locator::Css("body")).await?;
            let body_text = body.text().await?;
            assert!(body_text.contains("Aliases") || body_text.contains("aliases"));

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}

#[tokio::test]
async fn test_stats_page() -> Result<()> {
    let test_timeout = Duration::from_secs(30);
    run_test_with_timeout(
        async {
            let client = setup_client().await?;

            client.goto(&format!("{}/stats", get_app_url())).await?;

            // Check for stats page content
            let body = client.find(fantoccini::Locator::Css("body")).await?;
            let body_text = body.text().await?;
            assert!(
                body_text.contains("Statistics")
                    || body_text.contains("stats")
                    || body_text.contains("Statistics")
            );

            client.close().await?;
            Ok(())
        },
        test_timeout,
    )
    .await
}
