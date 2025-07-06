use crate::models::*;
use askama::Template;

#[derive(Template)]
#[template(path = "backups/list.html")]
pub struct BackupListTemplate {
    pub title: &'static str,
    pub backups: Vec<Backup>,
}

#[derive(Template)]
#[template(path = "backups/show.html")]
pub struct BackupShowTemplate {
    pub title: &'static str,
    pub backup: Backup,
}

#[derive(Template)]
#[template(path = "backups/form.html")]
pub struct BackupFormTemplate {
    pub title: &'static str,
    pub backup: Option<Backup>,
    pub form: BackupForm,
    pub error: Option<String>,
} 
