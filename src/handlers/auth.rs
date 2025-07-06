use crate::templates::auth::LoginTemplate;
use crate::{db, AppState};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};
use bcrypt;
use serde::Deserialize;

pub async fn login_form() -> Html<String> {
    let template = LoginTemplate {
        title: "Login",
        error: "",
    };
    Html(template.render().unwrap())
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub id: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Form(request): Form<LoginRequest>,
) -> Result<Redirect, Html<String>> {
    let pool = &state.pool;
    if let Ok(user) = db::get_user_by_id(pool, &request.id) {
        if bcrypt::verify(&request.password, &user.password).unwrap_or(false) {
            // TODO: Set session
            return Ok(Redirect::to("/"));
        }
    }

    let template = LoginTemplate {
        title: "Login",
        error: "Invalid id or password",
    };
    Err(Html(template.render().unwrap()))
}

pub async fn logout() -> Redirect {
    Redirect::to("/login")
}
