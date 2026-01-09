use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskMode {
    Interactive, // CLI step-by-step with user input
    Callback,    // Async callback-driven execution
    GitHub,      // GitHub issues/actions integration
    Batch,       // Fire-and-forget batch processing
    Stream,      // Real-time streaming updates
    WebHook,     // HTTP webhook notifications
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub task_id: Uuid,
    pub mode: TaskMode,
    pub config: TaskConfig,
    pub state: ExecutionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub auto_continue: bool,
    pub max_iterations: u32,
    pub timeout_seconds: u64,
    pub callback_url: Option<String>,
    pub github_repo: Option<String>,
    pub github_issue: Option<u32>,
    pub webhook_url: Option<String>,
    pub interactive_prompts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionState {
    Pending,
    Running { current_iteration: u32 },
    Paused { reason: String },
    WaitingInput { prompt: String },
    Completed { success: bool },
    Failed { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub iteration: u32,
    pub code: String,
    pub compile_success: bool,
    pub errors: Vec<String>,
    pub next_action: NextAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NextAction {
    Continue,
    Pause,
    RequestInput { prompt: String },
    Complete,
    Fail { reason: String },
}

pub trait TaskExecutor {
    async fn execute_step(&mut self, input: Option<String>) -> Result<StepResult, String>;
    async fn pause(&mut self) -> Result<(), String>;
    async fn resume(&mut self) -> Result<(), String>;
    async fn abort(&mut self) -> Result<(), String>;
}

pub struct MultiModalTaskRunner {
    executions: HashMap<Uuid, TaskExecution>,
    callbacks: HashMap<Uuid, mpsc::UnboundedSender<StepResult>>,
}

impl MultiModalTaskRunner {
    pub fn new() -> Self {
        Self {
            executions: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }

    pub async fn start_task(&mut self, prompt: &str, mode: TaskMode, config: TaskConfig) -> Uuid {
        let task_id = Uuid::new_v4();

        let execution = TaskExecution {
            task_id,
            mode: mode.clone(),
            config: config.clone(),
            state: ExecutionState::Pending,
        };

        self.executions.insert(task_id, execution);

        match mode {
            TaskMode::Interactive => {
                println!("🎮 Interactive mode started for task {}", task_id);
                println!("Use: ./zos_server task step {} [input]", task_id);
            }
            TaskMode::Callback => {
                let (tx, mut rx) = mpsc::unbounded_channel();
                self.callbacks.insert(task_id, tx);

                // Spawn callback handler
                let task_id_clone = task_id;
                tokio::spawn(async move {
                    while let Some(result) = rx.recv().await {
                        println!(
                            "📞 Callback for {}: {:?}",
                            task_id_clone, result.next_action
                        );
                    }
                });
            }
            TaskMode::GitHub => {
                if let Some(repo) = &config.github_repo {
                    println!("🐙 GitHub mode for repo: {}", repo);
                    if let Some(issue) = config.github_issue {
                        println!("📋 Linked to issue #{}", issue);
                    }
                }
            }
            TaskMode::Batch => {
                println!("🔄 Batch mode - auto-executing...");
                self.auto_execute_task(task_id).await;
            }
            TaskMode::Stream => {
                println!("📡 Stream mode - real-time updates enabled");
            }
            TaskMode::WebHook => {
                if let Some(url) = &config.webhook_url {
                    println!("🪝 WebHook mode - will notify: {}", url);
                }
            }
        }

        task_id
    }

    pub async fn execute_step(
        &mut self,
        task_id: Uuid,
        input: Option<String>,
    ) -> Result<StepResult, String> {
        let execution = self.executions.get_mut(&task_id).ok_or("Task not found")?;

        match &execution.mode {
            TaskMode::Interactive => self.handle_interactive_step(task_id, input).await,
            TaskMode::Callback => {
                let result = self.handle_callback_step(task_id, input).await?;
                if let Some(tx) = self.callbacks.get(&task_id) {
                    let _ = tx.send(result.clone());
                }
                Ok(result)
            }
            TaskMode::GitHub => self.handle_github_step(task_id, input).await,
            TaskMode::Stream => {
                let result = self.handle_stream_step(task_id, input).await?;
                self.broadcast_stream_update(task_id, &result).await;
                Ok(result)
            }
            _ => self.handle_generic_step(task_id, input).await,
        }
    }

    async fn handle_interactive_step(
        &mut self,
        task_id: Uuid,
        input: Option<String>,
    ) -> Result<StepResult, String> {
        println!("🎮 Interactive Step for task {}", task_id);

        if let Some(ref user_input) = input {
            println!("📝 User input: {}", user_input);
        }

        // Mock step execution
        let result = StepResult {
            iteration: 1,
            code: "fn main() { println!(\"Interactive step\"); }".to_string(),
            compile_success: true,
            errors: vec![],
            next_action: if input.is_some() {
                NextAction::Continue
            } else {
                NextAction::RequestInput {
                    prompt: "Enter next instruction or 'continue' to proceed:".to_string(),
                }
            },
        };

        match &result.next_action {
            NextAction::RequestInput { prompt } => {
                println!("❓ {}", prompt);
                self.update_state(
                    task_id,
                    ExecutionState::WaitingInput {
                        prompt: prompt.clone(),
                    },
                );
            }
            NextAction::Continue => {
                println!("▶️ Continuing to next step...");
            }
            _ => {}
        }

        Ok(result)
    }

    async fn handle_callback_step(
        &mut self,
        task_id: Uuid,
        _input: Option<String>,
    ) -> Result<StepResult, String> {
        println!("📞 Callback step for task {}", task_id);

        Ok(StepResult {
            iteration: 1,
            code: "fn main() { println!(\"Callback step\"); }".to_string(),
            compile_success: true,
            errors: vec![],
            next_action: NextAction::Continue,
        })
    }

    async fn handle_github_step(
        &mut self,
        task_id: Uuid,
        _input: Option<String>,
    ) -> Result<StepResult, String> {
        let execution = self.executions.get(&task_id).unwrap();

        if let Some(repo) = &execution.config.github_repo {
            println!("🐙 GitHub step for repo: {}", repo);

            if let Some(issue) = execution.config.github_issue {
                println!("📋 Processing issue #{}", issue);
                // Here you would integrate with GitHub API
            }
        }

        Ok(StepResult {
            iteration: 1,
            code: "// GitHub integration step\nfn main() { println!(\"GitHub step\"); }"
                .to_string(),
            compile_success: true,
            errors: vec![],
            next_action: NextAction::Continue,
        })
    }

    async fn handle_stream_step(
        &mut self,
        task_id: Uuid,
        _input: Option<String>,
    ) -> Result<StepResult, String> {
        println!("📡 Stream step for task {}", task_id);

        Ok(StepResult {
            iteration: 1,
            code: "fn main() { println!(\"Stream step\"); }".to_string(),
            compile_success: true,
            errors: vec![],
            next_action: NextAction::Continue,
        })
    }

    async fn handle_generic_step(
        &mut self,
        task_id: Uuid,
        _input: Option<String>,
    ) -> Result<StepResult, String> {
        println!("🔄 Generic step for task {}", task_id);

        Ok(StepResult {
            iteration: 1,
            code: "fn main() { println!(\"Generic step\"); }".to_string(),
            compile_success: true,
            errors: vec![],
            next_action: NextAction::Complete,
        })
    }

    async fn broadcast_stream_update(&self, task_id: Uuid, result: &StepResult) {
        println!(
            "📡 Broadcasting update for task {}: iteration {}",
            task_id, result.iteration
        );
        // Here you would send to WebSocket connections, SSE, etc.
    }

    async fn auto_execute_task(&mut self, task_id: Uuid) {
        println!("🔄 Auto-executing batch task {}", task_id);

        loop {
            match self.execute_step(task_id, None).await {
                Ok(result) => match result.next_action {
                    NextAction::Continue => continue,
                    NextAction::Complete => {
                        println!("✅ Batch task {} completed", task_id);
                        break;
                    }
                    NextAction::Fail { reason } => {
                        println!("❌ Batch task {} failed: {}", task_id, reason);
                        break;
                    }
                    _ => break,
                },
                Err(e) => {
                    println!("❌ Batch task {} error: {}", task_id, e);
                    break;
                }
            }
        }
    }

    fn update_state(&mut self, task_id: Uuid, state: ExecutionState) {
        if let Some(execution) = self.executions.get_mut(&task_id) {
            execution.state = state;
        }
    }

    pub fn get_task_state(&self, task_id: Uuid) -> Option<&ExecutionState> {
        self.executions.get(&task_id).map(|e| &e.state)
    }

    pub fn list_active_tasks(&self) -> Vec<(Uuid, &TaskMode, &ExecutionState)> {
        self.executions
            .iter()
            .map(|(id, exec)| (*id, &exec.mode, &exec.state))
            .collect()
    }
}
