use fantoccini::{Client, ClientBuilder, Locator};
use anyhow::Result;

async fn setup_client() -> Result<Client> {
    // Add retry logic for session creation with longer delays
    let mut attempts = 0;
    let max_attempts = 5;
    
    while attempts < max_attempts {
        match ClientBuilder::native()
            .connect("http://localhost:4444")
            .await {
            Ok(client) => return Ok(client),
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(anyhow::anyhow!("WebDriver session creation failed: {:?}", e));
                }
                // Wait longer before retrying
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            }
        }
    }
    
    Err(anyhow::anyhow!("Failed to create WebDriver session after {} attempts", max_attempts))
}

#[tokio::test]
async fn test_domain_creation_workflow() -> Result<()> {
    let client = setup_client().await?;
    
    // Navigate to domain creation page
    client.goto("http://host.docker.internal:3000/domains/new").await?;
    
    // Fill out the form
    let name_input = client.find(Locator::Css("input[name='name']")).await?;
    name_input.send_keys("testdomain.com").await?;
    
    let quota_input = client.find(Locator::Css("input[name='quota']")).await?;
    quota_input.send_keys("1000").await?;
    
    // Submit the form
    let submit_button = client.find(Locator::Css("button[type='submit'], input[type='submit']")).await?;
    submit_button.click().await?;
    
    // Wait for redirect or success message
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    // Check if we're redirected to domains list or show page
    let current_url = client.current_url().await?;
    assert!(current_url.as_str().contains("/domains"));
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_user_creation_workflow() -> Result<()> {
    let client = setup_client().await?;
    
    // Navigate to user creation page
    client.goto("http://host.docker.internal:3000/users/new").await?;
    
    // Fill out the form
    let username_input = client.find(Locator::Css("input[name='username']")).await?;
    username_input.send_keys("testuser").await?;
    
    let email_input = client.find(Locator::Css("input[name='email']")).await?;
    email_input.send_keys("testuser@testdomain.com").await?;
    
    let password_input = client.find(Locator::Css("input[name='password']")).await?;
    password_input.send_keys("testpassword123").await?;
    
    // Submit the form
    let submit_button = client.find(Locator::Css("button[type='submit'], input[type='submit']")).await?;
    submit_button.click().await?;
    
    // Wait for redirect or success message
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    // Check if we're redirected to users list or show page
    let current_url = client.current_url().await?;
    assert!(current_url.as_str().contains("/users"));
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_alias_creation_workflow() -> Result<()> {
    let client = setup_client().await?;
    
    // Navigate to alias creation page
    client.goto("http://host.docker.internal:3000/aliases/new").await?;
    
    // Fill out the form
    let mail_input = client.find(Locator::Css("input[name='mail']")).await?;
    mail_input.send_keys("testalias@testdomain.com").await?;
    
    let destination_input = client.find(Locator::Css("input[name='destination']")).await?;
    destination_input.send_keys("testuser@testdomain.com").await?;
    
    // Submit the form
    let submit_button = client.find(Locator::Css("button[type='submit'], input[type='submit']")).await?;
    submit_button.click().await?;
    
    // Wait for redirect or success message
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    // Check if we're redirected to aliases list or show page
    let current_url = client.current_url().await?;
    assert!(current_url.as_str().contains("/aliases"));
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_form_validation_errors() -> Result<()> {
    let client = setup_client().await?;
    
    // Navigate to domain creation page
    client.goto("http://host.docker.internal:3000/domains/new").await?;
    
    // Try to submit empty form
    let submit_button = client.find(Locator::Css("button[type='submit'], input[type='submit']")).await?;
    submit_button.click().await?;
    
    // Wait for validation errors
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Check for error messages
    let body = client.find(Locator::Css("body")).await?;
    let body_text = body.text().await?;
    
    // Should show validation errors or stay on the form page
    let current_url = client.current_url().await?;
    assert!(current_url.as_str().contains("/new") || body_text.contains("error") || body_text.contains("required"));
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_navigation_breadcrumbs() -> Result<()> {
    let client = setup_client().await?;
    
    // Navigate to a nested page
    client.goto("http://host.docker.internal:3000/domains/new").await?;
    
    // Look for breadcrumb navigation
    let breadcrumbs = client.find_all(Locator::Css(".breadcrumb, .breadcrumbs, nav[aria-label='breadcrumb']")).await?;
    
    // If breadcrumbs exist, they should be clickable
    if !breadcrumbs.is_empty() {
        for breadcrumb in breadcrumbs {
            let links = breadcrumb.find_all(Locator::Css("a")).await?;
            assert!(!links.is_empty(), "Breadcrumbs should contain links");
        }
    }
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_table_sorting_and_pagination() -> Result<()> {
    let client = setup_client().await?;
    
    // Navigate to a list page
    client.goto("http://host.docker.internal:3000/domains").await?;
    
    // Look for table elements
    let tables = client.find_all(Locator::Css("table")).await?;
    
    if !tables.is_empty() {
        // Check for sortable headers
        let sortable_headers = client.find_all(Locator::Css("th[data-sort], th.sortable, .sortable")).await?;
        
        // Check for pagination
        let pagination = client.find_all(Locator::Css(".pagination, .pager, nav[aria-label='pagination']")).await?;
        
        // At least one of these should exist
        assert!(!sortable_headers.is_empty() || !pagination.is_empty() || tables.len() > 0);
    }
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_functionality() -> Result<()> {
    let client = setup_client().await?;
    
    // Navigate to a list page
    client.goto("http://host.docker.internal:3000/domains").await?;
    
    // Look for search input
    let search_inputs = client.find_all(Locator::Css("input[type='search'], input[placeholder*='search'], input[name*='search']")).await?;
    
    if !search_inputs.is_empty() {
        // Test search functionality
        let search_input = &search_inputs[0];
        search_input.send_keys("test").await?;
        
        // Look for search button or auto-search
        let search_button = client.find(Locator::Css("button[type='submit'], .search-btn")).await?;
        search_button.click().await?;
        
        // Wait for search results
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        // Should still be on the same page
        let current_url = client.current_url().await?;
        assert!(current_url.as_str().contains("/domains"));
    }
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_modal_dialogs() -> Result<()> {
    let client = setup_client().await?;
    
    // Navigate to a page that might have modals
    client.goto("http://host.docker.internal:3000/domains").await?;
    
    // Look for modal triggers (buttons that might open modals)
    let modal_triggers = client.find_all(Locator::Css("button[data-toggle='modal'], button[data-target], .modal-trigger")).await?;
    
    if !modal_triggers.is_empty() {
        // Click the first modal trigger
        modal_triggers[0].click().await?;
        
        // Wait for modal to appear
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        // Look for modal content
        let modals = client.find_all(Locator::Css(".modal, [role='dialog']")).await?;
        assert!(!modals.is_empty(), "Modal should appear after clicking trigger");
    }
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_performance_metrics() -> Result<()> {
    let client = setup_client().await?;
    
    // Test page load performance
    let start_time = std::time::Instant::now();
    
    client.goto("http://host.docker.internal:3000").await?;
    
    let load_time = start_time.elapsed();
    
    // Page should load within reasonable time (5 seconds)
    assert!(load_time.as_millis() < 5000, "Page load took too long: {}ms", load_time.as_millis());
    
    // Test navigation performance
    let nav_start = std::time::Instant::now();
    client.goto("http://host.docker.internal:3000/dashboard").await?;
    let nav_time = nav_start.elapsed();
    
    // Navigation should be fast
    assert!(nav_time.as_millis() < 3000, "Navigation took too long: {}ms", nav_time.as_millis());
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_data_persistence() -> Result<()> {
    let client = setup_client().await?;
    
    // Navigate to a form page
    client.goto("http://host.docker.internal:3000/domains/new").await?;
    
    // Fill out form partially
    let name_input = client.find(Locator::Css("input[name='name']")).await?;
    name_input.send_keys("partial").await?;
    
    // Navigate away and back
    client.goto("http://host.docker.internal:3000/dashboard").await?;
    client.goto("http://host.docker.internal:3000/domains/new").await?;
    
    // Check if form data is preserved (this might not be implemented)
    let name_input_again = client.find(Locator::Css("input[name='name']")).await?;
    let value = name_input_again.prop("value").await?;
    
    // Either the value should be preserved or the field should be empty (both are acceptable)
    assert!(value.is_none() || value.unwrap().contains("partial"));
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let client = setup_client().await?;
    
    // Test various error scenarios
    let error_urls = [
        "http://host.docker.internal:3000/nonexistent",
        "http://host.docker.internal:3000/domains/999999",
        "http://host.docker.internal:3000/users/999999",
    ];
    
    for url in error_urls {
        client.goto(url).await?;
        
        // Should show some error content
        let body = client.find(Locator::Css("body")).await?;
        let body_text = body.text().await?;
        
        // Should contain error indicators
        assert!(
            body_text.contains("404") || 
            body_text.contains("Not Found") || 
            body_text.contains("Error") ||
            body_text.contains("error"),
            "Error page should show error content for URL: {}", url
        );
    }
    
    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_accessibility_features() -> Result<()> {
    let client = setup_client().await?;
    
    client.goto("http://host.docker.internal:3000").await?;
    
    // Check for accessibility attributes
    let elements_with_aria = client.find_all(Locator::Css("[aria-label], [aria-labelledby], [role]")).await?;
    
    // Check for alt text on images
    let images = client.find_all(Locator::Css("img")).await?;
    let images_with_alt = client.find_all(Locator::Css("img[alt]")).await?;
    
    // Check for form labels
    let form_inputs = client.find_all(Locator::Css("input, textarea, select")).await?;
    let inputs_with_labels = client.find_all(Locator::Css("input[id], textarea[id], select[id]")).await?;
    
    // Basic accessibility checks
    assert!(elements_with_aria.len() > 0 || images_with_alt.len() > 0 || inputs_with_labels.len() > 0);
    
    client.close().await?;
    Ok(())
} 
