use crate::{db, AppState};
use axum::http::HeaderMap;

/// Find the most common aliases from existing aliases in the database
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

            println!("[ANALYTICS DEBUG] Alias name counts: {:?}", alias_counts);

            // Find the most common aliases that meet the minimum occurrence threshold
            let mut sorted_aliases: Vec<_> = alias_counts
                .into_iter()
                .filter(|(_, count)| *count >= min_occurrence_count)
                .collect();

            sorted_aliases.sort_by(|a, b| b.1.cmp(&a.1));

            let common_aliases: Vec<String> = sorted_aliases
                .into_iter()
                .take(limit)
                .map(|(alias, count)| {
                    println!(
                        "[ANALYTICS DEBUG] Common alias: {} (count: {})",
                        alias, count
                    );
                    alias
                })
                .collect();

            println!(
                "[ANALYTICS DEBUG] Found {} common aliases from database analysis",
                common_aliases.len()
            );

            common_aliases
        }
        Err(e) => {
            // Error getting aliases, return empty vector
            println!(
                "[ANALYTICS DEBUG] Error getting aliases for common alias analysis: {:?}",
                e
            );
            Vec::new()
        }
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

            // Count destinations
            let mut destination_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            for alias in aliases {
                if !alias.destination.is_empty() {
                    *destination_counts.entry(alias.destination).or_insert(0) += 1;
                }
            }

            println!(
                "[ANALYTICS DEBUG] Destination counts: {:?}",
                destination_counts
            );

            // Find the most common destination
            if let Some((most_common_dest, count)) = destination_counts
                .into_iter()
                .max_by_key(|&(_, count)| count)
            {
                println!(
                    "[ANALYTICS DEBUG] Most common destination: {} (count: {})",
                    most_common_dest, count
                );
                // Only use this destination if it appears at least 3 times
                if count >= 3 {
                    println!(
                        "[ANALYTICS DEBUG] Using most common destination: {}",
                        most_common_dest
                    );
                    most_common_dest
                } else {
                    println!("[ANALYTICS DEBUG] Most common destination count ({}) is less than 3, using empty", count);
                    String::new()
                }
            } else {
                println!("[ANALYTICS DEBUG] No destinations found, using empty");
                String::new()
            }
        }
        Err(e) => {
            // Error getting aliases, return empty string
            println!("[ANALYTICS DEBUG] Error getting aliases: {:?}", e);
            String::new()
        }
    }
}
