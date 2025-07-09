use crate::config::Config;
use askama::Template;

#[derive(Template)]
#[template(path = "config.html", escape = "html")]
pub struct ConfigTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub required_aliases_header: &'a str,
    pub common_aliases_header: &'a str,
    pub domain_overrides_header: &'a str,
    pub save_button: &'a str,
    pub cancel_button: &'a str,
    pub add_required_alias_button: &'a str,
    pub add_common_alias_button: &'a str,
    pub remove_alias_button: &'a str,
    pub promote_button: &'a str,
    pub demote_button: &'a str,
    pub required_aliases_description: &'a str,
    pub common_aliases_description: &'a str,
    pub domain_overrides_description: &'a str,
    pub add_domain_override_button: &'a str,
    pub remove_domain_button: &'a str,
    pub required_aliases_label: &'a str,
    pub common_aliases_label: &'a str,
    pub remove_button: &'a str,
    pub add_alias_button: &'a str,
    pub placeholder_required_alias: &'a str,
    pub placeholder_common_alias: &'a str,
    pub placeholder_domain: &'a str,
    pub placeholder_domain_alias: &'a str,
    pub config: &'a Config,
    pub domain_overrides_vec: Vec<(&'a String, &'a crate::config::DomainOverride)>,
}
