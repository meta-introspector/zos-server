// Execution Provenance and Audit System
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Complete execution provenance tracker
pub struct ExecutionProvenance {
    execution_records: HashMap<String, ExecutionRecord>,
    code_lineage: HashMap<String, CodeLineage>,
    data_flow_graph: DataFlowGraph,
    audit_trail: Vec<AuditEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub execution_id: String,
    pub code_hash: String,
    pub user_id: String,
    pub timestamp: u64,
    pub duration_ns: u64,
    pub memory_peak: u64,
    pub cpu_cycles: u64,
    pub inputs: Vec<DataProvenance>,
    pub outputs: Vec<DataProvenance>,
    pub call_stack: Vec<FunctionCall>,
    pub system_calls: Vec<SystemCall>,
    pub network_calls: Vec<NetworkCall>,
    pub file_operations: Vec<FileOperation>,
    pub environment: ExecutionEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLineage {
    pub code_hash: String,
    pub source_origin: SourceOrigin,
    pub compilation_info: CompilationInfo,
    pub dependencies: Vec<DependencyInfo>,
    pub security_classification: String,
    pub approval_chain: Vec<ApprovalRecord>,
    pub deployment_history: Vec<DeploymentRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProvenance {
    pub data_id: String,
    pub data_hash: String,
    pub source_execution: Option<String>,
    pub source_user: Option<String>,
    pub creation_timestamp: u64,
    pub data_type: String,
    pub size_bytes: u64,
    pub classification: DataClassification,
    pub lineage_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub function_name: String,
    pub module_path: String,
    pub entry_timestamp: u64,
    pub exit_timestamp: u64,
    pub memory_delta: i64,
    pub return_value_hash: String,
    pub exception: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCall {
    pub syscall_name: String,
    pub arguments: Vec<String>,
    pub return_value: i64,
    pub timestamp: u64,
    pub blocked: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCall {
    pub destination: String,
    pub port: u16,
    pub protocol: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub timestamp: u64,
    pub allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub operation: String, // read, write, create, delete
    pub file_path: String,
    pub bytes_affected: u64,
    pub timestamp: u64,
    pub allowed: bool,
    pub file_hash_before: Option<String>,
    pub file_hash_after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEnvironment {
    pub container_id: String,
    pub security_layer: String,
    pub orbit_class: String,
    pub energy_consumed: u64,
    pub resource_limits: ResourceLimits,
    pub environment_variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceOrigin {
    pub repository: String,
    pub commit_hash: String,
    pub author: String,
    pub timestamp: u64,
    pub branch: String,
    pub pull_request: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationInfo {
    pub compiler_version: String,
    pub compilation_flags: Vec<String>,
    pub target_architecture: String,
    pub optimization_level: String,
    pub compilation_timestamp: u64,
    pub build_environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub source: String,
    pub hash: String,
    pub security_audit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub approver: String,
    pub approval_type: String,
    pub timestamp: u64,
    pub signature: String,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub deployment_id: String,
    pub target_environment: String,
    pub timestamp: u64,
    pub deployer: String,
    pub rollback_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
    TopSecret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory: u64,
    pub max_cpu_time: u64,
    pub max_file_operations: u32,
    pub max_network_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: AuditEventType,
    pub timestamp: u64,
    pub user_id: String,
    pub execution_id: Option<String>,
    pub details: HashMap<String, String>,
    pub risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    CodeExecution,
    DataAccess,
    SecurityViolation,
    ResourceExceeded,
    UnauthorizedAccess,
    SystemCall,
    NetworkAccess,
    FileAccess,
}

/// Data flow graph for tracking data lineage
pub struct DataFlowGraph {
    nodes: HashMap<String, DataNode>,
    edges: Vec<DataEdge>,
}

#[derive(Debug, Clone)]
pub struct DataNode {
    pub data_id: String,
    pub creation_execution: String,
    pub data_type: String,
    pub classification: DataClassification,
}

#[derive(Debug, Clone)]
pub struct DataEdge {
    pub from_data: String,
    pub to_data: String,
    pub transformation: String,
    pub execution_id: String,
}

impl ExecutionProvenance {
    pub fn new() -> Self {
        Self {
            execution_records: HashMap::new(),
            code_lineage: HashMap::new(),
            data_flow_graph: DataFlowGraph {
                nodes: HashMap::new(),
                edges: Vec::new(),
            },
            audit_trail: Vec::new(),
        }
    }

    /// Start tracking a new execution
    pub fn start_execution(&mut self, code_hash: &str, user_id: &str) -> String {
        let execution_id = format!("exec_{}_{}",
            self.current_timestamp(),
            self.generate_random_id());

        let record = ExecutionRecord {
            execution_id: execution_id.clone(),
            code_hash: code_hash.to_string(),
            user_id: user_id.to_string(),
            timestamp: self.current_timestamp(),
            duration_ns: 0,
            memory_peak: 0,
            cpu_cycles: 0,
            inputs: Vec::new(),
            outputs: Vec::new(),
            call_stack: Vec::new(),
            system_calls: Vec::new(),
            network_calls: Vec::new(),
            file_operations: Vec::new(),
            environment: ExecutionEnvironment {
                container_id: "container_123".to_string(),
                security_layer: "L2Service".to_string(),
                orbit_class: "Cyclic".to_string(),
                energy_consumed: 0,
                resource_limits: ResourceLimits {
                    max_memory: 1024 * 1024,
                    max_cpu_time: 1000,
                    max_file_operations: 10,
                    max_network_connections: 5,
                },
                environment_variables: HashMap::new(),
            },
        };

        self.execution_records.insert(execution_id.clone(), record);

        // Log audit event
        self.log_audit_event(AuditEventType::CodeExecution, user_id, Some(&execution_id),
            HashMap::from([("action".to_string(), "start".to_string())]));

        execution_id
    }

    /// Record function call in execution
    pub fn record_function_call(&mut self, execution_id: &str, function_call: FunctionCall) {
        if let Some(record) = self.execution_records.get_mut(execution_id) {
            record.call_stack.push(function_call);
        }
    }

    /// Record system call attempt
    pub fn record_system_call(&mut self, execution_id: &str, syscall: SystemCall) {
        if let Some(record) = self.execution_records.get_mut(execution_id) {
            record.system_calls.push(syscall.clone());
        }

        // Log security event if blocked
        if syscall.blocked {
            self.log_audit_event(AuditEventType::SecurityViolation,
                &self.get_user_for_execution(execution_id).unwrap_or_default(),
                Some(execution_id),
                HashMap::from([
                    ("syscall".to_string(), syscall.syscall_name),
                    ("blocked".to_string(), "true".to_string()),
                ]));
        }
    }

    /// Record data input with provenance
    pub fn record_input(&mut self, execution_id: &str, data: DataProvenance) {
        if let Some(record) = self.execution_records.get_mut(execution_id) {
            record.inputs.push(data.clone());
        }

        // Add to data flow graph
        self.data_flow_graph.nodes.insert(data.data_id.clone(), DataNode {
            data_id: data.data_id.clone(),
            creation_execution: data.source_execution.unwrap_or_default(),
            data_type: data.data_type,
            classification: data.classification,
        });
    }

    /// Record data output with provenance
    pub fn record_output(&mut self, execution_id: &str, data: DataProvenance) {
        if let Some(record) = self.execution_records.get_mut(execution_id) {
            record.outputs.push(data.clone());
        }

        // Create data flow edges from inputs to outputs
        if let Some(record) = self.execution_records.get(execution_id) {
            for input in &record.inputs {
                self.data_flow_graph.edges.push(DataEdge {
                    from_data: input.data_id.clone(),
                    to_data: data.data_id.clone(),
                    transformation: "execution_transform".to_string(),
                    execution_id: execution_id.to_string(),
                });
            }
        }
    }

    /// Complete execution tracking
    pub fn complete_execution(&mut self, execution_id: &str, duration_ns: u64, memory_peak: u64) {
        if let Some(record) = self.execution_records.get_mut(execution_id) {
            record.duration_ns = duration_ns;
            record.memory_peak = memory_peak;
        }

        self.log_audit_event(AuditEventType::CodeExecution,
            &self.get_user_for_execution(execution_id).unwrap_or_default(),
            Some(execution_id),
            HashMap::from([
                ("action".to_string(), "complete".to_string()),
                ("duration_ns".to_string(), duration_ns.to_string()),
            ]));
    }

    /// Register code lineage
    pub fn register_code_lineage(&mut self, code_hash: &str, lineage: CodeLineage) {
        self.code_lineage.insert(code_hash.to_string(), lineage);
    }

    /// Generate complete provenance report
    pub fn generate_provenance_report(&self, execution_id: &str) -> Option<ProvenanceReport> {
        let execution = self.execution_records.get(execution_id)?;
        let code_lineage = self.code_lineage.get(&execution.code_hash);

        Some(ProvenanceReport {
            execution_summary: ExecutionSummary {
                execution_id: execution_id.to_string(),
                user_id: execution.user_id.clone(),
                duration_ms: execution.duration_ns / 1_000_000,
                memory_peak_mb: execution.memory_peak / 1024 / 1024,
                function_calls: execution.call_stack.len(),
                system_calls: execution.system_calls.len(),
                blocked_operations: execution.system_calls.iter().filter(|s| s.blocked).count(),
            },
            code_origin: code_lineage.cloned(),
            data_lineage: DataLineageSummary {
                inputs_count: execution.inputs.len(),
                outputs_count: execution.outputs.len(),
                data_classifications: self.summarize_data_classifications(&execution.inputs, &execution.outputs),
            },
            security_events: self.get_security_events_for_execution(execution_id),
            compliance_status: self.assess_compliance(execution),
        })
    }

    /// Trace data lineage back to origin
    pub fn trace_data_lineage(&self, data_id: &str) -> Vec<String> {
        let mut lineage = Vec::new();
        let mut current_data = data_id.to_string();
        let mut visited = std::collections::HashSet::new();

        while !visited.contains(&current_data) {
            visited.insert(current_data.clone());
            lineage.push(current_data.clone());

            // Find parent data
            if let Some(edge) = self.data_flow_graph.edges.iter()
                .find(|e| e.to_data == current_data) {
                current_data = edge.from_data.clone();
            } else {
                break;
            }
        }

        lineage.reverse();
        lineage
    }

    fn log_audit_event(&mut self, event_type: AuditEventType, user_id: &str, execution_id: Option<&str>, details: HashMap<String, String>) {
        let event = AuditEvent {
            event_id: format!("audit_{}", self.current_timestamp()),
            event_type,
            timestamp: self.current_timestamp(),
            user_id: user_id.to_string(),
            execution_id: execution_id.map(|s| s.to_string()),
            details,
            risk_score: 0.5, // Would calculate based on event type and context
        };

        self.audit_trail.push(event);
    }

    fn get_user_for_execution(&self, execution_id: &str) -> Option<String> {
        self.execution_records.get(execution_id).map(|r| r.user_id.clone())
    }

    fn summarize_data_classifications(&self, inputs: &[DataProvenance], outputs: &[DataProvenance]) -> HashMap<String, u32> {
        let mut classifications = HashMap::new();

        for data in inputs.iter().chain(outputs.iter()) {
            let class_name = format!("{:?}", data.classification);
            *classifications.entry(class_name).or_insert(0) += 1;
        }

        classifications
    }

    fn get_security_events_for_execution(&self, execution_id: &str) -> Vec<AuditEvent> {
        self.audit_trail.iter()
            .filter(|event| event.execution_id.as_ref() == Some(&execution_id.to_string()))
            .filter(|event| matches!(event.event_type, AuditEventType::SecurityViolation))
            .cloned()
            .collect()
    }

    fn assess_compliance(&self, execution: &ExecutionRecord) -> ComplianceStatus {
        let blocked_syscalls = execution.system_calls.iter().filter(|s| s.blocked).count();
        let unauthorized_files = execution.file_operations.iter().filter(|f| !f.allowed).count();

        if blocked_syscalls > 0 || unauthorized_files > 0 {
            ComplianceStatus::Violation
        } else {
            ComplianceStatus::Compliant
        }
    }

    fn current_timestamp(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    fn generate_random_id(&self) -> String {
        format!("{:x}", self.current_timestamp() % 0xFFFF)
    }
}

#[derive(Debug, Clone)]
pub struct ProvenanceReport {
    pub execution_summary: ExecutionSummary,
    pub code_origin: Option<CodeLineage>,
    pub data_lineage: DataLineageSummary,
    pub security_events: Vec<AuditEvent>,
    pub compliance_status: ComplianceStatus,
}

#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub execution_id: String,
    pub user_id: String,
    pub duration_ms: u64,
    pub memory_peak_mb: u64,
    pub function_calls: usize,
    pub system_calls: usize,
    pub blocked_operations: usize,
}

#[derive(Debug, Clone)]
pub struct DataLineageSummary {
    pub inputs_count: usize,
    pub outputs_count: usize,
    pub data_classifications: HashMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComplianceStatus {
    Compliant,
    Violation,
    UnderReview,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_tracking() {
        let mut provenance = ExecutionProvenance::new();

        let execution_id = provenance.start_execution("code_hash_123", "user_alice");

        // Record function call
        provenance.record_function_call(&execution_id, FunctionCall {
            function_name: "main".to_string(),
            module_path: "src/main.rs".to_string(),
            entry_timestamp: 1000,
            exit_timestamp: 2000,
            memory_delta: 1024,
            return_value_hash: "return_hash".to_string(),
            exception: None,
        });

        provenance.complete_execution(&execution_id, 1_000_000, 2048);

        let report = provenance.generate_provenance_report(&execution_id).unwrap();
        assert_eq!(report.execution_summary.function_calls, 1);
    }

    #[test]
    fn test_data_lineage() {
        let mut provenance = ExecutionProvenance::new();

        let data_id = "data_123";
        let lineage = provenance.trace_data_lineage(data_id);

        // Empty lineage for non-existent data
        assert_eq!(lineage, vec![data_id]);
    }
}
