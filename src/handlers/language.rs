use crate::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
    Form,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LanguageForm {
    language: String,
}

pub async fn set_language(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<LanguageForm>,
) -> Response {
    // Validate the language
    let locale = match form.language.as_str() {
        "en-US" | "es-ES" | "fr-FR" | "nb-NO" => &form.language,
        _ => "en-US", // Default fallback
    };

    // Load the locale if it's not already loaded
    if locale != "en-US" {
        let _ = state.i18n.load_locale(locale).await;
    }

    // Get the referer to redirect back to the previous page
    let redirect_url = headers
        .get("referer")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("/");

    // Set language cookie and redirect
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", redirect_url)
        .header(
            "Set-Cookie",
            format!("language={locale}; Path=/; Max-Age=31536000; SameSite=Lax"),
        )
        .body("".into())
        .unwrap()
}

// Helper function to get locale from request (cookie, header, or default)
pub fn get_user_locale(headers: &HeaderMap) -> String {
    // First check for language cookie
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("language=") {
                    let language = cookie.strip_prefix("language=").unwrap_or("en-US");
                    // Validate the language
                    if language == "en-US"
                        || language == "es-ES"
                        || language == "fr-FR"
                        || language == "nb-NO"
                    {
                        return language.to_string();
                    }
                }
            }
        }
    }

    // Fall back to Accept-Language header
    crate::i18n::get_locale_from_headers(headers)
}
