//! paste-browse — ZOS plugin: Browse and search paste archive
//!
//! Commands: browse, list, search

pub fn execute(command: &str, args: &[String]) -> Result<serde_json::Value, String> {
    let query = args.first().map(|s| s.as_str()).unwrap_or("");
    let limit: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20);

    match command {
        "browse" => Ok(serde_json::json!({
            "page": 1, "limit": limit, "total": 0, "items": [],
        })),
        "list" => Ok(serde_json::json!({
            "limit": limit, "items": [], "total": 0,
        })),
        "search" => {
            if query.is_empty() {
                return Err("missing search query".into());
            }
            Ok(serde_json::json!({
                "query": query, "limit": limit, "results": [], "total": 0,
            }))
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

pub fn commands() -> Vec<&'static str> {
    vec!["browse", "list", "search"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browse() {
        let r = execute("browse", &[]).unwrap();
        assert_eq!(r["limit"], 20);
    }

    #[test]
    fn test_list_limit() {
        let r = execute("list", &["".into(), "5".into()]).unwrap();
        assert_eq!(r["limit"], 5);
    }

    #[test]
    fn test_search() {
        let r = execute("search", &["monster".into()]).unwrap();
        assert_eq!(r["query"], "monster");
    }

    #[test]
    fn test_search_empty() {
        assert!(execute("search", &[]).is_err());
    }

    #[test]
    fn test_unknown() {
        assert!(execute("bad", &[]).is_err());
    }
}
