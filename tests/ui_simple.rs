use anyhow::Result;
use fantoccini::{Client, ClientBuilder};

async fn setup_client() -> Result<Client> {
    // Simple client setup without retry logic
    let client = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await?;
    Ok(client)
}

#[tokio::test]
async fn test_basic_connection() -> Result<()> {
    let client = setup_client().await?;

    // Just test that we can connect and navigate to a simple page
    client.goto("http://host.docker.internal:3000/").await?;

    // Check that we get a response (even if it's an error page)
    let current_url = client.current_url().await?;
    println!("Current URL: {}", current_url);

    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_page_title() -> Result<()> {
    let client = setup_client().await?;

    client.goto("http://host.docker.internal:3000/").await?;

    // Get the page title
    let title = client.title().await?;
    println!("Page title: {}", title);

    // Basic assertion - title should not be empty
    assert!(!title.is_empty());

    client.close().await?;
    Ok(())
}
