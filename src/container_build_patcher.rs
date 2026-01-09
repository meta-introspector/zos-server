// Build.rs integration for automatic container security
use crate::mkbuildrs_patcher::SecurityPatcher;
use crate::container_runtime::SyscallStripper;

/// Enhanced mkbuildrs! macro with container support
#[macro_export]
macro_rules! mkbuildrs_container {
    () => {
        fn main() {
            // Apply security patching
            zos_security_patcher::patch_cargo_project();

            // Strip syscalls and generate container interface
            let stripper = zos_container::SyscallStripper::new();
            let secure_interface = stripper.generate_secure_interface();

            // Write secure interface
            std::fs::create_dir_all("src/generated").unwrap();
            std::fs::write("src/generated/secure_git.rs", secure_interface).unwrap();

            println!("cargo:warning=Generated secure container interface");
        }
    };
}

/// Container-aware security patcher
pub struct ContainerSecurityPatcher {
    base_patcher: SecurityPatcher,
    stripper: SyscallStripper,
}

impl ContainerSecurityPatcher {
    pub fn new() -> Self {
        Self {
            base_patcher: SecurityPatcher::new(),
            stripper: SyscallStripper::new(),
        }
    }

    /// Patch project with container security
    pub fn patch_with_containers(&self) {
        // Apply base security patching
        self.base_patcher.patch_cargo_project();

        // Generate container-specific patches
        self.generate_container_patches();

        // Strip syscalls from git operations
        self.patch_git_operations();
    }

    fn generate_container_patches(&self) {
        let container_mod = r#"
// Auto-generated container security module
pub mod container {
    use crate::container_runtime::{ContainerRuntime, SecurityLevel};

    /// Initialize LLM container with git access
    pub fn init_llm_git_container(repo_path: &str) -> Result<String, String> {
        let mut runtime = ContainerRuntime::new();
        let container_id = runtime.create_llm_container("llm_git");
        runtime.load_git_repo(&container_id, repo_path)?;
        Ok(container_id)
    }

    /// Secure git operations for LLM
    pub mod git {
        use super::*;

        pub fn log(container_id: &str) -> Result<String, String> {
            let runtime = ContainerRuntime::new();
            runtime.git_command(container_id, "log", &[])
        }

        pub fn show(container_id: &str, commit: &str) -> Result<String, String> {
            let runtime = ContainerRuntime::new();
            runtime.git_command(container_id, "show", &[commit])
        }

        pub fn diff(container_id: &str) -> Result<String, String> {
            let runtime = ContainerRuntime::new();
            runtime.git_command(container_id, "diff", &[])
        }
    }
}
"#;

        std::fs::create_dir_all("src/security").unwrap();
        std::fs::write("src/security/container.rs", container_mod).unwrap();
    }

    fn patch_git_operations(&self) {
        // Find and patch git2 crate usage
        let git_patches = vec![
            ("git2::Repository::open", "crate::security::container::git::open"),
            ("git2::Repository::clone", "crate::security::container::git::clone"),
            ("repository.revwalk()", "crate::security::container::git::log"),
        ];

        // Apply patches to source files
        if let Ok(entries) = std::fs::read_dir("src") {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        self.patch_git_file(&entry.path(), &git_patches);
                    }
                }
            }
        }
    }

    fn patch_git_file(&self, path: &std::path::Path, patches: &[(&str, &str)]) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let mut patched = content;
            let mut needs_patch = false;

            for (original, replacement) in patches {
                if patched.contains(original) {
                    patched = patched.replace(original, replacement);
                    needs_patch = true;
                }
            }

            if needs_patch {
                std::fs::write(path, patched).unwrap();
                println!("cargo:warning=Patched git operations in {:?}", path);
            }
        }
    }
}

/// Convenience function for container-aware patching
pub fn patch_cargo_project_with_containers() {
    let patcher = ContainerSecurityPatcher::new();
    patcher.patch_with_containers();
}
