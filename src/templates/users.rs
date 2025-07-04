use askama::Template;
use crate::models::{User, UserForm};

#[derive(Template)]
#[template(path = "users/list.html", escape = "html")]
pub struct UserListTemplate<'a> {
    pub title: &'a str,
    pub users: Vec<User>,
}

#[derive(Template)]
#[template(path = "users/show.html", escape = "html")]
pub struct UserShowTemplate<'a> {
    pub title: &'a str,
    pub user: User,
}

#[derive(Template)]
#[template(path = "users/form.html", escape = "html")]
pub struct UserFormTemplate<'a> {
    pub title: &'a str,
    pub user: Option<User>,
    pub form: UserForm,
    pub error: Option<String>,
} 
