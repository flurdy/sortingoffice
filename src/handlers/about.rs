use crate::templates::about::AboutTemplate;
use crate::templates::layout::BaseTemplate;
use crate::{AppState, i18n::get_translation};
use askama::Template;
use axum::{extract::State, response::Html};

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let locale = "en-US"; // For now, use default locale
    
    let content_template = AboutTemplate {
        title: &get_translation(&state, locale, "nav-about").await,
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate::with_i18n(
        get_translation(&state, locale, "nav-about").await,
        content,
        &state,
        locale,
    ).await.unwrap();
    
    Html(template.render().unwrap())
} 
