# DNS Lookup Integration Research

## Overview
This document outlines the research and implementation plan for integrating DNS lookup functionality into the Sorting Office application to display NS, MX, TXT, DKIM, and other DNS records for domains.

## Recommended Rust DNS Libraries

### 1. **hickory-dns** (Recommended)
- **Status**: Active, modern successor to trust-dns
- **Repository**: https://github.com/hickory-dns/hickory-dns
- **Pros**:
  - Modern async/await support
  - Comprehensive DNS record type support
  - Good performance
  - Well-maintained
  - Supports both sync and async operations
- **Cons**:
  - Relatively new (less mature than trust-dns)
  - Smaller community

### 2. **trust-dns-resolver** (Alternative)
- **Status**: Mature but in maintenance mode
- **Repository**: https://github.com/bluejekyll/trust-dns
- **Pros**:
  - Very mature and stable
  - Extensive documentation
  - Large community
  - Battle-tested
- **Cons**:
  - Maintenance mode (not actively developed)
  - Older async patterns

## Implementation Plan

### Phase 1: Basic DNS Lookup Service
```rust
// src/services/dns_lookup.rs
use hickory_dns::{
    client::{Client, SyncClient},
    rr::{DNSClass, Name, RecordType},
    udp::UdpClientStream,
};
use std::net::IpAddr;
use tokio::net::UdpSocket;

pub struct DnsLookupService {
    client: SyncClient,
}

impl DnsLookupService {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize DNS client
        let address = "8.8.8.8:53".parse()?;
        let stream = UdpClientStream::<UdpSocket>::new(address);
        let client = SyncClient::new(stream);
        
        Ok(Self { client })
    }

    pub async fn lookup_ns(&self, domain: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let name = Name::from_str(domain)?;
        let response = self.client.query(&name, DNSClass::IN, RecordType::NS).await?;
        
        let ns_records: Vec<String> = response
            .answers()
            .iter()
            .filter_map(|record| record.data().and_then(|data| data.as_ns()))
            .map(|ns| ns.to_string())
            .collect();
            
        Ok(ns_records)
    }

    pub async fn lookup_mx(&self, domain: &str) -> Result<Vec<MxRecord>, Box<dyn std::error::Error>> {
        let name = Name::from_str(domain)?;
        let response = self.client.query(&name, DNSClass::IN, RecordType::MX).await?;
        
        let mx_records: Vec<MxRecord> = response
            .answers()
            .iter()
            .filter_map(|record| record.data().and_then(|data| data.as_mx()))
            .map(|mx| MxRecord {
                priority: mx.preference(),
                exchange: mx.exchange().to_string(),
            })
            .collect();
            
        Ok(mx_records)
    }

    pub async fn lookup_txt(&self, domain: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let name = Name::from_str(domain)?;
        let response = self.client.query(&name, DNSClass::IN, RecordType::TXT).await?;
        
        let txt_records: Vec<String> = response
            .answers()
            .iter()
            .filter_map(|record| record.data().and_then(|data| data.as_txt()))
            .flat_map(|txt| txt.iter())
            .map(|txt| String::from_utf8_lossy(txt).to_string())
            .collect();
            
        Ok(txt_records)
    }

    pub async fn lookup_dkim(&self, domain: &str, selector: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let dkim_domain = format!("{}.{}", selector, domain);
        self.lookup_txt(&dkim_domain).await
    }

    pub async fn lookup_spf(&self, domain: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.lookup_txt(domain).await
    }
}

#[derive(Debug, Clone)]
pub struct MxRecord {
    pub priority: u16,
    pub exchange: String,
}

#[derive(Debug, Clone)]
pub struct DnsLookupResult {
    pub domain: String,
    pub ns_records: Vec<String>,
    pub mx_records: Vec<MxRecord>,
    pub txt_records: Vec<String>,
    pub dkim_records: Vec<String>,
    pub spf_records: Vec<String>,
}
```

### Phase 2: Database Models
```rust
// src/models.rs (additions)
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: Option<i32>,
    pub domain_id: i32,
    pub record_type: String, // "NS", "MX", "TXT", "DKIM", "SPF"
    pub name: String,
    pub value: String,
    pub priority: Option<u16>, // For MX records
    pub ttl: Option<u32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsLookupCache {
    pub id: Option<i32>,
    pub domain_id: i32,
    pub lookup_type: String,
    pub result: String, // JSON serialized result
    pub expires_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
}
```

### Phase 3: Database Schema
```sql
-- Add to migrations
CREATE TABLE dns_records (
    id INT AUTO_INCREMENT PRIMARY KEY,
    domain_id INT NOT NULL,
    record_type VARCHAR(10) NOT NULL,
    name VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    priority INT NULL,
    ttl INT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (domain_id) REFERENCES domains(pkid) ON DELETE CASCADE,
    INDEX idx_domain_record_type (domain_id, record_type)
);

CREATE TABLE dns_lookup_cache (
    id INT AUTO_INCREMENT PRIMARY KEY,
    domain_id INT NOT NULL,
    lookup_type VARCHAR(20) NOT NULL,
    result TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (domain_id) REFERENCES domains(pkid) ON DELETE CASCADE,
    UNIQUE KEY unique_domain_lookup (domain_id, lookup_type)
);
```

### Phase 4: UI Integration
```html
<!-- templates/domains/show.html additions -->
<div class="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">DNS Records</h3>
    
    <!-- DNS Lookup Button -->
    <div class="mb-4">
        <button 
            hx-post="/domains/{{ domain.pkid }}/dns-lookup" 
            hx-target="#dns-results"
            hx-indicator="#dns-loading"
            class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
            <span id="dns-loading" class="htmx-indicator">Loading...</span>
            Lookup DNS Records
        </button>
    </div>
    
    <!-- DNS Results -->
    <div id="dns-results">
        <!-- Results will be loaded here -->
    </div>
</div>
```

### Phase 5: Handler Implementation
```rust
// src/handlers/dns_lookup.rs
use crate::services::dns_lookup::DnsLookupService;
use crate::models::DnsLookupResult;
use axum::{extract::State, response::Html};
use askama::Template;

#[derive(Template)]
#[template(path = "domains/dns_results.html")]
pub struct DnsResultsTemplate {
    pub domain: String,
    pub ns_records: Vec<String>,
    pub mx_records: Vec<MxRecord>,
    pub txt_records: Vec<String>,
    pub dkim_records: Vec<String>,
    pub spf_records: Vec<String>,
    pub error: Option<String>,
}

pub async fn lookup_dns_records(
    State(state): State<AppState>,
    Path(domain_id): Path<i32>,
) -> Result<Html<String>, StatusCode> {
    // Get domain from database
    let domain = get_domain_by_id(&state.db_pool, domain_id).await?;
    
    // Initialize DNS lookup service
    let dns_service = DnsLookupService::new().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Perform DNS lookups
    let result = match dns_service.lookup_all_records(&domain.domain).await {
        Ok(result) => result,
        Err(e) => {
            return Ok(Html(render_dns_error(&domain.domain, &e.to_string())));
        }
    };
    
    // Cache results in database
    cache_dns_results(&state.db_pool, domain_id, &result).await?;
    
    // Render results
    let template = DnsResultsTemplate {
        domain: domain.domain,
        ns_records: result.ns_records,
        mx_records: result.mx_records,
        txt_records: result.txt_records,
        dkim_records: result.dkim_records,
        spf_records: result.spf_records,
        error: None,
    };
    
    Ok(Html(template.render().unwrap()))
}
```

## Configuration Options

### DNS Servers
```toml
# config.toml additions
[dns_lookup]
enabled = true
timeout_seconds = 10
cache_ttl_hours = 24
servers = ["8.8.8.8:53", "1.1.1.1:53", "8.8.4.4:53"]
default_dkim_selectors = ["default", "google", "mail", "k1", "selector1", "selector2"]
```

### Environment Variables
```bash
DNS_LOOKUP_ENABLED=true
DNS_LOOKUP_TIMEOUT=10
DNS_LOOKUP_CACHE_TTL=24
DNS_SERVERS=8.8.8.8:53,1.1.1.1:53
```

## Security Considerations

1. **Rate Limiting**: Implement rate limiting to prevent DNS abuse
2. **Input Validation**: Validate domain names before DNS queries
3. **Timeout Handling**: Set reasonable timeouts to prevent hanging
4. **Error Handling**: Don't expose internal DNS errors to users
5. **Caching**: Cache results to reduce DNS load and improve performance

## Performance Considerations

1. **Async Operations**: Use async DNS lookups to prevent blocking
2. **Caching**: Cache DNS results with appropriate TTL
3. **Batch Operations**: Support batch DNS lookups for multiple domains
4. **Background Jobs**: Consider background DNS lookup jobs for large datasets
5. **Connection Pooling**: Reuse DNS client connections

## Testing Strategy

1. **Unit Tests**: Test individual DNS lookup functions
2. **Integration Tests**: Test with real DNS servers
3. **Mock Tests**: Test error conditions with mocked DNS responses
4. **Performance Tests**: Test with high-volume DNS lookups
5. **UI Tests**: Test DNS lookup UI components

## Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
hickory-dns = "0.24"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Implementation Timeline

1. **Week 1**: Basic DNS lookup service and models
2. **Week 2**: Database schema and caching
3. **Week 3**: UI integration and handlers
4. **Week 4**: Testing and optimization

## Future Enhancements

1. **DKIM Validation**: Validate DKIM signatures
2. **SPF Validation**: Parse and validate SPF records
3. **DMARC Records**: Lookup and parse DMARC records
4. **DNS Security**: Implement DNSSEC validation
5. **Historical Data**: Track DNS record changes over time
6. **Alerting**: Alert on DNS record changes
7. **Bulk Operations**: Bulk DNS lookup for multiple domains
8. **API Endpoints**: REST API for DNS lookup functionality

## Conclusion

This implementation provides a comprehensive DNS lookup system that integrates well with the existing Sorting Office architecture. The modular design allows for incremental implementation and future enhancements while maintaining good performance and security practices.
