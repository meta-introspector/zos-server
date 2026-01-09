// Rust Compiler Integration - Pure Trait-Based Approach
use std::path::Path;
use std::process::Command;

/// Core trait for compiler operations - implementation agnostic
pub trait CompilerInterface: Send + Sync {
    fn compile(&self, source: &str, output: &str) -> Result<(), String>;
    fn check(&self, source: &str) -> Result<bool, String>;
    fn build_project(&self, project_path: &str, features: &[&str]) -> Result<(), String>;
    fn get_version(&self) -> Result<String, String>;
    fn get_target_list(&self) -> Result<Vec<String>, String>;
}

/// Implementation 1: Command execution (always available)
pub struct CommandCompiler;

impl CompilerInterface for CommandCompiler {
    fn compile(&self, source: &str, output: &str) -> Result<(), String> {
        let output = Command::new("rustc")
            .args(&[source, "-o", output])
            .output()
            .map_err(|e| format!("Failed to execute rustc: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    fn check(&self, source: &str) -> Result<bool, String> {
        let output = Command::new("rustc")
            .args(&["--emit=metadata", "--crate-type=lib", source])
            .output()
            .map_err(|e| format!("Failed to execute rustc check: {}", e))?;

        Ok(output.status.success())
    }

    fn build_project(&self, project_path: &str, features: &[&str]) -> Result<(), String> {
        let mut cmd = Command::new("cargo");
        cmd.args(&["build", "--release"]);
        cmd.current_dir(project_path);

        if !features.is_empty() {
            cmd.args(&["--features", &features.join(",")]);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute cargo: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    fn get_version(&self) -> Result<String, String> {
        let output = Command::new("rustc")
            .args(&["--version"])
            .output()
            .map_err(|e| format!("Failed to get rustc version: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn get_target_list(&self) -> Result<Vec<String>, String> {
        let output = Command::new("rustc")
            .args(&["--print", "target-list"])
            .output()
            .map_err(|e| format!("Failed to get target list: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect())
    }
}

/// Implementation 2: Static embedded compiler (when available)
pub struct EmbeddedCompiler {
    // This would contain embedded rustc functionality
    // For now, it's a placeholder that delegates to command
}

impl EmbeddedCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn is_available() -> bool {
        // Check if we have embedded compiler capabilities
        // This could check for specific features or compiled-in support
        false // For now, not available
    }
}

impl CompilerInterface for EmbeddedCompiler {
    fn compile(&self, source: &str, output: &str) -> Result<(), String> {
        // TODO: Use embedded rustc functionality
        // For now, fall back to command execution
        CommandCompiler.compile(source, output)
    }

    fn check(&self, source: &str) -> Result<bool, String> {
        CommandCompiler.check(source)
    }

    fn build_project(&self, project_path: &str, features: &[&str]) -> Result<(), String> {
        CommandCompiler.build_project(project_path, features)
    }

    fn get_version(&self) -> Result<String, String> {
        Ok("embedded-rustc-0.1.0".to_string())
    }

    fn get_target_list(&self) -> Result<Vec<String>, String> {
        Ok(vec![
            "x86_64-unknown-linux-gnu".to_string(),
            "aarch64-unknown-linux-gnu".to_string(),
        ])
    }
}

/// Implementation 3: Dynamic library loading (optional feature)
#[cfg(feature = "dynamic-loading")]
pub struct DynamicCompiler {
    lib_handle: Option<libloading::Library>,
}

#[cfg(feature = "dynamic-loading")]
impl DynamicCompiler {
    pub fn new() -> Self {
        Self { lib_handle: None }
    }

    pub fn load_library(&mut self, lib_path: &str) -> Result<(), String> {
        unsafe {
            let lib = libloading::Library::new(lib_path)
                .map_err(|e| format!("Failed to load library: {}", e))?;
            self.lib_handle = Some(lib);
            Ok(())
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.lib_handle.is_some()
    }
}

#[cfg(feature = "dynamic-loading")]
impl CompilerInterface for DynamicCompiler {
    fn compile(&self, source: &str, output: &str) -> Result<(), String> {
        if let Some(ref _lib) = self.lib_handle {
            // TODO: Use dynamic library functions
            // For now, fall back to command
            CommandCompiler.compile(source, output)
        } else {
            Err("Dynamic library not loaded".to_string())
        }
    }

    fn check(&self, source: &str) -> Result<bool, String> {
        if self.lib_handle.is_some() {
            CommandCompiler.check(source)
        } else {
            Err("Dynamic library not loaded".to_string())
        }
    }

    fn build_project(&self, project_path: &str, features: &[&str]) -> Result<(), String> {
        if self.lib_handle.is_some() {
            CommandCompiler.build_project(project_path, features)
        } else {
            Err("Dynamic library not loaded".to_string())
        }
    }

    fn get_version(&self) -> Result<String, String> {
        Ok("dynamic-rustc".to_string())
    }

    fn get_target_list(&self) -> Result<Vec<String>, String> {
        CommandCompiler.get_target_list()
    }
}

/// Implementation 4: In-process compiler (future)
pub struct InProcessCompiler;

impl CompilerInterface for InProcessCompiler {
    fn compile(&self, _source: &str, _output: &str) -> Result<(), String> {
        Err("In-process compiler not yet implemented".to_string())
    }

    fn check(&self, _source: &str) -> Result<bool, String> {
        Err("In-process compiler not yet implemented".to_string())
    }

    fn build_project(&self, _project_path: &str, _features: &[&str]) -> Result<(), String> {
        Err("In-process compiler not yet implemented".to_string())
    }

    fn get_version(&self) -> Result<String, String> {
        Ok("in-process-rustc-future".to_string())
    }

    fn get_target_list(&self) -> Result<Vec<String>, String> {
        Ok(vec!["future-target".to_string()])
    }
}

/// Compiler factory - chooses best available implementation
pub struct CompilerFactory;

impl CompilerFactory {
    /// Create the best available compiler implementation
    pub fn create_best() -> Box<dyn CompilerInterface> {
        // Try implementations in order of preference

        // 1. Try embedded compiler if available
        if EmbeddedCompiler::is_available() {
            return Box::new(EmbeddedCompiler::new());
        }

        // 2. Try dynamic loading if feature is enabled
        #[cfg(feature = "dynamic-loading")]
        {
            let mut dynamic = DynamicCompiler::new();
            // Try common library paths
            let lib_paths = [
                "/usr/lib/librustc_driver.so",
                "/usr/local/lib/librustc_driver.so",
                "./librustc_driver.so",
            ];

            for path in &lib_paths {
                if Path::new(path).exists() {
                    if dynamic.load_library(path).is_ok() {
                        return Box::new(dynamic);
                    }
                }
            }
        }

        // 3. Fall back to command execution
        Box::new(CommandCompiler)
    }

    /// Create specific implementation
    pub fn create_command() -> Box<dyn CompilerInterface> {
        Box::new(CommandCompiler)
    }

    pub fn create_embedded() -> Box<dyn CompilerInterface> {
        Box::new(EmbeddedCompiler::new())
    }

    #[cfg(feature = "dynamic-loading")]
    pub fn create_dynamic() -> Box<dyn CompilerInterface> {
        Box::new(DynamicCompiler::new())
    }

    pub fn create_in_process() -> Box<dyn CompilerInterface> {
        Box::new(InProcessCompiler)
    }
}

/// High-level self-building system
pub struct SelfBuildingSystem {
    compiler: Box<dyn CompilerInterface>,
}

impl SelfBuildingSystem {
    pub fn new() -> Self {
        Self {
            compiler: CompilerFactory::create_best(),
        }
    }

    pub fn with_compiler(compiler: Box<dyn CompilerInterface>) -> Self {
        Self { compiler }
    }

    pub fn self_compile(&self) -> Result<(), String> {
        println!("🔧 Starting self-compilation...");
        println!("📋 Compiler version: {}", self.compiler.get_version()?);

        // Build the current project with self-build feature
        self.compiler.build_project(".", &["self-build"])?;

        println!("✅ Self-compilation successful!");
        Ok(())
    }

    pub fn compile_source(&self, source_file: &str, output: &str) -> Result<(), String> {
        self.compiler.compile(source_file, output)
    }

    pub fn check_syntax(&self, source_file: &str) -> Result<bool, String> {
        self.compiler.check(source_file)
    }

    pub fn get_compiler_info(&self) -> Result<CompilerInfo, String> {
        Ok(CompilerInfo {
            version: self.compiler.get_version()?,
            targets: self.compiler.get_target_list()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CompilerInfo {
    pub version: String,
    pub targets: Vec<String>,
}

impl Default for SelfBuildingSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_command_compiler() {
        let compiler = CommandCompiler;

        // Test version
        let version = compiler.get_version();
        assert!(version.is_ok());
        println!("Rustc version: {}", version.unwrap());
    }

    #[test]
    fn test_compiler_factory() {
        let compiler = CompilerFactory::create_best();
        let info = SelfBuildingSystem::with_compiler(compiler).get_compiler_info();
        assert!(info.is_ok());
    }

    #[test]
    fn test_self_building_system() {
        let system = SelfBuildingSystem::new();

        // Test getting compiler info
        let info = system.get_compiler_info();
        assert!(info.is_ok());

        let info = info.unwrap();
        assert!(!info.version.is_empty());
        assert!(!info.targets.is_empty());
    }

    #[test]
    fn test_syntax_check() {
        let system = SelfBuildingSystem::new();

        // Create a valid Rust file
        fs::write("test_valid.rs", "fn main() { println!(\"Hello\"); }").unwrap();
        let result = system.check_syntax("test_valid.rs");
        assert!(result.is_ok() && result.unwrap());

        // Create an invalid Rust file
        fs::write("test_invalid.rs", "fn main( { invalid syntax }").unwrap();
        let result = system.check_syntax("test_invalid.rs");
        assert!(result.is_ok() && !result.unwrap());

        // Cleanup
        fs::remove_file("test_valid.rs").ok();
        fs::remove_file("test_invalid.rs").ok();
    }
}
