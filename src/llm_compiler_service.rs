use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::time::{timeout, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMRequest {
    pub prompt: String,
    pub code: Option<String>,
    pub error: Option<String>,
    pub iteration: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMResponse {
    pub code: String,
    pub explanation: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompilerResult {
    pub success: bool,
    pub output: String,
    pub errors: Vec<String>,
}

pub struct LLMCompilerService {
    max_iterations: u32,
    timeout_seconds: u64,
}

impl LLMCompilerService {
    pub fn new() -> Self {
        Self {
            max_iterations: 5,
            timeout_seconds: 30,
        }
    }

    pub async fn improve_code(
        &self,
        initial_prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut current_code = String::new();
        let mut current_error = None;

        for iteration in 1..=self.max_iterations {
            println!("🤖 Iteration {}/{}", iteration, self.max_iterations);

            // Call LLM
            let llm_response = self
                .call_llm(&LLMRequest {
                    prompt: initial_prompt.to_string(),
                    code: if current_code.is_empty() {
                        None
                    } else {
                        Some(current_code.clone())
                    },
                    error: current_error.clone(),
                    iteration,
                })
                .await?;

            current_code = llm_response.code;
            println!(
                "📝 Generated code (confidence: {:.1}%)",
                llm_response.confidence * 100.0
            );

            // Compile code
            let compile_result = self.compile_code(&current_code).await?;

            if compile_result.success {
                println!("✅ Compilation successful!");
                return Ok(current_code);
            }

            println!("❌ Compilation failed: {}", compile_result.output);
            current_error = Some(compile_result.output);

            if iteration == self.max_iterations {
                return Err(
                    format!("Failed to compile after {} iterations", self.max_iterations).into(),
                );
            }
        }

        Err("Unexpected end of iterations".into())
    }

    async fn call_llm(
        &self,
        request: &LLMRequest,
    ) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        let prompt = self.build_prompt(request);

        // Mock LLM call - replace with actual API call
        let response = timeout(
            Duration::from_secs(self.timeout_seconds),
            self.mock_llm_call(&prompt),
        )
        .await??;

        Ok(response)
    }

    fn build_prompt(&self, request: &LLMRequest) -> String {
        let mut prompt = format!("Iteration {}: {}\n\n", request.iteration, request.prompt);

        if let Some(code) = &request.code {
            prompt.push_str(&format!("Previous code:\n```rust\n{}\n```\n\n", code));
        }

        if let Some(error) = &request.error {
            prompt.push_str(&format!("Compiler error:\n```\n{}\n```\n\n", error));
            prompt.push_str("Please fix the error and provide improved code.\n");
        } else {
            prompt.push_str("Please provide Rust code that compiles successfully.\n");
        }

        prompt.push_str("Respond with valid Rust code only.");
        prompt
    }

    async fn mock_llm_call(&self, prompt: &str) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        // Mock implementation - replace with actual LLM API
        tokio::time::sleep(Duration::from_millis(500)).await;

        let code = if prompt.contains("error") {
            // Generate improved code based on error
            r#"fn main() {
    println!("Hello, ZOS Server!");
    let result = add_numbers(5, 3);
    println!("Result: {}", result);
}

fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}"#
            .to_string()
        } else {
            // Generate initial code
            r#"fn main() {
    println!("Hello, World!");
}"#
            .to_string()
        };

        Ok(LLMResponse {
            code,
            explanation: "Generated Rust code with basic functionality".to_string(),
            confidence: 0.85,
        })
    }

    async fn compile_code(&self, code: &str) -> Result<CompilerResult, Box<dyn std::error::Error>> {
        // Write code to temporary file
        let temp_file = format!("/tmp/zos_temp_{}.rs", std::process::id());
        tokio::fs::write(&temp_file, code).await?;

        // Compile with rustc
        let output = timeout(
            Duration::from_secs(self.timeout_seconds),
            tokio::task::spawn_blocking({
                let temp_file = temp_file.clone();
                move || {
                    Command::new("rustc")
                        .arg(&temp_file)
                        .arg("-o")
                        .arg(format!("/tmp/zos_compiled_{}", std::process::id()))
                        .output()
                }
            }),
        )
        .await???;

        // Clean up
        let _ = tokio::fs::remove_file(&temp_file).await;

        let success = output.status.success();
        let output_str = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CompilerResult {
            success,
            output: output_str.clone(),
            errors: if success { vec![] } else { vec![output_str] },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_compiler_service() {
        let service = LLMCompilerService::new();
        let result = service
            .improve_code("Create a simple Rust program that prints hello world")
            .await;
        assert!(result.is_ok());
    }
}
