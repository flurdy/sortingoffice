use crate::models::*;
use askama::Template;

#[derive(Template)]
#[template(path = "domain_backup/show.html")]
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
    pub not_available: String,
    pub backup: Backup,
    // Cross-database domain information
    pub cross_database_info: Vec<crate::models::CrossDatabaseDomainInfo>,
    pub other_databases_header: String,
    pub other_databases_description: String,
    pub other_databases_database_label: String,
    pub other_databases_domain_type: String,
    pub other_databases_primary_domain: String,
    pub other_databases_backup_domain: String,
    pub other_databases_users_count: String,
    pub other_databases_aliases_count: String,
    pub status_enabled: String,
    pub status_disabled: String,
    // DNS section
    pub dns_section_header: String,
    pub dns_section_description: String,
    pub dns_lookup_button: String,
    pub dns_loading_label: String,
}

#[derive(Template)]
#[template(path = "domain_backup/form.html")]
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
