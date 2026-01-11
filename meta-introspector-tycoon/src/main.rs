use axum::{routing::get, Router, Json};
use serde_json::json;
use std::collections::HashMap;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌌 Meta-Introspector Tycoon Server Starting...");

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/api/tycoon-stats", get(tycoon_stats))
        .route("/api/community-data", get(community_data))
        .route("/api/vote", get(handle_vote))
        .route("/api/join-node", get(join_node));

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("🚀 Server running on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn dashboard() -> &'static str {
    r#"
<!DOCTYPE html>
<html>
<head><title>Meta-Introspector Tycoon</title>
<style>
body { font-family: monospace; background: #0a0a0a; color: #00ff00; padding: 20px; }
.factory { border: 1px solid #00ff00; padding: 15px; margin: 10px; background: #001100; }
.revenue { color: #ffff00; font-weight: bold; }
.infinite { color: #ff00ff; animation: pulse 2s infinite; }
@keyframes pulse { 0% { opacity: 1; } 50% { opacity: 0.5; } 100% { opacity: 1; } }
</style></head>
<body>
<h1>🌌 META-INTROSPECTOR TYCOON 🌌</h1>
<div class="factory">
<h3>🏭 Revolutionary Factories</h3>
<p>Security Lattice Factory: <span class="revenue">$100/sec</span></p>
<p>Kleene Algebra Mine: <span class="revenue">$250/sec</span></p>
<p>Monster Group Foundry: <span class="revenue">$500/sec</span></p>
<p>Unity Convergence Center: <span class="revenue">$2000/sec</span></p>
<p class="infinite">Infinite Complexity Engine: ∞/sec</p>
</div>
<div class="factory">
<h3>🌐 Community Participation</h3>
<p>Commands: !vote !node !feedback !invest</p>
<p>Active Nodes: <span id="node-count">0</span></p>
<p>Total Votes: <span id="vote-count">0</span></p>
</div>
<script>
setInterval(() => {
    fetch('/api/tycoon-stats').then(r => r.json()).then(data => {
        document.getElementById('node-count').textContent = data.nodes;
        document.getElementById('vote-count').textContent = data.votes;
    });
}, 5000);
</script>
</body>
</html>
"#
}

async fn tycoon_stats() -> Json<serde_json::Value> {
    Json(json!({
        "nodes": 42,
        "votes": 156,
        "revenue": 50000.0,
        "factories": 8
    }))
}

async fn community_data() -> Json<serde_json::Value> {
    Json(json!({
        "active_votes": [
            {"topic": "Next Factory", "options": ["Quantum", "Blockchain", "AI"], "votes": [15, 8, 12]}
        ],
        "community_nodes": 42,
        "recent_feedback": ["Great stream!", "Add more particles", "Love the math!"]
    }))
}

async fn handle_vote() -> &'static str {
    "Vote recorded!"
}

async fn join_node() -> &'static str {
    "Node joined successfully!"
}
