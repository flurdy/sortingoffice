use axum::http::HeaderMap;

/// Check if the request is an HTMX request
pub fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .map(|v| v.as_bytes() == b"true")
        .unwrap_or(false)
}

/// Get user locale from headers
pub fn get_user_locale(headers: &HeaderMap) -> String {
    headers
        .get("Accept-Language")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("en")
        .to_string()
}
