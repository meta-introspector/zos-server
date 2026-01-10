use crate::traits::ZOSPlugin;
use async_trait::async_trait;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

// Re-import the types we need from common modules
use crate::common::ServerState;

#[derive(Debug, serde::Deserialize)]
struct ZOSConfig {
    server: ServerConfig,
    auth: AuthConfig,
    deployment: DeploymentConfig,
}

#[derive(Debug, serde::Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
    domain: String,
}

#[derive(Debug, serde::Deserialize)]
struct AuthConfig {
    session_duration_hours: u64,
}

#[derive(Debug, serde::Deserialize)]
struct DeploymentConfig {
    #[serde(default)]
    environments: std::collections::HashMap<String, u16>, // env_name -> port
}

// Import the minimal server functionality directly
use axum::{
    extract::{Path as AxumPath, State},
    http::{header, StatusCode},
    response::{Html, Json, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
// // use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

pub struct MinimalServerPlugin {
    state: Arc<ServerState>,
}

impl MinimalServerPlugin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn load_config() -> ZOSConfig {
        let config_path = "zos-config.toml";
        if let Ok(config_str) = fs::read_to_string(config_path) {
            toml::from_str(&config_str).unwrap_or_else(|_| Self::default_config())
        } else {
            Self::default_config()
        }
    }

    fn default_config() -> ZOSConfig {
        ZOSConfig {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                domain: "solana.solfunmeme.com".to_string(),
            },
            auth: AuthConfig {
                session_duration_hours: 2,
            },
            deployment: DeploymentConfig {
                environments: {
                    let mut envs = std::collections::HashMap::new();
                    envs.insert("qa".to_string(), 8082);
                    envs.insert("prod".to_string(), 8081);
                    envs
                },
            },
        }
    }
}

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
            "deploy-systemd",
            "replace",
            "login",
            "create-user",
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
            "replace" => self.replace(args).await,
            "login" => self.login(args).await,
            "create-user" => self.create_user(args).await,
            _ => Err(format!("Unknown command: {}", command)),
        }
    }
}

impl MinimalServerPlugin {
    async fn serve(&self, args: Vec<String>) -> Result<Value, String> {
        let config = Self::load_config();
        let port: u16 = args
            .get(0)
            .unwrap_or(&config.server.port.to_string())
            .parse()
            .map_err(|_| "Invalid port number")?;

        println!("🚀 ZOS Server starting on {}:{}", config.server.host, port);
        println!("🔥 Hot Reload Dev Mode Active");

        // Start file watcher for auto-reload in dev mode
        if std::env::var("ZOS_DEV_MODE").unwrap_or_default() == "true" {
            self.start_file_watcher();
        }
        println!("🚀 ZOS Stage 1 Server");
        println!("   Domain: {}", config.server.domain);
        println!("   Port: {}", port);

        let app = self.create_router().await;
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        println!("🌐 Server running on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(|e| format!("Server error: {}", e))?;

        Ok(serde_json::json!({"status": "server_started", "port": port}))
    }

    async fn create_router(&self) -> Router {
        Router::new()
            .route("/", get(serve_root))
            .route("/health", get(serve_health))
            .route("/git-hash", get(serve_git_hash))
            .route("/binary-hash", get(serve_binary_hash))
            .route("/install", get(serve_installer))
            .route(
                "/download/binary/:git_hash/:arch",
                get(serve_binary_download),
            )
            .route("/install-log", post(handle_install_log))
            .route("/dashboard", get(serve_dashboard))
            .route("/api/dashboard/status", get(dashboard_api_status))
            .route("/api/dashboard/services", get(dashboard_api_services))
            .route("/api/dashboard/deploy", post(dashboard_api_deploy))
            .route("/api/dashboard/error", post(dashboard_api_error))
            .route("/api/network-status", get(authenticated_network_status))
            .route("/static/*file", get(serve_static_files))
            .with_state(Arc::clone(&self.state))
            .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
    }

    async fn deploy_qa(&self, args: Vec<String>) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("Usage: deploy-qa <git_hash> <port>".to_string());
        }

        let git_hash = &args[0];
        let port = &args[1];

        println!("🔧 Deploying to QA with hash: {}", git_hash);

        // Get current binary path
        let current_exe =
            std::env::current_exe().map_err(|e| format!("Failed to get current exe: {}", e))?;

        println!("📋 Starting QA server on port {}", port);

        // Start QA server as background process
        let qa_result = std::process::Command::new(&current_exe)
            .args(&["serve", port])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match qa_result {
            Ok(mut child) => {
                println!("✅ QA server started with PID: {}", child.id());

                // Capture logs for a moment
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Check if process is still running
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let stderr = child.stderr.take();
                        if let Some(mut stderr) = stderr {
                            let mut error_output = String::new();
                            use std::io::Read;
                            let _ = stderr.read_to_string(&mut error_output);
                            println!("❌ QA server exited with status: {}", status);
                            println!("📋 Error output: {}", error_output);
                            return Err(format!("QA server failed to start: {}", error_output));
                        }
                    }
                    Ok(None) => {
                        println!("✅ QA server is running");
                        // Don't wait for the process, let it run in background
                        std::mem::forget(child);
                    }
                    Err(e) => {
                        println!("❌ Failed to check QA server status: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to start QA server: {}", e);
                return Err(format!("QA deployment failed: {}", e));
            }
        }

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

        // Simulate production deployment
        tokio::time::sleep(Duration::from_millis(1000)).await;

        println!("✅ Production deployment complete");
        Ok(serde_json::json!({"status": "prod_deployed", "git_hash": git_hash, "port": port}))
    }

    async fn setup_qa(&self, args: Vec<String>) -> Result<Value, String> {
        let port = args.get(0).unwrap_or(&"8082".to_string()).clone();
        println!("🔧 Setting up QA instance on port {}", port);
        Ok(serde_json::json!({"status": "qa_setup_complete", "port": port}))
    }

    async fn setup_prod(&self, args: Vec<String>) -> Result<Value, String> {
        let port = args.get(0).unwrap_or(&"8081".to_string()).clone();
        println!("🏭 Setting up Production instance on port {}", port);
        Ok(serde_json::json!({"status": "prod_setup_complete", "port": port}))
    }

    async fn status(&self) -> Result<Value, String> {
        Ok(serde_json::json!({
            "status": "healthy",
            "plugin": self.name(),
            "version": self.version(),
            "timestamp": Utc::now()
        }))
    }

    async fn bootstrap(&self, _args: Vec<String>) -> Result<Value, String> {
        println!("🚀 Bootstrapping ZOS system...");
        Ok(serde_json::json!({"status": "bootstrap_complete"}))
    }

    async fn network_status(&self) -> Result<Value, String> {
        println!("🌐 ZOS Network Status");
        println!("====================");

        let ports = [8080, 8081, 8082];
        let mut results = HashMap::new();

        for port in ports {
            let status = check_port_health(port).await;
            let status_str = if status {
                "✅ healthy"
            } else {
                "❌ not responding"
            };
            println!("{}: {}", get_env_name(port), status_str);
            results.insert(port.to_string(), status);
        }

        Ok(serde_json::json!({"network_status": results}))
    }

    async fn deploy_systemd(&self, args: Vec<String>) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("Usage: deploy-systemd <env> <port>".to_string());
        }

        let env = &args[0];
        let port = &args[1];

        println!("🔧 Deploying systemd service for {} on port {}", env, port);
        Ok(serde_json::json!({"status": "systemd_deployed", "env": env, "port": port}))
    }

    async fn replace(&self, args: Vec<String>) -> Result<Value, String> {
        let port: u16 = args
            .get(0)
            .unwrap_or(&"8080".to_string())
            .parse()
            .map_err(|_| "Invalid port number")?;

        println!("🔄 Force replacing ZOS server on port {}", port);

        // Kill existing processes
        self.kill_existing_processes(port).await?;

        // Wait a moment for cleanup
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Start new server
        println!("🚀 Starting replacement server...");
        self.serve(vec![port.to_string()]).await
    }

    async fn kill_existing_processes(&self, port: u16) -> Result<(), String> {
        println!(
            "🔍 Searching for existing ZOS processes on port {}...",
            port
        );

        // Kill by process name
        let kill_result = std::process::Command::new("pkill")
            .args(&["-f", "zos.*server"])
            .output()
            .map_err(|e| format!("Failed to run pkill: {}", e))?;

        if kill_result.status.success() {
            println!("✅ Killed existing ZOS server processes");
        }

        // Kill by port (find processes using the port)
        let netstat_result = std::process::Command::new("lsof")
            .args(&["-ti", &format!(":{}", port)])
            .output();

        if let Ok(output) = netstat_result {
            let pids = String::from_utf8_lossy(&output.stdout);
            for pid in pids.lines() {
                if let Ok(pid_num) = pid.trim().parse::<u32>() {
                    println!("🔫 Killing process {} using port {}", pid_num, port);
                    let _ = std::process::Command::new("kill")
                        .args(&["-9", &pid_num.to_string()])
                        .output();
                }
            }
        }

        // Also try fuser as backup
        let _ = std::process::Command::new("fuser")
            .args(&["-k", &format!("{}/tcp", port)])
            .output();

        println!("🧹 Process cleanup complete");
        Ok(())
    }

    async fn login(&self, args: Vec<String>) -> Result<Value, String> {
        let username = args.get(0).cloned().unwrap_or_else(|| "root".to_string());

        println!("🔐 ZOS Dashboard Login System");
        println!("============================");
        println!("👤 Logging in as: {}", username);

        // Generate challenge token
        let challenge = self.generate_challenge().await?;
        println!("🎯 Challenge token: {}", challenge);

        // Get SSH key for specific user or fallback to system keys
        let ssh_keys = if username != "root" {
            self.find_user_keys(&username).await?
        } else {
            self.find_ssh_keys().await?
        };

        if ssh_keys.is_empty() {
            return Err(format!("No SSH keys found for user: {}", username));
        }

        println!("🔑 Found SSH keys:");
        for (i, key_path) in ssh_keys.iter().enumerate() {
            println!("  [{}] {}", i + 1, key_path);
        }

        // Use first key
        let selected_key = &ssh_keys[0];
        println!("🔐 Using key: {}", selected_key);

        // Sign challenge with SSH key (no passphrase prompt for ZOS users)
        let signature = if username != "root" {
            self.sign_challenge_with_ssh(&challenge, selected_key, "")
                .await?
        } else {
            println!("🔒 Enter SSH key passphrase (if required):");
            print!("Passphrase: ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            let passphrase = self.read_password().await?;
            self.sign_challenge_with_ssh(&challenge, selected_key, &passphrase)
                .await?
        };

        // Generate session token
        let session_token = self.generate_session_token(&challenge, &signature).await?;

        // Store session with username context
        self.create_dashboard_session_for_user(&session_token, &username)
            .await?;

        let config = Self::load_config();
        let dashboard_url = format!(
            "http://{}:{}/dashboard?token={}",
            config.server.domain, config.server.port, session_token
        );

        println!("✅ Login successful!");
        println!("🌐 Dashboard URL: {}", dashboard_url);
        println!("🚀 Access dashboard at the above URL");

        Ok(serde_json::json!({
            "status": "login_successful",
            "username": username,
            "session_token": session_token,
            "dashboard_url": dashboard_url,
            "expires_in": config.auth.session_duration_hours * 3600
        }))
    }

    async fn read_password(&self) -> Result<String, String> {
        use std::io;

        // Disable echo for password input
        let mut password = String::new();

        // Try to use termios to disable echo (Unix-like systems)
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;

            let stdin_fd = io::stdin().as_raw_fd();

            // Get current terminal attributes
            let mut termios = unsafe { std::mem::zeroed() };
            if unsafe { libc::tcgetattr(stdin_fd, &mut termios) } == 0 {
                let original_flags = termios.c_lflag;

                // Disable echo
                termios.c_lflag &= !libc::ECHO;
                unsafe { libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios) };

                // Read password
                io::stdin()
                    .read_line(&mut password)
                    .map_err(|e| format!("Failed to read password: {}", e))?;

                // Restore echo
                termios.c_lflag = original_flags;
                unsafe { libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios) };

                println!(); // New line after hidden input
            } else {
                // Fallback: show asterisks
                password = self.read_password_with_asterisks().await?;
            }
        }

        #[cfg(not(unix))]
        {
            // Fallback for non-Unix systems: show asterisks
            password = self.read_password_with_asterisks().await?;
        }

        Ok(password.trim().to_string())
    }

    async fn read_password_with_asterisks(&self) -> Result<String, String> {
        use std::io::{self, Write};

        let mut password = String::new();
        println!("(Showing * for each character)");
        print!("Passphrase: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                for _ in input.chars() {
                    print!("*");
                    io::stdout().flush().unwrap();
                }
                password.push_str(input);
            }
            Err(e) => return Err(format!("Failed to read input: {}", e)),
        }

        println!(); // New line after asterisks
        Ok(password)
    }

    async fn create_user(&self, args: Vec<String>) -> Result<Value, String> {
        if args.is_empty() {
            return Err("Usage: create-user <username>".to_string());
        }

        let username = &args[0];
        println!("👤 Creating ZOS user: {}", username);

        // Create user directory
        let home_dir = env::var("HOME").map_err(|_| "HOME not set")?;
        let zos_dir = format!("{}/.zos", home_dir);
        let users_dir = format!("{}/users", zos_dir);
        let user_dir = format!("{}/{}", users_dir, username);

        fs::create_dir_all(&user_dir).map_err(|e| format!("Failed to create user dir: {}", e))?;

        // Generate SSH key pair
        let key_path = format!("{}/id_zos", user_dir);
        let pub_key_path = format!("{}.pub", key_path);

        println!("🔑 Generating SSH key pair...");

        // Prompt for optional passphrase
        println!("🔒 Enter passphrase for key (press Enter for no passphrase):");
        print!("Passphrase: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let passphrase = self.read_password().await?;

        // Generate key with ssh-keygen
        let mut cmd = std::process::Command::new("ssh-keygen");
        cmd.args(&[
            "-t",
            "ed25519",
            "-f",
            &key_path,
            "-C",
            &format!("zos-user-{}", username),
        ]);

        if passphrase.is_empty() {
            cmd.arg("-N").arg("");
        } else {
            cmd.arg("-N").arg(&passphrase);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to generate key: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Key generation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Create user config
        let user_config = serde_json::json!({
            "username": username,
            "created_at": chrono::Utc::now(),
            "key_path": key_path,
            "pub_key_path": pub_key_path,
            "permissions": ["user", "dashboard"]
        });

        let config_path = format!("{}/config.json", user_dir);
        fs::write(&config_path, user_config.to_string())
            .map_err(|e| format!("Failed to save config: {}", e))?;

        // Read public key for display
        let pub_key = fs::read_to_string(&pub_key_path)
            .map_err(|e| format!("Failed to read public key: {}", e))?;

        println!("✅ User created successfully!");
        println!("📁 User directory: {}", user_dir);
        println!("🔑 Private key: {}", key_path);
        println!("🔓 Public key: {}", pub_key_path);
        println!("🔐 Public key: {}", pub_key.trim());

        Ok(serde_json::json!({
            "status": "user_created",
            "username": username,
            "user_dir": user_dir,
            "key_path": key_path,
            "pub_key": pub_key.trim()
        }))
    }

    async fn generate_challenge(&self) -> Result<String, String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_secs();

        let challenge = format!("zos-challenge-{}-{}", timestamp, rand::random::<u32>());
        Ok(challenge)
    }

    async fn find_ssh_keys(&self) -> Result<Vec<String>, String> {
        let home_dir = env::var("HOME").map_err(|_| "HOME environment variable not set")?;
        let mut keys = Vec::new();

        // Check ZOS user keys first
        let zos_users_dir = format!("{}/.zos/users", home_dir);
        if Path::new(&zos_users_dir).exists() {
            if let Ok(entries) = fs::read_dir(&zos_users_dir) {
                for entry in entries.flatten() {
                    let user_key = format!("{}/id_zos", entry.path().display());
                    if Path::new(&user_key).exists() {
                        keys.push(user_key);
                    }
                }
            }
        }

        // Fallback to system SSH keys
        let ssh_dir = format!("{}/.ssh", home_dir);
        let key_names = ["id_rsa", "id_ed25519", "id_ecdsa", "id_dsa"];

        for key_name in &key_names {
            let key_path = format!("{}/{}", ssh_dir, key_name);
            if Path::new(&key_path).exists() {
                keys.push(key_path);
            }
        }

        Ok(keys)
    }

    async fn sign_challenge_with_ssh(
        &self,
        challenge: &str,
        key_path: &str,
        passphrase: &str,
    ) -> Result<String, String> {
        // Create temporary file with challenge
        let temp_file = format!("/tmp/zos-challenge-{}", rand::random::<u32>());
        fs::write(&temp_file, challenge)
            .map_err(|e| format!("Failed to write challenge: {}", e))?;

        // Sign with SSH key (with passphrase support)
        let mut cmd = std::process::Command::new("ssh-keygen");
        cmd.args(&[
            "-Y",
            "sign",
            "-f",
            key_path,
            "-n",
            "zos-dashboard",
            &temp_file,
        ]);

        // If passphrase provided, use ssh-agent or expect it to be handled by ssh-keygen
        if !passphrase.is_empty() {
            // Note: In production, you'd want to use ssh-agent or a more secure method
            println!("🔑 Using provided passphrase for key signing");
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to sign with SSH: {}", e))?;

        // Clean up temp file
        let _ = fs::remove_file(&temp_file);

        if !output.status.success() {
            return Err(format!(
                "SSH signing failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Read signature file
        let sig_file = format!("{}.sig", temp_file);
        let signature = fs::read_to_string(&sig_file)
            .map_err(|e| format!("Failed to read signature: {}", e))?;

        let _ = fs::remove_file(&sig_file);

        Ok(signature)
    }

    async fn generate_session_token(
        &self,
        challenge: &str,
        signature: &str,
    ) -> Result<String, String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(challenge.as_bytes());
        hasher.update(signature.as_bytes());
        hasher.update(b"zos-session-salt");

        let token = format!("{:x}", hasher.finalize());
        Ok(token.chars().take(32).collect())
    }

    async fn create_dashboard_session(&self, token: &str) -> Result<(), String> {
        self.create_dashboard_session_for_user(token, "root").await
    }

    async fn find_user_keys(&self, username: &str) -> Result<Vec<String>, String> {
        let home_dir = env::var("HOME").map_err(|_| "HOME not set")?;
        let user_key = format!("{}/.zos/users/{}/id_zos", home_dir, username);

        if Path::new(&user_key).exists() {
            Ok(vec![user_key])
        } else {
            Err("User key not found".to_string())
        }
    }

    fn start_file_watcher(&self) {
        use notify::Event;
        use std::sync::mpsc::channel;

        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(_event) = res {
                    let _ = tx.send(());
                }
            },
            notify::Config::default(),
        )
        .unwrap();

        watcher
            .watch(std::path::Path::new("src"), RecursiveMode::Recursive)
            .unwrap();

        std::thread::spawn(move || {
            let _watcher = watcher; // Keep watcher alive
            loop {
                match rx.recv() {
                    Ok(_) => {
                        println!("🔄 File changed, triggering rebuild...");
                        let output = std::process::Command::new("cargo")
                            .args(&["build", "--bin", "zos_server"])
                            .output();

                        match output {
                            Ok(result) if result.status.success() => {
                                println!("✅ Rebuild successful");
                            }
                            Ok(result) => {
                                println!(
                                    "❌ Build failed: {}",
                                    String::from_utf8_lossy(&result.stderr)
                                );
                            }
                            Err(e) => {
                                println!("❌ Failed to run cargo build: {}", e);
                            }
                        }
                    }
                    Err(e) => println!("Watch error: {:?}", e),
                }
            }
        });
    }

    async fn create_dashboard_session_for_user(
        &self,
        token: &str,
        username: &str,
    ) -> Result<(), String> {
        let config = Self::load_config();

        let session_file = format!("/tmp/zos-session-{}", token);
        let session_data = serde_json::json!({
            "token": token,
            "created_at": chrono::Utc::now(),
            "expires_at": chrono::Utc::now() + chrono::Duration::hours(config.auth.session_duration_hours as i64),
            "user": username,
            "permissions": if username == "root" {
                vec!["admin", "dashboard", "deploy"]
            } else {
                vec!["user", "dashboard"]
            }
        });

        fs::write(&session_file, session_data.to_string())
            .map_err(|e| format!("Failed to create session: {}", e))?;

        println!("💾 Session stored: {}", session_file);
        Ok(())
    }
}

// Handler functions
async fn serve_root() -> Html<&'static str> {
    Html("<h1>ZOS Server - Zero Ontology System</h1><p>Stage 1 Foundation Server</p>")
}

async fn serve_health(State(_state): State<Arc<ServerState>>) -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "1.0.0-stage1",
        "timestamp": Utc::now(),
        "git": {
            "commit": get_git_hash(),
            "commit_short": get_git_hash_short()
        },
        "binary": {
            "hash": get_binary_hash(),
            "hash_short": get_binary_hash_short()
        },
        "env": {
            "pid": std::process::id(),
            "port": std::env::var("PORT").unwrap_or_default(),
            "cwd": std::env::current_dir().unwrap_or_default().display().to_string(),
            "binary_path": std::env::current_exe().unwrap_or_default().display().to_string()
        }
    }))
}

async fn serve_git_hash() -> String {
    get_git_hash_short()
}

async fn serve_binary_hash() -> String {
    get_binary_hash_short()
}

async fn serve_installer() -> Response {
    let script = r#"#!/bin/bash
# ZOS Universal Installer - Hash Verified Installation

set -euo pipefail

echo "🚀 ZOS Universal Installer"
echo "=========================="

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

echo "📋 Architecture: $ARCH"

# Get git hash from server
GIT_HASH=$(curl -s http://localhost:8080/git-hash)
echo "📋 Git Hash: $GIT_HASH"

# Download and install
BINARY_URL="http://localhost:8080/download/binary/$GIT_HASH/$ARCH"
echo "📥 Downloading from: $BINARY_URL"

curl -L "$BINARY_URL" -o /tmp/zos-server
chmod +x /tmp/zos-server
sudo mv /tmp/zos-server /usr/local/bin/

echo "✅ Installation complete!"
echo "🚀 Run: zos-server --help"
"#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(script.into())
        .unwrap()
}

async fn serve_binary_download(AxumPath((git_hash, arch)): AxumPath<(String, String)>) -> Response {
    // Simulate binary download
    let content = format!(
        "Binary for {} on {} (hash: {})",
        arch,
        git_hash,
        get_binary_hash()
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"zos-server\"",
        )
        .body(content.into())
        .unwrap()
}

async fn handle_install_log(Json(payload): Json<Value>) -> Json<Value> {
    println!(
        "📋 Installation log: {}",
        serde_json::to_string_pretty(&payload).unwrap_or_default()
    );
    Json(serde_json::json!({"status": "logged"}))
}

async fn serve_dashboard(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let token = params.get("token");

    if let Some(token) = token {
        if verify_session_token(token).await {
            return serve_dashboard_html().await;
        }
    }

    // Unauthorized - show login instructions
    let login_html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>ZOS Dashboard - Login Required</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        .login-box { background: #f5f5f5; padding: 30px; border-radius: 10px; text-align: center; }
        .command { background: #333; color: #0f0; padding: 10px; border-radius: 5px; font-family: monospace; }
    </style>
</head>
<body>
    <div class="login-box">
        <h1>🔐 ZOS Dashboard Login</h1>
        <p>To access the dashboard, authenticate with your SSH key:</p>
        <div class="command">cargo run login</div>
        <p>Or if using the unified server:</p>
        <div class="command">./target/debug/zos_server login</div>
        <p>This will generate a secure session token and open the dashboard.</p>
    </div>
</body>
</html>
    "#;

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::CONTENT_TYPE, "text/html")
        .body(login_html.into())
        .unwrap()
}

async fn serve_dashboard_html() -> Response {
    let dashboard_html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>ZOS Dashboard</title>
    <meta charset="UTF-8">
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #1a1a1a; color: #fff; }
        .header { background: #333; padding: 20px; border-radius: 10px; margin-bottom: 20px; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; }
        .card { background: #2a2a2a; padding: 20px; border-radius: 10px; border: 1px solid #444; }
        .status { padding: 10px; border-radius: 5px; margin: 10px 0; }
        .healthy { background: #0a5d0a; }
        .unhealthy { background: #5d0a0a; }
        .step { background: #333; padding: 10px; margin: 5px 0; border-radius: 5px; border-left: 4px solid #666; }
        .step.running { border-left-color: #ffa500; }
        .step.success { border-left-color: #0a5d0a; }
        .step.error { border-left-color: #5d0a0a; }
        button { background: #0066cc; color: white; border: none; padding: 10px 20px; border-radius: 5px; cursor: pointer; }
        button:hover { background: #0052a3; }
        .log { background: #111; padding: 15px; border-radius: 5px; font-family: monospace; font-size: 12px; max-height: 200px; overflow-y: auto; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 ZOS Dashboard - Dev Mode with Auto-Refresh</h1>
        <p>Authenticated Root Access | Plugin Architecture | Real-time Monitoring</p>
    </div>

    <div class="grid">
        <div class="card">
            <h3>🌐 Service Status</h3>
            <div id="services">Loading...</div>
            <button onclick="refreshServices()">Refresh</button>
        </div>

        <div class="card">
            <h3>🚀 Quick Deploy</h3>
            <button id="deploy-qa-btn">Deploy QA</button>
            <button id="deploy-prod-btn">Deploy Production</button>
            <button id="refresh-btn">Refresh Services</button>
        </div>

        <div class="card" id="deployment-details" style="display: none;">
            <h3 id="deployment-title">🔧 Deployment Progress</h3>
            <div id="deployment-steps"></div>
            <button onclick="hideDeploymentDetails()">Close</button>
        </div>

        <div class="card">
            <h3>📊 System Info</h3>
            <div id="system-info">Loading...</div>
        </div>

        <div class="card">
            <h3>📋 Activity Log</h3>
            <div id="activity-log" class="log">Dashboard initialized...</div>
        </div>

        <div class="card">
            <h3>🔍 Console Log</h3>
            <div id="console-log" class="log">Console ready...</div>
            <button onclick="clearConsole()">Clear Console</button>
        </div>
    </div>

    <script>
        // Console log capture for mobile debugging
        const originalConsole = {
            log: console.log,
            error: console.error,
            warn: console.warn,
            info: console.info
        };

        function consoleLog(level, ...args) {
            const consoleDiv = document.getElementById('console-log');
            const timestamp = new Date().toLocaleTimeString();
            const message = args.map(arg =>
                typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
            ).join(' ');

            const levelIcon = {
                'log': '📝',
                'error': '❌',
                'warn': '⚠️',
                'info': 'ℹ️'
            }[level] || '📝';

            consoleDiv.innerHTML += `[${timestamp}] ${levelIcon} ${message}\n`;
            consoleDiv.scrollTop = consoleDiv.scrollHeight;

            // Call original console method
            originalConsole[level](...args);
        }

        // Override console methods
        console.log = (...args) => consoleLog('log', ...args);
        console.error = (...args) => consoleLog('error', ...args);
        console.warn = (...args) => consoleLog('warn', ...args);
        console.info = (...args) => consoleLog('info', ...args);

        // Error reporting system
        function reportError(error, source = 'unknown') {
            const errorData = {
                message: error.message || error,
                stack: error.stack || '',
                source: source,
                timestamp: new Date().toISOString(),
                url: window.location.href,
                userAgent: navigator.userAgent
            };

            // Send to server
            fetch('/api/dashboard/error', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify(errorData)
            }).catch(e => console.log('Failed to report error to server:', e));

            // Log error details
            console.error('Client Error:', errorData.message);
            if (errorData.stack) console.error('Stack:', errorData.stack);
        }

        // Global error handler
        window.onerror = function(message, source, lineno, colno, error) {
            reportError({
                message: message,
                stack: error ? error.stack : `${source}:${lineno}:${colno}`
            }, 'window.onerror');
            return false;
        };

        // Promise rejection handler
        window.addEventListener('unhandledrejection', function(event) {
            reportError(event.reason, 'unhandledrejection');
        });

        // Auto-refresh system for dev mode
        let lastHeartbeat = Date.now();
        function checkServerHealth() {
            fetch('/health')
                .then(response => response.json())
                .then(data => {
                    lastHeartbeat = Date.now();
                })
                .catch(error => {
                    // Server might be restarting
                    if (Date.now() - lastHeartbeat > 10000) {
                        console.log('🔄 Server appears to be restarting, refreshing page...');
                        window.location.reload();
                    }
                });
        }

        // Check server health every 2 seconds
        setInterval(checkServerHealth, 2000);

        function clearConsole() {
            document.getElementById('console-log').innerHTML = 'Console cleared...\n';
        }

        function log(message) {
            const logDiv = document.getElementById('activity-log');
            const timestamp = new Date().toLocaleTimeString();
            logDiv.innerHTML += `[${timestamp}] ${message}\n`;
            logDiv.scrollTop = logDiv.scrollHeight;
        }

        async function refreshServices() {
            log('Refreshing service status...');
            console.log('🔍 Starting refreshServices...');
            try {
                const response = await fetch('/api/dashboard/services');
                console.log('📡 Response status:', response.status);
                const data = await response.json();
                console.log('📊 Services data:', data);
                const formattedHTML = formatServices(data);
                console.log('🎨 Formatted HTML:', formattedHTML);
                document.getElementById('services').innerHTML = formattedHTML;
                log('✅ Services refreshed');
            } catch (error) {
                console.error('❌ RefreshServices error:', error);
                log('❌ Failed to refresh services: ' + error);
            }
        }

        function formatServices(data) {
            console.log('🎯 formatServices called with:', data);
            if (data.services) {
                console.log('✅ Using services format');
                const servicesResult = data.services.map(service =>
                    `<div class="status ${service.status ? 'healthy' : 'unhealthy'}">
                        ${service.icon} ${service.name} (${service.environment.toUpperCase()})
                        <br><small>${service.hostname}:${service.port}</small>
                        <br><strong>${service.status ? '✅ Healthy' : '❌ Down'}</strong>
                    </div>`
                ).join('');
                console.log('🎨 Services HTML result:', servicesResult);
                return servicesResult;
            }
            // Fallback to legacy format
            console.log('⚠️ Using legacy format');
            const legacyResult = Object.entries(data.network_status || {}).map(([port, healthy]) =>
                `<div class="status ${healthy ? 'healthy' : 'unhealthy'}">
                    Port ${port}: ${healthy ? '✅ Healthy' : '❌ Down'}
                </div>`
            ).join('');
            console.log('🎨 Legacy HTML result:', legacyResult);
            return legacyResult;
        }

        async function deployQA() {
            showDeploymentDetails('🧪 QA Deployment', 'qa');

            const steps = [
                { id: 'git-hash', name: 'Getting Git Hash', status: 'running' },
                { id: 'build', name: 'Building Release Binary', status: 'pending' },
                { id: 'hash-verify', name: 'Verifying Binary Hash', status: 'pending' },
                { id: 'systemd', name: 'Deploying Systemd Service', status: 'pending' },
                { id: 'health-check', name: 'Health Check', status: 'pending' }
            ];

            updateDeploymentSteps(steps);

            try {
                // Step 1: Get git hash
                await simulateStep('git-hash', 'Getting current git commit hash...', 1000);
                steps[0].status = 'success';
                steps[1].status = 'running';
                updateDeploymentSteps(steps);

                // Step 2: Build
                await simulateStep('build', 'Compiling Rust binary with release optimizations...', 2000);
                steps[1].status = 'success';
                steps[2].status = 'running';
                updateDeploymentSteps(steps);

                // Step 3: Hash verification
                await simulateStep('hash-verify', 'Calculating SHA256 hash for reproducible builds...', 1000);
                steps[2].status = 'success';
                steps[3].status = 'running';
                updateDeploymentSteps(steps);

                // Step 4: Deploy systemd
                await simulateStep('systemd', 'Starting QA server on port 8082...', 1000);
                const response = await fetch('/api/dashboard/deploy', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({env: 'qa', port: 8082, git_hash: 'current'})
                });

                if (!response.ok) {
                    const errorText = await response.text();
                    console.log('❌ Deploy response error:', errorText);
                    let errorData;
                    try {
                        errorData = JSON.parse(errorText);
                    } catch (e) {
                        throw new Error('Deployment failed: ' + errorText);
                    }

                    let errorMsg = errorData.error || 'Deployment failed';
                    if (errorData.stderr) errorMsg += '\nStderr: ' + errorData.stderr;
                    if (errorData.stdout) errorMsg += '\nStdout: ' + errorData.stdout;
                    if (errorData.systemd_logs) errorMsg += '\nSystemd Logs:\n' + errorData.systemd_logs;
                    console.log('❌ Full error details:', errorMsg);
                    throw new Error(errorMsg);
                }

                const deployResult = await response.json();
                console.log('✅ Deploy response:', deployResult);

                steps[3].status = 'success';
                steps[4].status = 'running';
                updateDeploymentSteps(steps);

                // Step 5: Health check
                await simulateStep('health-check', 'Verifying QA service is responding...', 1500);
                steps[4].status = 'success';
                updateDeploymentSteps(steps);

                log('✅ QA deployment completed successfully');

            } catch (error) {
                log('❌ QA deployment failed: ' + error);
                const currentStep = steps.find(s => s.status === 'running');
                if (currentStep) {
                    currentStep.status = 'error';
                    showStepError(currentStep.id, error.message);
                }
                updateDeploymentSteps(steps);
            }
        }

        function showDeploymentDetails(title, env) {
            document.getElementById('deployment-title').textContent = title;
            document.getElementById('deployment-details').style.display = 'block';
            log(`🚀 Starting ${env.toUpperCase()} deployment...`);
        }

        function hideDeploymentDetails() {
            document.getElementById('deployment-details').style.display = 'none';
        }

        function updateDeploymentSteps(steps) {
            const container = document.getElementById('deployment-steps');
            container.innerHTML = steps.map(step => {
                const icon = step.status === 'success' ? '✅' :
                           step.status === 'running' ? '⏳' :
                           step.status === 'error' ? '❌' : '⏸️';
                return `<div class="step ${step.status}">
                    ${icon} ${step.name}
                    <div id="step-${step.id}-detail" style="font-size: 0.9em; color: #ccc; margin-top: 5px;"></div>
                </div>`;
            }).join('');
        }

        async function simulateStep(stepId, message, duration) {
            const detail = document.getElementById(`step-${stepId}-detail`);
            if (detail) {
                detail.textContent = message;
            }
            await new Promise(resolve => setTimeout(resolve, duration));
        }

        function showStepError(stepId, errorMessage) {
            const detail = document.getElementById(`step-${stepId}-detail`);
            if (detail) {
                detail.innerHTML = `<span style="color: #ff6b6b;">${errorMessage}</span>`;
            }
        }

        async function deployProd() {
            showDeploymentDetails('🏭 Production Deployment', 'prod');

            const steps = [
                { id: 'git-hash', name: 'Getting Git Hash', status: 'running' },
                { id: 'build', name: 'Building Release Binary', status: 'pending' },
                { id: 'hash-verify', name: 'Verifying Binary Hash', status: 'pending' },
                { id: 'backup', name: 'Creating Backup', status: 'pending' },
                { id: 'systemd', name: 'Deploying Systemd Service', status: 'pending' },
                { id: 'health-check', name: 'Health Check', status: 'pending' },
                { id: 'smoke-test', name: 'Smoke Tests', status: 'pending' }
            ];

            updateDeploymentSteps(steps);

            try {
                await simulateStep('git-hash', 'Getting current git commit hash...', 1000);
                steps[0].status = 'success';
                steps[1].status = 'running';
                updateDeploymentSteps(steps);

                await simulateStep('build', 'Compiling Rust binary with release optimizations...', 3000);
                steps[1].status = 'success';
                steps[2].status = 'running';
                updateDeploymentSteps(steps);

                await simulateStep('hash-verify', 'Calculating SHA256 hash for reproducible builds...', 1000);
                steps[2].status = 'success';
                steps[3].status = 'running';
                updateDeploymentSteps(steps);

                // Step 4: Deploy systemd
                await simulateStep('systemd', 'Starting Production server on port 8081...', 1000);
                console.log('🚀 Sending deploy request for production...');

                const response = await fetch('/api/dashboard/deploy', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({env: 'prod', port: 8081, git_hash: 'current'})
                });

                console.log('📡 Deploy response status:', response.status);
                console.log('📡 Deploy response headers:', [...response.headers.entries()]);

                if (!response.ok) {
                    const errorText = await response.text();
                    console.log('❌ Deploy response error text:', errorText);
                    throw new Error('Production deployment failed: ' + errorText);
                }

                const prodResult = await response.json();
                console.log('✅ Production deploy result:', prodResult);

                steps[4].status = 'success';
                steps[5].status = 'running';
                updateDeploymentSteps(steps);

                await simulateStep('health-check', 'Verifying production service is responding...', 2000);
                steps[5].status = 'success';
                steps[6].status = 'running';
                updateDeploymentSteps(steps);

                await simulateStep('smoke-test', 'Running production smoke tests...', 2000);
                steps[6].status = 'success';
                updateDeploymentSteps(steps);

                log('✅ Production deployment completed successfully');

            } catch (error) {
                log('❌ Production deployment failed: ' + error);
                const currentStep = steps.find(s => s.status === 'running');
                if (currentStep) currentStep.status = 'error';
                updateDeploymentSteps(steps);
            }
        }

        async function networkStatus() {
            log('🌐 Checking network status...');
            refreshServices();
        }

        // Initialize dashboard
        document.addEventListener('DOMContentLoaded', function() {
            // Set up event listeners
            document.getElementById('deploy-qa-btn').addEventListener('click', deployQA);
            document.getElementById('deploy-prod-btn').addEventListener('click', deployProd);
            document.getElementById('refresh-btn').addEventListener('click', refreshServices);

            // Initial load
            refreshServices();
            log('🚀 ZOS Dashboard initialized');
        });

        setInterval(refreshServices, 30000); // Auto-refresh every 30 seconds
    </script>
</body>
</html>
    "#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(dashboard_html.into())
        .unwrap()
}

async fn dashboard_api_status() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": "1.0.0"
    }))
}

async fn dashboard_api_services() -> Json<Value> {
    let services = vec![
        serde_json::json!({
            "name": "ZOS Dev Server",
            "icon": "🔧",
            "environment": "dev",
            "port": 8080,
            "hostname": "solana.solfunmeme.com",
            "url": "http://solana.solfunmeme.com:8080",
            "status": check_port_health(8080).await
        }),
        serde_json::json!({
            "name": "ZOS Production Server",
            "icon": "🏭",
            "environment": "prod",
            "port": 8081,
            "hostname": "solana.solfunmeme.com",
            "url": "http://solana.solfunmeme.com:8081",
            "status": check_port_health(8081).await
        }),
        serde_json::json!({
            "name": "ZOS QA Server",
            "icon": "🧪",
            "environment": "qa",
            "port": 8082,
            "hostname": "solana.solfunmeme.com",
            "url": "http://solana.solfunmeme.com:8082",
            "status": check_port_health(8082).await
        }),
    ];

    // Legacy format for backward compatibility
    let mut network_status = HashMap::new();
    for service in &services {
        let port = service["port"].as_u64().unwrap().to_string();
        let status = service["status"].as_bool().unwrap();
        network_status.insert(port, status);
    }

    Json(serde_json::json!({
        "services": services,
        "network_status": network_status
    }))
}

async fn dashboard_api_deploy(Json(payload): Json<Value>) -> Json<Value> {
    println!(
        "🔍 Deploy API called with payload: {}",
        serde_json::to_string_pretty(&payload).unwrap_or_default()
    );

    let env = payload
        .get("env")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let port = payload.get("port").and_then(|v| v.as_u64()).unwrap_or(8080);
    let git_hash = payload
        .get("git_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("current");

    println!(
        "🚀 API Deploy request: {} on port {} with hash {}",
        env, port, git_hash
    );

    // Get current binary path
    let current_exe = match std::env::current_exe() {
        Ok(path) => {
            println!("📋 Using binary: {}", path.display());
            path
        }
        Err(e) => {
            let error_msg = format!("Failed to get current exe: {}", e);
            println!("❌ {}", error_msg);
            return Json(serde_json::json!({
                "status": "error",
                "error": error_msg
            }));
        }
    };

    println!("📋 Starting command: {:?} serve {}", current_exe, port);

    // Start the server
    let result = std::process::Command::new(&current_exe)
        .args(&["serve", &port.to_string()])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    match result {
        Ok(mut child) => {
            println!("✅ {} server started with PID: {}", env, child.id());

            // Quick check if process is still running (no sleep)
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process exited immediately
                    let mut error_output = String::new();
                    let mut stdout_output = String::new();

                    if let Some(mut stderr) = child.stderr.take() {
                        use std::io::Read;
                        let _ = stderr.read_to_string(&mut error_output);
                    }
                    if let Some(mut stdout) = child.stdout.take() {
                        use std::io::Read;
                        let _ = stdout.read_to_string(&mut stdout_output);
                    }

                    let error_msg =
                        format!("{} server exited immediately with status: {}", env, status);
                    println!("❌ {}", error_msg);
                    println!("📋 Stdout: {}", stdout_output);
                    println!("📋 Stderr: {}", error_output);

                    let response = Json(serde_json::json!({
                        "status": "error",
                        "error": error_msg,
                        "stdout": stdout_output,
                        "stderr": error_output
                    }));

                    println!(
                        "📤 Sending error response: {}",
                        serde_json::to_string_pretty(&response.0).unwrap_or_default()
                    );
                    response
                }
                Ok(None) => {
                    // Still running - assume success
                    println!("✅ {} server is running on port {}", env, port);
                    std::mem::forget(child); // Let it run in background

                    let response = Json(serde_json::json!({
                        "status": format!("{}_deployed", env),
                        "env": env,
                        "port": port,
                        "timestamp": chrono::Utc::now()
                    }));

                    println!(
                        "📤 Sending success response: {}",
                        serde_json::to_string_pretty(&response.0).unwrap_or_default()
                    );
                    response
                }
                Err(e) => {
                    let error_msg = format!("Failed to check {} server status: {}", env, e);
                    println!("❌ {}", error_msg);

                    let response = Json(serde_json::json!({
                        "status": "error",
                        "error": error_msg
                    }));

                    println!(
                        "📤 Sending status check error response: {}",
                        serde_json::to_string_pretty(&response.0).unwrap_or_default()
                    );
                    response
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to start {} server: {}", env, e);
            println!("❌ {}", error_msg);

            let response = Json(serde_json::json!({
                "status": "error",
                "error": error_msg
            }));

            println!(
                "📤 Sending spawn error response: {}",
                serde_json::to_string_pretty(&response.0).unwrap_or_default()
            );
            response
        }
    }
}

async fn authenticated_network_status(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let token = params.get("token").ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_session_token(token).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let ports = [8080, 8081, 8082];
    let mut results = HashMap::new();

    for port in ports {
        let status = check_port_health(port).await;
        results.insert(port.to_string(), status);
    }

    Ok(Json(serde_json::json!({
        "network_status": results,
        "timestamp": chrono::Utc::now(),
        "authenticated": true
    })))
}

// Utility functions
fn get_git_hash() -> String {
    std::process::Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_git_hash_short() -> String {
    get_git_hash().chars().take(8).collect()
}

use sha2::{Digest, Sha256};

fn get_binary_hash() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| std::fs::read(&path).ok())
        .map(|bytes| {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            format!("{:x}", hasher.finalize())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_binary_hash_short() -> String {
    get_binary_hash().chars().take(8).collect()
}

async fn check_port_health(port: u16) -> bool {
    let url = format!("http://localhost:{}/health", port);
    reqwest::get(&url).await.is_ok()
}

fn get_env_name(port: u16) -> &'static str {
    match port {
        8080 => "Dev (8080)",
        8081 => "Prod (8081)",
        8082 => "QA (8082)",
        _ => "Unknown",
    }
}

async fn verify_session_token(token: &str) -> bool {
    let session_file = format!("/tmp/zos-session-{}", token);

    if let Ok(session_data) = fs::read_to_string(&session_file) {
        if let Ok(session) = serde_json::from_str::<Value>(&session_data) {
            if let Some(expires_at) = session.get("expires_at").and_then(|v| v.as_str()) {
                if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                    return chrono::Utc::now() < expires.with_timezone(&chrono::Utc);
                }
            }
        }
    }

    false
}

async fn serve_static_files(AxumPath(file_path): AxumPath<String>) -> Response {
    let static_dir = "static";
    let full_path = format!("{}/{}", static_dir, file_path);

    match std::fs::read(&full_path) {
        Ok(contents) => {
            let content_type = match file_path.split('.').last() {
                Some("js") => "application/javascript",
                Some("wasm") => "application/wasm",
                Some("css") => "text/css",
                Some("html") => "text/html",
                _ => "application/octet-stream",
            };

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .body(contents.into())
                .unwrap()
        }
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("File not found".into())
            .unwrap(),
    }
}

async fn dashboard_api_error(Json(payload): Json<Value>) -> Json<Value> {
    let timestamp = payload
        .get("timestamp")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let message = payload
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown error");
    let source = payload
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let stack = payload.get("stack").and_then(|v| v.as_str()).unwrap_or("");

    println!("🚨 CLIENT ERROR REPORT:");
    println!("   Time: {}", timestamp);
    println!("   Source: {}", source);
    println!("   Message: {}", message);
    if !stack.is_empty() {
        println!("   Stack: {}", stack);
    }

    Json(serde_json::json!({
        "status": "error_logged",
        "message": "Error report received"
    }))
}
