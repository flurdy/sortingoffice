use askama::Template;

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate {
    pub title: String,
    pub content: String,
}

#[derive(Template)]
#[template(path = "layout.html")]
pub struct LayoutTemplate {
    pub title: String,
    pub content: String,
} 
