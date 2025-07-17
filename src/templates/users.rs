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
    pub users_maildir: String,
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
    pub password_change_required_label: String,
    pub password_change_required_yes: String,
    pub password_change_required_no: String,
    pub password_management_title: String,
    pub change_password_button: String,
    pub require_password_change_button: String,
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
    pub password_management_title: String,
    pub change_password_button: String,
    pub toggle_change_password_button: String,
    pub cancel: String,
    pub create_user: String,
    pub update_user: String,
    pub new_user: String,
    pub edit_user_title: String,
    pub user: Option<User>,
    pub form: UserForm,
    pub error: Option<String>,
    pub users_maildir: String,
    pub users_tooltip_maildir: String,
    pub users_placeholder_maildir: String,
}

#[derive(Template)]
#[template(path = "users/change_password.html", escape = "html")]
pub struct ChangePasswordTemplate {
    pub user: User,
    pub error: Option<String>,
    pub change_password_title: String,
    pub new_password_label: String,
    pub new_password_placeholder: String,
    pub confirm_password_label: String,
    pub confirm_password_placeholder: String,
    pub cancel_button: String,
    pub change_password_button: String,
}
