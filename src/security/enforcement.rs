// Security Enforcement Module
pub mod instruction_filter;
pub mod bytecode_manipulator;
pub mod syscall_stripper_macros;
pub mod container_runtime;
pub mod server_lattice;

pub use instruction_filter::{InstructionFilter, CodeAnalysis};
pub use bytecode_manipulator::{BytecodeManipulator, MonsterDriverIntegration};
pub use container_runtime::{ContainerRuntime, SecureContainer};
pub use server_lattice::{ServerLattice, ServerLayer};

/// Main security enforcement interface
pub struct SecurityEnforcement {
    instruction_filter: InstructionFilter,
    bytecode_manipulator: BytecodeManipulator,
    container_runtime: ContainerRuntime,
    server_lattice: ServerLattice,
}

impl SecurityEnforcement {
    pub fn new(current_layer: ServerLayer) -> Self {
        Self {
            instruction_filter: InstructionFilter::new(),
            bytecode_manipulator: BytecodeManipulator::new(),
            container_runtime: ContainerRuntime::new(),
            server_lattice: ServerLattice::new(current_layer),
        }
    }

    pub fn filter_code(&mut self, code: &str, user_role: &str) -> Result<CodeAnalysis, String> {
        self.instruction_filter.filter_code(code, user_role)
    }

    pub fn create_secure_container(&mut self, user_id: &str) -> Result<String, String> {
        Ok(self.container_runtime.create_llm_container(user_id))
    }

    pub fn execute_in_layer(&self, feature: &str, user_clearance: &str) -> Result<String, String> {
        let result = self.server_lattice.execute_feature(feature, user_clearance)?;
        Ok(result.result)
    }
}
