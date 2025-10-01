use askama::Template;
use axum::{http::HeaderMap, response::Html};

use crate::AppState;

#[derive(serde::Deserialize, Default)]
pub struct DnsLookupForm {
    pub selector: Option<String>,
}

pub async fn render_dns_fragment(
    state: &AppState,
    headers: &HeaderMap,
    domain_name: &str,
    selector: Option<String>,
) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(headers);

    let resolver = match crate::services::dns_lookup::DnsLookupService::new_system().await {
        Ok(r) => r,
        Err(_) => return crate::handlers::errors::render_500_page(state, headers).await,
    };

    let result = match resolver.lookup_all(domain_name).await {
        Ok(r) => r,
        Err(_) => crate::services::dns_lookup::DnsLookupResult::default(),
    };

    // Gather DKIM data
    let mut dkim_records: Vec<String> = Vec::new();
    let mut selectors_results: Vec<crate::templates::dns::SelectorDkimRecords> = Vec::new();
    let mut dkim_fallback_description: Option<String> = None;

    if let Some(sel) = selector.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        if let Ok(records) = resolver.lookup_dkim(sel, domain_name).await {
            dkim_records = records;
        }
    } else {
        let common_selectors = [
            "s1",
            "s2",
            "default",
            "selector1",
            "selector",
            "k1",
            "google",
        ];
        for s in common_selectors.iter() {
            if let Ok(records) = resolver.lookup_dkim(s, domain_name).await {
                if !records.is_empty() {
                    selectors_results.push(crate::templates::dns::SelectorDkimRecords {
                        selector: s.to_string(),
                        records,
                    });
                }
            }
        }
        if !selectors_results.is_empty() {
            dkim_fallback_description = Some(
                crate::i18n::get_translation(state, &locale, "dns-dkim-fallback-description").await,
            );
        }
    }

    // Build Askama template
    let tpl = crate::templates::dns::DnsResultsTemplate {
        dns_records_title: crate::i18n::get_translation(state, &locale, "dns-records-title").await,
        dns_ns_header: crate::i18n::get_translation(state, &locale, "dns-ns-header").await,
        dns_mx_header: crate::i18n::get_translation(state, &locale, "dns-mx-header").await,
        dns_txt_header: crate::i18n::get_translation(state, &locale, "dns-txt-header").await,
        dns_dkim_header: crate::i18n::get_translation(state, &locale, "dns-dkim-header").await,
        dkim_fallback_description,
        ns_records: result.ns_records,
        mx_records: result.mx_records,
        txt_records: result.txt_records,
        dkim_records,
        selectors_results,
    };

    match tpl.render() {
        Ok(html) => Html(html),
        Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
    }
}
