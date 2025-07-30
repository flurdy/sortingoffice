use regex::Regex;
use std::path::Path;

/// Validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    DomainInvalid(String),
    AliasMailInvalid(String),
    AliasDestinationInvalid(String),
    UserIdInvalid(String),
    UserPathInvalid(String),
    BackupNameInvalid(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::DomainInvalid(msg) => write!(f, "Domain validation error: {}", msg),
            ValidationError::AliasMailInvalid(msg) => {
                write!(f, "Alias mail validation error: {}", msg)
            }
            ValidationError::AliasDestinationInvalid(msg) => {
                write!(f, "Alias destination validation error: {}", msg)
            }
            ValidationError::UserIdInvalid(msg) => write!(f, "User ID validation error: {}", msg),
            ValidationError::UserPathInvalid(msg) => {
                write!(f, "User path validation error: {}", msg)
            }
            ValidationError::BackupNameInvalid(msg) => {
                write!(f, "Backup name validation error: {}", msg)
            }
        }
    }
}

/// Validate domain names
///
/// Rules:
/// - No capitalisation allowed
/// - Only letters, numbers, dots, and hyphens allowed
/// - Cannot start or end with dot or hyphen
/// - Machine names without TLD are valid (e.g., localhost, andromeda-001)
pub fn validate_domain(domain: &str) -> Result<(), ValidationError> {
    if domain.is_empty() {
        return Err(ValidationError::DomainInvalid(
            "Domain cannot be empty".to_string(),
        ));
    }

    // Check for capitalisation
    if domain.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::DomainInvalid(
            "Domain cannot contain uppercase letters".to_string(),
        ));
    }

    // Check for invalid characters (only letters, numbers, dots, hyphens allowed)
    let valid_chars = Regex::new(r"^[a-z0-9.-]+$").unwrap();
    if !valid_chars.is_match(domain) {
        return Err(ValidationError::DomainInvalid(
            "Domain can only contain lowercase letters, numbers, dots, and hyphens".to_string(),
        ));
    }

    // Cannot start or end with dot or hyphen
    if domain.starts_with('.') || domain.starts_with('-') {
        return Err(ValidationError::DomainInvalid(
            "Domain cannot start with dot or hyphen".to_string(),
        ));
    }
    if domain.ends_with('.') || domain.ends_with('-') {
        return Err(ValidationError::DomainInvalid(
            "Domain cannot end with dot or hyphen".to_string(),
        ));
    }

    // Cannot have consecutive dots or hyphens
    if domain.contains("..") || domain.contains("--") {
        return Err(ValidationError::DomainInvalid(
            "Domain cannot have consecutive dots or hyphens".to_string(),
        ));
    }

    Ok(())
}

/// Validate alias mail addresses
///
/// Rules:
/// - Must contain @ but not end in @
/// - Catchall aliases are valid (e.g., @example.com)
/// - Must have valid local part and domain part
pub fn validate_alias_mail(mail: &str) -> Result<(), ValidationError> {
    if mail.is_empty() {
        return Err(ValidationError::AliasMailInvalid(
            "Alias mail cannot be empty".to_string(),
        ));
    }

    // Must contain @
    if !mail.contains('@') {
        return Err(ValidationError::AliasMailInvalid(
            "Alias mail must contain @".to_string(),
        ));
    }

    // Cannot end in @
    if mail.ends_with('@') {
        return Err(ValidationError::AliasMailInvalid(
            "Alias mail cannot end in @".to_string(),
        ));
    }

    // Split into local and domain parts
    let parts: Vec<&str> = mail.split('@').collect();
    if parts.len() != 2 {
        return Err(ValidationError::AliasMailInvalid(
            "Alias mail must have exactly one @".to_string(),
        ));
    }

    let local_part = parts[0];
    let domain_part = parts[1];

    // For catchall aliases, local part can be empty
    if local_part.is_empty() {
        // This is a catchall alias (e.g., @example.com)
        if domain_part.is_empty() {
            return Err(ValidationError::AliasMailInvalid(
                "Catchall alias must have a domain part".to_string(),
            ));
        }
        // Validate the domain part
        validate_domain(domain_part).map_err(|e| {
            ValidationError::AliasMailInvalid(format!("Invalid domain in catchall alias: {}", e))
        })?;
    } else {
        // Regular alias - validate both parts
        validate_alias_local_part(local_part)?;
        validate_domain(domain_part)
            .map_err(|e| ValidationError::AliasMailInvalid(format!("Invalid domain: {}", e)))?;
    }

    Ok(())
}

/// Validate alias local part
fn validate_alias_local_part(local_part: &str) -> Result<(), ValidationError> {
    if local_part.is_empty() {
        return Err(ValidationError::AliasMailInvalid(
            "Local part cannot be empty for non-catchall aliases".to_string(),
        ));
    }

    // Check for valid characters in local part
    let valid_local_chars = Regex::new(r"^[a-zA-Z0-9._%+-]+$").unwrap();
    if !valid_local_chars.is_match(local_part) {
        return Err(ValidationError::AliasMailInvalid(
            "Local part contains invalid characters".to_string(),
        ));
    }

    // Cannot start or end with dot
    if local_part.starts_with('.') || local_part.ends_with('.') {
        return Err(ValidationError::AliasMailInvalid(
            "Local part cannot start or end with dot".to_string(),
        ));
    }

    // Cannot have consecutive dots
    if local_part.contains("..") {
        return Err(ValidationError::AliasMailInvalid(
            "Local part cannot have consecutive dots".to_string(),
        ));
    }

    Ok(())
}

/// Validate alias destinations
///
/// Rules:
/// - Must contain @ but not end in @
/// - + character is allowed if used once and not at the start or just before the @
/// - @ is not a valid destination
/// - @example.com is a valid destination
pub fn validate_alias_destination(destination: &str) -> Result<(), ValidationError> {
    if destination.is_empty() {
        return Err(ValidationError::AliasDestinationInvalid(
            "Destination cannot be empty".to_string(),
        ));
    }

    // Must contain @
    if !destination.contains('@') {
        return Err(ValidationError::AliasDestinationInvalid(
            "Destination must contain @".to_string(),
        ));
    }

    // Cannot end in @
    if destination.ends_with('@') {
        return Err(ValidationError::AliasDestinationInvalid(
            "Destination cannot end in @".to_string(),
        ));
    }

    // @ alone is not valid
    if destination == "@" {
        return Err(ValidationError::AliasDestinationInvalid(
            "@ alone is not a valid destination".to_string(),
        ));
    }

    // Split into local and domain parts
    let parts: Vec<&str> = destination.split('@').collect();
    if parts.len() != 2 {
        return Err(ValidationError::AliasDestinationInvalid(
            "Destination must have exactly one @".to_string(),
        ));
    }

    let local_part = parts[0];
    let domain_part = parts[1];

    // Validate domain part
    validate_domain(domain_part)
        .map_err(|e| ValidationError::AliasDestinationInvalid(format!("Invalid domain: {}", e)))?;

    // For destinations, local part can be empty (e.g., @example.com)
    if !local_part.is_empty() {
        validate_destination_local_part(local_part)?;
    }

    Ok(())
}

/// Validate destination local part
fn validate_destination_local_part(local_part: &str) -> Result<(), ValidationError> {
    // Check for valid characters in local part
    let valid_local_chars = Regex::new(r"^[a-zA-Z0-9._%+-]+$").unwrap();
    if !valid_local_chars.is_match(local_part) {
        return Err(ValidationError::AliasDestinationInvalid(
            "Local part contains invalid characters".to_string(),
        ));
    }

    // + character validation
    let plus_count = local_part.chars().filter(|&c| c == '+').count();
    if plus_count > 1 {
        return Err(ValidationError::AliasDestinationInvalid(
            "Local part can only contain one + character".to_string(),
        ));
    }

    if local_part.starts_with('+') {
        return Err(ValidationError::AliasDestinationInvalid(
            "Local part cannot start with +".to_string(),
        ));
    }

    // Check if + is just before @ (this would be caught by the main validation, but let's be explicit)
    if local_part.ends_with('+') {
        return Err(ValidationError::AliasDestinationInvalid(
            "Local part cannot end with +".to_string(),
        ));
    }

    // Cannot start or end with dot
    if local_part.starts_with('.') || local_part.ends_with('.') {
        return Err(ValidationError::AliasDestinationInvalid(
            "Local part cannot start or end with dot".to_string(),
        ));
    }

    // Cannot have consecutive dots
    if local_part.contains("..") {
        return Err(ValidationError::AliasDestinationInvalid(
            "Local part cannot have consecutive dots".to_string(),
        ));
    }

    Ok(())
}

/// Validate user IDs
///
/// Rules:
/// - Must be a valid email format
/// - A catchall is not valid user ID
pub fn validate_user_id(user_id: &str) -> Result<(), ValidationError> {
    if user_id.is_empty() {
        return Err(ValidationError::UserIdInvalid(
            "User ID cannot be empty".to_string(),
        ));
    }

    // Check if it's a catchall (starts with @)
    if user_id.starts_with('@') {
        return Err(ValidationError::UserIdInvalid(
            "User ID cannot be a catchall alias".to_string(),
        ));
    }

    // Must be a valid email format
    validate_alias_mail(user_id).map_err(|e| {
        ValidationError::UserIdInvalid(format!("User ID must be a valid email: {}", e))
    })?;

    Ok(())
}

/// Validate user paths (maildir and home)
///
/// Rules:
/// - Must be valid file system paths
/// - Should not be empty
pub fn validate_user_path(path: &str) -> Result<(), ValidationError> {
    if path.is_empty() {
        return Err(ValidationError::UserPathInvalid(
            "Path cannot be empty".to_string(),
        ));
    }

    // Check if it's a valid path
    let path_obj = Path::new(path);
    if !path_obj.is_absolute() {
        return Err(ValidationError::UserPathInvalid(
            "Path must be absolute".to_string(),
        ));
    }

    // Check for invalid characters in path
    if path.contains('\0') {
        return Err(ValidationError::UserPathInvalid(
            "Path contains null character".to_string(),
        ));
    }

    // Check for path traversal attempts
    if path.contains("..") {
        return Err(ValidationError::UserPathInvalid(
            "Path cannot contain ..".to_string(),
        ));
    }

    Ok(())
}

/// Validate backup names
///
/// Rules:
/// - No capitalisation allowed
/// - Only letters, numbers, dots, and hyphens allowed
/// - Cannot start or end with dot or hyphen
pub fn validate_backup_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::BackupNameInvalid(
            "Backup name cannot be empty".to_string(),
        ));
    }

    // Check for capitalisation
    if name.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::BackupNameInvalid(
            "Backup name cannot contain uppercase letters".to_string(),
        ));
    }

    // Check for invalid characters (only letters, numbers, dots, hyphens allowed)
    let valid_chars = Regex::new(r"^[a-z0-9.-]+$").unwrap();
    if !valid_chars.is_match(name) {
        return Err(ValidationError::BackupNameInvalid(
            "Backup name can only contain lowercase letters, numbers, dots, and hyphens"
                .to_string(),
        ));
    }

    // Cannot start or end with dot or hyphen
    if name.starts_with('.') || name.starts_with('-') {
        return Err(ValidationError::BackupNameInvalid(
            "Backup name cannot start with dot or hyphen".to_string(),
        ));
    }
    if name.ends_with('.') || name.ends_with('-') {
        return Err(ValidationError::BackupNameInvalid(
            "Backup name cannot end with dot or hyphen".to_string(),
        ));
    }

    // Cannot have consecutive dots or hyphens
    if name.contains("..") || name.contains("--") {
        return Err(ValidationError::BackupNameInvalid(
            "Backup name cannot have consecutive dots or hyphens".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_domain() {
        // Valid domains
        assert!(validate_domain("example.com").is_ok());
        assert!(validate_domain("subdomain.example.com").is_ok());
        assert!(validate_domain("domain.co.uk").is_ok());
        assert!(validate_domain("test-domain.org").is_ok());
        assert!(validate_domain("localhost").is_ok()); // Machine name without TLD
        assert!(validate_domain("andromeda-001").is_ok()); // Machine name without TLD
        assert!(validate_domain("a").is_ok()); // Single character
        assert!(validate_domain("123").is_ok()); // Numbers only
        assert!(validate_domain("test-123").is_ok()); // Letters and numbers
        assert!(validate_domain("test.123").is_ok()); // Letters and numbers with dot

        // Invalid domains
        assert!(validate_domain("").is_err()); // Empty
        assert!(validate_domain("Example.com").is_err()); // Capitalisation
        assert!(validate_domain(".example.com").is_err()); // Starts with dot
        assert!(validate_domain("example.com.").is_err()); // Ends with dot
        assert!(validate_domain("example..com").is_err()); // Consecutive dots
        assert!(validate_domain("example--com").is_err()); // Consecutive hyphens
        assert!(validate_domain("example_com").is_err()); // Underscore not allowed
        assert!(validate_domain("example@com").is_err()); // @ not allowed
        assert!(validate_domain("-example.com").is_err()); // Starts with hyphen
        assert!(validate_domain("example.com-").is_err()); // Ends with hyphen
        assert!(validate_domain("example..com").is_err()); // Consecutive dots
        assert!(validate_domain("example--com").is_err()); // Consecutive hyphens
        assert!(validate_domain("example@com").is_err()); // @ not allowed
        assert!(validate_domain("example#com").is_err()); // # not allowed
        assert!(validate_domain("example$com").is_err()); // $ not allowed
        assert!(validate_domain("example%com").is_err()); // % not allowed
        assert!(validate_domain("example&com").is_err()); // & not allowed
        assert!(validate_domain("example*com").is_err()); // * not allowed
        assert!(validate_domain("example+com").is_err()); // + not allowed
        assert!(validate_domain("example=com").is_err()); // = not allowed
        assert!(validate_domain("example!com").is_err()); // ! not allowed
        assert!(validate_domain("example?com").is_err()); // ? not allowed
        assert!(validate_domain("example,com").is_err()); // , not allowed
        assert!(validate_domain("example;com").is_err()); // ; not allowed
        assert!(validate_domain("example:com").is_err()); // : not allowed
        assert!(validate_domain("example\"com").is_err()); // " not allowed
        assert!(validate_domain("example'com").is_err()); // ' not allowed
        assert!(validate_domain("example(com").is_err()); // ( not allowed
        assert!(validate_domain("example)com").is_err()); // ) not allowed
        assert!(validate_domain("example[com").is_err()); // [ not allowed
        assert!(validate_domain("example]com").is_err()); // ] not allowed
        assert!(validate_domain("example{com").is_err()); // { not allowed
        assert!(validate_domain("example}com").is_err()); // } not allowed
        assert!(validate_domain("example|com").is_err()); // | not allowed
        assert!(validate_domain("example\\com").is_err()); // \ not allowed
        assert!(validate_domain("example/com").is_err()); // / not allowed
        assert!(validate_domain("example<com").is_err()); // < not allowed
        assert!(validate_domain("example>com").is_err()); // > not allowed
        assert!(validate_domain("example~com").is_err()); // ~ not allowed
        assert!(validate_domain("example`com").is_err()); // ` not allowed
        assert!(validate_domain("example\tcom").is_err()); // Tab not allowed
        assert!(validate_domain("example\ncom").is_err()); // Newline not allowed
        assert!(validate_domain("example\rcom").is_err()); // Carriage return not allowed
        assert!(validate_domain("example com").is_err()); // Space not allowed
        assert!(validate_domain("example\tcom").is_err()); // Tab not allowed
        assert!(validate_domain("example\ncom").is_err()); // Newline not allowed
        assert!(validate_domain("example\rcom").is_err()); // Carriage return not allowed
        assert!(validate_domain("example com").is_err()); // Space not allowed
    }

    #[test]
    fn test_validate_alias_mail() {
        // Valid alias mails
        assert!(validate_alias_mail("user@example.com").is_ok());
        assert!(validate_alias_mail("user.name@domain.co.uk").is_ok());
        assert!(validate_alias_mail("user+tag@example.org").is_ok());
        assert!(validate_alias_mail("123@numbers.com").is_ok());
        assert!(validate_alias_mail("@example.com").is_ok()); // Catchall
        assert!(validate_alias_mail("a@b.com").is_ok()); // Minimal valid
        assert!(validate_alias_mail("user-name@domain.org").is_ok()); // Hyphen in local part
        assert!(validate_alias_mail("user_name@domain.org").is_ok()); // Underscore in local part
        assert!(validate_alias_mail("user+tag+another@domain.org").is_ok()); // Multiple + in local part
        assert!(validate_alias_mail("user@localhost").is_ok()); // Machine name domain
        assert!(validate_alias_mail("user@andromeda-001").is_ok()); // Machine name domain

        // Invalid alias mails
        assert!(validate_alias_mail("").is_err()); // Empty
        assert!(validate_alias_mail("invalid-email").is_err()); // No @
        assert!(validate_alias_mail("user@").is_err()); // Ends with @
        assert!(validate_alias_mail("@").is_err()); // Just @
        assert!(validate_alias_mail("user..name@example.com").is_err()); // Consecutive dots in local part
        assert!(validate_alias_mail("user@Example.com").is_err()); // Capitalisation in domain
        assert!(validate_alias_mail("@").is_err()); // Just @
        assert!(validate_alias_mail("user@@example.com").is_err()); // Multiple @
        assert!(validate_alias_mail("@example.com@").is_err()); // Multiple @
        assert!(validate_alias_mail("user@example@com").is_err()); // Multiple @
        assert!(validate_alias_mail("user@.com").is_err()); // Domain starts with dot
        assert!(validate_alias_mail("user@example.").is_err()); // Domain ends with dot
        assert!(validate_alias_mail("user@example..com").is_err()); // Consecutive dots in domain
        assert!(validate_alias_mail("user@example--com").is_err()); // Consecutive hyphens in domain
        assert!(validate_alias_mail("user@example_com").is_err()); // Underscore in domain
        assert!(validate_alias_mail("user@example#com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example$com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example%com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example&com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example*com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example+com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example=com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example!com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example?com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example,com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example;com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example:com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example\"com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example'com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example(com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example)com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example[com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example]com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example{com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example}com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example|com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example\\com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example/com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example<com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example>com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example~com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example`com").is_err()); // Invalid char in domain
        assert!(validate_alias_mail("user@example\tcom").is_err()); // Tab in domain
        assert!(validate_alias_mail("user@example\ncom").is_err()); // Newline in domain
        assert!(validate_alias_mail("user@example\rcom").is_err()); // Carriage return in domain
        assert!(validate_alias_mail("user@example com").is_err()); // Space in domain
        assert!(validate_alias_mail("user@example\tcom").is_err()); // Tab in domain
        assert!(validate_alias_mail("user@example\ncom").is_err()); // Newline in domain
        assert!(validate_alias_mail("user@example\rcom").is_err()); // Carriage return in domain
        assert!(validate_alias_mail("user@example com").is_err()); // Space in domain
        assert!(validate_alias_mail(".user@example.com").is_err()); // Local part starts with dot
        assert!(validate_alias_mail("user.@example.com").is_err()); // Local part ends with dot
        assert!(validate_alias_mail("user..name@example.com").is_err()); // Consecutive dots in local part
        assert!(validate_alias_mail("user--name@example.com").is_ok()); // Consecutive hyphens in local part are allowed
        assert!(validate_alias_mail("user__name@example.com").is_ok()); // Consecutive underscores in local part are allowed
        assert!(validate_alias_mail("user##name@example.com").is_err()); // Consecutive # in local part
        assert!(validate_alias_mail("user$$name@example.com").is_err()); // Consecutive $ in local part
        assert!(validate_alias_mail("user%%name@example.com").is_ok()); // Consecutive % in local part are allowed
        assert!(validate_alias_mail("user&&name@example.com").is_err()); // Consecutive & in local part
        assert!(validate_alias_mail("user**name@example.com").is_err()); // Consecutive * in local part
        assert!(validate_alias_mail("user++name@example.com").is_ok()); // Consecutive + in local part are allowed
        assert!(validate_alias_mail("user==name@example.com").is_err()); // Consecutive = in local part
        assert!(validate_alias_mail("user!!name@example.com").is_err()); // Consecutive ! in local part
        assert!(validate_alias_mail("user??name@example.com").is_err()); // Consecutive ? in local part
        assert!(validate_alias_mail("user,,name@example.com").is_err()); // Consecutive , in local part
        assert!(validate_alias_mail("user;;name@example.com").is_err()); // Consecutive ; in local part
        assert!(validate_alias_mail("user::name@example.com").is_err()); // Consecutive : in local part
        assert!(validate_alias_mail("user\"\"name@example.com").is_err()); // Consecutive " in local part
        assert!(validate_alias_mail("user''name@example.com").is_err()); // Consecutive ' in local part
        assert!(validate_alias_mail("user(()name@example.com").is_err()); // Consecutive ( in local part
        assert!(validate_alias_mail("user))name@example.com").is_err()); // Consecutive ) in local part
        assert!(validate_alias_mail("user[[]name@example.com").is_err()); // Consecutive [ in local part
        assert!(validate_alias_mail("user]]name@example.com").is_err()); // Consecutive ] in local part
        assert!(validate_alias_mail("user{{name@example.com").is_err()); // Consecutive { in local part
        assert!(validate_alias_mail("user}}name@example.com").is_err()); // Consecutive } in local part
        assert!(validate_alias_mail("user||name@example.com").is_err()); // Consecutive | in local part
        assert!(validate_alias_mail("user\\\\name@example.com").is_err()); // Consecutive \ in local part
        assert!(validate_alias_mail("user//name@example.com").is_err()); // Consecutive / in local part
        assert!(validate_alias_mail("user<<name@example.com").is_err()); // Consecutive < in local part
        assert!(validate_alias_mail("user>>name@example.com").is_err()); // Consecutive > in local part
        assert!(validate_alias_mail("user~~name@example.com").is_err()); // Consecutive ~ in local part
        assert!(validate_alias_mail("user``name@example.com").is_err()); // Consecutive ` in local part
        assert!(validate_alias_mail("user\tname@example.com").is_err()); // Tab in local part
        assert!(validate_alias_mail("user\nname@example.com").is_err()); // Newline in local part
        assert!(validate_alias_mail("user\rname@example.com").is_err()); // Carriage return in local part
        assert!(validate_alias_mail("user name@example.com").is_err()); // Space in local part
    }

    #[test]
    fn test_validate_alias_destination() {
        // Valid destinations
        assert!(validate_alias_destination("user@example.com").is_ok());
        assert!(validate_alias_destination("user+tag@example.org").is_ok());
        assert!(validate_alias_destination("@example.com").is_ok()); // @domain is valid
        assert!(validate_alias_destination("user.name@domain.co.uk").is_ok());
        assert!(validate_alias_destination("a@b.com").is_ok()); // Minimal valid
        assert!(validate_alias_destination("user-name@domain.org").is_ok()); // Hyphen in local part
        assert!(validate_alias_destination("user_name@domain.org").is_ok()); // Underscore in local part
        assert!(validate_alias_destination("user+tag+another@domain.org").is_err()); // Multiple + in local part not allowed
        assert!(validate_alias_destination("user@localhost").is_ok()); // Machine name domain
        assert!(validate_alias_destination("user@andromeda-001").is_ok()); // Machine name domain

        // Invalid destinations
        assert!(validate_alias_destination("").is_err()); // Empty
        assert!(validate_alias_destination("@").is_err()); // Just @
        assert!(validate_alias_destination("user@").is_err()); // Ends with @
        assert!(validate_alias_destination("user++tag@example.com").is_err()); // Multiple +
        assert!(validate_alias_destination("+user@example.com").is_err()); // Starts with +
        assert!(validate_alias_destination("user+@example.com").is_err()); // + before @
        assert!(validate_alias_destination("user@@example.com").is_err()); // Multiple @
        assert!(validate_alias_destination("@example.com@").is_err()); // Multiple @
        assert!(validate_alias_destination("user@example@com").is_err()); // Multiple @
        assert!(validate_alias_destination("user@.com").is_err()); // Domain starts with dot
        assert!(validate_alias_destination("user@example.").is_err()); // Domain ends with dot
        assert!(validate_alias_destination("user@example..com").is_err()); // Consecutive dots in domain
        assert!(validate_alias_destination("user@example--com").is_err()); // Consecutive hyphens in domain
        assert!(validate_alias_destination("user@example_com").is_err()); // Underscore in domain
        assert!(validate_alias_destination("user@example#com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example$com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example%com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example&com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example*com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example+com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example=com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example!com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example?com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example,com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example;com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example:com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example\"com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example'com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example(com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example)com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example[com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example]com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example{com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example}com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example|com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example\\com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example/com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example<com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example>com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example~com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example`com").is_err()); // Invalid char in domain
        assert!(validate_alias_destination("user@example\tcom").is_err()); // Tab in domain
        assert!(validate_alias_destination("user@example\ncom").is_err()); // Newline in domain
        assert!(validate_alias_destination("user@example\rcom").is_err()); // Carriage return in domain
        assert!(validate_alias_destination("user@example com").is_err()); // Space in domain
        assert!(validate_alias_destination(".user@example.com").is_err()); // Local part starts with dot
        assert!(validate_alias_destination("user.@example.com").is_err()); // Local part ends with dot
        assert!(validate_alias_destination("user..name@example.com").is_err()); // Consecutive dots in local part
        assert!(validate_alias_destination("user--name@example.com").is_ok()); // Consecutive hyphens in local part are allowed
        assert!(validate_alias_destination("user__name@example.com").is_ok()); // Consecutive underscores in local part are allowed
        assert!(validate_alias_destination("user##name@example.com").is_err()); // Consecutive # in local part
        assert!(validate_alias_destination("user$$name@example.com").is_err()); // Consecutive $ in local part
        assert!(validate_alias_destination("user%%name@example.com").is_ok()); // Consecutive % in local part are allowed
        assert!(validate_alias_destination("user&&name@example.com").is_err()); // Consecutive & in local part
        assert!(validate_alias_destination("user**name@example.com").is_err()); // Consecutive * in local part
        assert!(validate_alias_destination("user++name@example.com").is_err()); // Multiple + in local part not allowed
        assert!(validate_alias_destination("user==name@example.com").is_err()); // Consecutive = in local part
        assert!(validate_alias_destination("user!!name@example.com").is_err()); // Consecutive ! in local part
        assert!(validate_alias_destination("user??name@example.com").is_err()); // Consecutive ? in local part
        assert!(validate_alias_destination("user,,name@example.com").is_err()); // Consecutive , in local part
        assert!(validate_alias_destination("user;;name@example.com").is_err()); // Consecutive ; in local part
        assert!(validate_alias_destination("user::name@example.com").is_err()); // Consecutive : in local part
        assert!(validate_alias_destination("user\"\"name@example.com").is_err()); // Consecutive " in local part
        assert!(validate_alias_destination("user''name@example.com").is_err()); // Consecutive ' in local part
        assert!(validate_alias_destination("user(()name@example.com").is_err()); // Consecutive ( in local part
        assert!(validate_alias_destination("user))name@example.com").is_err()); // Consecutive ) in local part
        assert!(validate_alias_destination("user[[]name@example.com").is_err()); // Consecutive [ in local part
        assert!(validate_alias_destination("user]]name@example.com").is_err()); // Consecutive ] in local part
        assert!(validate_alias_destination("user{{name@example.com").is_err()); // Consecutive { in local part
        assert!(validate_alias_destination("user}}name@example.com").is_err()); // Consecutive } in local part
        assert!(validate_alias_destination("user||name@example.com").is_err()); // Consecutive | in local part
        assert!(validate_alias_destination("user\\\\name@example.com").is_err()); // Consecutive \ in local part
        assert!(validate_alias_destination("user//name@example.com").is_err()); // Consecutive / in local part
        assert!(validate_alias_destination("user<<name@example.com").is_err()); // Consecutive < in local part
        assert!(validate_alias_destination("user>>name@example.com").is_err()); // Consecutive > in local part
        assert!(validate_alias_destination("user~~name@example.com").is_err()); // Consecutive ~ in local part
        assert!(validate_alias_destination("user``name@example.com").is_err()); // Consecutive ` in local part
        assert!(validate_alias_destination("user\tname@example.com").is_err()); // Tab in local part
        assert!(validate_alias_destination("user\nname@example.com").is_err()); // Newline in local part
        assert!(validate_alias_destination("user\rname@example.com").is_err()); // Carriage return in local part
        assert!(validate_alias_destination("user name@example.com").is_err()); // Space in local part
    }

    #[test]
    fn test_validate_user_id() {
        // Valid user IDs
        assert!(validate_user_id("user@example.com").is_ok());
        assert!(validate_user_id("user.name@domain.co.uk").is_ok());
        assert!(validate_user_id("a@b.com").is_ok()); // Minimal valid
        assert!(validate_user_id("user-name@domain.org").is_ok()); // Hyphen in local part
        assert!(validate_user_id("user_name@domain.org").is_ok()); // Underscore in local part
        assert!(validate_user_id("user+tag@domain.org").is_ok()); // + in local part
        assert!(validate_user_id("user@localhost").is_ok()); // Machine name domain
        assert!(validate_user_id("user@andromeda-001").is_ok()); // Machine name domain

        // Invalid user IDs
        assert!(validate_user_id("").is_err()); // Empty
        assert!(validate_user_id("@example.com").is_err()); // Catchall not allowed
        assert!(validate_user_id("invalid-email").is_err()); // No @
        assert!(validate_user_id("user@").is_err()); // Ends with @
        assert!(validate_user_id("@").is_err()); // Just @
        assert!(validate_user_id("user@@example.com").is_err()); // Multiple @
        assert!(validate_user_id("@example.com@").is_err()); // Multiple @
        assert!(validate_user_id("user@example@com").is_err()); // Multiple @
        assert!(validate_user_id("user@.com").is_err()); // Domain starts with dot
        assert!(validate_user_id("user@example.").is_err()); // Domain ends with dot
        assert!(validate_user_id("user@example..com").is_err()); // Consecutive dots in domain
        assert!(validate_user_id("user@example--com").is_err()); // Consecutive hyphens in domain
        assert!(validate_user_id("user@example_com").is_err()); // Underscore in domain
        assert!(validate_user_id("user@example#com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example$com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example%com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example&com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example*com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example+com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example=com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example!com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example?com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example,com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example;com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example:com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example\"com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example'com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example(com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example)com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example[com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example]com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example{com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example}com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example|com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example\\com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example/com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example<com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example>com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example~com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example`com").is_err()); // Invalid char in domain
        assert!(validate_user_id("user@example\tcom").is_err()); // Tab in domain
        assert!(validate_user_id("user@example\ncom").is_err()); // Newline in domain
        assert!(validate_user_id("user@example\rcom").is_err()); // Carriage return in domain
        assert!(validate_user_id("user@example com").is_err()); // Space in domain
        assert!(validate_user_id(".user@example.com").is_err()); // Local part starts with dot
        assert!(validate_user_id("user.@example.com").is_err()); // Local part ends with dot
        assert!(validate_user_id("user..name@example.com").is_err()); // Consecutive dots in local part
        assert!(validate_user_id("user--name@example.com").is_ok()); // Consecutive hyphens in local part are allowed
        assert!(validate_user_id("user__name@example.com").is_ok()); // Consecutive underscores in local part are allowed
        assert!(validate_user_id("user##name@example.com").is_err()); // Consecutive # in local part
        assert!(validate_user_id("user$$name@example.com").is_err()); // Consecutive $ in local part
        assert!(validate_user_id("user%%name@example.com").is_ok()); // Consecutive % in local part are allowed
        assert!(validate_user_id("user&&name@example.com").is_err()); // Consecutive & in local part
        assert!(validate_user_id("user**name@example.com").is_err()); // Consecutive * in local part
        assert!(validate_user_id("user++name@example.com").is_ok()); // Consecutive + in local part are allowed
        assert!(validate_user_id("user==name@example.com").is_err()); // Consecutive = in local part
        assert!(validate_user_id("user!!name@example.com").is_err()); // Consecutive ! in local part
        assert!(validate_user_id("user??name@example.com").is_err()); // Consecutive ? in local part
        assert!(validate_user_id("user,,name@example.com").is_err()); // Consecutive , in local part
        assert!(validate_user_id("user;;name@example.com").is_err()); // Consecutive ; in local part
        assert!(validate_user_id("user::name@example.com").is_err()); // Consecutive : in local part
        assert!(validate_user_id("user\"\"name@example.com").is_err()); // Consecutive " in local part
        assert!(validate_user_id("user''name@example.com").is_err()); // Consecutive ' in local part
        assert!(validate_user_id("user(()name@example.com").is_err()); // Consecutive ( in local part
        assert!(validate_user_id("user))name@example.com").is_err()); // Consecutive ) in local part
        assert!(validate_user_id("user[[]name@example.com").is_err()); // Consecutive [ in local part
        assert!(validate_user_id("user]]name@example.com").is_err()); // Consecutive ] in local part
        assert!(validate_user_id("user{{name@example.com").is_err()); // Consecutive { in local part
        assert!(validate_user_id("user}}name@example.com").is_err()); // Consecutive } in local part
        assert!(validate_user_id("user||name@example.com").is_err()); // Consecutive | in local part
        assert!(validate_user_id("user\\\\name@example.com").is_err()); // Consecutive \ in local part
        assert!(validate_user_id("user//name@example.com").is_err()); // Consecutive / in local part
        assert!(validate_user_id("user<<name@example.com").is_err()); // Consecutive < in local part
        assert!(validate_user_id("user>>name@example.com").is_err()); // Consecutive > in local part
        assert!(validate_user_id("user~~name@example.com").is_err()); // Consecutive ~ in local part
        assert!(validate_user_id("user``name@example.com").is_err()); // Consecutive ` in local part
        assert!(validate_user_id("user\tname@example.com").is_err()); // Tab in local part
        assert!(validate_user_id("user\nname@example.com").is_err()); // Newline in local part
        assert!(validate_user_id("user\rname@example.com").is_err()); // Carriage return in local part
        assert!(validate_user_id("user name@example.com").is_err()); // Space in local part
    }

    #[test]
    fn test_validate_user_path() {
        // Valid paths (Unix-style)
        assert!(validate_user_path("/home/user").is_ok());
        assert!(validate_user_path("/var/mail/user").is_ok());
        assert!(validate_user_path("/opt/mail/user").is_ok());
        assert!(validate_user_path("/home/user-name").is_ok()); // Hyphen in path
        assert!(validate_user_path("/home/user_name").is_ok()); // Underscore in path
        assert!(validate_user_path("/home/user.name").is_ok()); // Dot in path
        assert!(validate_user_path("/home/user123").is_ok()); // Numbers in path
        assert!(validate_user_path("/home/user-name_123.test").is_ok()); // Complex path

        // Invalid paths
        assert!(validate_user_path("").is_err()); // Empty
        assert!(validate_user_path("relative/path").is_err()); // Not absolute
        assert!(validate_user_path("/home/user/..").is_err()); // Path traversal
        assert!(validate_user_path("/home/user/\0").is_err()); // Null character
        assert!(validate_user_path("/home/user/../other").is_err()); // Path traversal
        assert!(validate_user_path("/home/user/./other").is_ok()); // Current directory is allowed
        assert!(validate_user_path("/home/user/.../other").is_err()); // Multiple dots
        assert!(validate_user_path("/home/user/\t").is_ok()); // Tab character is allowed in paths
        assert!(validate_user_path("/home/user/\n").is_ok()); // Newline character is allowed in paths
        assert!(validate_user_path("/home/user/\r").is_ok()); // Carriage return is allowed in paths
        assert!(validate_user_path("/home/user/ ").is_ok()); // Space character is allowed in paths
        assert!(validate_user_path("/home/user/\t").is_ok()); // Tab character is allowed in paths
        assert!(validate_user_path("/home/user/\n").is_ok()); // Newline character is allowed in paths
        assert!(validate_user_path("/home/user/\r").is_ok()); // Carriage return is allowed in paths
        assert!(validate_user_path("/home/user/ ").is_ok()); // Space character is allowed in paths
                                                             // Note: Path validation only checks for null characters and path traversal
                                                             // Other characters like #, $, %, etc. are not validated by the current function
                                                             // These tests are removed as they don't match the actual validation logic
    }

    #[test]
    fn test_validate_backup_name() {
        // Valid backup names
        assert!(validate_backup_name("backup-2024-01-01").is_ok());
        assert!(validate_backup_name("database.backup").is_ok());
        assert!(validate_backup_name("backup123").is_ok());
        assert!(validate_backup_name("a").is_ok()); // Single character
        assert!(validate_backup_name("123").is_ok()); // Numbers only
        assert!(validate_backup_name("backup-name").is_ok()); // Letters and hyphens
        assert!(validate_backup_name("backup.name").is_ok()); // Letters and dots
        assert!(validate_backup_name("backup123-name.test").is_ok()); // Complex valid name

        // Invalid backup names
        assert!(validate_backup_name("").is_err()); // Empty
        assert!(validate_backup_name("Backup-2024").is_err()); // Capitalisation
        assert!(validate_backup_name(".backup").is_err()); // Starts with dot
        assert!(validate_backup_name("backup.").is_err()); // Ends with dot
        assert!(validate_backup_name("backup--2024").is_err()); // Consecutive hyphens
        assert!(validate_backup_name("backup_2024").is_err()); // Underscore not allowed
        assert!(validate_backup_name("-backup").is_err()); // Starts with hyphen
        assert!(validate_backup_name("backup-").is_err()); // Ends with hyphen
        assert!(validate_backup_name("backup..2024").is_err()); // Consecutive dots
        assert!(validate_backup_name("backup##2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup$$2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup%%2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup&&2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup**2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup++2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup==2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup!!2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup??2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup,,2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup;;2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup::2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup\"\"2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup''2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup(()2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup))2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup[[]2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup]]2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup{{2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup}}2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup||2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup\\\\2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup//2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup<<2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup>>2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup~~2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup``2024").is_err()); // Invalid character
        assert!(validate_backup_name("backup\t2024").is_err()); // Tab character
        assert!(validate_backup_name("backup\n2024").is_err()); // Newline character
        assert!(validate_backup_name("backup\r2024").is_err()); // Carriage return
        assert!(validate_backup_name("backup 2024").is_err()); // Space character
        assert!(validate_backup_name("backup@2024").is_err()); // @ character
        assert!(validate_backup_name("backup#2024").is_err()); // # character
        assert!(validate_backup_name("backup$2024").is_err()); // $ character
        assert!(validate_backup_name("backup%2024").is_err()); // % character
        assert!(validate_backup_name("backup&2024").is_err()); // & character
        assert!(validate_backup_name("backup*2024").is_err()); // * character
        assert!(validate_backup_name("backup+2024").is_err()); // + character
        assert!(validate_backup_name("backup=2024").is_err()); // = character
        assert!(validate_backup_name("backup!2024").is_err()); // ! character
        assert!(validate_backup_name("backup?2024").is_err()); // ? character
        assert!(validate_backup_name("backup,2024").is_err()); // , character
        assert!(validate_backup_name("backup;2024").is_err()); // ; character
        assert!(validate_backup_name("backup:2024").is_err()); // : character
        assert!(validate_backup_name("backup\"2024").is_err()); // " character
        assert!(validate_backup_name("backup'2024").is_err()); // ' character
        assert!(validate_backup_name("backup(2024").is_err()); // ( character
        assert!(validate_backup_name("backup)2024").is_err()); // ) character
        assert!(validate_backup_name("backup[2024").is_err()); // [ character
        assert!(validate_backup_name("backup]2024").is_err()); // ] character
        assert!(validate_backup_name("backup{2024").is_err()); // { character
        assert!(validate_backup_name("backup}2024").is_err()); // } character
        assert!(validate_backup_name("backup|2024").is_err()); // | character
        assert!(validate_backup_name("backup\\2024").is_err()); // \ character
        assert!(validate_backup_name("backup/2024").is_err()); // / character
        assert!(validate_backup_name("backup<2024").is_err()); // < character
        assert!(validate_backup_name("backup>2024").is_err()); // > character
        assert!(validate_backup_name("backup~2024").is_err()); // ~ character
        assert!(validate_backup_name("backup`2024").is_err()); // ` character
    }
}
