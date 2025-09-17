use crate::models::{Domain, DomainWizardSession, WizardProgress, WizardSummary};
use askama::Template;

#[derive(Template)]
#[template(path = "wizard/index.html")]
pub struct WizardIndexTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub start_button: &'a str,
    pub step_1_box_title: &'a str,
    pub step_1_box_description: &'a str,
    pub step_2_box_title: &'a str,
    pub step_2_box_description: &'a str,
    pub step_3_box_title: &'a str,
    pub step_3_box_description: &'a str,
}

#[derive(Template)]
#[template(path = "wizard/domain_config.html")]
pub struct WizardDomainConfigTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub form: &'a crate::models::DomainConfigForm,
    pub error: &'a str,
    pub domains_label: &'a str,
    pub domains_description: &'a str,
    pub domains_placeholder: &'a str,
    pub transport_label: &'a str,
    pub transport_description: &'a str,
    pub transport_placeholder: &'a str,
    pub enabled_description: &'a str,
    pub domain_status_label: &'a str,
    pub enabled_label: &'a str,
    pub disabled_label: &'a str,
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
    pub analytics_common_aliases: &'a [String],
    pub config_common_aliases: &'a [String],
    pub required_aliases_label: &'a str,
    pub common_aliases_label: &'a str,
    pub analytics_common_aliases_label: &'a str,
    pub config_common_aliases_label: &'a str,
    pub custom_aliases_label: &'a str,
    pub custom_aliases_placeholder: &'a str,
    pub custom_aliases_description: &'a str,
    pub catchall_title: &'a str,
    pub catchall_description: &'a str,
    pub destination_title: &'a str,
    pub destination_description: &'a str,
    pub destination_placeholder: &'a str,
    pub domains_to_configure_label: &'a str,
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
    pub configuration_summary_title: &'a str,
    pub summary_domains_label: &'a str,
    pub summary_aliases_label: &'a str,
    pub summary_total_label: &'a str,
    pub destination_label: &'a str,
    pub domains_plural: &'a str,
    pub aliases_plural: &'a str,
    pub new_badge: &'a str,
    pub confirm_button: &'a str,
    pub back_button: &'a str,
}

#[derive(Template)]
#[template(path = "wizard/executing.html")]
pub struct WizardExecutingTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub progress: &'a WizardProgress,
    pub creating_domains_text: &'a str,
    pub creating_aliases_text: &'a str,
}

#[derive(Template)]
#[template(path = "wizard/complete.html")]
pub struct WizardCompleteTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub domains_created: i32,
    pub aliases_created: i32,
    pub has_errors: bool,

    pub setup_results_title: &'a str,
    pub domains_created_label: &'a str,
    pub aliases_created_label: &'a str,
    pub domains_plural: &'a str,
    pub created_domains: &'a Vec<String>,
    pub created_domain_ids: &'a Vec<i32>,
    pub created_domains_title: &'a str,
    pub errors_title: &'a str,
    pub errors_description: &'a str,
    pub view_domains_button: &'a str,
    pub new_wizard_button: &'a str,
}

// Duplicate Wizard Templates

#[derive(Template)]
#[template(path = "wizard/duplicate_domain_selection.html")]
pub struct DuplicateDomainSelectionTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub error: &'a str,
    pub domains: &'a [Domain],
    pub source_domain_label: &'a str,
    pub source_domain_placeholder: &'a str,
    pub source_domain_description: &'a str,
    pub new_domain_section_title: &'a str,
    pub new_domain_label: &'a str,
    pub new_domain_placeholder: &'a str,
    pub new_domain_description: &'a str,
    pub enabled_label: &'a str,
    pub next_button: &'a str,
    pub cancel_button: &'a str,
}

#[derive(Template)]
#[template(path = "wizard/duplicate_review.html")]
pub struct DuplicateReviewTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub source_domain_title: &'a str,
    pub source_domain: &'a str,
    pub source_transport: &'a str,
    pub source_enabled: bool,
    pub new_domain_title: &'a str,
    pub new_domain: &'a str,
    pub new_transport: &'a str,
    pub new_enabled: bool,
    pub items_to_duplicate_title: &'a str,
    pub duplicate_aliases: bool,
    pub aliases_count: usize,
    pub aliases_to_duplicate: &'a [crate::models::Alias],
    pub duplicate_relays: bool,
    pub relays_count: usize,
    pub relays_to_duplicate: &'a [crate::models::Relay],
    pub domain_label: &'a str,
    pub transport_label: &'a str,
    pub enabled_label: &'a str,
    pub enabled_status: &'a str,
    pub disabled_status: &'a str,
    pub aliases_label: &'a str,
    pub relays_label: &'a str,
    pub yes_status: &'a str,
    pub no_status: &'a str,
    pub items_label: &'a str,
    pub back_button: &'a str,
    pub cancel_button: &'a str,
    pub confirm_button: &'a str,
}

#[derive(Template)]
#[template(path = "wizard/duplicate_complete.html")]
pub struct DuplicateCompleteTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub success_message: &'a str,
    pub source_domain: &'a str,
    pub new_domain: &'a str,
    pub new_domain_id: i32,
    pub source_label: &'a str,
    pub destination_label: &'a str,
    pub view_domain_button: &'a str,
    pub back_to_domains_button: &'a str,
    pub duplicate_another_button: &'a str,
}
