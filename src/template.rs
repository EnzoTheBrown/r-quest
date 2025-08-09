pub const TEMPLATE: &str = r#"
[api]
name = "{name}"
base_url = ""

[[requests]]
name   = "docs"
method = "GET"
path   = "/docs"
"#;
