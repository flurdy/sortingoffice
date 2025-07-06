use crate::templates::about::AboutTemplate;
use crate::templates::layout::BaseTemplate;
use askama::Template;
use axum::response::Html;

pub async fn index() -> Html<String> {
    let content_template = AboutTemplate {
        title: "About",
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate {
        title: "About".to_string(),
        content,
    };
    Html(template.render().unwrap())
} 
