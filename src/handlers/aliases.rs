use crate::templates::aliases::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState};
use askama::Template;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
    Form,
};

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").map_or(false, |v| v == "true")
}

pub async fn list(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;

    let aliases = match db::get_aliases(pool) {
        Ok(aliases) => aliases,
        Err(_) => vec![],
    };

    let content_template = AliasListTemplate {
        title: "Aliases",
        aliases,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate {
            title: "Aliases".to_string(),
            content,
        };
        Html(template.render().unwrap())
    }
}

pub async fn new(headers: HeaderMap) -> Html<String> {
    let form = AliasForm {
        mail: "".to_string(),
        destination: "".to_string(),
        domain: "example.com".to_string(),
        enabled: true,
    };

    let content_template = AliasFormTemplate {
        title: "New Alias",
        alias: None,
        form,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate {
            title: "New Alias".to_string(),
            content,
        };
        Html(template.render().unwrap())
    }
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    let alias = match db::get_alias(pool, id) {
        Ok(alias) => alias,
        Err(_) => return Html("Alias not found".to_string()),
    };

    let content_template = AliasShowTemplate {
        title: "Show Alias",
        alias,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate {
            title: "Show Alias".to_string(),
            content,
        };
        Html(template.render().unwrap())
    }
}

pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    let alias = match db::get_alias(pool, id) {
        Ok(alias) => alias,
        Err(_) => return Html("Alias not found".to_string()),
    };

    let form = AliasForm {
        mail: alias.mail.clone(),
        destination: alias.destination.clone(),
        domain: alias.domain.clone(),
        enabled: alias.enabled,
    };

    let content_template = AliasFormTemplate {
        title: "Edit Alias",
        alias: Some(alias),
        form,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate {
            title: "Edit Alias".to_string(),
            content,
        };
        Html(template.render().unwrap())
    }
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<AliasForm>,
) -> Html<String> {
    let pool = &state.pool;

    match db::create_alias(pool, form) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(e) => {
                    eprintln!("Error getting aliases: {:?}", e);
                    vec![]
                }
            };
            let content_template = AliasListTemplate {
                title: "Aliases",
                aliases,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let template = BaseTemplate {
                    title: "Aliases".to_string(),
                    content,
                };
                Html(template.render().unwrap())
            }
        }
        Err(e) => {
            eprintln!("Error creating alias: {:?}", e);
            Html(format!("Error creating alias: {:?}", e))
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<AliasForm>,
) -> Html<String> {
    let pool = &state.pool;

    match db::update_alias(pool, id, form) {
        Ok(_) => {
            let alias = match db::get_alias(pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };
            let content_template = AliasShowTemplate {
                title: "Show Alias",
                alias,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let template = BaseTemplate {
                    title: "Show Alias".to_string(),
                    content,
                };
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error updating alias".to_string()),
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    match db::delete_alias(pool, id) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(e) => {
                    eprintln!("Error getting aliases: {:?}", e);
                    vec![]
                }
            };
            let content_template = AliasListTemplate {
                title: "Aliases",
                aliases,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let template = BaseTemplate {
                    title: "Aliases".to_string(),
                    content,
                };
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error deleting alias".to_string()),
    }
}

pub async fn toggle_active(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    match db::toggle_alias_active(pool, id) {
        Ok(_) => {
            let alias = match db::get_alias(pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };
            let content_template = AliasShowTemplate {
                title: "Show Alias",
                alias,
            };
            let content = content_template.render().unwrap();
            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let template = BaseTemplate {
                    title: "Show Alias".to_string(),
                    content,
                };
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}

pub async fn toggle_active_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_alias_active(pool, id) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(e) => {
                    eprintln!("Error getting aliases: {:?}", e);
                    vec![]
                }
            };
            let content_template = AliasListTemplate {
                title: "Aliases",
                aliases,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let template = BaseTemplate {
                    title: "Aliases".to_string(),
                    content,
                };
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}

pub async fn toggle_active_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_alias_active(pool, id) {
        Ok(_) => {
            let alias = match db::get_alias(pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };
            let content_template = AliasShowTemplate {
                title: "Show Alias",
                alias,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let template = BaseTemplate {
                    title: "Show Alias".to_string(),
                    content,
                };
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}
