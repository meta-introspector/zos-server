// Service management module
use serde_json::Value;

pub struct ServiceManager;

impl ServiceManager {
    pub async fn get_service_config() -> Result<Value, String> {
        let config_file = "/tmp/zos-services.json";

        if let Ok(content) = std::fs::read_to_string(config_file) {
            serde_json::from_str(&content).map_err(|e| format!("Invalid config: {}", e))
        } else {
            let default_config = serde_json::json!({
                "services": [
                    {"name": "zos-dev", "type": "process", "description": "Development Server", "port": 8080},
                    {"name": "zos-qa", "type": "systemd", "description": "QA Server", "port": 8082},
                    {"name": "zos-prod", "type": "systemd", "description": "Production Server", "port": 8081}
                ]
            });

            std::fs::write(
                config_file,
                serde_json::to_string_pretty(&default_config).unwrap(),
            )
            .ok();
            Ok(default_config)
        }
    }

    pub async fn get_service_logs(service_name: &str) -> Result<String, String> {
        let output = std::process::Command::new("journalctl")
            .args(&["-u", service_name, "--no-pager", "-n", "50", "--reverse"])
            .output()
            .map_err(|e| format!("Failed to get logs: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Ok(format!("No logs found for service: {}", service_name))
        }
    }
}
