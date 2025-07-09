use crate::templates::auth::LoginTemplate;
use crate::{AppState};
use crate::config::AdminRole;
use askama::Template;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, Response},
    Form,
};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn login_form(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    
    let title = crate::i18n::get_translation(&state, &locale, "login-title").await;
    let user_id = crate::i18n::get_translation(&state, &locale, "login-user-id").await;
    let password = crate::i18n::get_translation(&state, &locale, "login-password").await;
    let sign_in = crate::i18n::get_translation(&state, &locale, "login-sign-in").await;
    let app_title = crate::i18n::get_translation(&state, &locale, "app-title").await;
    let app_subtitle = crate::i18n::get_translation(&state, &locale, "app-subtitle").await;
    let language_selector = crate::i18n::get_translation(&state, &locale, "language-selector").await;
    let theme_toggle = crate::i18n::get_translation(&state, &locale, "theme-toggle").await;
    let language_english = crate::i18n::get_translation(&state, &locale, "language-english").await;
    let language_spanish = crate::i18n::get_translation(&state, &locale, "language-spanish").await;
    let language_french = crate::i18n::get_translation(&state, &locale, "language-french").await;
    let language_norwegian = crate::i18n::get_translation(&state, &locale, "language-norwegian").await;
    
    let template = LoginTemplate {
        title: &title,
        error: "",
        login_title: &title,
        user_id: &user_id,
        password: &password,
        sign_in: &sign_in,
        app_title: &app_title,
        app_subtitle: &app_subtitle,
        language_selector: &language_selector,
        theme_toggle: &theme_toggle,
        language_english: &language_english,
        language_spanish: &language_spanish,
        language_french: &language_french,
        language_norwegian: &language_norwegian,
        current_locale: &locale,
    };
    
    Html(template.render().unwrap())
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub id: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(request): Form<LoginRequest>,
) -> Result<Response, Html<String>> {
    println!("🔐 [LOGIN DEBUG] Login attempt received");
    println!("🔐 [LOGIN DEBUG] Request ID: '{}'", request.id);
    println!("🔐 [LOGIN DEBUG] Password length: {} characters", request.password.len());
    println!("🔐 [LOGIN DEBUG] Is HTMX request: {}", headers.get("hx-request").is_some());
    
    let locale = crate::handlers::language::get_user_locale(&headers);
    let is_htmx = headers.get("hx-request").is_some();
    
    // Validate input
    if request.id.trim().is_empty() || request.password.trim().is_empty() {
        println!("🔐 [LOGIN DEBUG] ❌ Empty fields detected - ID empty: {}, Password empty: {}", 
                 request.id.trim().is_empty(), request.password.trim().is_empty());
        let error = crate::i18n::get_translation(&state, &locale, "login-error-empty-fields").await;
        
        if is_htmx {
            // Return just the error message for HTMX requests
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(format!(
                    r#"<div id="error-message">
                        <div class="bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 px-4 py-3 rounded mb-4">
                            <div class="flex">
                                <div class="flex-shrink-0">
                                    <svg class="h-5 w-5 text-red-400 dark:text-red-300" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                    </svg>
                                </div>
                                <div class="ml-3">
                                    <p class="text-sm">{}</p>
                                </div>
                            </div>
                        </div>
                    </div>"#,
                    error
                ).into())
                .unwrap());
        } else {
            // Return full page for regular requests
            let title = crate::i18n::get_translation(&state, &locale, "login-title").await;
            let user_id = crate::i18n::get_translation(&state, &locale, "login-user-id").await;
            let password = crate::i18n::get_translation(&state, &locale, "login-password").await;
            let sign_in = crate::i18n::get_translation(&state, &locale, "login-sign-in").await;
            let app_title = crate::i18n::get_translation(&state, &locale, "app-title").await;
            let app_subtitle = crate::i18n::get_translation(&state, &locale, "app-subtitle").await;
            let language_selector = crate::i18n::get_translation(&state, &locale, "language-selector").await;
            let theme_toggle = crate::i18n::get_translation(&state, &locale, "theme-toggle").await;
            let language_english = crate::i18n::get_translation(&state, &locale, "language-english").await;
            let language_spanish = crate::i18n::get_translation(&state, &locale, "language-spanish").await;
            let language_french = crate::i18n::get_translation(&state, &locale, "language-french").await;
            let language_norwegian = crate::i18n::get_translation(&state, &locale, "language-norwegian").await;

            let template = LoginTemplate {
                title: &title,
                error: &error,
                login_title: &title,
                user_id: &user_id,
                password: &password,
                sign_in: &sign_in,
                app_title: &app_title,
                app_subtitle: &app_subtitle,
                language_selector: &language_selector,
                theme_toggle: &theme_toggle,
                language_english: &language_english,
                language_spanish: &language_spanish,
                language_french: &language_french,
                language_norwegian: &language_norwegian,
                current_locale: &locale,
            };
            return Err(Html(template.render().unwrap()));
        }
    }
    
    // Verify admin credentials from config
    println!("🔐 [LOGIN DEBUG] Calling verify_admin_credentials with trimmed ID: '{}'", request.id.trim());
    if let Some(role) = state.config.verify_admin_credentials(&request.id.trim(), &request.password) {
        println!("🔐 [LOGIN DEBUG] ✅ Authentication successful! Role: {:?}", role);
        // Set authentication cookie with role
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiry = now + (24 * 60 * 60); // 24 hours
        let role_str = match role {
            AdminRole::ReadOnly => "read-only",
            AdminRole::Edit => "edit",
        };
        let cookie_value = format!("authenticated={}:{}; Path=/; Max-Age=86400; HttpOnly; SameSite=Lax", expiry, role_str);
        println!("🔐 [LOGIN DEBUG] Setting authentication cookie: {}", cookie_value);
        if is_htmx {
            // For htmx, use HX-Redirect header to force a full page reload
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("HX-Redirect", "/")
                .header("Set-Cookie", cookie_value)
                .body("".into())
                .unwrap());
        } else {
            // For normal requests, use 302 redirect
            return Ok(Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/")
                .header("Set-Cookie", cookie_value)
                .body("".into())
                .unwrap());
        }
    }

    println!("🔐 [LOGIN DEBUG] ❌ Authentication failed - no valid credentials found");

    let error = crate::i18n::get_translation(&state, &locale, "login-error-invalid-credentials").await;
    
    if is_htmx {
        // Return just the error message for HTMX requests
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(format!(
                r#"<div id="error-message">
                    <div class="bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 px-4 py-3 rounded mb-4">
                        <div class="flex">
                            <div class="flex-shrink-0">
                                <svg class="h-5 w-5 text-red-400 dark:text-red-300" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                </svg>
                            </div>
                            <div class="ml-3">
                                <p class="text-sm">{}</p>
                            </div>
                        </div>
                    </div>
                </div>"#,
                error
            ).into())
            .unwrap())
    } else {
        // Return full page for regular requests
        let title = crate::i18n::get_translation(&state, &locale, "login-title").await;
        let user_id = crate::i18n::get_translation(&state, &locale, "login-user-id").await;
        let password = crate::i18n::get_translation(&state, &locale, "login-password").await;
        let sign_in = crate::i18n::get_translation(&state, &locale, "login-sign-in").await;
        let app_title = crate::i18n::get_translation(&state, &locale, "app-title").await;
        let app_subtitle = crate::i18n::get_translation(&state, &locale, "app-subtitle").await;
        let language_selector = crate::i18n::get_translation(&state, &locale, "language-selector").await;
        let theme_toggle = crate::i18n::get_translation(&state, &locale, "theme-toggle").await;
        let language_english = crate::i18n::get_translation(&state, &locale, "language-english").await;
        let language_spanish = crate::i18n::get_translation(&state, &locale, "language-spanish").await;
        let language_french = crate::i18n::get_translation(&state, &locale, "language-french").await;
        let language_norwegian = crate::i18n::get_translation(&state, &locale, "language-norwegian").await;

    let template = LoginTemplate {
            title: &title,
            error: &error,
            login_title: &title,
            user_id: &user_id,
            password: &password,
            sign_in: &sign_in,
            app_title: &app_title,
            app_subtitle: &app_subtitle,
            language_selector: &language_selector,
            theme_toggle: &theme_toggle,
            language_english: &language_english,
            language_spanish: &language_spanish,
            language_french: &language_french,
            language_norwegian: &language_norwegian,
            current_locale: &locale,
    };
    Err(Html(template.render().unwrap()))
}
}

pub async fn logout() -> Response {
    // Clear authentication cookie
    let cookie_value = "authenticated=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax";
    
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/login")
        .header("Set-Cookie", cookie_value)
        .body("".into())
        .unwrap()
}

/// Check if user is authenticated and return their role
pub fn get_user_role(headers: &HeaderMap) -> Option<AdminRole> {
    println!("🔐 [COOKIE DEBUG] Checking authentication cookie");
    
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            println!("🔐 [COOKIE DEBUG] Raw cookie header: '{}'", cookie_str);
            
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                println!("🔐 [COOKIE DEBUG] Processing cookie: '{}'", cookie);
                
                if cookie.starts_with("authenticated=") {
                    println!("🔐 [COOKIE DEBUG] Found authenticated cookie!");
                    // Correctly extract the value after 'authenticated='
                    let value_part = &cookie[14..].split(';').next().unwrap_or("");
                    println!("🔐 [COOKIE DEBUG] Cookie value part: '{}'", value_part);
                    
                    let parts: Vec<&str> = value_part.split(':').collect();
                    println!("🔐 [COOKIE DEBUG] Cookie parts: {:?}", parts);
                    
                    if parts.len() >= 2 {
                        if let Ok(expiry) = parts[0].parse::<u64>() {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            
                            println!("🔐 [COOKIE DEBUG] Cookie expiry: {}, Current time: {}", expiry, now);
                            
                            if expiry > now {
                                println!("🔐 [COOKIE DEBUG] Cookie is valid, parsing role: '{}'", parts[1]);
                                // Parse role
                                match parts[1] {
                                    "read-only" => {
                                        println!("🔐 [COOKIE DEBUG] ✅ Returning ReadOnly role");
                                        return Some(AdminRole::ReadOnly);
                                    }
                                    "edit" => {
                                        println!("🔐 [COOKIE DEBUG] ✅ Returning Edit role");
                                        return Some(AdminRole::Edit);
                                    }
                                    _ => {
                                        println!("🔐 [COOKIE DEBUG] ❌ Unknown role: '{}'", parts[1]);
                                        return None;
                                    }
                                }
                            } else {
                                println!("🔐 [COOKIE DEBUG] ❌ Cookie has expired");
                            }
                        } else {
                            println!("🔐 [COOKIE DEBUG] ❌ Failed to parse expiry: '{}'", parts[0]);
                        }
                    } else {
                        println!("🔐 [COOKIE DEBUG] ❌ Invalid cookie format - not enough parts");
                    }
                }
            }
        } else {
            println!("🔐 [COOKIE DEBUG] ❌ Failed to convert cookie header to string");
        }
    } else {
        println!("🔐 [COOKIE DEBUG] ❌ No cookie header found");
    }
    
    println!("🔐 [COOKIE DEBUG] ❌ No valid authentication found");
    None
}

/// Check if user is authenticated
pub fn is_authenticated(headers: &HeaderMap) -> bool {
    let result = get_user_role(headers).is_some();
    println!("🔐 [AUTH DEBUG] is_authenticated: {}", result);
    result
}

/// Check if user has edit permissions
pub fn has_edit_permissions(headers: &HeaderMap) -> bool {
    let result = matches!(get_user_role(headers), Some(AdminRole::Edit));
    println!("🔐 [AUTH DEBUG] has_edit_permissions: {}", result);
    result
}

/// Authentication middleware
pub async fn require_auth(
    State(_state): State<AppState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    println!("🔐 [MIDDLEWARE DEBUG] require_auth checking path: {}", path);
    
    if is_authenticated(&headers) {
        println!("🔐 [MIDDLEWARE DEBUG] ✅ User is authenticated, allowing access to: {}", path);
        Ok(next.run(request).await)
    } else {
        println!("🔐 [MIDDLEWARE DEBUG] ❌ User not authenticated, redirecting to login from: {}", path);
        // Redirect to login
        Ok(Response::builder()
            .status(StatusCode::FOUND)
            .header("Location", "/login")
            .body("".into())
            .unwrap())
    }
}

/// Edit permissions middleware
pub async fn require_edit_permissions(
    State(_state): State<AppState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    println!("🔐 [MIDDLEWARE DEBUG] require_edit_permissions checking path: {}", path);
    
    if has_edit_permissions(&headers) {
        println!("🔐 [MIDDLEWARE DEBUG] ✅ User has edit permissions, allowing access to: {}", path);
        Ok(next.run(request).await)
    } else {
        println!("🔐 [MIDDLEWARE DEBUG] ❌ User lacks edit permissions, denying access to: {}", path);
        // Return 403 Forbidden
        Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("Insufficient permissions".into())
            .unwrap())
    }
}
