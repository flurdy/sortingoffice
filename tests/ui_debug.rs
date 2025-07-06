use anyhow::Result;
use fantoccini::{Client, ClientBuilder};

async fn setup_client() -> Result<Client> {
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
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            }
        }
    }

    Err(anyhow::anyhow!(
        "Failed to create WebDriver session after {} attempts",
        max_attempts
    ))
}

#[tokio::test]
async fn test_domain_form_debug() -> Result<()> {
    let client = setup_client().await?;

    // Navigate to domain creation page
    println!("Navigating to domain creation page...");
    client
        .goto("http://host.docker.internal:3000/domains/new")
        .await?;

    // Wait for page to load
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    // Get page title
    let title = client.title().await?;
    println!("Page title: '{}'", title);

    // Get current URL
    let current_url = client.current_url().await?;
    println!("Current URL: {}", current_url);

    // Check for form elements
    println!("Checking for form elements...");

    // Look for any input elements
    let all_inputs = client.find_all(fantoccini::Locator::Css("input")).await?;
    println!("Found {} input elements", all_inputs.len());

    for (i, input) in all_inputs.iter().enumerate() {
        let name = input.attr("name").await?.unwrap_or_default();
        let id = input.attr("id").await?.unwrap_or_default();
        let input_type = input.attr("type").await?.unwrap_or_default();
        println!(
            "  Input {}: name='{}', id='{}', type='{}'",
            i, name, id, input_type
        );
    }

    // Look for submit buttons
    let submit_buttons = client
        .find_all(fantoccini::Locator::Css(
            "button[type='submit'], input[type='submit']",
        ))
        .await?;
    println!("Found {} submit buttons", submit_buttons.len());

    // Look for forms
    let forms = client.find_all(fantoccini::Locator::Css("form")).await?;
    println!("Found {} forms", forms.len());

    // Get page source to see what's actually there
    let page_source = client.source().await?;
    println!("Page source length: {} characters", page_source.len());

    if page_source.contains("input[name='domain']") {
        println!("✓ Found input[name='domain'] in page source");
    } else {
        println!("✗ input[name='domain'] NOT found in page source");
    }

    if page_source.contains("button[type='submit']") {
        println!("✓ Found submit button in page source");
    } else {
        println!("✗ Submit button NOT found in page source");
    }

    client.close().await?;
    Ok(())
}
