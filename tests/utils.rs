#[cfg(test)]
mod tests {
    use sortingoffice::models::*;

    #[test]
    fn test_checkbox_deserialization_utility() {
        // Test various checkbox values in form data
        let test_cases = vec![
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&enabled=on", true),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&enabled=true", true),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&enabled=1", true),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&enabled=false", false),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&enabled=off", false),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost&enabled=0", false),
            ("domain=test.com&description=Test&aliases=10&maxquota=1000000&quota=500000&transport=smtp:localhost", false), // No checkbox field
        ];

        for (form_data, expected) in test_cases {
            let form: DomainForm = serde_urlencoded::from_str(form_data).unwrap();
            assert_eq!(form.enabled, expected, "Failed for input: {form_data}");
        }
    }
}
