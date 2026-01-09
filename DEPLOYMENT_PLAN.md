# ZOS Server Deployment Plan - Staged Rollout

## Stage 1: Foundation Server (Week 1-2)
**Goal**: Get basic server running with core functionality

### Core Components
- [ ] **Unified HTTP Server**: Single Rust process with embedded HTTP server
- [ ] **Basic Authentication**: Wallet signature verification
- [ ] **Port Allocation**: Simple block-based port assignment
- [ ] **Free Tier Services**: Pi calculator, Fibonacci, Prime poetry
- [ ] **Simple Dashboard**: Basic user status and port info

### Architecture Decision: Embedded HTTP Server
```rust
// Single process architecture
tokio::spawn(async {
    // LibP2P node
    let mut swarm = libp2p::Swarm::new(transport, behaviour, peer_id);

    // HTTP server in same process
    let app = axum::Router::new()
        .route("/:wallet/:service", get(handle_service))
        .route("/:wallet/:service/swap", post(handle_swap));

    // Run both concurrently
    tokio::select! {
        _ = swarm.next() => {},
        _ = axum::Server::bind(&addr).serve(app.into_make_service()) => {}
    }
});
```

### Deliverables
- Single binary: `zos-server`
- Config file: `zos-config.toml`
- Basic web UI: Static files served from binary
- Docker image: `zos-server:v1.0`

---

## Stage 2: Economic Layer (Week 3-4)
**Goal**: Add token economics and payment system

### New Features
- [ ] **Payment Processing**: HTTP 402 with USDC/SOL support
- [ ] **Swap Integration**: Basic USDC ↔ SOLFUNMEME swaps
- [ ] **Commission System**: Referral tracking and payouts
- [ ] **Rate Limiting**: Per-wallet usage quotas
- [ ] **Pricing Tiers**: Free/Basic/Premium service levels

### Integration Points
- Solana RPC for balance checking
- Simple AMM for token swaps
- Payment verification via transaction signatures

---

## Stage 3: Community Features (Week 5-6)
**Goal**: Add social and governance features

### New Features
- [ ] **Telegram Bot**: Wallet linking and group access
- [ ] **Unix Accounts**: Real user accounts with resource limits
- [ ] **Vouching System**: Community-driven onboarding
- [ ] **Governance DAO**: Basic proposal and voting system
- [ ] **Leaderboards**: Ranking and reputation system

---

## Stage 4: Advanced Services (Week 7-8)
**Goal**: Add AI marketplace and advanced features

### New Features
- [ ] **AI Prompt Marketplace**: LLM service hosting
- [ ] **Plugin System**: Dynamic service loading
- [ ] **Multi-server Network**: Federated server discovery
- [ ] **Advanced Analytics**: Usage metrics and optimization

---

## HTTP Server Architecture Options

### Option A: Embedded Axum (Recommended)
**Pros**:
- Single process, easier deployment
- Shared state between HTTP and LibP2P
- Lower resource usage
- Simpler configuration

**Cons**:
- Less flexibility for scaling
- HTTP and P2P coupled

```rust
use axum::{Router, extract::Path, Json};
use libp2p::Swarm;

#[tokio::main]
async fn main() {
    // Initialize LibP2P
    let swarm = create_libp2p_swarm().await;

    // Initialize HTTP server
    let app = Router::new()
        .route("/:wallet/:service", get(proxy_to_libp2p))
        .route("/:wallet/:service/swap", post(handle_swap))
        .route("/dashboard/:wallet", get(serve_dashboard));

    // Run both in same process
    tokio::select! {
        _ = run_libp2p(swarm) => {},
        _ = axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
            .serve(app.into_make_service()) => {}
    }
}
```

### Option B: Separate Nginx + Rust
**Pros**:
- Better for high traffic
- Can use Nginx features (SSL, caching, etc.)
- More traditional architecture

**Cons**:
- More complex deployment
- Need IPC between processes
- Higher resource usage

---

## Recommended Stage 1 Implementation

### Single Binary Architecture
```
zos-server (single Rust binary)
├── HTTP Server (Axum)
│   ├── Public API endpoints
│   ├── Payment processing
│   ├── Static file serving
│   └── WebSocket for real-time updates
├── LibP2P Node
│   ├── Service discovery
│   ├── Peer communication
│   └── Protocol handlers
├── Embedded Database (SQLite)
│   ├── User accounts
│   ├── Transaction history
│   └── Configuration
└── Background Tasks
    ├── Block monitoring
    ├── Payment verification
    └── Cleanup jobs
```

### Configuration File
```toml
[server]
http_port = 3000
libp2p_port = 4001
domain = "node1.solfunmeme.com"

[blockchain]
solana_rpc = "https://api.mainnet-beta.solana.com"
solfunmeme_mint = "SoLFuNMeMeTokenAddress123456789"

[services]
max_concurrent_users = 50
block_duration_ms = 400
free_tier_credits = 100

[payments]
usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
swap_fee_percentage = 0.3
commission_rates = { swap = 20.0, referral = 10.0, service = 5.0 }
```

### Deployment Strategy
1. **Development**: `cargo run` with local config
2. **Testing**: Docker container with test tokens
3. **Production**: Systemd service with real tokens

### Build Commands
```bash
# Stage 1 build
cargo build --release --features "stage1"

# Create deployment package
./scripts/package-stage1.sh

# Deploy to server
./scripts/deploy-stage1.sh node1.solfunmeme.com
```

This gives us a solid foundation that we can iterate on quickly, with clear upgrade paths for each stage.
