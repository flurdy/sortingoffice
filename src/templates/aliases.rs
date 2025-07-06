use crate::models::{Alias, AliasForm};
use askama::Template;

#[derive(Template)]
#[template(path = "aliases/list.html", escape = "html")]
pub struct AliasListTemplate<'a> {
    pub title: &'a str,
    pub aliases: Vec<Alias>,
}

#[derive(Template)]
#[template(path = "aliases/show.html", escape = "html")]
pub struct AliasShowTemplate<'a> {
    pub title: &'a str,
    pub alias: Alias,
}

#[derive(Template)]
#[template(path = "aliases/form.html", escape = "html")]
pub struct AliasFormTemplate<'a> {
    pub title: &'a str,
    pub alias: Option<Alias>,
    pub form: AliasForm,
    pub error: Option<String>,
}
