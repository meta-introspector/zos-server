//! paste-preview — ZOS plugin: Render paste content as HTML preview
//!
//! Commands: preview, render

pub fn execute(command: &str, args: &[String]) -> Result<serde_json::Value, String> {
    let content = args.first().map(|s| s.as_str()).unwrap_or("");

    match command {
        "preview" => {
            let is_html = content.contains("<html") || content.contains("<!DOCTYPE");
            let html = if is_html {
                content.to_string()
            } else {
                format!("<html><head><style>body{{font-family:sans-serif;padding:20px;line-height:1.6}}</style></head><body><pre style=\"white-space:pre-wrap;word-wrap:break-word\">{}</pre></body></html>",
                    html_escape(content))
            };
            Ok(serde_json::json!({"html": html, "is_raw_html": is_html, "size": html.len()}))
        }
        "render" => {
            let escaped = html_escape(content);
            Ok(serde_json::json!({"text": escaped, "size": escaped.len()}))
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

pub fn commands() -> Vec<&'static str> {
    vec!["preview", "render"]
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_text() {
        let r = execute("preview", &["hello world".into()]).unwrap();
        assert!(r["html"].as_str().unwrap().contains("<pre"));
        assert!(!r["is_raw_html"].as_bool().unwrap());
    }

    #[test]
    fn test_preview_html() {
        let r = execute("preview", &["<html><body>hi</body></html>".into()]).unwrap();
        assert!(r["is_raw_html"].as_bool().unwrap());
    }

    #[test]
    fn test_render_escapes() {
        let r = execute("render", &["<script>alert(1)</script>".into()]).unwrap();
        assert!(!r["text"].as_str().unwrap().contains("<script>"));
        assert!(r["text"].as_str().unwrap().contains("&lt;script&gt;"));
    }

    #[test]
    fn test_unknown() {
        assert!(execute("bad", &[]).is_err());
    }
}
