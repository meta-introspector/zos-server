//! paste-splitter — ZOS plugin: Split content into chunks
//!
//! Commands: split, chunk

pub fn execute(command: &str, args: &[String]) -> Result<serde_json::Value, String> {
    let content = args.first().map(|s| s.as_str()).unwrap_or("");
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(71);

    match command {
        "split" => {
            let lines: Vec<&str> = content.lines().collect();
            let chunks: Vec<Vec<&str>> = lines.chunks(n).map(|c| c.to_vec()).collect();
            Ok(serde_json::json!({
                "total_lines": lines.len(),
                "chunk_size": n,
                "chunks": chunks.len(),
                "preview": chunks.first().map(|c| c.join("\n")).unwrap_or_default(),
            }))
        }
        "chunk" => {
            let bytes = content.as_bytes();
            let chunks: Vec<String> = bytes
                .chunks(n)
                .map(|c| String::from_utf8_lossy(c).to_string())
                .collect();
            Ok(serde_json::json!({
                "total_bytes": bytes.len(),
                "chunk_size": n,
                "chunks": chunks.len(),
                "preview": chunks.first().cloned().unwrap_or_default(),
            }))
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

pub fn commands() -> Vec<&'static str> {
    vec!["split", "chunk"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_lines() {
        let r = execute("split", &["a\nb\nc\nd\ne".into(), "2".into()]).unwrap();
        assert_eq!(r["total_lines"], 5);
        assert_eq!(r["chunks"], 3);
    }

    #[test]
    fn test_chunk_bytes() {
        let r = execute("chunk", &["hello world".into(), "5".into()]).unwrap();
        assert_eq!(r["total_bytes"], 11);
        assert_eq!(r["chunks"], 3);
        assert_eq!(r["preview"], "hello");
    }

    #[test]
    fn test_default_71() {
        let r = execute("split", &["one line".into()]).unwrap();
        assert_eq!(r["chunk_size"], 71);
    }

    #[test]
    fn test_unknown() {
        assert!(execute("bad", &[]).is_err());
    }
}
