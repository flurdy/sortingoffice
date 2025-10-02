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

    // Use short-lived cache for DNS lookups
    let ttl = std::time::Duration::from_secs(300);
    let ns_records = if let Some(cached) = state.db_manager.get_dns_ns(domain_name).await {
        cached
    } else {
        let fetched = resolver.lookup_ns(domain_name).await.unwrap_or_default();
        state.db_manager.set_dns_ns(domain_name, fetched.clone(), ttl).await;
        fetched
    };

    let mx_records = if let Some(cached) = state.db_manager.get_dns_mx(domain_name).await {
        cached
    } else {
        let fetched = resolver.lookup_mx(domain_name).await.unwrap_or_default();
        state.db_manager.set_dns_mx(domain_name, fetched.clone(), ttl).await;
        fetched
    };

    let txt_records = if let Some(cached) = state.db_manager.get_dns_txt(domain_name).await {
        cached
    } else {
        let fetched = resolver.lookup_txt(domain_name).await.unwrap_or_default();
        state.db_manager.set_dns_txt(domain_name, fetched.clone(), ttl).await;
        fetched
    };

    // DMARC TXT (_dmarc.domain)
    let dmarc_host = format!("_dmarc.{}", domain_name);
    let dmarc_records = if let Some(cached) = state.db_manager.get_dns_txt(&dmarc_host).await {
        cached
    } else {
        let fetched = resolver.lookup_dmarc(domain_name).await.unwrap_or_default();
        if !fetched.is_empty() {
            state
                .db_manager
                .set_dns_txt(&dmarc_host, fetched.clone(), ttl)
                .await;
        }
        fetched
    };

    // Gather DKIM data
    let mut dkim_records: Vec<String> = Vec::new();
    let mut selectors_results: Vec<crate::templates::dns::SelectorDkimRecords> = Vec::new();
    let mut dkim_fallback_description: Option<String> = None;

    if let Some(sel) = selector.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let cache_key = format!("{}:{}", sel, domain_name);
        if let Some(records) = state.db_manager.get_dns_dkim(&cache_key).await {
            dkim_records = records;
        } else if let Ok(records) = resolver.lookup_dkim(sel, domain_name).await {
            dkim_records = records;
            state
                .db_manager
                .set_dns_dkim(&cache_key, dkim_records.clone(), ttl)
                .await;
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
            let cache_key = format!("{}:{}", s, domain_name);
            if let Some(records) = state.db_manager.get_dns_dkim(&cache_key).await {
                if !records.is_empty() {
                    selectors_results.push(crate::templates::dns::SelectorDkimRecords {
                        selector: s.to_string(),
                        records,
                    });
                }
            } else if let Ok(records) = resolver.lookup_dkim(s, domain_name).await {
                if !records.is_empty() {
                    selectors_results.push(crate::templates::dns::SelectorDkimRecords {
                        selector: s.to_string(),
                        records,
                    });
                    // Store DKIM cache
                    if let Some(last) = selectors_results.last() {
                        state
                            .db_manager
                            .set_dns_dkim(&cache_key, last.records.clone(), ttl)
                            .await;
                    }
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
        dns_dmarc_header: crate::i18n::get_translation(state, &locale, "dns-dmarc-header").await,
        dkim_fallback_description,
        ns_records,
        mx_records,
        txt_records,
        dmarc_records,
        dkim_records,
        selectors_results,
    };

    match tpl.render() {
        Ok(html) => Html(html),
        Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
    }
}
