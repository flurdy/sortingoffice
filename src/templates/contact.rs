use crate::config::ContactInfo;
use askama::Template;

#[derive(Template)]
#[template(path = "contact.html", escape = "html")]
pub struct ContactTemplate<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    pub app_maintainer_title: &'a str,
    pub app_maintainer_desc: &'a str,
    pub project_maintainer_title: &'a str,
    pub project_maintainer_desc: &'a str,
    pub contact_name: &'a str,
    pub contact_email: &'a str,
    pub contact_role: &'a str,
    pub github_issues: &'a str,
    pub github_issues_desc: &'a str,
    pub security_advisories: &'a str,
    pub security_advisories_desc: &'a str,
    pub contact_form: &'a str,
    pub contact_form_desc: &'a str,
    pub pgp_keys: &'a str,
    pub pgp_keys_desc: &'a str,
    pub documentation: &'a str,
    pub documentation_desc: &'a str,
    pub sponsorship: &'a str,
    pub sponsorship_desc: &'a str,
    pub app_contact: Option<ContactInfo>,
}
