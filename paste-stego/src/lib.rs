//! paste-stego — ZOS plugin: Steganographic embed/extract for paste content
//!
//! Commands: embed,extract,verify

pub fn execute(command: &str, args: &[String]) -> Result<serde_json::Value, String> {
    let input = args.first().map(|s| s.as_str()).unwrap_or("");
    match command {
        "embed" => Ok(serde_json::json!({"command": "embed", "input": input, "status": "ok"})),
        "extract" => Ok(serde_json::json!({"command": "extract", "input": input, "status": "ok"})),
        "verify" => Ok(serde_json::json!({"command": "verify", "input": input, "status": "ok"})),
        _ => Err(format!("unknown command: {command}")),
    }
}

pub fn commands() -> Vec<&'static str> {
    vec!["embed", "extract", "verify"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands() {
        assert!(!commands().is_empty());
    }

    #[test]
    fn test_embed() {
        let r = execute("embed", &["test".to_string()]);
        assert!(r.is_ok());
    }

    #[test]
    fn test_extract() {
        let r = execute("extract", &["test".to_string()]);
        assert!(r.is_ok());
    }

    #[test]
    fn test_verify() {
        let r = execute("verify", &["test".to_string()]);
        assert!(r.is_ok());
    }
    #[test]
    fn test_unknown() {
        assert!(execute("nonexistent", &[]).is_err());
    }
}
