use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use libp2p::{Swarm, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Clone)]
pub struct AppState {
    pub libp2p_swarm: Arc<RwLock<Swarm<ZosBehaviour>>>,
    pub user_sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    pub service_registry: Arc<RwLock<HashMap<String, ServiceEndpoint>>>,
    pub config: ZosConfig,
    pub ddns_client: Arc<RwLock<NamecheapDDNS>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZosConfig {
    pub http_port: u16,
    pub https_port: u16,
    pub domain: String,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub solana_rpc: String,
    pub max_concurrent_users: u32,
    pub block_duration_ms: u64,
    pub ddns: Option<DDNSConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDNSConfig {
    pub enabled: bool,
    pub domain: String,
    pub host: String,
    pub password: String,
    pub update_interval_minutes: u64,
}

#[derive(Debug, Clone)]
pub struct NamecheapDDNS {
    pub config: DDNSConfig,
    pub last_ip: Option<String>,
    pub client: reqwest::Client,
}

impl NamecheapDDNS {
    pub fn new(config: DDNSConfig) -> Self {
        Self {
            config,
            last_ip: None,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_current_ip(&self) -> Result<String, Box<dyn std::error::Error>> {
        let services = [
            "https://api.ipify.org",
            "https://icanhazip.com",
            "https://ipecho.net/plain",
            "https://checkip.amazonaws.com",
        ];

        for service in &services {
            match self.client.get(*service).send().await {
                Ok(response) if response.status().is_success() => {
                    if let Ok(ip) = response.text().await {
                        let ip = ip.trim();
                        if !ip.is_empty() {
                            return Ok(ip.to_string());
                        }
                    }
                }
                _ => continue,
            }
        }

        Err("All IP services failed".into())
    }

    pub async fn update_dns(&self, ip: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let url = "https://dynamicdns.park-your-domain.com/update";

        let params = [
            ("host", self.config.host.as_str()),
            ("domain", self.config.domain.as_str()),
            ("password", self.config.password.as_str()),
            ("ip", ip),
        ];

        let response = self.client
            .get(url)
            .query(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let text = response.text().await?;

            if text.contains("<ErrCount>0</ErrCount>") {
                println!("✅ DNS updated: {}.{} → {}", self.config.host, self.config.domain, ip);
                Ok(true)
            } else {
                println!("❌ DNS update failed: {}", text);
                Ok(false)
            }
        } else {
            println!("❌ HTTP error {}: {}", response.status(), response.text().await?);
            Ok(false)
        }
    }

    pub async fn check_and_update(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let current_ip = self.get_current_ip().await?;

        if Some(&current_ip) != self.last_ip.as_ref() {
            println!("🔄 IP changed: {:?} → {}", self.last_ip, current_ip);

            if self.update_dns(&current_ip).await? {
                self.last_ip = Some(current_ip);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            println!("✓ IP unchanged: {}", current_ip);
            Ok(true)
        }
    }
}

impl ZosConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load from config file, fallback to environment variables
        if let Ok(config_str) = std::fs::read_to_string("/opt/zos/zos-config.toml") {
            let config: ZosConfig = toml::from_str(&config_str)?;
            Ok(config)
        } else {
            // Fallback to environment variables
            let ddns_config = if let (Ok(domain), Ok(host), Ok(password)) = (
                std::env::var("NAMECHEAP_DOMAIN"),
                std::env::var("NAMECHEAP_HOST"),
                std::env::var("NAMECHEAP_PASSWORD"),
            ) {
                Some(DDNSConfig {
                    enabled: true,
                    domain,
                    host,
                    password,
                    update_interval_minutes: 5,
                })
            } else {
                None
            };

            Ok(ZosConfig {
                http_port: std::env::var("ZOS_HTTP_PORT")
                    .unwrap_or("8080".to_string())
                    .parse()
                    .unwrap_or(8080),
                https_port: std::env::var("ZOS_HTTPS_PORT")
                    .unwrap_or("8443".to_string())
                    .parse()
                    .unwrap_or(8443),
                domain: std::env::var("ZOS_DOMAIN")
                    .unwrap_or("localhost".to_string()),
                cert_path: std::env::var("ZOS_CERT_PATH").ok(),
                key_path: std::env::var("ZOS_KEY_PATH").ok(),
                solana_rpc: "https://api.mainnet-beta.solana.com".to_string(),
                max_concurrent_users: 50,
                block_duration_ms: 400,
                ddns: ddns_config,
            })
        }
    }
}

pub async fn create_zos_server() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = ZosConfig::load()?;

    println!("🚀 ZOS Server starting...");
    println!("   Domain: {}", config.domain);
    println!("   HTTP Port: {}", config.http_port);
    println!("   HTTPS Port: {}", config.https_port);

    // Initialize DDNS client
    let ddns_client = if let Some(ddns_config) = &config.ddns {
        if ddns_config.enabled {
            println!("🌐 DDNS enabled for {}.{}", ddns_config.host, ddns_config.domain);
            Arc::new(RwLock::new(NamecheapDDNS::new(ddns_config.clone())))
        } else {
            Arc::new(RwLock::new(NamecheapDDNS::new(DDNSConfig {
                enabled: false,
                domain: "localhost".to_string(),
                host: "@".to_string(),
                password: "".to_string(),
                update_interval_minutes: 5,
            })))
        }
    } else {
        println!("🌐 DDNS disabled");
        Arc::new(RwLock::new(NamecheapDDNS::new(DDNSConfig {
            enabled: false,
            domain: "localhost".to_string(),
            host: "@".to_string(),
            password: "".to_string(),
            update_interval_minutes: 5,
        })))
    };

    // Initialize LibP2P (placeholder)
    let swarm = create_libp2p_swarm().await?;

    // Create shared state
    let state = AppState {
        libp2p_swarm: Arc::new(RwLock::new(swarm)),
        user_sessions: Arc::new(RwLock::new(HashMap::new())),
        service_registry: Arc::new(RwLock::new(HashMap::new())),
        config: config.clone(),
        ddns_client,
    };

    // Create HTTP router
    let app = Router::new()
        // Public API endpoints
        .route("/", get(serve_homepage))
        .route("/health", get(health_check))

        // Service endpoints
        .route("/:wallet/:service", get(handle_service_get).post(handle_service_post))
        .route("/:wallet/:service/swap", post(handle_swap))
        .route("/:wallet/:service/quote", get(handle_quote))

        // Dashboard endpoints
        .route("/dashboard/:wallet", get(serve_dashboard))
        .route("/api/status/:wallet", get(get_user_status))
        .route("/api/allocate-port", post(allocate_port))

        // DDNS management endpoints
        .route("/api/ddns/status", get(get_ddns_status))
        .route("/api/ddns/update", post(force_ddns_update))

        // Static files
        .route("/static/*file", get(serve_static))

        .with_state(state.clone());

    // Setup HTTPS if certificates are available
    if let (Some(cert_path), Some(key_path)) = (&config.cert_path, &config.key_path) {
        if std::path::Path::new(cert_path).exists() && std::path::Path::new(key_path).exists() {
            println!("🔐 Starting HTTPS server with SSL certificates");

            let rustls_config = RustlsConfig::from_pem_file(cert_path, key_path).await?;
            let https_addr = format!("0.0.0.0:{}", config.https_port);

            // Run HTTPS server and other tasks concurrently
            tokio::select! {
                result = axum_server::bind_rustls(https_addr.parse()?, rustls_config)
                    .serve(app.into_make_service()) => {
                    println!("HTTPS server error: {:?}", result);
                },
                _ = run_libp2p_loop(state.libp2p_swarm.clone()) => {
                    println!("LibP2P loop ended");
                },
                _ = run_background_tasks(state.clone()) => {
                    println!("Background tasks ended");
                },
                _ = run_ddns_loop(state.ddns_client.clone(), &config) => {
                    println!("DDNS loop ended");
                }
            }
        } else {
            println!("⚠️  SSL certificates not found, falling back to HTTP");
            run_http_server(app, &config, state).await?;
        }
    } else {
        println!("📡 Starting HTTP server (no SSL configured)");
        run_http_server(app, &config, state).await?;
    }

    Ok(())
}

async fn run_ddns_loop(ddns_client: Arc<RwLock<NamecheapDDNS>>, config: &ZosConfig) {
    if let Some(ddns_config) = &config.ddns {
        if ddns_config.enabled {
            let mut interval = interval(Duration::from_secs(ddns_config.update_interval_minutes * 60));

            // Initial update
            {
                let mut client = ddns_client.write().await;
                if let Err(e) = client.check_and_update().await {
                    println!("❌ Initial DDNS update failed: {}", e);
                }
            }

            loop {
                interval.tick().await;

                let mut client = ddns_client.write().await;
                if let Err(e) = client.check_and_update().await {
                    println!("❌ DDNS update failed: {}", e);
                }
            }
        }
    }
}

async fn get_ddns_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let ddns = state.ddns_client.read().await;

    Json(serde_json::json!({
        "enabled": ddns.config.enabled,
        "domain": format!("{}.{}", ddns.config.host, ddns.config.domain),
        "last_ip": ddns.last_ip,
        "update_interval_minutes": ddns.config.update_interval_minutes
    }))
}

async fn force_ddns_update(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut ddns = state.ddns_client.write().await;

    if !ddns.config.enabled {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": "DDNS is disabled"
        })));
    }

    match ddns.check_and_update().await {
        Ok(updated) => Ok(Json(serde_json::json!({
            "success": true,
            "updated": updated,
            "current_ip": ddns.last_ip
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })))
    }
}

#[derive(Clone)]
pub struct AppState {
    pub libp2p_swarm: Arc<RwLock<Swarm<ZosBehaviour>>>,
    pub user_sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    pub service_registry: Arc<RwLock<HashMap<String, ServiceEndpoint>>>,
    pub config: ZosConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZosConfig {
    pub http_port: u16,
    pub https_port: u16,
    pub domain: String,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub solana_rpc: String,
    pub max_concurrent_users: u32,
    pub block_duration_ms: u64,
}

impl ZosConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load from config file, fallback to environment variables
        if let Ok(config_str) = std::fs::read_to_string("/opt/zos/zos-config.toml") {
            let config: ZosConfig = toml::from_str(&config_str)?;
            Ok(config)
        } else {
            // Fallback to environment variables
            Ok(ZosConfig {
                http_port: std::env::var("ZOS_HTTP_PORT")
                    .unwrap_or("8080".to_string())
                    .parse()
                    .unwrap_or(8080),
                https_port: std::env::var("ZOS_HTTPS_PORT")
                    .unwrap_or("8443".to_string())
                    .parse()
                    .unwrap_or(8443),
                domain: std::env::var("ZOS_DOMAIN")
                    .unwrap_or("localhost".to_string()),
                cert_path: std::env::var("ZOS_CERT_PATH").ok(),
                key_path: std::env::var("ZOS_KEY_PATH").ok(),
                solana_rpc: "https://api.mainnet-beta.solana.com".to_string(),
                max_concurrent_users: 50,
                block_duration_ms: 400,
            })
        }
    }
}

pub async fn create_zos_server() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = ZosConfig::load()?;

    println!("🚀 ZOS Server starting...");
    println!("   Domain: {}", config.domain);
    println!("   HTTP Port: {}", config.http_port);
    println!("   HTTPS Port: {}", config.https_port);

    // Initialize LibP2P (placeholder)
    let swarm = create_libp2p_swarm().await?;

    // Create shared state
    let state = AppState {
        libp2p_swarm: Arc::new(RwLock::new(swarm)),
        user_sessions: Arc::new(RwLock::new(HashMap::new())),
        service_registry: Arc::new(RwLock::new(HashMap::new())),
        config: config.clone(),
    };

    // Create HTTP router
    let app = Router::new()
        // Public API endpoints
        .route("/", get(serve_homepage))
        .route("/health", get(health_check))

        // Service endpoints
        .route("/:wallet/:service", get(handle_service_get).post(handle_service_post))
        .route("/:wallet/:service/swap", post(handle_swap))
        .route("/:wallet/:service/quote", get(handle_quote))

        // Dashboard endpoints
        .route("/dashboard/:wallet", get(serve_dashboard))
        .route("/api/status/:wallet", get(get_user_status))
        .route("/api/allocate-port", post(allocate_port))

        // Static files
        .route("/static/*file", get(serve_static))

        .with_state(state.clone());

    // Setup HTTPS if certificates are available
    if let (Some(cert_path), Some(key_path)) = (&config.cert_path, &config.key_path) {
        if std::path::Path::new(cert_path).exists() && std::path::Path::new(key_path).exists() {
            println!("🔐 Starting HTTPS server with SSL certificates");

            let rustls_config = RustlsConfig::from_pem_file(cert_path, key_path).await?;
            let https_addr = format!("0.0.0.0:{}", config.https_port);

            // Run HTTPS server and other tasks concurrently
            tokio::select! {
                result = axum_server::bind_rustls(https_addr.parse()?, rustls_config)
                    .serve(app.into_make_service()) => {
                    println!("HTTPS server error: {:?}", result);
                },
                _ = run_libp2p_loop(state.libp2p_swarm.clone()) => {
                    println!("LibP2P loop ended");
                },
                _ = run_background_tasks(state.clone()) => {
                    println!("Background tasks ended");
                }
            }
        } else {
            println!("⚠️  SSL certificates not found, falling back to HTTP");
            run_http_server(app, &config, state).await?;
        }
    } else {
        println!("📡 Starting HTTP server (no SSL configured)");
        run_http_server(app, &config, state).await?;
    }

    Ok(())
}

async fn run_http_server(
    app: Router,
    config: &ZosConfig,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    let http_addr = format!("0.0.0.0:{}", config.http_port);

    tokio::select! {
        result = axum::Server::bind(&http_addr.parse()?)
            .serve(app.into_make_service()) => {
            println!("HTTP server error: {:?}", result);
        },
        _ = run_libp2p_loop(state.libp2p_swarm.clone()) => {
            println!("LibP2P loop ended");
        },
        _ = run_background_tasks(state.clone()) => {
            println!("Background tasks ended");
        }
    }

    Ok(())
}

async fn serve_homepage(State(state): State<AppState>) -> Html<String> {
    let domain = &state.config.domain;
    let https_port = state.config.https_port;

    let homepage = format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>ZOS - Zero Ontology System</title>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <style>
            body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 0; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; }}
            .container {{ max-width: 1200px; margin: 0 auto; padding: 20px; }}
            .hero {{ text-align: center; color: white; padding: 60px 20px; }}
            .hero h1 {{ font-size: 3.5em; margin: 0; text-shadow: 2px 2px 4px rgba(0,0,0,0.3); }}
            .hero p {{ font-size: 1.3em; margin: 20px 0; opacity: 0.9; }}
            .features {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin: 40px 0; }}
            .feature {{ background: rgba(255,255,255,0.95); padding: 30px; border-radius: 15px; box-shadow: 0 8px 32px rgba(0,0,0,0.1); }}
            .feature h3 {{ color: #333; margin-top: 0; }}
            .endpoint {{ background: #f8f9fa; padding: 15px; font-family: 'Courier New', monospace; margin: 10px 0; border-radius: 8px; border-left: 4px solid #667eea; }}
            .cta {{ text-align: center; margin: 40px 0; }}
            .btn {{ display: inline-block; background: #ff6b6b; color: white; padding: 15px 30px; text-decoration: none; border-radius: 25px; font-weight: bold; transition: all 0.3s; }}
            .btn:hover {{ background: #ff5252; transform: translateY(-2px); }}
            .status {{ background: rgba(255,255,255,0.1); color: white; padding: 20px; border-radius: 10px; margin: 20px 0; }}
        </style>
    </head>
    <body>
        <div class="container">
            <div class="hero">
                <h1>🚀 ZOS Server</h1>
                <p>Decentralized Compute Platform with Token Economics</p>
                <div class="status">
                    <strong>🌐 Live at:</strong> https://{}<br>
                    <strong>🔐 SSL:</strong> Let's Encrypt Secured<br>
                    <strong>⚡ Status:</strong> <span id="status">Checking...</span>
                </div>
            </div>

            <div class="features">
                <div class="feature">
                    <h3>🔌 Service Endpoints</h3>
                    <div class="endpoint">GET /{{wallet}}/{{service}}</div>
                    <div class="endpoint">POST /{{wallet}}/{{service}}/swap</div>
                    <div class="endpoint">GET /{{wallet}}/{{service}}/quote</div>
                    <p>Access decentralized services through simple HTTP APIs with automatic micropayments.</p>
                </div>

                <div class="feature">
                    <h3>📊 Real-time Dashboard</h3>
                    <div class="endpoint">GET /dashboard/{{wallet}}</div>
                    <div class="endpoint">GET /api/status/{{wallet}}</div>
                    <p>Monitor your allocations, earnings, and service usage in real-time.</p>
                </div>

                <div class="feature">
                    <h3>🎮 Free Tier Services</h3>
                    <p>Try our free computational services:</p>
                    <ul>
                        <li>🥧 Pi Calculator - Leibniz formula implementation</li>
                        <li>🐰 Fibonacci Generator - With rabbit memes</li>
                        <li>🎭 Prime Poetry - Mathematical art</li>
                    </ul>
                </div>

                <div class="feature">
                    <h3>💰 Token Economics</h3>
                    <p>Earn commissions from:</p>
                    <ul>
                        <li>20% of swap fees on your endpoints</li>
                        <li>10% of referred users' lifetime fees</li>
                        <li>5% of service payments you host</li>
                        <li>Tier multipliers up to 2x earnings</li>
                    </ul>
                </div>
            </div>

            <div class="cta">
                <a href="/dashboard/demo" class="btn">🎯 Try Demo Dashboard</a>
                <a href="/api/health" class="btn">📊 API Health Check</a>
            </div>
        </div>

        <script>
            // Check server status
            fetch('/health')
                .then(r => r.json())
                .then(data => {{
                    document.getElementById('status').textContent = data.status === 'healthy' ? 'Online ✅' : 'Issues ⚠️';
                }})
                .catch(() => {{
                    document.getElementById('status').textContent = 'Offline ❌';
                }});
        </script>
    </body>
    </html>
    "#, domain);

    Html(homepage)
}

#[derive(Clone)]
pub struct AppState {
    pub libp2p_swarm: Arc<RwLock<Swarm<ZosBehaviour>>>,
    pub user_sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    pub service_registry: Arc<RwLock<HashMap<String, ServiceEndpoint>>>,
    pub config: ZosConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZosConfig {
    pub http_port: u16,
    pub libp2p_port: u16,
    pub domain: String,
    pub solana_rpc: String,
    pub max_concurrent_users: u32,
    pub block_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub wallet_address: String,
    pub allocated_port: Option<u16>,
    pub credits: u64,
    pub last_activity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub service_name: String,
    pub wallet_address: String,
    pub libp2p_port: u16,
    pub pricing_tier: String,
}

// Placeholder for LibP2P behaviour
pub struct ZosBehaviour;

pub async fn create_zos_server() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = ZosConfig {
        http_port: 3000,
        libp2p_port: 4001,
        domain: "node1.solfunmeme.com".to_string(),
        solana_rpc: "https://api.mainnet-beta.solana.com".to_string(),
        max_concurrent_users: 50,
        block_duration_ms: 400,
    };

    // Initialize LibP2P (placeholder)
    let swarm = create_libp2p_swarm().await?;

    // Create shared state
    let state = AppState {
        libp2p_swarm: Arc::new(RwLock::new(swarm)),
        user_sessions: Arc::new(RwLock::new(HashMap::new())),
        service_registry: Arc::new(RwLock::new(HashMap::new())),
        config: config.clone(),
    };

    // Create HTTP router
    let app = Router::new()
        // Public API endpoints
        .route("/", get(serve_homepage))
        .route("/health", get(health_check))

        // Service endpoints
        .route("/:wallet/:service", get(handle_service_get).post(handle_service_post))
        .route("/:wallet/:service/swap", post(handle_swap))
        .route("/:wallet/:service/quote", get(handle_quote))

        // Dashboard endpoints
        .route("/dashboard/:wallet", get(serve_dashboard))
        .route("/api/status/:wallet", get(get_user_status))
        .route("/api/allocate-port", post(allocate_port))

        // Static files
        .route("/static/*file", get(serve_static))

        .with_state(state.clone());

    let addr = format!("0.0.0.0:{}", config.http_port);
    println!("🚀 ZOS Server starting on {}", addr);

    // Run HTTP server and LibP2P concurrently
    tokio::select! {
        result = axum::Server::bind(&addr.parse()?)
            .serve(app.into_make_service()) => {
            println!("HTTP server error: {:?}", result);
        },
        _ = run_libp2p_loop(state.libp2p_swarm.clone()) => {
            println!("LibP2P loop ended");
        },
        _ = run_background_tasks(state.clone()) => {
            println!("Background tasks ended");
        }
    }

    Ok(())
}

async fn serve_homepage() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>ZOS - Zero Ontology System</title>
        <style>
            body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
            .hero { text-align: center; background: linear-gradient(45deg, #ff6b6b, #4ecdc4);
                    color: white; padding: 40px; border-radius: 10px; margin-bottom: 30px; }
            .feature { background: #f8f9fa; padding: 20px; margin: 10px 0; border-radius: 8px; }
            .endpoint { background: #e9ecef; padding: 10px; font-family: monospace; margin: 5px 0; }
        </style>
    </head>
    <body>
        <div class="hero">
            <h1>🚀 ZOS Server v1.0</h1>
            <p>Decentralized compute platform with token economics</p>
        </div>

        <div class="feature">
            <h3>🔌 Service Endpoints</h3>
            <div class="endpoint">GET /{wallet}/{service} - Call service</div>
            <div class="endpoint">POST /{wallet}/{service}/swap - Swap tokens</div>
            <div class="endpoint">GET /{wallet}/{service}/quote - Get swap quote</div>
        </div>

        <div class="feature">
            <h3>📊 Dashboard</h3>
            <div class="endpoint">GET /dashboard/{wallet} - User dashboard</div>
            <div class="endpoint">GET /api/status/{wallet} - API status</div>
        </div>

        <div class="feature">
            <h3>🎮 Free Tier Services</h3>
            <p>Try our free services: Pi calculator, Fibonacci generator, Prime poetry</p>
            <button onclick="window.location='/dashboard/demo'">Try Demo</button>
        </div>
    </body>
    </html>
    "#)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    }))
}

async fn handle_service_get(
    Path((wallet, service)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {

    // Check if service exists
    let registry = state.service_registry.read().await;
    let service_key = format!("{}_{}", wallet, service);

    if let Some(endpoint) = registry.get(&service_key) {
        // Forward to LibP2P service (simplified)
        let response = serde_json::json!({
            "service": service,
            "wallet": wallet,
            "result": "Service response from LibP2P",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(Json(response))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn handle_service_post(
    Path((wallet, service)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {

    // Similar to GET but with payload processing
    let response = serde_json::json!({
        "service": service,
        "wallet": wallet,
        "input": payload,
        "result": "POST service response",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(response))
}

async fn handle_swap(
    Path((wallet, service)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(swap_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {

    // Simplified swap handling
    let response = serde_json::json!({
        "transaction_id": format!("tx_{}", chrono::Utc::now().timestamp()),
        "from_token": swap_request.get("from_token").unwrap_or(&serde_json::Value::Null),
        "to_token": swap_request.get("to_token").unwrap_or(&serde_json::Value::Null),
        "input_amount": swap_request.get("amount").unwrap_or(&serde_json::Value::Null),
        "output_amount": 95.2,
        "fee": 2.0,
        "status": "completed"
    });

    Ok(Json(response))
}

async fn handle_quote(
    Path((wallet, service)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {

    let from_token = params.get("from").unwrap_or(&"SOLFUNMEME".to_string());
    let to_token = params.get("to").unwrap_or(&"USDC".to_string());
    let amount: f64 = params.get("amount").unwrap_or(&"100".to_string()).parse().unwrap_or(100.0);

    let response = serde_json::json!({
        "from_token": from_token,
        "to_token": to_token,
        "input_amount": amount,
        "quoted_price": amount * 0.95,
        "slippage": 2.1,
        "expires_at": chrono::Utc::now().timestamp() + 30
    });

    Ok(Json(response))
}

async fn serve_dashboard(
    Path(wallet): Path<String>,
    State(state): State<AppState>,
) -> Html<String> {

    let dashboard_html = format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>ZOS Dashboard - {}</title>
        <style>
            body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
            .container {{ max-width: 1200px; margin: 0 auto; }}
            .card {{ background: white; padding: 20px; margin: 10px 0; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
            .stat {{ display: inline-block; margin: 10px 20px; text-align: center; }}
            .stat-value {{ font-size: 2em; font-weight: bold; color: #4ecdc4; }}
            .stat-label {{ color: #666; }}
            button {{ background: #4ecdc4; color: white; border: none; padding: 10px 20px; border-radius: 5px; cursor: pointer; }}
            button:hover {{ background: #45b7aa; }}
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🎯 ZOS Dashboard</h1>
            <p>Wallet: <code>{}</code></p>

            <div class="card">
                <h3>📊 Quick Stats</h3>
                <div class="stat">
                    <div class="stat-value">-</div>
                    <div class="stat-label">Rank</div>
                </div>
                <div class="stat">
                    <div class="stat-value">100</div>
                    <div class="stat-label">Credits</div>
                </div>
                <div class="stat">
                    <div class="stat-value">0</div>
                    <div class="stat-label">Points</div>
                </div>
            </div>

            <div class="card">
                <h3>🔌 Current Allocation</h3>
                <p>No active port. <button onclick="allocatePort()">Allocate Port</button></p>
            </div>

            <div class="card">
                <h3>🎯 Quick Actions</h3>
                <button onclick="executeService('pi_calculator')">🥧 Calculate Pi</button>
                <button onclick="executeService('fibonacci')">🐰 Fibonacci</button>
                <button onclick="executeService('primes')">🎭 Prime Poetry</button>
            </div>
        </div>

        <script>
            async function allocatePort() {{
                try {{
                    const response = await fetch('/api/allocate-port', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ wallet: '{}' }})
                    }});
                    const result = await response.json();
                    alert('Port allocated: ' + result.port);
                    location.reload();
                }} catch (e) {{
                    alert('Error: ' + e.message);
                }}
            }}

            async function executeService(service) {{
                alert('Executing ' + service + '...');
                // Would make actual API call
            }}
        </script>
    </body>
    </html>
    "#, wallet, wallet, wallet);

    Html(dashboard_html)
}

async fn get_user_status(
    Path(wallet): Path<String>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {

    let sessions = state.user_sessions.read().await;

    if let Some(session) = sessions.get(&wallet) {
        Json(serde_json::json!({
            "wallet": wallet,
            "credits": session.credits,
            "allocated_port": session.allocated_port,
            "last_activity": session.last_activity,
            "status": "active"
        }))
    } else {
        Json(serde_json::json!({
            "wallet": wallet,
            "status": "not_found"
        }))
    }
}

async fn allocate_port(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {

    let wallet = request.get("wallet")
        .and_then(|w| w.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Simplified port allocation
    let port = 20001 + (wallet.len() % 1000) as u16;

    let mut sessions = state.user_sessions.write().await;
    let session = sessions.entry(wallet.to_string()).or_insert(UserSession {
        wallet_address: wallet.to_string(),
        allocated_port: None,
        credits: 100,
        last_activity: chrono::Utc::now().timestamp() as u64,
    });

    session.allocated_port = Some(port);

    Ok(Json(serde_json::json!({
        "success": true,
        "port": port,
        "expires_in_blocks": 1
    })))
}

async fn serve_static(Path(file): Path<String>) -> Result<String, StatusCode> {
    // Serve static files (CSS, JS, images)
    match file.as_str() {
        "style.css" => Ok("/* ZOS Styles */".to_string()),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_libp2p_swarm() -> Result<Swarm<ZosBehaviour>, Box<dyn std::error::Error>> {
    // Placeholder LibP2P setup
    // In real implementation, would create actual LibP2P swarm
    todo!("Implement LibP2P swarm creation")
}

async fn run_libp2p_loop(swarm: Arc<RwLock<Swarm<ZosBehaviour>>>) {
    // LibP2P event loop
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        // Process LibP2P events
    }
}

async fn run_background_tasks(state: AppState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(state.config.block_duration_ms));

    loop {
        interval.tick().await;

        // Block advancement logic
        println!("⏰ Block tick - cleaning up expired ports");

        // Clean up expired sessions
        let mut sessions = state.user_sessions.write().await;
        let current_time = chrono::Utc::now().timestamp() as u64;

        sessions.retain(|_, session| {
            current_time - session.last_activity < 3600 // Keep active sessions
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting ZOS Server v1.0");
    create_zos_server().await
}
