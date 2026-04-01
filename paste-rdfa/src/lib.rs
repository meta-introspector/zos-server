//! paste-rdfa — ZOS plugin: Generate RDFa and eRDFa shard URLs
//!
//! Commands: rdfa-url, erdfa-shard

pub fn execute(command: &str, args: &[String]) -> Result<serde_json::Value, String> {
    let paste_url = args.first().map(|s| s.as_str()).unwrap_or("");
    let title = args.get(1).map(|s| s.as_str()).unwrap_or("untitled");
    let cid = args.get(2).map(|s| s.as_str()).unwrap_or("");

    match command {
        "rdfa-url" => {
            let mut url = format!(
                "{}#typeof=schema:CreativeWork&property=schema:name={}",
                paste_url,
                urlenc(title)
            );
            if !cid.is_empty() {
                url.push_str(&format!("&property=schema:identifier={}", urlenc(cid)));
            }
            Ok(serde_json::json!({"url": url, "format": "rdfa"}))
        }
        "erdfa-shard" => {
            let url = format!("{}&erdfa=1&cid={}", paste_url, urlenc(cid));
            Ok(serde_json::json!({"url": url, "format": "erdfa"}))
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

pub fn commands() -> Vec<&'static str> {
    vec!["rdfa-url", "erdfa-shard"]
}

fn urlenc(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdfa_url() {
        let r = execute(
            "rdfa-url",
            &[
                "https://example.com/paste/123".into(),
                "My Paste".into(),
                "bafk1234".into(),
            ],
        )
        .unwrap();
        let url = r["url"].as_str().unwrap();
        assert!(url.contains("typeof=schema:CreativeWork"));
        assert!(url.contains("My%20Paste"));
        assert!(url.contains("bafk1234"));
    }

    #[test]
    fn test_erdfa_shard() {
        let r = execute(
            "erdfa-shard",
            &[
                "https://example.com/data".into(),
                "".into(),
                "bafkABC".into(),
            ],
        )
        .unwrap();
        assert!(r["url"].as_str().unwrap().contains("erdfa=1"));
    }

    #[test]
    fn test_rdfa_no_cid() {
        let r = execute("rdfa-url", &["https://x.com".into(), "title".into()]).unwrap();
        assert!(!r["url"].as_str().unwrap().contains("identifier"));
    }

    #[test]
    fn test_unknown() {
        assert!(execute("bad", &[]).is_err());
    }
}
