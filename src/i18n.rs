use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct I18n {
    messages: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    default_locale: String,
}

impl I18n {
    pub fn new(default_locale: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut messages = HashMap::new();

        // Load the default locale
        let default_messages = Self::load_messages(default_locale)?;
        messages.insert(default_locale.to_string(), default_messages);

        Ok(I18n {
            messages: Arc::new(RwLock::new(messages)),
            default_locale: default_locale.to_string(),
        })
    }

    fn load_messages(locale: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let resource_path = format!("resources/locales/{}/messages.ftl", locale);
        let resource_str = fs::read_to_string(&resource_path)?;

        let mut messages = HashMap::new();
        for line in resource_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // Remove quotes if present
                let value = if value.starts_with('"') && value.ends_with('"') {
                    &value[1..value.len() - 1]
                } else {
                    value
                };

                messages.insert(key.to_string(), value.to_string());
            }
        }

        Ok(messages)
    }

    pub async fn translate(&self, locale: &str, message_id: &str) -> String {
        let messages = self.messages.read().await;

        // Try to get the requested locale
        if let Some(locale_messages) = messages.get(locale) {
            if let Some(message) = locale_messages.get(message_id) {
                return message.clone();
            }
        }

        // Fall back to default locale
        if let Some(default_messages) = messages.get(&self.default_locale) {
            if let Some(message) = default_messages.get(message_id) {
                return message.clone();
            }
        }

        // Fallback to message ID if translation fails
        message_id.to_string()
    }

    pub async fn translate_with_args(
        &self,
        locale: &str,
        message_id: &str,
        args: HashMap<String, String>,
    ) -> String {
        let mut message = self.translate(locale, message_id).await;

        // Simple variable substitution: { $variable }
        for (key, value) in args {
            let placeholder = format!("{{ ${{{}}} }}", key);
            message = message.replace(&placeholder, &value);
        }

        message
    }

    pub async fn load_locale(&self, locale: &str) -> Result<(), Box<dyn std::error::Error>> {
        let messages = Self::load_messages(locale)?;
        let mut all_messages = self.messages.write().await;
        all_messages.insert(locale.to_string(), messages);
        Ok(())
    }

    pub fn get_default_locale(&self) -> &str {
        &self.default_locale
    }
}

// Helper function to get locale from request headers
pub fn get_locale_from_headers(headers: &axum::http::HeaderMap) -> String {
    if let Some(accept_language) = headers.get("accept-language") {
        if let Ok(lang_str) = accept_language.to_str() {
            // Simple parsing - take the first language code
            if let Some(lang) = lang_str.split(',').next() {
                let lang = lang.split(';').next().unwrap_or(lang).trim();
                if lang.len() >= 2 {
                    return lang.to_string();
                }
            }
        }
    }

    "en-US".to_string()
}

// Helper function to get locale from session or default
// Note: This function needs to be implemented based on your session store type
// pub fn get_locale_from_session(session: &YourSessionType) -> String {
//     session
//         .get::<String>("locale")
//         .unwrap_or_else(|| "en-US".to_string())
// }

// Helper function to get translations in handlers
pub async fn get_translation(state: &crate::AppState, locale: &str, message_id: &str) -> String {
    state.i18n.translate(locale, message_id).await
}

// Helper function to get translations with arguments in handlers
pub async fn get_translation_with_args(
    state: &crate::AppState,
    locale: &str,
    message_id: &str,
    args: HashMap<String, String>,
) -> String {
    state
        .i18n
        .translate_with_args(locale, message_id, args)
        .await
}

// Helper function to create BaseTemplate with i18n support
pub async fn create_base_template(
    state: &crate::AppState,
    locale: &str,
    title_key: &str,
    content: String,
) -> Result<crate::templates::layout::BaseTemplate, Box<dyn std::error::Error>> {
    let title = get_translation(state, locale, title_key).await;
    crate::templates::layout::BaseTemplate::with_i18n(title, content, state, locale).await
}
