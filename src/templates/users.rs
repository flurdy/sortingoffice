use askama::Template;
use crate::models::User;

#[derive(Template)]
#[template(path = "users/list.html")]
pub struct UserListTemplate {
    pub users: Vec<User>,
}

#[derive(Template)]
#[template(path = "users/show.html")]
pub struct UserShowTemplate {
    pub user: User,
}

#[derive(Template)]
#[template(path = "users/form.html")]
pub struct UserFormTemplate {
    pub user: Option<User>,
} 
