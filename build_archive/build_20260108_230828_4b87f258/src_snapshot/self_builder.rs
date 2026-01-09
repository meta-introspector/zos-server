// Self-Building System with LLM Error Fixing
use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileError {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub error_code: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMRequest {
    pub error: CompileError,
    pub context: String,
    pub file_content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMResponse {
    pub fixed_code: String,
    pub explanation: String,
    pub confidence: f64,
}

pub struct SelfBuilder {
    project_root: String,
    max_iterations: u32,
    llm_endpoint: String,
}

impl SelfBuilder {
    pub fn new(project_root: String) -> Self {
        Self {
            project_root,
            max_iterations: 10,
            llm_endpoint: "http://localhost:11434/api/generate".to_string(), // Ollama default
        }
    }

    pub async fn self_build(&mut self) -> Result<bool, String> {
        println!("🔧 Starting self-build process...");

        for iteration in 1..=self.max_iterations {
            println!("🔄 Build iteration {}/{}", iteration, self.max_iterations);

            match self.attempt_build().await {
                Ok(true) => {
                    println!("✅ Build successful after {} iterations!", iteration);
                    return Ok(true);
                }
                Ok(false) => {
                    println!("❌ Build failed, analyzing errors...");
                    if !self.fix_errors_with_llm().await? {
                        println!("⚠️ Could not fix errors automatically");
                        break;
                    }
                }
                Err(e) => {
                    println!("💥 Build system error: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(false)
    }

    async fn attempt_build(&self) -> Result<bool, String> {
        let output = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(&self.project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run cargo build: {}", e))?;

        if output.status.success() {
            Ok(true)
        } else {
            // Save error log for analysis
            let stderr = String::from_utf8_lossy(&output.stderr);
            fs::write(format!("{}/build_errors.log", self.project_root), stderr.as_bytes())
                .map_err(|e| format!("Failed to write error log: {}", e))?;
            Ok(false)
        }
    }

    async fn fix_errors_with_llm(&self) -> Result<bool, String> {
        let errors = self.parse_compile_errors()?;
        println!("🔍 Found {} compile errors", errors.len());

        let mut fixed_any = false;

        for error in errors.iter().take(3) { // Fix top 3 errors per iteration
            println!("🤖 Asking LLM to fix: {}", error.message);

            match self.fix_single_error(error).await {
                Ok(true) => {
                    println!("✅ Fixed error in {}", error.file);
                    fixed_any = true;
                }
                Ok(false) => {
                    println!("⚠️ LLM couldn't fix error in {}", error.file);
                }
                Err(e) => {
                    println!("❌ Error fixing {}: {}", error.file, e);
                }
            }
        }

        Ok(fixed_any)
    }

    fn parse_compile_errors(&self) -> Result<Vec<CompileError>, String> {
        let log_path = format!("{}/build_errors.log", self.project_root);
        let content = fs::read_to_string(&log_path)
            .map_err(|e| format!("Failed to read error log: {}", e))?;

        let mut errors = Vec::new();

        for line in content.lines() {
            if line.contains("error[E") {
                if let Some(error) = self.parse_error_line(line) {
                    errors.push(error);
                }
            }
        }

        Ok(errors)
    }

    fn parse_error_line(&self, line: &str) -> Option<CompileError> {
        // Parse: "error[E0277]: the trait bound `SdfBehaviour: NetworkBehaviour` is not satisfied"
        // and: "  --> src/mini_sdf_server.rs:12:10"

        if let Some(start) = line.find("error[") {
            if let Some(end) = line.find("]:") {
                let error_code = line[start+6..end].to_string();
                let message = line[end+2..].trim().to_string();

                return Some(CompileError {
                    file: "unknown".to_string(), // Will be updated from next line
                    line: 0,
                    column: 0,
                    error_code,
                    message,
                    suggestion: None,
                });
            }
        }

        None
    }

    async fn fix_single_error(&self, error: &CompileError) -> Result<bool, String> {
        let file_path = format!("{}/{}", self.project_root, error.file);
        let file_content = match fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(_) => return Ok(false), // Skip if file doesn't exist
        };

        let request = LLMRequest {
            error: error.clone(),
            context: self.get_error_context(&file_content, error.line as usize),
            file_content: file_content.clone(),
        };

        let response = self.query_llm(&request).await?;

        if response.confidence > 0.7 {
            fs::write(&file_path, response.fixed_code)
                .map_err(|e| format!("Failed to write fixed file: {}", e))?;

            println!("📝 Applied fix: {}", response.explanation);
            Ok(true)
        } else {
            println!("🤔 LLM confidence too low: {:.2}", response.confidence);
            Ok(false)
        }
    }

    fn get_error_context(&self, content: &str, error_line: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let start = error_line.saturating_sub(5);
        let end = (error_line + 5).min(lines.len());

        lines[start..end].join("\n")
    }

    async fn query_llm(&self, request: &LLMRequest) -> Result<LLMResponse, String> {
        let prompt = format!(
            "Fix this Rust compile error:\n\nError: {}\nCode: {}\n\nFile: {}\n\nProvide only the corrected code and a brief explanation.",
            request.error.message,
            request.error.error_code,
            request.context
        );

        // Simple HTTP request to LLM (Ollama format)
        let client = reqwest::Client::new();
        let llm_request = serde_json::json!({
            "model": "codellama",
            "prompt": prompt,
            "stream": false
        });

        let response = client
            .post(&self.llm_endpoint)
            .json(&llm_request)
            .send()
            .await
            .map_err(|e| format!("LLM request failed: {}", e))?;

        let response_text = response.text().await
            .map_err(|e| format!("Failed to read LLM response: {}", e))?;

        // Parse LLM response (simplified)
        Ok(LLMResponse {
            fixed_code: request.file_content.clone(), // Placeholder - would parse actual fix
            explanation: "LLM suggested fix".to_string(),
            confidence: 0.8,
        })
    }

    pub fn set_llm_endpoint(&mut self, endpoint: String) {
        self.llm_endpoint = endpoint;
    }

    pub fn set_max_iterations(&mut self, max: u32) {
        self.max_iterations = max;
    }
}
