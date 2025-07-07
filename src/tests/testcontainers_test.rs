#[cfg(test)]
mod tests {
    use crate::tests::testcontainers_setup::{setup_test_db, cleanup_test_db};
    use diesel::{RunQueryDsl, QueryableByName};

    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    #[derive(QueryableByName)]
    struct TestResult {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        result: i32,
    }

    #[test]
    fn test_testcontainers_mysql_works() {
        let container = setup_test_db();
        let pool = container.get_pool();
        
        // Test that we can connect and execute a simple query
        let mut conn = pool.get().expect("Failed to get connection");
        let result: TestResult = diesel::sql_query("SELECT 1 as result").get_result(&mut conn).expect("Failed to execute query");
        assert_eq!(result.result, 1);
        
        cleanup_test_db(&container);
    }

    #[test]
    fn test_testcontainers_tables_exist() {
        let container = setup_test_db();
        let pool = container.get_pool();
        
        let mut conn = pool.get().expect("Failed to get connection");
        
        // Test that tables exist by checking their row counts
        let domains_count: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM domains").get_result(&mut conn).expect("Failed to count domains");
        let users_count: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM users").get_result(&mut conn).expect("Failed to count users");
        let aliases_count: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM aliases").get_result(&mut conn).expect("Failed to count aliases");
        let backups_count: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM backups").get_result(&mut conn).expect("Failed to count backups");
        
        assert_eq!(domains_count.count, 0);
        assert_eq!(users_count.count, 0);
        assert_eq!(aliases_count.count, 0);
        assert_eq!(backups_count.count, 0);
        
        cleanup_test_db(&container);
    }

    #[test]
    fn test_testcontainers_isolation() {
        // Test that each test gets its own isolated database
        let container1 = setup_test_db();
        let pool1 = container1.get_pool();
        
        let container2 = setup_test_db();
        let pool2 = container2.get_pool();
        
        // Insert data in first container
        let mut conn1 = pool1.get().expect("Failed to get connection");
        diesel::sql_query("INSERT INTO domains (domain, transport, enabled, created, modified) VALUES ('test1.com', 'virtual', 1, NOW(), NOW())")
            .execute(&mut conn1)
            .expect("Failed to insert test data");
        
        // Check that data exists in first container
        let count1: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM domains").get_result(&mut conn1).expect("Failed to count domains");
        assert_eq!(count1.count, 1);
        
        // Check that data does NOT exist in second container (isolation)
        let mut conn2 = pool2.get().expect("Failed to get connection");
        let count2: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM domains").get_result(&mut conn2).expect("Failed to count domains");
        assert_eq!(count2.count, 0);
        
        cleanup_test_db(&container1);
        cleanup_test_db(&container2);
    }
} 
