// Session management module
use chrono::{DateTime, Utc};
use serde_json::Value;

pub struct SessionManager;

impl SessionManager {
    pub async fn validate_session(token: &str, _client_ip: &str) -> Result<bool, String> {
        let session_file = format!("/tmp/zos-session-{}", token);

        if let Ok(content) = std::fs::read_to_string(&session_file) {
            if let Ok(session_data) = serde_json::from_str::<Value>(&content) {
                // Check if session is expired
                if let Some(expires_at) = session_data.get("expires_at") {
                    if let Ok(expires) = expires_at.as_str().unwrap_or("").parse::<DateTime<Utc>>()
                    {
                        if Utc::now() > expires {
                            return Ok(false);
                        }
                    }
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn create_session(token: &str, username: &str) -> Result<(), String> {
        let session_file = format!("/tmp/zos-session-{}", token);
        let session_data = serde_json::json!({
            "token": token,
            "created_at": Utc::now(),
            "expires_at": Utc::now() + chrono::Duration::hours(2),
            "user": username,
            "permissions": if username == "root" {
                vec!["admin", "dashboard", "deploy"]
            } else {
                vec!["user", "dashboard"]
            }
        });

        std::fs::write(
            &session_file,
            serde_json::to_string_pretty(&session_data).unwrap(),
        )
        .map_err(|e| format!("Failed to create session: {}", e))?;

        Ok(())
    }
}
