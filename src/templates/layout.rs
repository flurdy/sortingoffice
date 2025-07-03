use askama::Template;

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate {
    pub title: String,
    pub content: String,
}

#[derive(Template)]
#[template(path = "base.html", escape = "html")]
pub struct LayoutTemplate<'a> {
    pub title: &'a str,
    pub content: &'a str,
}
