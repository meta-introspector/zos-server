//! paste-home — ZOS plugin: Home page and service status
//!
//! Commands: home,status

pub fn execute(command: &str, args: &[String]) -> Result<serde_json::Value, String> {
    let input = args.first().map(|s| s.as_str()).unwrap_or("");
    match command {
        "home" => Ok(serde_json::json!({"command": "home", "input": input, "status": "ok"})),
        "status" => Ok(serde_json::json!({"command": "status", "input": input, "status": "ok"})),
        _ => Err(format!("unknown command: {command}")),
    }
}

pub fn commands() -> Vec<&'static str> {
    vec!["home", "status"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands() {
        assert!(!commands().is_empty());
    }

    #[test]
    fn test_home() {
        let r = execute("home", &["test".to_string()]);
        assert!(r.is_ok());
    }

    #[test]
    fn test_status() {
        let r = execute("status", &["test".to_string()]);
        assert!(r.is_ok());
    }
    #[test]
    fn test_unknown() {
        assert!(execute("nonexistent", &[]).is_err());
    }
}
