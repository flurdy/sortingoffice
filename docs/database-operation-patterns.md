# Database Operation Patterns Documentation

This document provides comprehensive guidance on using the consolidated database helper functions and patterns in the Sorting Office application.

## Table of Contents

1. [Overview](#overview)
2. [Database Pool Management](#database-pool-management)
3. [Error Handling Strategies](#error-handling-strategies)
4. [Generic Entity Operations](#generic-entity-operations)
5. [Retry Configuration](#retry-configuration)
6. [Unified Error Handling](#unified-error-handling)
7. [Best Practices](#best-practices)
8. [Migration Guide](#migration-guide)
9. [Examples](#examples)

## Overview

The database helper functions have been refactored to provide consistent, maintainable patterns for database operations. The new system includes:

- **Generic pool retrieval** with configurable retry logic
- **Unified error handling** with multiple strategies
- **Standardized retry configurations** for different scenarios
- **Generic entity operation helpers** for CRUD operations
- **Comprehensive fallback mechanisms** for robust error recovery

## Database Pool Management

### Basic Pool Retrieval

```rust
use crate::handlers::database_ops::{get_db_pool_or_handle_error, get_db_pool_or_error};

// For HTML responses
let pool = get_db_pool_or_handle_error(&state, &headers).await?;

// For direct error handling
let pool = get_db_pool_or_error(&state, &headers).await?;
```

### Advanced Pool Retrieval with Configuration

```rust
use crate::handlers::database_ops::{
    get_db_pool_generic, PoolRetrievalConfig, HtmlErrorHandler
};

let config = PoolRetrievalConfig::with_fallback()
    .operation_name("user_creation");

let pool = get_db_pool_generic(
    &state, 
    &headers, 
    config, 
    HtmlErrorHandler
).await?;
```

### Pool Retrieval Strategies

- **Standard**: 3 attempts, 100ms base delay, 5s max delay
- **Fast**: 2 attempts, 50ms base delay, 1s max delay  
- **Aggressive**: 5 attempts, 200ms base delay, 10s max delay
- **Conservative**: 2 attempts, 100ms base delay, 2s max delay
- **With Fallback**: Enables automatic fallback to primary database

## Error Handling Strategies

### Error Handling Configuration

```rust
use crate::handlers::database_ops::{ErrorHandlingConfig, ErrorHandlingStrategy};

// HTML response strategy
let config = ErrorHandlingConfig::html_response()
    .with_error_message("error-user-creation")
    .with_logging(true);

// Empty result strategy (for lists)
let config = ErrorHandlingConfig::empty_result()
    .with_error_message("error-user-list");

// Fallback value strategy
let config = ErrorHandlingConfig::fallback_value("Default User")
    .with_error_details(false);
```

### Available Strategies

- **HtmlResponse**: Returns HTML error page
- **HttpError**: Returns HTTP status code with message
- **FallbackValue**: Returns configured fallback value
- **EmptyResult**: Returns empty collection/list
- **Redirect**: Redirects to error page
- **Custom**: Returns custom error response

## Generic Entity Operations

### Entity Creation

```rust
use crate::handlers::database_ops::create_entity_html;

let result = create_entity_html(
    || async { db::create_user(&pool, user_data) },
    |user| Html(format!("User created: {}", user.email)),
    |e| Html(format!("Failed to create user: {:?}", e)),
    "user",
    &user_data.email,
).await;
```

### Entity Updates

```rust
use crate::handlers::database_ops::update_entity_html;

let result = update_entity_html(
    || async { db::update_user(&pool, user_id, user_data) },
    |user| Html(format!("User updated: {}", user.email)),
    |e| Html(format!("Failed to update user: {:?}", e)),
    "user",
    &user_id,
).await;
```

### Entity Deletion

```rust
use crate::handlers::database_ops::delete_entity_html;

let result = delete_entity_html(
    || async { db::delete_user(&pool, user_id) },
    |_| Html("User deleted successfully".to_string()),
    |e| Html(format!("Failed to delete user: {:?}", e)),
    "user",
    &user_id,
).await;
```

### Entity Toggle Operations

```rust
use crate::handlers::database_ops::toggle_entity_html;

let result = toggle_entity_html(
    || async { db::toggle_user_enabled(&pool, user_id) },
    |user| Html(format!("User {} toggled", user.email)),
    |e| Html(format!("Failed to toggle user: {:?}", e)),
    "user",
    &user_id,
).await;
```

## Retry Configuration

### Using Predefined Configurations

```rust
use crate::handlers::errors::RetryConfig;

// Standard retry (3 attempts, 100ms-5s delay)
let config = RetryConfig::standard();

// Fast retry (2 attempts, 50ms-1s delay)
let config = RetryConfig::fast();

// Aggressive retry (5 attempts, 200ms-10s delay)
let config = RetryConfig::aggressive();

// Conservative retry (2 attempts, 100ms-2s delay)
let config = RetryConfig::conservative();
```

### Custom Retry Configuration

```rust
use crate::handlers::errors::RetryConfig;
use std::time::Duration;

let config = RetryConfig::custom()
    .max_attempts(5)
    .base_delay(Duration::from_millis(200))
    .max_delay(Duration::from_secs(10))
    .backoff_multiplier(1.5)
    .build();
```

## Unified Error Handling

### HTML Error Handler

```rust
use crate::handlers::database_ops::{
    HtmlErrorHandlerImpl, unified_entity_creation, ErrorHandlingConfig
};

let error_handler = HtmlErrorHandlerImpl {
    state: state.clone(),
    headers: headers.clone(),
};

let user = unified_entity_creation(
    || async { db::create_user(&pool, user_data) },
    error_handler,
    "user",
    &user_data.email,
).await?;
```

### HTTP Error Handler

```rust
use crate::handlers::database_ops::{
    HttpErrorHandlerImpl, unified_entity_retrieval
};

let error_handler = HttpErrorHandlerImpl;

let (user, status) = unified_entity_retrieval(
    || async { db::get_user(&pool, user_id) },
    error_handler,
    "user",
    &user_id,
).await?;
```

### List Operations with Fallback

```rust
use crate::handlers::database_ops::unified_list_operation;

let users = unified_list_operation(
    || async { db::get_users(&pool) },
    error_handler,
    "user",
    "retrieve",
).await?;
```

### Paginated Operations

```rust
use crate::handlers::database_ops::unified_paginated_operation;

let paginated_users = unified_paginated_operation(
    || async { db::get_users(&pool) },
    error_handler,
    "user",
    "retrieve",
    page,
    per_page,
).await?;
```

## Best Practices

### 1. Choose Appropriate Error Handling Strategy

- **HtmlResponse**: For user-facing web pages
- **HttpError**: For API endpoints
- **EmptyResult**: For list/collection operations
- **FallbackValue**: When a default value is acceptable

### 2. Use Consistent Logging

```rust
// Good: Consistent error logging
let config = ErrorHandlingConfig::html_response()
    .with_logging(true)
    .with_error_message("error-user-operation");

// Avoid: Inconsistent error handling
match db::create_user(&pool, user_data).await {
    Ok(user) => Ok(user),
    Err(e) => {
        // Inconsistent logging
        println!("Error: {}", e);
        Err(e)
    }
}
```

### 3. Leverage Fallback Mechanisms

```rust
// Good: Use fallback for non-critical operations
let users = get_entity_list_with_fallback(
    || async { db::get_users(&pool) },
    "retrieve users"
).await;

// Avoid: Failing completely on non-critical errors
let users = db::get_users(&pool).await?; // This will panic on error
```

### 4. Use Appropriate Retry Strategies

- **Fast**: For user-initiated actions that need quick feedback
- **Standard**: For most database operations
- **Aggressive**: For critical operations that must succeed
- **Conservative**: For operations that shouldn't retry often

### 5. Implement Proper Context

```rust
// Good: Provide meaningful context
let context = format!("create user with email {}", user_data.email);
let config = ErrorHandlingConfig::html_response()
    .with_error_message("error-user-creation");

// Avoid: Generic context
let context = "database operation";
```

## Migration Guide

### From Old Patterns to New Patterns

#### Before (Old Pattern)
```rust
match db::create_user(&pool, user_data) {
    Ok(user) => {
        info!("User created: {}", user.email);
        Ok(user)
    }
    Err(e) => {
        error!("Failed to create user: {:?}", e);
        let error_message = get_translation(&state, &locale, "error-user-creation").await;
        Err(Html(error_message))
    }
}
```

#### After (New Pattern)
```rust
let result = create_entity_html(
    || async { db::create_user(&pool, user_data) },
    |user| Html(format!("User created: {}", user.email)),
    |e| async {
        let error_message = get_translation(&state, &locale, "error-user-creation").await;
        Html(error_message)
    },
    "user",
    &user_data.email,
).await;
```

### Migration Checklist

- [ ] Replace manual `match` statements with generic helpers
- [ ] Use appropriate error handling strategies
- [ ] Implement consistent logging
- [ ] Add fallback mechanisms for non-critical operations
- [ ] Use standardized retry configurations
- [ ] Update error messages to use i18n keys
- [ ] Test error scenarios thoroughly

## Examples

### Complete User Creation Handler

```rust
pub async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<UserForm>,
) -> Html<String> {
    let pool = get_db_pool_or_handle_error(&state, &headers).await?;
    
    let result = create_entity_html(
        || async { db::create_user(&pool, form.clone()) },
        |_| {
            // Success: redirect to user list
            Html("<script>window.location.href='/users';</script>".to_string())
        },
        |e| async {
            let locale = get_user_locale(&headers);
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => get_translation(&state, &locale, "error-duplicate-user").await,
                _ => get_translation(&state, &locale, "error-user-creation").await,
            };
            
            let form_template = build_user_form_template(
                &state, &locale, None, form, Some(error_message)
            ).await;
            Html(form_template.render().unwrap())
        },
        "user",
        &form.email,
    ).await;

    result
}
```

### Complete User List Handler with Pagination

```rust
pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let pool = get_db_pool_or_handle_error(&state, &headers).await?;
    let locale = get_user_locale(&headers);
    
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    
    // Use fallback for non-critical list retrieval
    let users = get_entity_list_with_fallback(
        || async { db::get_users(&pool) },
        "retrieve users"
    ).await;
    
    let paginated = PaginatedResult::new(users.clone(), users.len() as i64, page, per_page);
    
    let content_template = build_user_list_template(
        &state, &locale, users, paginated, &headers
    ).await;
    
    Html(content_template.render().unwrap())
}
```

### Error Handler with Custom Strategy

```rust
pub async fn create_user_with_custom_error_handling(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<UserForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let pool = get_db_pool_or_handle_error(&state, &headers).await?;
    
    let error_handler = HttpErrorHandlerImpl;
    let config = ErrorHandlingConfig::http_error()
        .with_error_details(false)
        .with_logging(true);
    
    let result = execute_db_operation_with_unified_error_handling(
        || async { db::create_user(&pool, form) },
        error_handler,
        config,
        &format!("create user with email {}", form.email),
    ).await?;
    
    Ok(Html(format!("User created: {}", result.email)))
}
```

## Conclusion

The new database operation patterns provide:

1. **Consistency**: All database operations follow the same patterns
2. **Maintainability**: Centralized error handling and retry logic
3. **Flexibility**: Configurable strategies for different scenarios
4. **Robustness**: Comprehensive fallback mechanisms
5. **Testability**: Well-defined interfaces for unit testing

By following these patterns and best practices, you can create robust, maintainable database operations that handle errors gracefully and provide consistent user experiences.
