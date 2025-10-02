use askama::Template;

#[derive(Debug)]
pub struct SelectorDkimRecords {
    pub selector: String,
    pub records: Vec<String>,
}

#[derive(Template)]
#[template(path = "dns/results.html")]
pub struct DnsResultsTemplate {
    pub dns_records_title: String,
    pub dns_ns_header: String,
    pub dns_mx_header: String,
    pub dns_txt_header: String,
    pub dns_dkim_header: String,
    pub dns_dmarc_header: String,
    pub dkim_fallback_description: Option<String>,

    pub ns_records: Vec<String>,
    pub mx_records: Vec<crate::services::dns_lookup::MxRecord>,
    pub txt_records: Vec<String>,
    pub dmarc_records: Vec<String>,

    // Either direct DKIM records for a provided selector, or empty when using fallback
    pub dkim_records: Vec<String>,
    // Fallback per-selector results (only populated when no selector was provided and any were found)
    pub selectors_results: Vec<SelectorDkimRecords>,
}
