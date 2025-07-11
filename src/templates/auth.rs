use askama::Template;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    pub title: &'a str,
    pub error: &'a str,
    pub login_title: &'a str,
    pub user_id: &'a str,
    pub password: &'a str,
    pub sign_in: &'a str,
    pub app_title: &'a str,
    pub app_subtitle: &'a str,
    pub language_selector: &'a str,
    pub theme_toggle: &'a str,
    pub language_english: &'a str,
    pub language_spanish: &'a str,
    pub language_french: &'a str,
    pub language_norwegian: &'a str,
    pub language_german: &'a str,
    pub current_locale: &'a str,
}
