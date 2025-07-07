use crate::templates::aliases::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState, i18n::get_translation};
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
        let locale = "en-US"; // For now, use default locale
        let template = BaseTemplate::with_i18n(
            get_translation(&state, locale, "aliases-title").await,
            content,
            &state,
            locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
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
        error: None,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = "en-US"; // For now, use default locale
        let template = BaseTemplate::with_i18n(
            get_translation(&state, locale, "aliases-add-title").await,
            content,
            &state,
            locale,
        ).await.unwrap();
        
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
        title: "Show Alias", // Use static string for now to avoid borrowing issues
        alias,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = "en-US"; // For now, use default locale
        let template = BaseTemplate::with_i18n(
            get_translation(&state, locale, "aliases-title").await,
            content,
            &state,
            locale,
        ).await.unwrap();
        
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
        error: None,
    };
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = "en-US"; // For now, use default locale
        let template = BaseTemplate::with_i18n(
            get_translation(&state, locale, "aliases-title").await,
            content,
            &state,
            locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<AliasForm>,
) -> Html<String> {
    let pool = &state.pool;

    match db::create_alias(pool, form.clone()) {
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
                let locale = "en-US"; // For now, use default locale
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, locale, "aliases-title").await,
                    content,
                    &state,
                    locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(e) => {
            eprintln!("Error creating alias: {:?}", e);
            
            // Handle specific database errors with user-friendly messages
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                    _,
                ) => format!("The domain '{}' does not exist. Please create the domain first before adding aliases.", form.domain),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("An alias with the email '{}' already exists.", form.mail),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The alias data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while creating the alias. Please try again.".to_string(),
            };

            // Return to form with error message
            let form_template = AliasFormTemplate {
                title: "New Alias",
                alias: None,
                form: form.clone(),
                error: Some(error_message),
            };
            let content = form_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = "en-US"; // For now, use default locale
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, locale, "aliases-add-title").await,
                    content,
                    &state,
                    locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
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

    match db::update_alias(pool, id, form.clone()) {
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
                let locale = "en-US"; // For now, use default locale
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, locale, "aliases-show-title").await,
                    content,
                    &state,
                    locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(e) => {
            eprintln!("Error updating alias: {:?}", e);
            
            // Handle specific database errors with user-friendly messages
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                    _,
                ) => format!("The domain '{}' does not exist. Please create the domain first before updating the alias.", form.domain),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("An alias with the email '{}' already exists.", form.mail),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The alias data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while updating the alias. Please try again.".to_string(),
            };

            // Get the original alias for the form
            let original_alias = match db::get_alias(pool, id) {
                Ok(alias) => Some(alias),
                Err(_) => None,
            };

            // Return to form with error message
            let form_template = AliasFormTemplate {
                title: "Edit Alias",
                alias: original_alias,
                form: form.clone(),
                error: Some(error_message),
            };
            let content = form_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = "en-US"; // For now, use default locale
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, locale, "aliases-edit-title").await,
                    content,
                    &state,
                    locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
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
                let locale = "en-US"; // For now, use default locale
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, locale, "aliases-title").await,
                    content,
                    &state,
                    locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error deleting alias".to_string()),
    }
}

pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    match db::toggle_alias_enabled(pool, id) {
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
                let locale = "en-US"; // For now, use default locale
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, locale, "aliases-show-title").await,
                    content,
                    &state,
                    locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_alias_enabled(pool, id) {
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
                let locale = "en-US"; // For now, use default locale
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, locale, "aliases-title").await,
                    content,
                    &state,
                    locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_alias_enabled(pool, id) {
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
                let locale = "en-US"; // For now, use default locale
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, locale, "aliases-show-title").await,
                    content,
                    &state,
                    locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}
