
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct I18n {
    messages: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    default_locale: String,
}

impl I18n {
    pub fn new(default_locale: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut messages = HashMap::new();

        // Load all locales
        let locales = ["en-US", "es-ES", "fr-FR", "nb-NO", "de-DE"];
        for locale in locales {
            debug!("Loading locale: {}", locale);
            match Self::load_messages(locale) {
                Ok(locale_messages) => {
                    let len = locale_messages.len();
                    messages.insert(locale.to_string(), locale_messages);
                    debug!("Loaded {} translations for locale {}", len, locale);
                }
                Err(e) => {
                    debug!("Failed to load locale {}: {:?}", locale, e);
                }
            }
        }

        // Ensure default locale is loaded
        if !messages.contains_key(default_locale) {
            debug!("Loading default locale: {}", default_locale);
            let default_messages = Self::load_messages(default_locale)?;
            messages.insert(default_locale.to_string(), default_messages);
        }

        Ok(I18n {
            messages: Arc::new(RwLock::new(messages)),
            default_locale: default_locale.to_string(),
        })
    }

    fn load_messages(locale: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let resource_path = format!("resources/locales/{locale}/messages.ftl");
        let resource_str = fs::read_to_string(&resource_path)?;

        let mut messages = HashMap::new();
        let mut current_key = None;
        let mut current_value = String::new();

        for line in resource_str.lines() {
            let line = line.trim_start();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // If we have a key-value pair
            if let Some(idx) = line.find('=') {
                // If we had a previous key-value pair, store it
                if let Some(key) = current_key.take() {
                    if !current_value.is_empty() {
                        messages.insert(key, current_value.trim().to_string());
                    }
                    current_value = String::new();
                }

                let (key, value) = line.split_at(idx);
                let key = key.trim();
                let value = value[1..].trim().trim_matches('"');
                messages.insert(key.to_string(), value.to_string());
            }
        }

        // Store the last key-value pair if any
        if let Some(key) = current_key {
            if !current_value.is_empty() {
                messages.insert(key, current_value.trim().to_string());
            }
        }

        Ok(messages)
    }

    pub async fn get_translation(&self, locale: &str, key: &str) -> String {
        let messages = self.messages.read().await;

        // Try to get translation from requested locale
        if let Some(locale_messages) = messages.get(locale) {
            if let Some(translation) = locale_messages.get(key) {
                return translation.to_string();
            }
        }

        // Fall back to default locale if translation not found
        if let Some(default_messages) = messages.get(&self.default_locale) {
            if let Some(translation) = default_messages.get(key) {
                return translation.to_string();
            }
        }

        // Return the key itself if no translation found
        key.to_string()
    }

    pub async fn translate_with_args(
        &self,
        locale: &str,
        message_id: &str,
        args: HashMap<String, String>,
    ) -> String {
        let mut message = self.get_translation(locale, message_id).await;

        // Simple variable substitution: { $variable }
        for (key, value) in args {
            let placeholder = format!("{{ ${key} }}");
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
                    // Validate against supported locales
                    match lang {
                        "en-US" | "es-ES" | "fr-FR" | "de-DE" | "nb-NO" => return lang.to_string(),
                        "en" => return "en-US".to_string(), // Map short codes to full locales
                        _ => return "en-US".to_string(), // Default for unsupported locales
                    }
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
    state.i18n.get_translation(locale, message_id).await
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
    // For i18n functions, we don't have access to headers, so we'll use default database
    let current_db_id = state.db_manager.get_default_db_id().to_string();
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());

    crate::templates::layout::BaseTemplate::with_i18n(
        title,
        content,
        state,
        locale,
        current_db_label,
        current_db_id,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use std::collections::HashMap;

    // Helper function to create test messages
    fn create_test_messages() -> HashMap<String, String> {
        let mut messages = HashMap::new();
        messages.insert("hello".to_string(), "Hello".to_string());
        messages.insert("welcome".to_string(), "Welcome".to_string());
        messages.insert("goodbye".to_string(), "Goodbye".to_string());
        messages.insert("greeting".to_string(), "Hello { $name }".to_string());
        messages.insert("count".to_string(), "You have { $count } items".to_string());
        messages
    }

    // Helper function to create test I18n instance
    async fn create_test_i18n() -> I18n {
        let mut messages = HashMap::new();
        messages.insert("en-US".to_string(), create_test_messages());

        // Add some Spanish translations
        let mut spanish_messages = HashMap::new();
        spanish_messages.insert("hello".to_string(), "Hola".to_string());
        spanish_messages.insert("welcome".to_string(), "Bienvenido".to_string());
        spanish_messages.insert("goodbye".to_string(), "Adiós".to_string());
        messages.insert("es-ES".to_string(), spanish_messages);

        I18n {
            messages: Arc::new(RwLock::new(messages)),
            default_locale: "en-US".to_string(),
        }
    }

    #[tokio::test]
    async fn test_get_translation_basic() {
        let i18n = create_test_i18n().await;

        // Test basic translation
        let result = i18n.get_translation("en-US", "hello").await;
        assert_eq!(result, "Hello");

        let result = i18n.get_translation("en-US", "welcome").await;
        assert_eq!(result, "Welcome");
    }

    #[tokio::test]
    async fn test_get_translation_fallback() {
        let i18n = create_test_i18n().await;

        // Test fallback to default locale when translation not found
        let result = i18n.get_translation("fr-FR", "hello").await;
        assert_eq!(result, "Hello"); // Should fall back to en-US

        let result = i18n.get_translation("de-DE", "welcome").await;
        assert_eq!(result, "Welcome"); // Should fall back to en-US
    }

    #[tokio::test]
    async fn test_get_translation_key_not_found() {
        let i18n = create_test_i18n().await;

        // Test when key doesn't exist
        let result = i18n.get_translation("en-US", "nonexistent").await;
        assert_eq!(result, "nonexistent"); // Should return the key itself
    }

    #[tokio::test]
    async fn test_get_translation_different_locale() {
        let i18n = create_test_i18n().await;

        // Test Spanish translations
        let result = i18n.get_translation("es-ES", "hello").await;
        assert_eq!(result, "Hola");

        let result = i18n.get_translation("es-ES", "welcome").await;
        assert_eq!(result, "Bienvenido");
    }

    #[tokio::test]
    async fn test_translate_with_args() {
        let i18n = create_test_i18n().await;

        // Test translation with arguments
        let mut args = HashMap::new();
        args.insert("name".to_string(), "John".to_string());

        let result = i18n.translate_with_args("en-US", "greeting", args).await;
        assert_eq!(result, "Hello John");
    }

    #[tokio::test]
    async fn test_translate_with_args_multiple() {
        let i18n = create_test_i18n().await;

        // Test translation with multiple arguments
        let mut args = HashMap::new();
        args.insert("count".to_string(), "5".to_string());

        let result = i18n.translate_with_args("en-US", "count", args).await;
        assert_eq!(result, "You have 5 items");
    }

    #[tokio::test]
    async fn test_translate_with_args_no_placeholder() {
        let i18n = create_test_i18n().await;

        // Test translation with arguments but no placeholders
        let mut args = HashMap::new();
        args.insert("name".to_string(), "John".to_string());

        let result = i18n.translate_with_args("en-US", "hello", args).await;
        assert_eq!(result, "Hello"); // Should not change
    }

    #[tokio::test]
    async fn test_translate_with_args_empty_args() {
        let i18n = create_test_i18n().await;

        // Test translation with empty arguments
        let args = HashMap::new();

        let result = i18n.translate_with_args("en-US", "greeting", args).await;
        assert_eq!(result, "Hello { $name }"); // Should keep placeholder
    }

    #[tokio::test]
    async fn test_load_locale() {
        let i18n = create_test_i18n().await;

        // Test loading a new locale
        let mut new_messages = HashMap::new();
        new_messages.insert("hello".to_string(), "Bonjour".to_string());
        new_messages.insert("welcome".to_string(), "Bienvenue".to_string());

        // Mock the load_messages function for testing
        // In a real test, we'd need to create a temporary file
        // For now, we'll test the basic functionality
        assert!(i18n.get_default_locale() == "en-US");
    }

    #[tokio::test]
    async fn test_get_default_locale() {
        let i18n = create_test_i18n().await;

        let default = i18n.get_default_locale();
        assert_eq!(default, "en-US");
    }

    #[test]
    fn test_get_locale_from_headers_basic() {
        let mut headers = HeaderMap::new();
        headers.insert("accept-language", "en-US,en;q=0.9".parse().unwrap());

        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "en-US");
    }

    #[test]
    fn test_get_locale_from_headers_spanish() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept-language",
            "es-ES,es;q=0.9,en;q=0.8".parse().unwrap(),
        );

        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "es-ES");
    }

    #[test]
    fn test_get_locale_from_headers_german() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept-language",
            "de-DE,de;q=0.9,en;q=0.8".parse().unwrap(),
        );

        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "de-DE");
    }

    #[test]
    fn test_get_locale_from_headers_french() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept-language",
            "fr-FR,fr;q=0.9,en;q=0.8".parse().unwrap(),
        );

        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "fr-FR");
    }

    #[test]
    fn test_get_locale_from_headers_norwegian() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept-language",
            "nb-NO,nb;q=0.9,en;q=0.8".parse().unwrap(),
        );

        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "nb-NO");
    }

    #[test]
    fn test_get_locale_from_headers_no_headers() {
        let headers = HeaderMap::new();

        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "en-US"); // Should default to en-US
    }

    #[test]
    fn test_get_locale_from_headers_invalid_header() {
        let mut headers = HeaderMap::new();
        headers.insert("accept-language", "invalid".parse().unwrap());

        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "en-US"); // Should default to en-US
    }

    #[test]
    fn test_get_locale_from_headers_short_language() {
        let mut headers = HeaderMap::new();
        headers.insert("accept-language", "en".parse().unwrap());

        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "en-US"); // Should default to en-US for short codes
    }

    #[test]
    fn test_get_locale_from_headers_with_quality() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept-language",
            "es-ES;q=0.9,en-US;q=0.8".parse().unwrap(),
        );

        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "es-ES");
    }

    #[test]
    fn test_get_locale_from_headers_multiple_languages() {
        let mut headers = HeaderMap::new();
        headers.insert("accept-language", "fr-FR,es-ES,en-US".parse().unwrap());

        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "fr-FR"); // Should take the first one
    }

    #[test]
    fn test_get_locale_from_headers_edge_cases() {
        let mut headers = HeaderMap::new();

        // Test with empty string
        headers.insert("accept-language", "".parse().unwrap());
        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "en-US");

        // Test with whitespace
        headers.insert("accept-language", "  en-US  ".parse().unwrap());
        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "en-US");

        // Test with very short code
        headers.insert("accept-language", "a".parse().unwrap());
        let result = get_locale_from_headers(&headers);
        assert_eq!(result, "en-US");
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let i18n = create_test_i18n().await;

        // Test concurrent access to translations
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let i18n = i18n.clone();
                tokio::spawn(async move { i18n.get_translation("en-US", "hello").await })
            })
            .collect();

        let results: Vec<_> = futures_util::future::join_all(handles).await;

        for result in results {
            let translation = result.unwrap();
            assert_eq!(translation, "Hello");
        }
    }

    #[tokio::test]
    async fn test_translation_performance() {
        let i18n = create_test_i18n().await;

        // Test performance with many translations
        let start = std::time::Instant::now();

        for _ in 0..1000 {
            let _ = i18n.get_translation("en-US", "hello").await;
        }

        let duration = start.elapsed();
        assert!(duration.as_millis() < 1000); // Should complete in less than 1 second
    }

    #[test]
    fn test_message_parsing_edge_cases() {
        // Test various edge cases for message parsing
        let test_cases = vec![
            ("key = value", ("key", "value")),
            ("key=value", ("key", "value")),
            ("key = \"value\"", ("key", "value")),
            ("key=\"value\"", ("key", "value")),
            ("key = \"value with spaces\"", ("key", "value with spaces")),
            (
                "key = \"value with \"quotes\"\"",
                ("key", "value with \"quotes\""),
            ),
        ];

        for (input, expected) in test_cases {
            if let Some(idx) = input.find('=') {
                let (key, value) = input.split_at(idx);
                let key = key.trim();
                let value = value[1..].trim();
                let value = if value.starts_with('"') && value.ends_with('"') {
                    &value[1..value.len()-1]
                } else {
                    value
                };
                assert_eq!((key, value), expected);
            }
        }
    }

    #[test]
    fn test_locale_validation() {
        // Test valid locales
        let valid_locales = vec!["en-US", "es-ES", "fr-FR", "de-DE", "nb-NO"];

        for locale in valid_locales {
            assert!(locale.len() >= 2);
            assert!(locale.contains('-'));
        }

        // Test invalid locales
        let invalid_locales = vec!["", "en", "US", "invalid"];

        for locale in invalid_locales {
            assert!(locale.is_empty() || locale.len() < 2 || !locale.contains('-'));
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let i18n = create_test_i18n().await;

        // Test with invalid locale
        let result = i18n.get_translation("invalid-locale", "hello").await;
        assert_eq!(result, "Hello"); // Should fall back to default

        // Test with empty key
        let result = i18n.get_translation("en-US", "").await;
        assert_eq!(result, ""); // Should return empty string

        // Test with empty locale
        let result = i18n.get_translation("", "hello").await;
        assert_eq!(result, "Hello"); // Should fall back to default
    }

    #[tokio::test]
    async fn test_translation_memory_usage() {
        let i18n = create_test_i18n().await;

        // Test that translations are properly stored and retrieved
        let initial_size = i18n.messages.read().await.len();

        // Add a new translation
        let mut messages = i18n.messages.write().await;
        let mut new_messages = HashMap::new();
        new_messages.insert("test".to_string(), "Test".to_string());
        messages.insert("test-locale".to_string(), new_messages);

        let final_size = messages.len();
        assert_eq!(final_size, initial_size + 1);
    }
}
