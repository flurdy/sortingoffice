use askama::Template;

#[derive(Template)]
#[template(path = "base.html", escape = "none")]
pub struct BaseTemplate {
    pub title: String,
    pub content: String,
    // i18n fields
    pub app_title: String,
    pub app_subtitle: String,
    pub nav_dashboard: String,
    pub nav_domains: String,
    pub nav_aliases: String,
    pub nav_users: String,
    pub nav_relays: String,
    pub nav_relocated: String,
    pub nav_clients: String,
    pub nav_statistics: String,
    pub nav_reports: String,
    pub nav_config: String,
    pub nav_about: String,
    pub nav_logout: String,
    pub theme_toggle: String,
    pub language_selector: String,
    pub language_english: String,
    pub language_spanish: String,
    pub language_french: String,
    pub language_norwegian: String,
    pub current_locale: String,
}

#[derive(Template)]
#[template(path = "base.html", escape = "html")]
pub struct LayoutTemplate<'a> {
    pub title: &'a str,
    pub content: &'a str,
    // i18n fields
    pub app_title: &'a str,
    pub app_subtitle: &'a str,
    pub nav_dashboard: &'a str,
    pub nav_domains: &'a str,
    pub nav_aliases: &'a str,
    pub nav_users: &'a str,
    pub nav_relays: &'a str,
    pub nav_relocated: &'a str,
    pub nav_clients: &'a str,
    pub nav_statistics: &'a str,
    pub nav_reports: &'a str,
    pub nav_config: &'a str,
    pub nav_about: &'a str,
    pub nav_logout: &'a str,
    pub theme_toggle: &'a str,
    pub language_selector: &'a str,
    pub language_english: &'a str,
    pub language_spanish: &'a str,
    pub language_french: &'a str,
    pub language_norwegian: &'a str,
    pub current_locale: &'a str,
}

impl BaseTemplate {
    pub async fn with_i18n(
        title: String,
        content: String,
        state: &crate::AppState,
        locale: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(BaseTemplate {
            title,
            content,
            app_title: crate::i18n::get_translation(state, locale, "app-title").await,
            app_subtitle: crate::i18n::get_translation(state, locale, "app-subtitle").await,
            nav_dashboard: crate::i18n::get_translation(state, locale, "nav-dashboard").await,
            nav_domains: crate::i18n::get_translation(state, locale, "nav-domains").await,
            nav_aliases: crate::i18n::get_translation(state, locale, "nav-aliases").await,
            nav_users: crate::i18n::get_translation(state, locale, "nav-users").await,
            nav_relays: crate::i18n::get_translation(state, locale, "nav-relays").await,
            nav_relocated: crate::i18n::get_translation(state, locale, "nav-relocated").await,
            nav_clients: crate::i18n::get_translation(state, locale, "nav-clients").await,
            nav_statistics: crate::i18n::get_translation(state, locale, "nav-statistics").await,
            nav_reports: crate::i18n::get_translation(state, locale, "nav-reports").await,
            nav_config: crate::i18n::get_translation(state, locale, "nav-config").await,
            nav_about: crate::i18n::get_translation(state, locale, "nav-about").await,
            nav_logout: crate::i18n::get_translation(state, locale, "nav-logout").await,
            theme_toggle: crate::i18n::get_translation(state, locale, "theme-toggle").await,
            language_selector: crate::i18n::get_translation(state, locale, "language-selector")
                .await,
            language_english: crate::i18n::get_translation(state, locale, "language-english").await,
            language_spanish: crate::i18n::get_translation(state, locale, "language-spanish").await,
            language_french: crate::i18n::get_translation(state, locale, "language-french").await,
            language_norwegian: crate::i18n::get_translation(state, locale, "language-norwegian")
                .await,
            current_locale: locale.to_string(),
        })
    }
}
