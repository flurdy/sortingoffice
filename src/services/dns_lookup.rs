use hickory_resolver::TokioAsyncResolver;

#[derive(Debug, Clone)]
pub struct MxRecord {
    pub priority: u16,
    pub exchange: String,
}

#[derive(Debug, Clone, Default)]
pub struct DnsLookupResult {
    pub domain: String,
    pub ns_records: Vec<String>,
    pub mx_records: Vec<MxRecord>,
    pub txt_records: Vec<String>,
    pub dkim_records: Vec<String>,
}

#[derive(Clone)]
pub struct DnsLookupService {
    resolver: TokioAsyncResolver,
}

impl DnsLookupService {
    pub async fn new_system() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Use system configuration (resolv.conf)
        let resolver = TokioAsyncResolver::tokio_from_system_conf()?;
        Ok(Self { resolver })
    }

    pub async fn lookup_ns(
        &self,
        domain: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.resolver.ns_lookup(domain).await?;
        Ok(response
            .iter()
            .map(|name| name.to_utf8())
            .collect::<Vec<_>>())
    }

    pub async fn lookup_mx(
        &self,
        domain: &str,
    ) -> Result<Vec<MxRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.resolver.mx_lookup(domain).await?;
        Ok(response
            .iter()
            .map(|mx| MxRecord {
                priority: mx.preference(),
                exchange: mx.exchange().to_utf8(),
            })
            .collect::<Vec<_>>())
    }

    pub async fn lookup_txt(
        &self,
        domain: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.resolver.txt_lookup(domain).await?;
        let mut results = Vec::new();
        for txt in response.iter() {
            for data in txt.txt_data() {
                results.push(String::from_utf8_lossy(data).to_string());
            }
        }
        Ok(results)
    }

    pub async fn lookup_dkim(
        &self,
        selector: &str,
        domain: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let hostname = format!("{}._domainkey.{}", selector.trim(), domain.trim());
        self.lookup_txt(&hostname).await
    }

    pub async fn lookup_all(
        &self,
        domain: &str,
    ) -> Result<DnsLookupResult, Box<dyn std::error::Error + Send + Sync>> {
        let (ns, mx, txt) = tokio::join!(
            self.lookup_ns(domain),
            self.lookup_mx(domain),
            self.lookup_txt(domain)
        );

        Ok(DnsLookupResult {
            domain: domain.to_string(),
            ns_records: ns.unwrap_or_default(),
            mx_records: mx.unwrap_or_default(),
            txt_records: txt.unwrap_or_default(),
            dkim_records: Vec::new(),
        })
    }
}
