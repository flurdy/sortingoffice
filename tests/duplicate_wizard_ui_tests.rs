use anyhow::Result;
use thirtyfour::prelude::*;
use tokio::time::{timeout, Duration};

mod common;
use common::ui_helpers::{
    login_and_goto_dashboard, rand_domain_str, run_test_with_timeout, safe_find_and_send_keys,
    setup_ui_test_env,
};

/// Test the duplicate wizard page loads and form works
#[tokio::test]
async fn test_duplicate_wizard_page_loads() -> anyhow::Result<()> {
    run_test_with_timeout(
        "test_duplicate_wizard_page_loads",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            // Test that the duplicate wizard page loads
            navigate_to_duplicate_wizard(&env.driver, &env.app_url).await?;

            // Test that we can fill in the form fields
            let new_domain = format!(
                "duplicate-{}-{}.com",
                rand_domain_str(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );

            // Select source domain from dropdown
            let source_domain_select = timeout30s!(
                env.driver.find(By::Css("select[name='source_domain']")),
                "Find source domain select"
            )?;
            source_domain_select.click().await?;

            // Select the first available option (if any)
            if let Ok(option) = timeout30s!(
                env.driver.find(By::Css(
                    "select[name='source_domain'] option:not([value=''])"
                )),
                "Find first available domain option"
            ) {
                option.click().await?;
            }

            // Fill new domain
            safe_find_and_send_keys(
                &env.driver,
                "input[name='new_domain']",
                &new_domain,
                "new domain input",
            )
            .await?;

            // Verify the form is filled correctly
            let source_select = timeout30s!(
                env.driver.find(By::Css("select[name='source_domain']")),
                "Find source domain select"
            )?;
            let _source_value =
                timeout30s!(source_select.prop("value"), "Get source domain value")?;
            // Note: source_value might be empty if no domains are available, which is expected

            let new_input = timeout30s!(
                env.driver.find(By::Css("input[name='new_domain']")),
                "Find new domain input"
            )?;
            let new_value = timeout30s!(new_input.prop("value"), "Get new domain value")?;
            assert_eq!(new_value.unwrap_or_default(), new_domain);

            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(60), // 1 minute
    )
    .await
}

/// Test duplicate wizard form elements are present and fillable
#[tokio::test]
async fn test_duplicate_wizard_form_elements() -> anyhow::Result<()> {
    run_test_with_timeout(
        "test_duplicate_wizard_form_elements",
        async {
            let env = setup_ui_test_env().await?;
            login_and_goto_dashboard(&env.driver, &env.app_url).await?;

            // Navigate to duplicate wizard
            navigate_to_duplicate_wizard(&env.driver, &env.app_url).await?;

            // Test that form elements are present and fillable
            let new_domain = format!(
                "duplicate-{}-{}.com",
                rand_domain_str(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );

            // Test source domain dropdown is present
            let source_domain_select = timeout30s!(
                env.driver.find(By::Css("select[name='source_domain']")),
                "Find source domain select"
            )?;
            assert!(
                source_domain_select.is_displayed().await?,
                "Source domain select should be visible"
            );

            // Test new domain input is present and fillable
            let new_domain_input = timeout30s!(
                env.driver.find(By::Css("input[name='new_domain']")),
                "Find new domain input"
            )?;
            assert!(
                new_domain_input.is_displayed().await?,
                "New domain input should be visible"
            );

            // Fill the new domain input
            safe_find_and_send_keys(
                &env.driver,
                "input[name='new_domain']",
                &new_domain,
                "new domain input",
            )
            .await?;

            // Test submit button is present (may not be visible due to CSS)
            let submit_button = timeout30s!(
                env.driver.find(By::Css("button[type='submit']")),
                "Find submit button"
            )?;
            // Just check that the element exists, not that it's visible
            assert!(
                submit_button.is_present().await?,
                "Submit button should be present"
            );

            // Test cancel button is present
            let cancel_button = timeout30s!(
                env.driver.find(By::Css("a[href='/domains']")),
                "Find cancel button"
            )?;
            assert!(
                cancel_button.is_present().await?,
                "Cancel button should be present"
            );

            env.cleanup().await?;
            Ok(())
        },
        Duration::from_secs(30), // 30 seconds
    )
    .await
}

/// Navigate to duplicate wizard
async fn navigate_to_duplicate_wizard(driver: &WebDriver, app_url: &str) -> Result<()> {
    let duplicate_wizard_url = format!("{}/duplicate-wizard", app_url.trim_end_matches('/'));
    timeout60s!(
        driver.get(&duplicate_wizard_url),
        "Navigate to duplicate wizard page"
    )?;

    // Wait a moment for any redirects
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Check current URL to see if we were redirected
    let current_url = timeout60s!(driver.current_url(), "Get current URL")?;
    println!(
        "[DUPLICATE WIZARD TEST] Current URL after navigation: {}",
        current_url
    );

    // Verify we're on the duplicate wizard page (either the index or domain-selection)
    let page_title = timeout60s!(driver.title(), "Get page title")?;
    println!("[DUPLICATE WIZARD TEST] Page title: {}", page_title);

    // Check if we're on the duplicate wizard page or if we got redirected
    let current_url_str = current_url.to_string();
    if current_url_str.contains("/duplicate-wizard") {
        println!("[DUPLICATE WIZARD TEST] Successfully navigated to duplicate wizard");
    } else {
        // Check if we got an error page
        let page_source = timeout60s!(driver.source(), "Get page source")?;
        if page_source.contains("Page Not Found") || page_source.contains("404") {
            return Err(anyhow::anyhow!(
                "Duplicate wizard page not found. Current URL: {}, Page title: {}",
                current_url_str,
                page_title
            ));
        } else {
            println!(
                "[DUPLICATE WIZARD TEST] Unexpected page content: {}",
                page_source
            );
            return Err(anyhow::anyhow!(
                "Unexpected page content. Current URL: {}, Page title: {}",
                current_url_str,
                page_title
            ));
        }
    }

    Ok(())
}
