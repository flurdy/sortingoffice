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
    pub nav_backups: String,
    pub nav_aliases: String,
    pub nav_users: String,
    pub nav_statistics: String,
    pub nav_about: String,
    pub nav_logout: String,
    pub theme_toggle: String,
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
    pub nav_backups: &'a str,
    pub nav_aliases: &'a str,
    pub nav_users: &'a str,
    pub nav_statistics: &'a str,
    pub nav_about: &'a str,
    pub nav_logout: &'a str,
    pub theme_toggle: &'a str,
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
            nav_backups: crate::i18n::get_translation(state, locale, "nav-backups").await,
            nav_aliases: crate::i18n::get_translation(state, locale, "nav-aliases").await,
            nav_users: crate::i18n::get_translation(state, locale, "nav-users").await,
            nav_statistics: crate::i18n::get_translation(state, locale, "nav-statistics").await,
            nav_about: crate::i18n::get_translation(state, locale, "nav-about").await,
            nav_logout: crate::i18n::get_translation(state, locale, "nav-logout").await,
            theme_toggle: crate::i18n::get_translation(state, locale, "theme-toggle").await,
        })
    }
}
