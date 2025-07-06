use askama::Template;

#[derive(Template)]
#[template(path = "about.html", escape = "html")]
pub struct AboutTemplate<'a> {
    pub title: &'a str,
} 
