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
}
