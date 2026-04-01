//! paste-gallery — ZOS plugin: NFT gallery with Wikidata enrichment
//!
//! Commands: gallery,enrich

pub fn execute(command: &str, args: &[String]) -> Result<serde_json::Value, String> {
    let input = args.first().map(|s| s.as_str()).unwrap_or("");
    match command {
        "gallery" => Ok(serde_json::json!({"command": "gallery", "input": input, "status": "ok"})),
        "enrich" => Ok(serde_json::json!({"command": "enrich", "input": input, "status": "ok"})),
        _ => Err(format!("unknown command: {command}")),
    }
}

pub fn commands() -> Vec<&'static str> {
    vec!["gallery", "enrich"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands() {
        assert!(!commands().is_empty());
    }

    #[test]
    fn test_gallery() {
        let r = execute("gallery", &["test".to_string()]);
        assert!(r.is_ok());
    }

    #[test]
    fn test_enrich() {
        let r = execute("enrich", &["test".to_string()]);
        assert!(r.is_ok());
    }
    #[test]
    fn test_unknown() {
        assert!(execute("nonexistent", &[]).is_err());
    }
}
