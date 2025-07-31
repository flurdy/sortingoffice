use crate::{db, AppState};
use axum::http::HeaderMap;

/// Find the most common aliases from existing aliases in the database
/// Core logic for finding common aliases from a list of aliases
fn find_common_aliases_from_list(
    aliases: &[crate::models::Alias],
    limit: usize,
    min_occurrence_count: usize,
) -> Vec<String> {
    // Count alias names (part before @)
    let mut alias_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for alias in aliases {
        if let Some(alias_name) = alias.mail.split('@').next() {
            if !alias_name.is_empty() {
                *alias_counts.entry(alias_name.to_string()).or_insert(0) += 1;
            }
        }
    }

    // Find the most common aliases that meet the minimum occurrence threshold
    let mut sorted_aliases: Vec<_> = alias_counts
        .into_iter()
        .filter(|(_, count)| *count >= min_occurrence_count)
        .collect();

    sorted_aliases.sort_by(|a, b| b.1.cmp(&a.1));

    sorted_aliases
        .into_iter()
        .take(limit)
        .map(|(alias, _)| alias)
        .collect()
}

pub async fn find_database_common_aliases(
    state: &AppState,
    headers: &HeaderMap,
    limit: usize,
    min_occurrence_count: usize,
) -> Vec<String> {
    // Try to get the database pool
    let pool = match crate::handlers::utils::get_current_db_pool(state, headers).await {
        Ok(pool) => pool,
        Err(_) => {
            // If we can't get the pool, return empty vector
            println!("[ANALYTICS DEBUG] Could not get database pool for common alias lookup");
            return Vec::new();
        }
    };

    // Get all aliases from the database
    match db::get_aliases(&pool) {
        Ok(aliases) => {
            println!(
                "[ANALYTICS DEBUG] Found {} aliases in database for common alias analysis",
                aliases.len()
            );

            let common_aliases =
                find_common_aliases_from_list(&aliases, limit, min_occurrence_count);

            println!(
                "[ANALYTICS DEBUG] Found {} common aliases from database analysis",
                common_aliases.len()
            );

            common_aliases
        }
        Err(e) => {
            // Error getting aliases, return empty vector
            println!("[ANALYTICS DEBUG] Error getting aliases for common alias analysis: {e:?}");
            Vec::new()
        }
    }
}

/// Core logic for finding the most common destination from a list of aliases
fn find_most_common_destination_from_list(aliases: &[crate::models::Alias]) -> String {
    // Count destinations
    let mut destination_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for alias in aliases {
        if !alias.destination.is_empty() {
            *destination_counts
                .entry(alias.destination.clone())
                .or_insert(0) += 1;
        }
    }

    // Find the most common destination
    if let Some((most_common_dest, count)) = destination_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
    {
        // Only use this destination if it appears at least 3 times
        if count >= 3 {
            most_common_dest
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

/// Find the most common destination from existing aliases in the database
pub async fn find_most_common_destination(state: &AppState, headers: &HeaderMap) -> String {
    // Try to get the database pool
    let pool = match crate::handlers::utils::get_current_db_pool(state, headers).await {
        Ok(pool) => pool,
        Err(_) => {
            // If we can't get the pool, return empty string
            println!("[ANALYTICS DEBUG] Could not get database pool for destination lookup");
            return String::new();
        }
    };

    // Get all aliases from the database
    match db::get_aliases(&pool) {
        Ok(aliases) => {
            println!(
                "[ANALYTICS DEBUG] Found {} aliases in database",
                aliases.len()
            );

            let result = find_most_common_destination_from_list(&aliases);

            println!(
                "[ANALYTICS DEBUG] Most common destination: {}",
                if result.is_empty() { "empty" } else { &result }
            );

            result
        }
        Err(e) => {
            // Error getting aliases, return empty string
            println!("[ANALYTICS DEBUG] Error getting aliases: {e:?}");
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Alias;

    use std::collections::HashMap;

    // Helper function to create test aliases
    fn create_test_aliases() -> Vec<Alias> {
        vec![
            Alias {
                pkid: 1,
                mail: "postmaster@example.com".to_string(),
                destination: "admin@example.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            Alias {
                pkid: 2,
                mail: "abuse@example.com".to_string(),
                destination: "admin@example.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            Alias {
                pkid: 3,
                mail: "webmaster@example.com".to_string(),
                destination: "admin@example.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            Alias {
                pkid: 4,
                mail: "postmaster@another.com".to_string(),
                destination: "admin@another.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            Alias {
                pkid: 5,
                mail: "abuse@another.com".to_string(),
                destination: "admin@another.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            Alias {
                pkid: 6,
                mail: "webmaster@another.com".to_string(),
                destination: "admin@another.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            Alias {
                pkid: 7,
                mail: "postmaster@third.com".to_string(),
                destination: "admin@third.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            Alias {
                pkid: 8,
                mail: "abuse@third.com".to_string(),
                destination: "admin@third.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            Alias {
                pkid: 9,
                mail: "webmaster@third.com".to_string(),
                destination: "admin@third.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            Alias {
                pkid: 10,
                mail: "unique@example.com".to_string(),
                destination: "admin@example.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
        ]
    }

    #[test]
    fn test_find_common_aliases_from_list_basic() {
        let aliases = create_test_aliases();
        let result = find_common_aliases_from_list(&aliases, 10, 3);

        // Should find common aliases that appear at least 3 times
        assert!(!result.is_empty());

        // postmaster, abuse, webmaster should be in the results
        let expected_aliases = vec!["postmaster", "abuse", "webmaster"];
        for expected in expected_aliases {
            assert!(result.contains(&expected.to_string()));
        }
    }

    #[test]
    fn test_find_common_aliases_from_list_with_limit() {
        let aliases = create_test_aliases();
        let result = find_common_aliases_from_list(&aliases, 2, 3);

        // Should return at most 2 results
        assert!(result.len() <= 2);
    }

    #[test]
    fn test_find_common_aliases_from_list_with_min_occurrence() {
        let aliases = create_test_aliases();
        let result = find_common_aliases_from_list(&aliases, 10, 4);

        // Should not contain unique aliases that only appear once
        assert!(!result.contains(&"unique".to_string()));
    }

    #[test]
    fn test_find_common_aliases_from_list_empty_input() {
        let aliases = vec![];
        let result = find_common_aliases_from_list(&aliases, 10, 3);

        // Should return empty vector
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_common_aliases_from_list_edge_cases() {
        let aliases = create_test_aliases();

        // Test with zero limit
        let result = find_common_aliases_from_list(&aliases, 0, 3);
        assert!(result.is_empty());

        // Test with zero minimum occurrence
        let result = find_common_aliases_from_list(&aliases, 10, 0);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_find_most_common_destination_from_list_basic() {
        let aliases = create_test_aliases();
        let result = find_most_common_destination_from_list(&aliases);

        // Should return the most common destination
        assert!(!result.is_empty());
        // admin@example.com should be the most common destination
        assert!(
            result == "admin@example.com"
                || result == "admin@another.com"
                || result == "admin@third.com"
        );
    }

    #[test]
    fn test_find_most_common_destination_from_list_with_insufficient_count() {
        // Create aliases with low occurrence counts
        let low_count_aliases = vec![
            Alias {
                pkid: 1,
                mail: "test1@example.com".to_string(),
                destination: "rare@example.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            Alias {
                pkid: 2,
                mail: "test2@example.com".to_string(),
                destination: "rare@example.com".to_string(),
                enabled: true,
                created: chrono::Utc::now().naive_utc(),
                modified: chrono::Utc::now().naive_utc(),
            },
            // Only 2 occurrences, less than minimum of 3
        ];

        let result = find_most_common_destination_from_list(&low_count_aliases);

        // Should return empty string since count (2) is less than minimum (3)
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_most_common_destination_from_list_empty_input() {
        let aliases = vec![];
        let result = find_most_common_destination_from_list(&aliases);

        // Should handle empty input gracefully
        assert!(result.is_empty());
    }

    #[test]
    fn test_alias_name_extraction() {
        // Test alias name extraction logic
        let test_cases = vec![
            ("postmaster@example.com", "postmaster"),
            ("abuse@another.com", "abuse"),
            ("webmaster@third.com", "webmaster"),
            ("@example.com", ""), // Empty local part
            ("", ""),             // Empty string
        ];

        for (input, expected) in test_cases {
            let result = input.split('@').next().unwrap_or("");
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_destination_counting_logic() {
        // Test destination counting logic
        let aliases = create_test_aliases();
        let mut destination_counts: HashMap<String, usize> = HashMap::new();

        for alias in aliases {
            if !alias.destination.is_empty() {
                *destination_counts.entry(alias.destination).or_insert(0) += 1;
            }
        }

        // Should have multiple destinations
        assert!(destination_counts.len() > 1);

        // Each destination should have a count
        for (_, count) in &destination_counts {
            assert!(*count > 0);
        }
    }

    #[test]
    fn test_alias_counting_logic() {
        // Test alias counting logic
        let aliases = create_test_aliases();
        let mut alias_counts: HashMap<String, usize> = HashMap::new();

        for alias in aliases {
            if let Some(alias_name) = alias.mail.split('@').next() {
                if !alias_name.is_empty() {
                    *alias_counts.entry(alias_name.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Should have multiple alias names
        assert!(alias_counts.len() > 1);

        // postmaster, abuse, webmaster should each appear 3 times
        assert_eq!(alias_counts.get("postmaster"), Some(&3));
        assert_eq!(alias_counts.get("abuse"), Some(&3));
        assert_eq!(alias_counts.get("webmaster"), Some(&3));

        // unique should appear only once
        assert_eq!(alias_counts.get("unique"), Some(&1));
    }

    #[test]
    fn test_sorting_logic() {
        // Test sorting logic for common aliases
        let mut alias_counts: Vec<(String, usize)> = vec![
            ("postmaster".to_string(), 3),
            ("abuse".to_string(), 3),
            ("webmaster".to_string(), 3),
            ("unique".to_string(), 1),
        ];

        // Sort by count descending
        alias_counts.sort_by(|a, b| b.1.cmp(&a.1));

        // Should be sorted by count descending
        assert_eq!(alias_counts[0].1, 3);
        assert_eq!(alias_counts[3].1, 1);
    }

    #[test]
    fn test_filtering_logic() {
        // Test filtering logic for minimum occurrence
        let alias_counts: Vec<(String, usize)> = vec![
            ("postmaster".to_string(), 3),
            ("abuse".to_string(), 3),
            ("webmaster".to_string(), 3),
            ("unique".to_string(), 1),
        ];

        let filtered: Vec<_> = alias_counts
            .into_iter()
            .filter(|(_, count)| *count >= 3)
            .collect();

        // Should only include aliases with count >= 3
        assert_eq!(filtered.len(), 3);
        for (_, count) in &filtered {
            assert!(*count >= 3);
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test edge cases for analytics functions

        // Empty alias mail
        let empty_alias = Alias {
            pkid: 1,
            mail: "".to_string(),
            destination: "test@example.com".to_string(),
            enabled: true,
            created: chrono::Utc::now().naive_utc(),
            modified: chrono::Utc::now().naive_utc(),
        };

        // Alias with empty destination
        let empty_destination = Alias {
            pkid: 2,
            mail: "test@example.com".to_string(),
            destination: "".to_string(),
            enabled: true,
            created: chrono::Utc::now().naive_utc(),
            modified: chrono::Utc::now().naive_utc(),
        };

        // Alias with @ in the middle
        let malformed_alias = Alias {
            pkid: 3,
            mail: "test@@example.com".to_string(),
            destination: "test@example.com".to_string(),
            enabled: true,
            created: chrono::Utc::now().naive_utc(),
            modified: chrono::Utc::now().naive_utc(),
        };

        // Test that these edge cases are handled gracefully
        assert!(empty_alias.mail.split('@').next().unwrap_or("").is_empty());
        assert!(empty_destination.destination.is_empty());
        // For malformed alias with @@, split('@') will return ["test", "@example.com"]
        // so .next() returns "test", which is not empty
        assert!(!malformed_alias
            .mail
            .split('@')
            .next()
            .unwrap_or("")
            .is_empty());
    }
}
