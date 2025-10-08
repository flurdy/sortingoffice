use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct WhoisLookupService;

impl WhoisLookupService {
    pub fn new() -> Self {
        Self
    }

    pub async fn lookup_whois(
        &self,
        domain: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Determine the appropriate WHOIS server based on TLD
        let whois_server = self.get_whois_server(domain);

        // Connect to WHOIS server on port 43
        let addr = format!("{}:43", whois_server);
        let mut stream =
            match tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(&addr)).await {
                Ok(Ok(s)) => s,
                Ok(Err(e)) => {
                    return Err(format!("Failed to connect to WHOIS server: {}", e).into())
                }
                Err(_) => return Err("WHOIS connection timeout".into()),
            };

        // Send the domain query
        let query = format!("{}\r\n", domain);
        stream.write_all(query.as_bytes()).await?;

        // Read the response
        let mut response = String::new();
        match tokio::time::timeout(
            Duration::from_secs(15),
            stream.read_to_string(&mut response),
        )
        .await
        {
            Ok(Ok(_)) => Ok(response),
            Ok(Err(e)) => Err(format!("Failed to read WHOIS response: {}", e).into()),
            Err(_) => Err("WHOIS read timeout".into()),
        }
    }

    fn get_whois_server(&self, domain: &str) -> &'static str {
        // Extract TLD from domain
        let parts: Vec<&str> = domain.split('.').collect();
        let tld = parts.last().unwrap_or(&"");

        // Return appropriate WHOIS server based on TLD
        match tld.to_lowercase().as_str() {
            "com" | "net" => "whois.verisign-grs.com",
            "org" => "whois.pir.org",
            "info" => "whois.afilias.net",
            "biz" => "whois.biz",
            "us" => "whois.nic.us",
            "uk" | "co.uk" => "whois.nic.uk",
            "de" => "whois.denic.de",
            "fr" => "whois.nic.fr",
            "nl" => "whois.domain-registry.nl",
            "no" => "whois.norid.no",
            "se" => "whois.iis.se",
            "es" => "whois.nic.es",
            "it" => "whois.nic.it",
            "eu" => "whois.eu",
            "io" => "whois.nic.io",
            "ai" => "whois.nic.ai",
            "dev" => "whois.nic.google",
            "app" => "whois.nic.google",
            _ => "whois.iana.org", // Fallback to IANA for unknown TLDs
        }
    }
}

impl Default for WhoisLookupService {
    fn default() -> Self {
        Self::new()
    }
}
