use crate::traits::ZOSPlugin;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

pub struct MinimalServerPlugin;

#[async_trait]
impl ZOSPlugin for MinimalServerPlugin {
    fn name(&self) -> &'static str {
        "minimal-server"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn commands(&self) -> Vec<&'static str> {
        vec![
            "serve",
            "deploy-qa",
            "deploy-prod",
            "setup-qa",
            "setup-prod",
            "status",
            "bootstrap",
            "network-status",
            "deploy-systemd"
        ]
    }

    async fn execute(&self, command: &str, args: Vec<String>) -> Result<Value, String> {
        match command {
            "serve" => self.serve(args).await,
            "deploy-qa" => self.deploy_qa(args).await,
            "deploy-prod" => self.deploy_prod(args).await,
            "setup-qa" => self.setup_qa(args).await,
            "setup-prod" => self.setup_prod(args).await,
            "status" => self.status().await,
            "bootstrap" => self.bootstrap(args).await,
            "network-status" => self.network_status().await,
            "deploy-systemd" => self.deploy_systemd(args).await,
            _ => Err(format!("Unknown command: {}", command))
        }
    }
}

impl MinimalServerPlugin {
    async fn serve(&self, args: Vec<String>) -> Result<Value, String> {
        let port = args.get(0).unwrap_or(&"8080".to_string()).clone();

        // Import the server logic from zos_minimal_server
        crate::zos_minimal_server::start_server(port).await
            .map_err(|e| format!("Server error: {}", e))?;

        Ok(serde_json::json!({"status": "server_started", "port": port}))
    }

    async fn deploy_qa(&self, args: Vec<String>) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("Usage: deploy-qa <git_hash> <port>".to_string());
        }

        let git_hash = &args[0];
        let port = &args[1];

        println!("🔧 Deploying to QA with hash: {}", git_hash);

        // Call the deployment logic
        crate::zos_minimal_server::deploy_qa_service(git_hash, port).await
            .map_err(|e| format!("QA deployment failed: {}", e))?;

        println!("✅ QA deployment complete");
        Ok(serde_json::json!({"status": "qa_deployed", "git_hash": git_hash, "port": port}))
    }

    async fn deploy_prod(&self, args: Vec<String>) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("Usage: deploy-prod <git_hash> <port>".to_string());
        }

        let git_hash = &args[0];
        let port = &args[1];

        println!("🏭 Deploying to Production with hash: {}", git_hash);

        crate::zos_minimal_server::deploy_prod_service(git_hash, port).await
            .map_err(|e| format!("Production deployment failed: {}", e))?;

        println!("✅ Production deployment complete");
        Ok(serde_json::json!({"status": "prod_deployed", "git_hash": git_hash, "port": port}))
    }

    async fn setup_qa(&self, args: Vec<String>) -> Result<Value, String> {
        let port = args.get(0).unwrap_or(&"8082".to_string()).clone();

        crate::zos_minimal_server::setup_qa_instance(&port).await
            .map_err(|e| format!("QA setup failed: {}", e))?;

        Ok(serde_json::json!({"status": "qa_setup_complete", "port": port}))
    }

    async fn setup_prod(&self, args: Vec<String>) -> Result<Value, String> {
        let port = args.get(0).unwrap_or(&"8081".to_string()).clone();

        crate::zos_minimal_server::setup_prod_instance(&port).await
            .map_err(|e| format!("Production setup failed: {}", e))?;

        Ok(serde_json::json!({"status": "prod_setup_complete", "port": port}))
    }

    async fn status(&self) -> Result<Value, String> {
        let status = crate::zos_minimal_server::get_server_status().await
            .map_err(|e| format!("Status check failed: {}", e))?;

        Ok(status)
    }

    async fn bootstrap(&self, args: Vec<String>) -> Result<Value, String> {
        crate::zos_minimal_server::bootstrap_system(args).await
            .map_err(|e| format!("Bootstrap failed: {}", e))?;

        Ok(serde_json::json!({"status": "bootstrap_complete"}))
    }

    async fn network_status(&self) -> Result<Value, String> {
        let status = crate::zos_minimal_server::check_network_status().await
            .map_err(|e| format!("Network status check failed: {}", e))?;

        Ok(status)
    }

    async fn deploy_systemd(&self, args: Vec<String>) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("Usage: deploy-systemd <env> <port>".to_string());
        }

        let env = &args[0];
        let port = &args[1];

        crate::zos_minimal_server::deploy_systemd_service(env, port).await
            .map_err(|e| format!("Systemd deployment failed: {}", e))?;

        Ok(serde_json::json!({"status": "systemd_deployed", "env": env, "port": port}))
    }
}
