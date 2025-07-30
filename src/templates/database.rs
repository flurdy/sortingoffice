use crate::config::DatabaseConfig;
use askama::Template;

#[derive(Template)]
#[template(path = "database/selection.html", escape = "html")]
pub struct DatabaseSelectionTemplate<'a> {
    pub databases: &'a [DatabaseConfig],
    pub current_db: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub switch_button: &'a str,
}

#[derive(Template)]
#[template(path = "database/dropdown.html", escape = "html")]
pub struct DatabaseDropdownTemplate<'a> {
    pub databases: &'a [crate::config::DatabaseConfig],
    pub current_db: &'a str,
    pub current_url: &'a str,
}
