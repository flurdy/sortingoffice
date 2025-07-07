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
    pub view_report: &'a str,
} 
