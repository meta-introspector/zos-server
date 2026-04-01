# ZOS Server

`zos-server` is the current peer-sync and bounded artifact-convergence runtime for the broader ZOS stack. In this repository, the active implementation focus is libp2p-backed inventory exchange, reconciliation, and bounded replay recovery rather than the whole conceptual platform.

## 🏗️ Architecture

### Repo-Facing Stack
- **Layer 4**: SL as truth and promotion boundary
- **Layer 3**: ZOS plus DASHI plus MDL as the structure and selection layer
- **Layer 2**: P2P as state movement, sync, and replication
- **Layer 1**: Storage and transport

Boundary notes:
- `SL` remains the authority for promoted truth.
- `ZOS` in this repo should be read as governed semantic state over promoted facts, not as a truth override layer.
- Peer sync in `zos-server` moves and reconciles bounded artifact state; it does not define semantic promotion policy.

### Plugin Layers
- **Layer -4**: Advanced ZK (Rollups, Lattice Folding, HME, MetaCoq, Lean4)
- **Layer -3**: Zero Knowledge (ZK-SNARKs, ZK-STARKs, Correctness Proofs)
- **Layer -2**: Regulatory (SEC, Quality, GDPR/HIPAA/SOX/ISO)
- **Layer -1**: Governance (Voting, Resources, ERP)
- **Layer 0**: Foundation (LMFDB, Wikidata, OSM, Archive.org, SDF.org)
- **Layer 1**: System (19 plugins: SystemD, Docker, Compilers, Blockchain, etc.)
- **Layer 2**: Data Formats (Parquet, HuggingFace, RDF, SQL, Protocols)
- **Layer ∞**: Recursive (Each layer exports to all others infinitely)

### Core Features
- **Universal Plugin Runtime**: Run any plugin on any architecture
- **Verified Execution**: Every plugin mathematically proven correct
- **Cross-Architecture**: Native ELF ↔ WASM ↔ ARM ↔ x86_64 ↔ RISC-V
- **Blockchain Integration**: Consume and create rollups from all major chains
- **Browser Extension**: Advanced UI helpers for web interaction
- **LMFDB Complexity Proofs**: Mathematical complexity verification

## 🚀 Quick Start

### Development Server
```bash
cd zos-minimal-server
cargo build --release
./target/release/zos-minimal-server
```

### CI/CD Pipeline Setup
```bash
# Install QA service
curl -X POST http://localhost:8080/install/qa-service

# Update QA from git
curl -X POST http://localhost:8080/manage/qa/update

# Deploy to production
curl -X POST http://localhost:8080/deploy/staging-to-prod
```

See [PIPELINE.md](PIPELINE.md) for complete CI/CD documentation.

### Build with Nix
```bash
nix-build default.nix
./result/bin/zos_server
```

### Build with Cargo
```bash
cargo build --release --features all-plugins
./target/release/zos_server
```

### Development Build
```bash
cargo run --features all-plugins
```

### mesh-sync-rs Compatibility Shim
`zos-server` now exposes a narrow HTTP compatibility shim for the sibling `mesh-sync-rs` repo. This shim is intended for quick cooperation only; the canonical sync path in this repo remains the libp2p-backed coordinator flow.

Supported endpoints on the `serve` surface:
- `GET /mesh/peers`
- `GET /mesh/logs`
- `POST /mesh/logs`

Environment contract:
```bash
export MESH_PEERS="10.0.0.12,peer.example.com,http://127.0.0.1:7780"
export MESH_SELF_ADDR="127.0.0.1:7780"
cargo run -- serve 7780
```

Notes:
- `GET /mesh/peers` returns bare host-style addresses because `mesh-sync-rs` appends `:7780` itself.
- `GET /mesh/logs` and `POST /mesh/logs` read and write JSON payloads under `~/.solfunmeme/mesh-logs/`.
- This shim does not yet translate mesh log JSON into `ZosNode`, `SyncWireMessage`, or coordinator inventory state.

## 🗺️ Roadmap

### Server Infrastructure
- **Build Server**: Multi-architecture compilation service
- **Compile Server**: Distributed compilation with caching
- **Verification Server**: ZK proof generation and validation
- **Package Repository**: Docker/dpkg/Nix store server
- **Binary Repository**: Bintray-compatible artifact storage

### AI-Powered Development
- **Security Scanner**: Automated vulnerability detection
- **AI Code Analysis**: Semantic code understanding and optimization
- **Auto Bug Fixing**: Intelligent patch generation and application
- **DevOps as a Service**: Complete CI/CD pipeline automation

### Network Services
- **P2P Build Clusters**: Distributed compilation networks
- **Rollup Aggregation**: Multi-chain transaction bundling
- **Proof Marketplace**: ZK proof trading and verification

### Sync Convergence Roadmap
- The repo-facing semantic boundary is `SL -> ZOS -> downstream consumers`: `SL` promotes truth, `ZOS` organizes promoted facts as governed semantic state, and the peer-sync layer only moves bounded artifact state plus replay metadata.
- `src/node_coordinator.rs` and `crates/zos-experimental/src/node_coordinator.rs` currently own the peer sync loop.
- `sync_with_peers()` now builds compact local inventory, computes a reconciliation plan per peer, emits announcement frames, and emits a serialized wire envelope for reconciliation and inventory traffic.
- The active coordinator now executes bounded replay recovery for canonical artifact and receipt gaps when acknowledged locators are present, verifies digest parity before admitting recovered items into local inventory, and keeps the recovered state ephemeral rather than ledger-like.
- The active and experimental coordinator paths both record bounded replay intent for missing canonical artifact and receipt identities and carry replay locator metadata for recovery planning.
- Replay in this repo is intended to follow the ITIR consumer pattern: acknowledged artifact identity plus bounded locator data should be enough for `objectRef`-style recovery and digest verification, but `zos-server` must not become a StatiBaker-style receipt or timeline ledger.
- Existing libp2p and gossipsub scaffolding in `src/extra_plugins/libp2p_c_interface.rs` and `zos-libp2p/src/server.rs` remains the transport substrate; the current sync slice binds outbound/inbound envelopes into a live `ZosNode` instance via `src/sync_transport.rs` and `main.rs`.
- The live bridge now accepts `ZOS_SYNC_LISTEN_ADDR` and `ZOS_SYNC_BOOTSTRAP_ADDRS` so separate local peers can actually listen and dial over libp2p instead of staying process-local.
- Producer-facing identity parsing now normalizes common zkperf and erdfa-shaped fields on the bridge surface so artifact and receipt identities arrive in canonical form more often.
- The live transport path now has bounded operational controls: startup visibility, duplicate inbound frame suppression, and clearer drop-path logging.
- `scripts/smoke_two_peer_sync.sh` now proves a same-host two-process libp2p smoke run with connection establishment, reconciliation, and artifact recovery.
- Full remote multi-operator convergence validation and full acknowledged-locator coverage across producer surfaces remain the next control gates before broader delta-sync claims.
- Trust scoring and MDL-aware replication remain explicitly deferred until basic cross-node convergence is testable.
- The governing architecture and release-gate note for this slice now lives in `docs/sync_convergence_architecture.md`.
- That architecture note governs sync and replay behavior only; truth and semantic promotion remain outside the scope of this repository's transport layer.

### Enterprise Services
- **LLM Routing & Proxy**: Vendor-agnostic AI model access and load balancing
- **Vector Storage APIs**: Embeddings and semantic search infrastructure
- **Storage APIs**: Unified object/block/file storage abstraction
- **ACL & Auth**: Role-based access control and authentication
- **API Gateway**: Rate limiting, routing, and service mesh
- **Security & Auditing**: Compliance monitoring (GDPR/HIPAA/SOX/ISO)
- **Storage Security**: Encryption, key management, and data governance

## 📦 Plugin System

### Canonical Plugin Structure
Every plugin implements:
- **Trait**: Rust trait with execute/verify/profile methods
- **Macro**: Code generation macro for plugin creation
- **ABI**: C-compatible interface for universal loading
- **LMFDB Proof**: Mathematical complexity verification

### Example Plugin
```rust
pub trait MyPlugin {
    fn execute(&self, args: &[u8]) -> Result<Vec<u8>, String>;
    fn verify(&self, proof: &str) -> Result<bool, String>;
    fn profile(&self) -> Result<ComplexityProfile, String>;
}

my_plugin!(MyPluginImpl);
```

## 🔐 Security Model

### Verification Pipeline
1. **Source Hash**: Cryptographic integrity verification
2. **Execution Review**: Static analysis of all code paths
3. **ZK Proof**: Zero-knowledge correctness proof
4. **Cost Profile**: Resource usage analysis with thresholds
5. **Binary Patching**: Automatic vulnerability fixes
6. **LMFDB Verification**: Mathematical complexity proof

### Zero Knowledge Properties
- **Zero Trust**: Mathematical proofs at every layer
- **Zero Secrets**: Homomorphic computation preserves privacy
- **Zero Doubt**: Formal verification in Coq/Lean4
- **Zero Overhead**: Rollups and folding for efficiency

## 🌐 Network Features

### P2P Cooperation
- LibP2P-based node coordination
- Automatic load balancing
- Cross-node plugin execution
- Blockchain rollup sharing

### Browser Integration
- Chrome/Firefox extension
- Real-time ZK proof generation
- Semantic entity extraction
- Compliance checking

## 📊 Supported Systems

### Blockchains
- Ethereum (Proof-of-Stake)
- Bitcoin (Proof-of-Work)
- Solana (Proof-of-History)
- Cosmos (Tendermint)
- Avalanche

### Architectures
- x86_64, ARM64, RISC-V, MIPS
- Native ELF, WASM, Docker
- Cross-compilation via LLVM IR

### Data Formats
- Parquet, HuggingFace, RDF, SQL
- MCP, SOAP, OpenAPI, REST
- LibP2P, Protobuf, JSON-LD

## 🔧 Configuration

### Environment Variables
```bash
export ZOS_NODE_ID="your-node-id"
export ZOS_NETWORK_PORT="8080"
export ZOS_PLUGIN_DIR="/nix/store/.../lib/zos-plugins"
export ZOS_LMFDB_ENDPOINT="https://lmfdb.org/api"
```

### Plugin Configuration
```toml
[plugins]
enable_all_layers = true
cost_threshold_usd = 0.01
verification_required = true
lmfdb_proofs_required = true
```

## 📈 Performance

### Benchmarks
- Plugin loading: <100ms per plugin
- ZK proof generation: <1s for most proofs
- Cross-architecture translation: <5s
- Blockchain rollup creation: <10s per 100 blocks

### Resource Usage
- Memory: ~500MB base + plugins
- CPU: Scales with plugin complexity
- Network: P2P gossip + blockchain sync
- Storage: Plugin cache + verification proofs

## 🤝 Contributing

### Development Setup
```bash
git clone https://github.com/meta-introspector/zos-server
cd zos-server
nix-shell
cargo test --all-features
```

### Plugin Development
1. Implement canonical trait
2. Add LMFDB complexity proof
3. Generate ZK correctness proof
4. Submit for verification

## 📄 License

AGPLv3 License - see LICENSE file for details.

This ensures that when you run ZOS Server as a network service, you remain part of the collective and any modifications must be shared with users of the service.

## 🔗 Links

- [LMFDB](https://lmfdb.org) - Mathematical database
- [Wikidata](https://wikidata.org) - Semantic knowledge
- [OpenStreetMap](https://openstreetmap.org) - Geographic data
- [Archive.org](https://archive.org) - Digital preservation
- [SDF.org](https://sdf.org) - Public access computing
