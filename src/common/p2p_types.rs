use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PVerb {
    // Plugin Management
    LoadSo(String, String),
    RegisterEvent(String, u32),
    AttachData(String, Vec<u8>),
    RunWithFiles(String, Vec<String>),
    CaptureResult(String),
    CompileSource(String, String),
    CompileFile(String, String),
    InvokeFunction(String, String, u32),

    // Telemetry & Analysis
    StartTrace(String, String),
    StopTrace(String),
    PerfRecord(String, String),
    GetMetrics(String),

    // Mathematical Operations
    ComputeEigenvalues(String),
    SolveLinearSystem(String, Vec<f64>),
    OptimizeFunction(String, HashMap<String, f64>),

    // Dataset Management
    LoadDataset(String, String),
    QueryDataset(String, String),
    TransformData(String, String),
    ExportResults(String, String),

    // Network Operations
    ConnectPeer(String),
    BroadcastMessage(String),
    SyncState(String),
    RequestResource(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub multiaddr: String,
    pub protocols: Vec<String>,
    pub last_seen: DateTime<Utc>,
    pub reputation: f64,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSeed {
    pub name: String,
    pub description: String,
    pub source_url: String,
    pub hash: String,
    pub size_bytes: u64,
    pub format: String,
    pub schema: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedSo {
    pub name: String,
    pub path: String,
    pub symbols: Vec<String>,
    pub loaded_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub compilation_time_ms: u64,
    pub binary_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionAnalysis {
    pub function_name: String,
    pub complexity_score: f64,
    pub memory_usage_bytes: u64,
    pub execution_time_ms: u64,
    pub dependencies: Vec<String>,
    pub risk_factors: Vec<String>,
}
