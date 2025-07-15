use crate::models::{PaginatedResult, User, UserForm};
use askama::Template;

#[derive(Template)]
#[template(path = "users/list.html", escape = "html")]
pub struct UsersListTemplate {
    pub title: String,
    pub description: String,
    pub add_user: String,
    pub table_header_username: String,
    pub table_header_domain: String,
    pub table_header_enabled: String,
    pub table_header_actions: String,
    pub status_active: String,
    pub status_inactive: String,
    pub action_view: String,
    pub enable_user: String,
    pub disable_user: String,
    pub empty_title: String,
    pub empty_description: String,
    pub users: Vec<User>,
    pub pagination: PaginatedResult<User>,
    pub page_range: Vec<i64>,
    pub max_item: i64,
}

#[derive(Template)]
#[template(path = "users/show.html", escape = "html")]
pub struct UserShowTemplate {
    pub title: String,
    pub view_edit_settings: String,
    pub back_to_users: String,
    pub user_information: String,
    pub user_details: String,
    pub user_id: String,
    pub full_name: String,

    pub status: String,
    pub created: String,
    pub modified: String,
    pub status_active: String,
    pub status_inactive: String,
    pub edit_user: String,
    pub enable_user: String,
    pub disable_user: String,
    pub delete_user: String,
    pub delete_confirm: String,
    pub user: User,
}

#[derive(Template)]
#[template(path = "users/form.html", escape = "html")]
pub struct UserFormTemplate {
    pub title: String,
    pub form_user_id: String,
    pub form_password: String,
    pub form_name: String,

    pub form_active: String,
    pub placeholder_user_email: String,
    pub placeholder_name: String,

    pub tooltip_user_id: String,
    pub tooltip_password: String,
    pub tooltip_name: String,

    pub tooltip_active: String,
    pub users_change_password: String,
    pub users_change_password_tooltip: String,
    pub users_placeholder_password: String,
    pub cancel: String,
    pub create_user: String,
    pub update_user: String,
    pub new_user: String,
    pub edit_user_title: String,
    pub user: Option<User>,
    pub form: UserForm,
    pub error: Option<String>,
}
