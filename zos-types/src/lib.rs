// ZOS Types - Zero dependency type foundation
// AGPL-3.0 License

/// Security levels (no external dependencies)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    Safe = 0,
    Controlled = 1,
    Privileged = 2,
    Critical = 3,
}

/// LMFDB orbit reference (string-based, no math dependencies)
#[derive(Debug, Clone)]
pub struct LMFDBOrbitRef {
    pub orbit_id: String,
    pub complexity_class: String,
    pub lmfdb_url: String,
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub security_level: SecurityLevel,
    pub lmfdb_orbit: Option<LMFDBOrbitRef>,
}
