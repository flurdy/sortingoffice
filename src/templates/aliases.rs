use askama::Template;
use crate::models::Alias;

#[derive(Template)]
#[template(path = "aliases/list.html")]
pub struct AliasListTemplate {
    pub aliases: Vec<Alias>,
}

#[derive(Template)]
#[template(path = "aliases/show.html")]
pub struct AliasShowTemplate {
    pub alias: Alias,
}

#[derive(Template)]
#[template(path = "aliases/form.html")]
pub struct AliasFormTemplate {
    pub alias: Option<Alias>,
} 
