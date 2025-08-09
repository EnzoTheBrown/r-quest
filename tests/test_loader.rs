pub const TEMPLATE_1: &str = r#"
[api]
name = "test_1"
base_url = "https://api.example.com"

[[request]]
name   = "docs"
method = "GET"
path   = "/docs"

[[request]]
name = "login"
method = "POST"
path = "/login?entity_id=${USER_ID}&email=elb@ibanfirst.com&user_id=1"
body = """
{
	"username": "${USERNAME}",
	"password": "${PASSWORD}"
}
"""
spell = """
let map = #{ TOKEN: data["access_token"].to_string() };
return map;
"""
	[[request.header]]
	key = "Content-Type"
	value = "application/x-www-form-urlencoded"
"#;

#[cfg(test)]
mod loader_tests {
    use super::TEMPLATE_1;
    use qwest::load_config;
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn test_load_config() {
        fs::write("/tmp/test-qwest.toml", TEMPLATE_1).expect("couldn't create test fixture");
        let mut vars = HashMap::new();
        let config = load_config("/tmp/test-qwest.toml", vars).expect("Failed to load config");
        assert_eq!(config.api.name, "test_1");
        assert_eq!(config.api.base_url, "https://api.example.com");
        assert_eq!(config.requests.len(), 2);

        let first_request = &config.requests[0];
        assert_eq!(first_request.name, "docs");
        assert_eq!(first_request.method, "GET");
        assert_eq!(first_request.path, "/docs");
        assert!(first_request.headers.is_empty());
        assert!(first_request.body.is_none());
        assert!(first_request.params.is_none());
        assert!(first_request.spell.is_none());

        let second_request = &config.requests[1];
        assert_eq!(second_request.name, "login");
        assert_eq!(second_request.method, "POST");
        assert_eq!(
            second_request.path,
            "/login?entity_id=${USER_ID}&email=elb@ibanfirst.com&user_id=1"
        );
        assert!(!second_request.headers.is_empty());
        assert!(second_request.body.is_some());
    }
}
