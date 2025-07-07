use crate::models::*;
use askama::Template;



#[derive(Template)]
#[template(path = "backups/show.html")]
pub struct BackupShowTemplate {
    pub title: String,
    pub view_edit_settings: String,
    pub back_to_domains: String,
    pub backup_information: String,
    pub backup_details: String,
    pub domain: String,
    pub transport: String,
    pub status: String,
    pub created: String,
    pub modified: String,
    pub status_active: String,
    pub status_inactive: String,
    pub edit_backup: String,
    pub enable_backup: String,
    pub disable_backup: String,
    pub delete_backup: String,
    pub delete_confirm: String,
    pub backup: Backup,
}

#[derive(Template)]
#[template(path = "backups/form.html")]
pub struct BackupFormTemplate {
    pub title: String,
    pub form_error: String,
    pub form_domain: String,
    pub form_transport: String,
    pub form_active: String,
    pub placeholder_domain: String,
    pub placeholder_transport: String,
    pub tooltip_domain: String,
    pub tooltip_transport: String,
    pub tooltip_active: String,
    pub cancel: String,
    pub create_backup: String,
    pub update_backup: String,
    pub new_backup: String,
    pub edit_backup_title: String,
    pub backup: Option<Backup>,
    pub form: BackupForm,
    pub error: Option<String>,
} 
