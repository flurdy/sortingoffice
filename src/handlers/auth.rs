use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;
use crate::AppState;
use crate::templates::auth::LoginTemplate;

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn login_page() -> Html<String> {
    let template = LoginTemplate {};
    Html(template.render().unwrap())
}

pub async fn login(
    State(_state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Redirect {
    // For now, we'll use a simple admin/admin login
    // In a real application, you'd validate against the database
    if form.username == "admin" && form.password == "admin" {
        Redirect::to("/")
    } else {
        Redirect::to("/login")
    }
}

pub async fn logout() -> Redirect {
    Redirect::to("/login")
} 
