use crate::config::DatabaseConfig;
use askama::Template;

#[derive(Template)]
#[template(path = "database/selection.html", escape = "html")]
pub struct DatabaseSelectionTemplate<'a> {
    pub databases: &'a [DatabaseConfig],
    pub current_db: &'a str,
}
