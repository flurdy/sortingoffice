use crate::models::DomainPresenceType::{Backup, Primary};
use crate::models::MigrationStatus;
use askama::Template;

#[derive(Template)]
#[template(path = "reports/matrix.html", escape = "html")]
pub struct MatrixReportTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub domain_header: &'a str,
    pub catch_all_header: &'a str,
    pub required_aliases_header: &'a str,
    pub status_present: &'a str,
    pub status_missing: &'a str,
    pub status_disabled: &'a str,
    pub legend_title: &'a str,
    pub no_domains: &'a str,
    pub no_domains_description: &'a str,
    pub report: &'a crate::models::DomainAliasMatrixReport,
}

#[derive(Template)]
#[template(path = "reports/list.html", escape = "html")]
pub struct ReportsListTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub matrix_report_title: &'a str,
    pub matrix_report_description: &'a str,
    pub orphaned_aliases_report_title: &'a str,
    pub orphaned_aliases_report_description: &'a str,
    pub external_forwarders_report_title: &'a str,
    pub external_forwarders_report_description: &'a str,
    pub alias_cross_domain_report_title: &'a str,
    pub alias_cross_domain_report_description: &'a str,
    pub cross_database_matrix_report_title: &'a str,
    pub cross_database_matrix_report_description: &'a str,
    pub cross_database_user_distribution_report_title: &'a str,
    pub cross_database_user_distribution_report_description: &'a str,
    pub cross_database_feature_toggle_report_title: &'a str,
    pub cross_database_feature_toggle_report_description: &'a str,
    pub cross_database_migration_report_title: &'a str,
    pub cross_database_migration_report_description: &'a str,
    pub view_report: &'a str,
}

#[derive(Template)]
#[template(path = "reports/orphaned.html", escape = "html")]
pub struct OrphanedReportTemplate<'a> {
    pub title: &'a str,
    pub report: &'a crate::models::OrphanedAliasReport,
}

#[derive(Template)]
#[template(path = "reports/external_forwarders.html", escape = "html")]
pub struct ExternalForwarderReportTemplate<'a> {
    pub title: &'a str,
    pub report: &'a crate::models::ExternalForwarderReport,
}

#[derive(Template)]
#[template(path = "reports/alias_cross_domain.html", escape = "html")]
pub struct AliasCrossDomainReportTemplate<'a> {
    pub title: &'a str,
    pub report: &'a crate::models::AliasCrossDomainReport,
}

#[derive(Template)]
#[template(path = "reports/cross_database_matrix.html", escape = "html")]
pub struct CrossDatabaseMatrixReportTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub domain_header: &'a str,
    pub database_header: &'a str,
    pub primary_domain: &'a str,
    pub backup_domain: &'a str,
    pub not_present: &'a str,
    pub legend_title: &'a str,
    pub no_domains: &'a str,
    pub no_domains_description: &'a str,
    pub report: &'a crate::models::CrossDatabaseDomainMatrixReport,
}

#[derive(Template)]
#[template(
    path = "reports/cross_database_user_distribution.html",
    escape = "html"
)]
pub struct CrossDatabaseUserDistributionReportTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub user_header: &'a str,
    pub database_header: &'a str,
    pub present: &'a str,
    pub not_present: &'a str,
    pub legend_title: &'a str,
    pub no_users: &'a str,
    pub no_users_description: &'a str,
    pub disabled: &'a str,
    pub report: &'a crate::models::CrossDatabaseUserDistributionReport,
}

#[derive(Template)]
#[template(path = "reports/cross_database_feature_toggle.html", escape = "html")]
pub struct CrossDatabaseFeatureToggleReportTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub database_header: &'a str,
    pub database_status_header: &'a str,
    pub read_only: &'a str,
    pub no_new_users: &'a str,
    pub no_new_domains: &'a str,
    pub no_password_updates: &'a str,
    pub enabled: &'a str,
    pub disabled: &'a str,
    pub report: &'a crate::models::CrossDatabaseFeatureToggleReport,
}

#[derive(Template)]
#[template(path = "reports/cross_database_migration.html", escape = "html")]
pub struct CrossDatabaseMigrationReportTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub database_header: &'a str,
    pub status_header: &'a str,
    pub last_migration_header: &'a str,
    pub migration_count_header: &'a str,
    pub report: &'a crate::models::CrossDatabaseMigrationReport,
}
