#![allow(unused)]

// Web layer - handles HTTP requests and responses
use crate::core::{User, ZOSCore};
use crate::telemetry::trace_user_operation;
use crate::*; // Import Clip2Secure macros
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, instrument};

pub type AppState = Arc<Mutex<ZOSCore>>;

// #[security_context(level = "Public", price_tier = 0.0, matrix_access = "DiagonalOnly")]
// #[complexity(level = "Trivial", orbit_size = 1, time = "O(1)", space = "O(1)")]
pub fn create_router(core: AppState) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/value-lattice", get(value_lattice_handler))
        .route("/health", get(health_handler))
        .with_state(core)
}

async fn root_handler() -> Response {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>ZOS Server</title>
</head>
<body>
    <h1>🚀 ZOS Server</h1>
    <p><a href="/dashboard">Dashboard</a></p>
    <p><a href="/value-lattice">Value Lattice</a></p>
</body>
</html>
    "#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.into())
        .unwrap()
}

async fn health_handler() -> &'static str {
    "OK"
}

#[instrument(skip_all)]
async fn dashboard_handler(
    Query(params): Query<HashMap<String, String>>,
    State(core): State<AppState>,
) -> Response {
    let token = params.get("token");

    if let Some(token) = token {
        let validation_result = {
            let core = core.lock().unwrap();
            core.validate_session(token).map(|u| u.clone())
        };

        if let Some(user) = validation_result {
            let username = user.username.clone();
            return trace_user_operation(&username, "dashboard_access", async move {
                info!("Dashboard access granted for user: {}", user.username);
                dashboard_html(&user)
            })
            .await;
        }
    }

    login_html()
}

fn dashboard_html(user: &User) -> Response {
    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>ZOS Dashboard</title>
</head>
<body>
    <h1>🎯 ZOS Dashboard</h1>
    <p>Welcome, {}!</p>
    <p>Permissions: {}</p>
</body>
</html>
    "#,
        user.username,
        user.permissions.join(", ")
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.into())
        .unwrap()
}

fn login_html() -> Response {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>ZOS Login</title>
</head>
<body>
    <h1>🔐 ZOS Login</h1>
    <p>Run: <code>cargo run --bin zos_server login &lt;username&gt;</code></p>
</body>
</html>
    "#;

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.into())
        .unwrap()
}

async fn value_lattice_handler() -> Response {
    use std::fs;
    use std::process::Command;

    // Check service status
    let service_status = Command::new("pgrep")
        .arg("-f")
        .arg("value_lattice_indexer")
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);

    // Get memory info
    let memory_info = Command::new("free")
        .arg("-h")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_else(|_| "Memory info unavailable".to_string());

    // Calculate lattice structure dynamically
    let lattice_path = "/mnt/data1/meta-introspector/value-lattice";
    let mut lattice_stats = Vec::new();
    let mut total_canonical_forms = 0;
    let mut convergence_levels = Vec::new();

    if let Ok(entries) = fs::read_dir(lattice_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                let count = fs::read_dir(entry.path())
                    .map(|entries| entries.count())
                    .unwrap_or(0);

                lattice_stats.push((dir_name.clone(), count));
                total_canonical_forms += count;

                // Map to convergence levels
                if dir_name.starts_with("length-1") {
                    convergence_levels.push(("Level 0: Literals", count, "Peano Axioms (ℕ)"));
                } else if dir_name.starts_with("length-2") {
                    convergence_levels.push(("Level 1: Functions", count, "Church Lambda (λ)"));
                } else if dir_name.starts_with("length-3") {
                    convergence_levels.push(("Level 2: Enums", count, "Finite Types"));
                }
            }
        }
    }

    // Sort by length for display
    lattice_stats.sort_by(|a, b| {
        let a_num: u32 =
            a.0.strip_prefix("length-")
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
        let b_num: u32 =
            b.0.strip_prefix("length-")
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
        a_num.cmp(&b_num)
    });

    let stats_html = lattice_stats
        .iter()
        .map(|(dir, count)| format!("<tr><td>{}</td><td>{}</td></tr>", dir, count))
        .collect::<Vec<_>>()
        .join("");

    // Calculate foundational mappings
    let peano_count = lattice_stats
        .iter()
        .find(|(d, _)| d == "length-1")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    let church_count = lattice_stats
        .iter()
        .filter(|(d, _)| d.starts_with("length-"))
        .map(|(_, c)| *c)
        .sum::<usize>();
    let kleene_count = lattice_stats
        .iter()
        .filter(|(d, _)| d.contains("length-"))
        .count();

    let convergence_html = format!(
        r#"
        <div class="level">Level 0: Literals → Peano Axioms ({} forms)</div>
        <div class="level">Level 1: Functions → Church Lambda ({} total)</div>
        <div class="level">Level 2-∞: Patterns → Kleene/Turing/Gödel ({} levels)</div>
        <div class="level">Level ∞: Types → MetaCoq Proofs (convergent)</div>
    "#,
        peano_count, church_count, kleene_count
    );

    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Zero Ontology Lattice Dashboard</title>
    <style>
        body {{ font-family: monospace; margin: 20px; background: #0a0a0a; color: #00ff00; }}
        .status {{ padding: 10px; border-radius: 5px; margin: 10px 0; }}
        .running {{ background: #1a4a1a; color: #00ff00; border: 1px solid #00ff00; }}
        .stopped {{ background: #4a1a1a; color: #ff4444; border: 1px solid #ff4444; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #333; padding: 8px; text-align: left; }}
        th {{ background-color: #1a1a1a; color: #00ffff; }}
        pre {{ background: #111; padding: 10px; border-radius: 5px; border: 1px solid #333; }}
        .convergence {{ background: #1a1a2a; padding: 15px; border-radius: 5px; margin: 15px 0; border: 1px solid #4444ff; }}
        .foundation {{ color: #ffaa00; }}
        .level {{ color: #aaaaff; margin: 5px 0; }}
        .stats {{ color: #00ffaa; }}
        a {{ color: #00ffff; }}
    </style>
</head>
<body>
    <h1>🔢 Zero Ontology Lattice Dashboard</h1>

    <div class="status {}">
        <strong>Service Status:</strong> {}
    </div>

    <div class="convergence">
        <h2 class="foundation">🧮 Foundational Convergence</h2>
        {}
        <div class="stats">Total Canonical Forms: {}</div>
    </div>

    <h2>Memory Usage</h2>
    <pre>{}</pre>

    <h2>Value Lattice Structure</h2>
    <table>
        <tr><th>Length Directory</th><th>Canonical Forms</th></tr>
        {}
    </table>

    <h2>Canonical Index Status</h2>
    <p>📊 Processing 1.51M Rust files from canonical index</p>
    <p>🔗 Mapping to foundational mathematical structures</p>
    <p>♾️ Converging to universal zero ontology</p>

    <p><a href="/">← Back to Home</a></p>
</body>
</html>
    "#,
        if service_status { "running" } else { "stopped" },
        if service_status {
            "🟢 Running"
        } else {
            "🔴 Stopped"
        },
        convergence_html,
        total_canonical_forms,
        memory_info,
        stats_html
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.into())
        .unwrap()
}
