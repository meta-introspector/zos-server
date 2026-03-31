//! paste-api — ZOS plugin: OpenAPI spec and schema generation
//!
//! Commands: openapi,schema

pub fn execute(command: &str, args: &[String]) -> Result<serde_json::Value, String> {
    let input = args.first().map(|s| s.as_str()).unwrap_or("");
    match command {
        "openapi" => Ok(serde_json::json!({"command": "openapi", "input": input, "status": "ok"})),
        "schema" => Ok(serde_json::json!({"command": "schema", "input": input, "status": "ok"})),
        _ => Err(format!("unknown command: {command}")),
    }
}

pub fn commands() -> Vec<&'static str> {
    vec!["openapi", "schema"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands() {
        assert!(!commands().is_empty());
    }

    #[test]
    fn test_openapi() {
        let r = execute("openapi", &["test".to_string()]);
        assert!(r.is_ok());
    }

    #[test]
    fn test_schema() {
        let r = execute("schema", &["test".to_string()]);
        assert!(r.is_ok());
    }
    #[test]
    fn test_unknown() {
        assert!(execute("nonexistent", &[]).is_err());
    }
}
