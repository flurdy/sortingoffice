use crate::models::{DomainWizardSession, WizardProgress, WizardSummary};
use askama::Template;

#[derive(Template)]
#[template(path = "wizard/index.html")]
pub struct WizardIndexTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub start_button: &'a str,
}

#[derive(Template)]
#[template(path = "wizard/domain_config.html")]
pub struct WizardDomainConfigTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub form: &'a crate::models::DomainConfigForm,
    pub error: &'a str,
    pub domains_placeholder: &'a str,
    pub transport_placeholder: &'a str,
    pub next_button: &'a str,
    pub cancel_button: &'a str,
}

#[derive(Template)]
#[template(path = "wizard/alias_config.html")]
pub struct WizardAliasConfigTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub domains: &'a [String],
    pub form: &'a crate::models::AliasConfigForm,
    pub error: &'a str,
    pub required_aliases: &'a [String],
    pub common_aliases: &'a [String],
    pub required_aliases_label: &'a str,
    pub common_aliases_label: &'a str,
    pub custom_aliases_placeholder: &'a str,
    pub destination_placeholder: &'a str,
    pub next_button: &'a str,
    pub back_button: &'a str,
}

#[derive(Template)]
#[template(path = "wizard/review.html")]
pub struct WizardReviewTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub session: &'a DomainWizardSession,
    pub summary: &'a WizardSummary,
    pub summary_domains_label: &'a str,
    pub summary_aliases_label: &'a str,
    pub summary_total_label: &'a str,
    pub destination_label: &'a str,
    pub confirm_button: &'a str,
    pub back_button: &'a str,
}

#[derive(Template)]
#[template(path = "wizard/executing.html")]
pub struct WizardExecutingTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub progress: &'a WizardProgress,
}

#[derive(Template)]
#[template(path = "wizard/complete.html")]
pub struct WizardCompleteTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub domains_created: i32,
    pub aliases_created: i32,
    pub has_errors: bool,
    pub view_domains_button: &'a str,
    pub new_wizard_button: &'a str,
}
