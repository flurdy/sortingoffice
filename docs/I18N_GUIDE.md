# Internationalization (i18n) Guide

This guide explains how to use the internationalization system in the Sorting Office application.

## Overview

The application now supports multiple languages through a simple resource bundle system. Text that was previously hardcoded in templates has been moved to resource files that can be easily translated.

## Resource Bundle Structure

Resource bundles are stored in the `resources/locales/` directory:

```
resources/
└── locales/
    ├── en-US/
    │   └── messages.ftl
    └── es-ES/
        └── messages.ftl
```

## Resource File Format

Resource files use a simple key-value format:

```ftl
# Application title and branding
app-title = Sorting Office
app-subtitle = Mail Server Administration Tool

# Navigation
nav-dashboard = Dashboard
nav-domains = Domains
nav-backups = Backups
nav-aliases = Aliases
nav-users = Users
nav-statistics = Statistics
nav-about = About
nav-logout = Logout

# Common actions
action-add = Add
action-edit = Edit
action-delete = Delete
action-view = View
action-save = Save
action-cancel = Cancel
action-back = Back
action-enable = Enable
action-disable = Disable
action-toggle = Toggle

# Status indicators
status-active = Active
status-inactive = Inactive
status-enabled = Enabled
status-disabled = Disabled
```

## Adding a New Language

To add support for a new language:

1. Create a new directory in `resources/locales/` with the locale code (e.g., `fr-FR/`)
2. Copy the `messages.ftl` file from `en-US/`
3. Translate all the values in the file
4. Update the application to load the new locale

Example for French:

```bash
mkdir -p resources/locales/fr-FR
cp resources/locales/en-US/messages.ftl resources/locales/fr-FR/
```

Then edit `resources/locales/fr-FR/messages.ftl`:

```ftl
# Application title and branding
app-title = Bureau de Tri
app-subtitle = Outil d'Administration du Serveur de Courrier

# Navigation
nav-dashboard = Tableau de Bord
nav-domains = Domaines
nav-backups = Sauvegardes
nav-aliases = Alias
nav-users = Utilisateurs
nav-statistics = Statistiques
nav-about = À Propos
nav-logout = Déconnexion
```

## Using i18n in Handlers

### Basic Translation

```rust
use crate::i18n::get_translation;

pub async fn my_handler(State(state): State<AppState>) -> Html<String> {
    let locale = "en-US"; // Get from headers or session
    let title = get_translation(&state, locale, "my-title").await;
    
    // Use the translated title...
}
```

### Translation with Arguments

```rust
use crate::i18n::get_translation_with_args;
use std::collections::HashMap;

pub async fn my_handler(State(state): State<AppState>) -> Html<String> {
    let locale = "en-US";
    let mut args = HashMap::new();
    args.insert("domain".to_string(), "example.com".to_string());
    
    let message = get_translation_with_args(
        &state, 
        locale, 
        "error-duplicate-backup", 
        args
    ).await;
    
    // message will be: "A backup server for domain 'example.com' already exists."
}
```

### Using with BaseTemplate

```rust
use crate::templates::layout::BaseTemplate;

pub async fn my_handler(State(state): State<AppState>) -> Html<String> {
    let locale = "en-US";
    
    let content = "Your page content here";
    
    let template = BaseTemplate::with_i18n(
        get_translation(&state, locale, "page-title").await,
        content,
        &state,
        locale,
    ).await.unwrap();
    
    Html(template.render().unwrap())
}
```

## Using i18n in Templates

### Base Template

The base template (`templates/base.html`) now uses i18n variables:

```html
<h1 class="text-xl font-semibold text-gray-900 dark:text-white">{{ app_title }}</h1>

<a href="/" class="...">{{ nav_dashboard }}</a>
<a href="/domains" class="...">{{ nav_domains }}</a>
<a href="/backups" class="...">{{ nav_backups }}</a>
```

### Content Templates

For content templates, you can pass translated strings from the handler:

```rust
// In handler
let content_template = MyTemplate {
    title: get_translation(&state, locale, "my-page-title").await,
    description: get_translation(&state, locale, "my-page-description").await,
    // ... other fields
};
```

```html
<!-- In template -->
<h1>{{ title }}</h1>
<p>{{ description }}</p>
```

## Locale Detection

The application includes helper functions for locale detection:

### From Headers

```rust
use crate::i18n::get_locale_from_headers;

pub async fn my_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let locale = get_locale_from_headers(&headers);
    // Use locale...
}
```

### From Session

```rust
// Note: This needs to be implemented based on your session store
// pub fn get_locale_from_session(session: &YourSessionType) -> String {
//     session
//         .get::<String>("locale")
//         .unwrap_or_else(|| "en-US".to_string())
// }
```

## Adding New Translation Keys

When adding new text to the application:

1. Add the key to `resources/locales/en-US/messages.ftl`
2. Add the key to all other language files
3. Update the template to use the new key
4. Update the handler to pass the translated value

Example:

```ftl
# Add to messages.ftl
new-feature-title = New Feature
new-feature-description = This is a new feature description
```

```rust
// In handler
let title = get_translation(&state, locale, "new-feature-title").await;
let description = get_translation(&state, locale, "new-feature-description").await;
```

```html
<!-- In template -->
<h1>{{ title }}</h1>
<p>{{ description }}</p>
```

## Best Practices

1. **Use descriptive keys**: Use keys that clearly describe the content (e.g., `nav-dashboard` instead of `nav1`)
2. **Group related keys**: Use prefixes to group related translations (e.g., `nav-`, `action-`, `status-`)
3. **Keep translations consistent**: Use the same terminology across the application
4. **Test translations**: Always test with different languages to ensure proper display
5. **Handle missing translations**: The system falls back to the message key if a translation is missing

## Current Supported Languages

- **English (en-US)**: Default language
- **Spanish (es-ES)**: Complete translation available

## Adding More Languages

To add support for additional languages:

1. Create the locale directory and messages file
2. Translate all keys
3. Test the application with the new language
4. Update this documentation

## Technical Details

The i18n system uses:
- Simple key-value storage in memory
- Async loading of resource files
- Thread-safe access with `RwLock`
- Fallback to default locale if requested locale is not available
- Fallback to message key if translation is missing

The system is designed to be simple and efficient, avoiding the complexity of more advanced i18n libraries while providing all necessary functionality for a web application. 
