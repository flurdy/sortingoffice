use crate::AppState;
use askama::Template;
use axum::http::HeaderMap;
use axum::response::Html;
use tracing::error;

/// Helper function to render template safely with error handling
pub fn render_template_safely<T: Template>(template: T) -> Result<String, String> {
    template.render().map_err(|e| {
        error!("Template rendering failed: {:?}", e);
        format!("Template rendering error: {e}")
    })
}

/// Helper function to render template safely and return Html or error
/// Note: This function should be used carefully as it doesn't have access to state and headers
/// for proper error page rendering. Consider using render_template_safely directly with
/// proper error page handling in the calling function.
pub fn render_template_to_html<T: Template>(template: T) -> Html<String> {
    match render_template_safely(template) {
        Ok(content) => Html(content),
        Err(_) => Html("Template rendering error".to_string()),
    }
}

/// Helper function to render template with layout
pub async fn render_template_with_layout<T>(
    template: T,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String>
where
    T: Template,
{
    let content = match render_template_safely(template) {
        Ok(content) => content,
        Err(_) => return crate::handlers::errors::render_500_page(state, headers).await,
    };

    if crate::handlers::http_helpers::is_htmx_request(headers) {
        Html(content)
    } else {
        let current_db_id = crate::handlers::auth::get_selected_database(headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = crate::templates::layout::BaseTemplate::with_i18n(
            crate::i18n::get_translation(state, locale, "aliases-title").await,
            content,
            state,
            locale,
            current_db_label,
            current_db_id,
        )
        .await
        .unwrap();
        match render_template_safely(template) {
            Ok(content) => Html(content),
            Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
        }
    }
}

/// Helper function to render list template
pub async fn render_list_template<T>(
    template: T,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String>
where
    T: Template,
{
    let content = match render_template_safely(template) {
        Ok(content) => content,
        Err(_) => return crate::handlers::errors::render_500_page(state, headers).await,
    };

    if crate::handlers::http_helpers::is_htmx_request(headers) {
        Html(content)
    } else {
        let current_db_id = crate::handlers::auth::get_selected_database(headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = crate::templates::layout::BaseTemplate::with_i18n(
            crate::i18n::get_translation(state, locale, "aliases-title").await,
            content,
            state,
            locale,
            current_db_label,
            current_db_id,
        )
        .await
        .unwrap();
        match render_template_safely(template) {
            Ok(content) => Html(content),
            Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
        }
    }
}

/// Helper function to render show template
pub async fn render_show_template<T>(
    template: T,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String>
where
    T: Template,
{
    let content = match render_template_safely(template) {
        Ok(content) => content,
        Err(_) => return crate::handlers::errors::render_500_page(state, headers).await,
    };

    if crate::handlers::http_helpers::is_htmx_request(headers) {
        Html(content)
    } else {
        let current_db_id = crate::handlers::auth::get_selected_database(headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = crate::templates::layout::BaseTemplate::with_i18n(
            crate::i18n::get_translation(state, locale, "aliases-show-title").await,
            content,
            state,
            locale,
            current_db_label,
            current_db_id,
        )
        .await
        .unwrap();
        match render_template_safely(template) {
            Ok(content) => Html(content),
            Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
        }
    }
}

/// Helper function to render form template
pub async fn render_form_template<T>(
    template: T,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
    title: String,
) -> Html<String>
where
    T: askama::Template,
{
    let content = match render_template_safely(template) {
        Ok(content) => content,
        Err(_) => return crate::handlers::errors::render_500_page(state, headers).await,
    };

    if crate::handlers::http_helpers::is_htmx_request(headers) {
        Html(content)
    } else {
        let current_db_id = crate::handlers::auth::get_selected_database(headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = crate::templates::layout::BaseTemplate::with_i18n(
            title,
            content,
            state,
            locale,
            current_db_label,
            current_db_id,
        )
        .await
        .unwrap();
        match render_template_safely(template) {
            Ok(content) => Html(content),
            Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
        }
    }
}

/// Helper function to create base template
pub async fn create_base_template(
    state: &AppState,
    locale: &str,
    title: String,
    content: String,
    headers: &HeaderMap,
) -> Result<Html<String>, Box<dyn std::error::Error>> {
    let current_db_id = crate::handlers::auth::get_selected_database(headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());

    let template = crate::templates::layout::BaseTemplate::with_i18n(
        title,
        content,
        state,
        locale,
        current_db_label,
        current_db_id,
    )
    .await?;

    let rendered = render_template_safely(template)?;
    Ok(Html(rendered))
}
