// Security Analysis Module
pub mod binary_inspector;
pub mod binary_classifier;
pub mod exploit_detector;
pub mod entropy_scanner;
pub mod packet_entropy_analyzer;
pub mod syscall_labeler;
pub mod lmfdb_orbit_filter;

pub use binary_inspector::{BinaryInspector, PluginSecurityLattice};
pub use binary_classifier::{BinaryClassifier, VerificationSystem};
pub use exploit_detector::ExploitDetector;
pub use entropy_scanner::{EntropyScanner, EntropyEnforcer};
pub use packet_entropy_analyzer::PacketEntropyAnalyzer;
pub use syscall_labeler::SyscallLabeler;
pub use lmfdb_orbit_filter::{LMFDBOrbitFilter, OrbitClass};

/// Main security analysis interface
pub struct SecurityAnalysis {
    binary_inspector: BinaryInspector,
    exploit_detector: ExploitDetector,
    entropy_scanner: EntropyScanner,
    packet_analyzer: PacketEntropyAnalyzer,
    orbit_filter: LMFDBOrbitFilter,
}

impl SecurityAnalysis {
    pub fn new() -> Self {
        Self {
            binary_inspector: BinaryInspector::new(),
            exploit_detector: ExploitDetector::new(),
            entropy_scanner: EntropyScanner::new(4.0), // 4.0 bit entropy limit
            packet_analyzer: PacketEntropyAnalyzer::new(4.0),
            orbit_filter: LMFDBOrbitFilter::new(),
        }
    }

    pub fn analyze_binary(&mut self, binary_path: &str) -> Result<String, String> {
        // Unified binary analysis
        let classification = self.binary_inspector.inspect_binary(binary_path)?;
        Ok(format!("Binary analyzed: {} functions classified", classification.len()))
    }

    pub fn analyze_packet(&mut self, user_id: &str, packet_data: &[u8]) -> Result<bool, String> {
        let analysis = self.packet_analyzer.analyze_packet(user_id, packet_data);
        Ok(analysis.allowed)
    }

    pub fn classify_orbit(&self, function_name: &str) -> OrbitClass {
        self.orbit_filter.classify_function(function_name)
    }
}
