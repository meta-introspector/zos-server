//! paste-reply — ZOS plugin: Thread management for paste replies
//!
//! Commands: reply, thread

pub fn execute(command: &str, args: &[String]) -> Result<serde_json::Value, String> {
    let paste_id = args.first().map(|s| s.as_str()).unwrap_or("");

    match command {
        "reply" => {
            let content = args.get(1).map(|s| s.as_str()).unwrap_or("");
            if paste_id.is_empty() {
                return Err("missing paste_id".into());
            }
            if content.is_empty() {
                return Err("missing content".into());
            }
            Ok(serde_json::json!({
                "reply_to": paste_id,
                "content_len": content.len(),
                "status": "ok",
            }))
        }
        "thread" => {
            if paste_id.is_empty() {
                return Err("missing paste_id".into());
            }
            Ok(serde_json::json!({
                "root": paste_id,
                "replies": [],
                "depth": 0,
            }))
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

pub fn commands() -> Vec<&'static str> {
    vec!["reply", "thread"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reply() {
        let r = execute("reply", &["paste123".into(), "my reply".into()]).unwrap();
        assert_eq!(r["reply_to"], "paste123");
    }

    #[test]
    fn test_reply_missing_content() {
        assert!(execute("reply", &["paste123".into()]).is_err());
    }

    #[test]
    fn test_thread() {
        let r = execute("thread", &["paste123".into()]).unwrap();
        assert_eq!(r["root"], "paste123");
    }

    #[test]
    fn test_thread_missing_id() {
        assert!(execute("thread", &[]).is_err());
    }

    #[test]
    fn test_unknown() {
        assert!(execute("bad", &[]).is_err());
    }
}
