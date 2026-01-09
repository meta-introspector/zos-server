use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: Uuid,
    pub prompt: String,
    pub final_code: String,
    pub iterations: Vec<TaskIteration>,
    pub metadata: TaskMetadata,
    pub ratings: Vec<TaskRating>,
    pub verification: Option<TaskVerification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskIteration {
    pub iteration: u32,
    pub code: String,
    pub compile_success: bool,
    pub errors: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub created_at: DateTime<Utc>,
    pub creator: String,
    pub tags: Vec<String>,
    pub difficulty: TaskDifficulty,
    pub language: String,
    pub total_iterations: u32,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRating {
    pub rater_id: String,
    pub score: u8, // 1-5 stars
    pub comment: Option<String>,
    pub aspects: RatingAspects,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingAspects {
    pub code_quality: u8,
    pub prompt_clarity: u8,
    pub learning_value: u8,
    pub innovation: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskVerification {
    pub verifier_id: String,
    pub verified: bool,
    pub tests_passed: u8,
    pub total_tests: u8,
    pub security_check: bool,
    pub performance_score: Option<f32>,
    pub timestamp: DateTime<Utc>,
}

pub struct TaskRegistry {
    tasks: HashMap<Uuid, TaskItem>,
    user_ratings: HashMap<String, Vec<Uuid>>, // user_id -> task_ids they rated
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            user_ratings: HashMap::new(),
        }
    }

    pub fn export_task(
        &mut self,
        prompt: &str,
        final_code: &str,
        iterations: Vec<TaskIteration>,
        creator: &str,
        tags: Vec<String>,
    ) -> Uuid {
        let task_id = Uuid::new_v4();

        let task = TaskItem {
            id: task_id,
            prompt: prompt.to_string(),
            final_code: final_code.to_string(),
            iterations: iterations.clone(),
            metadata: TaskMetadata {
                created_at: Utc::now(),
                creator: creator.to_string(),
                tags,
                difficulty: self.assess_difficulty(&iterations),
                language: "rust".to_string(),
                total_iterations: iterations.len() as u32,
                success: iterations
                    .last()
                    .map(|i| i.compile_success)
                    .unwrap_or(false),
            },
            ratings: Vec::new(),
            verification: None,
        };

        self.tasks.insert(task_id, task);
        println!("📤 Task exported with ID: {}", task_id);
        task_id
    }

    pub fn rate_task(
        &mut self,
        task_id: Uuid,
        rater_id: &str,
        score: u8,
        aspects: RatingAspects,
        comment: Option<String>,
    ) -> Result<(), String> {
        if score < 1 || score > 5 {
            return Err("Score must be between 1 and 5".to_string());
        }

        // Check if user already rated this task
        if let Some(user_tasks) = self.user_ratings.get(rater_id) {
            if user_tasks.contains(&task_id) {
                return Err("User has already rated this task".to_string());
            }
        }

        let task = self.tasks.get_mut(&task_id).ok_or("Task not found")?;

        let rating = TaskRating {
            rater_id: rater_id.to_string(),
            score,
            comment,
            aspects,
            timestamp: Utc::now(),
        };

        task.ratings.push(rating);

        // Track user ratings
        self.user_ratings
            .entry(rater_id.to_string())
            .or_insert_with(Vec::new)
            .push(task_id);

        println!("⭐ Task {} rated {} stars by {}", task_id, score, rater_id);
        Ok(())
    }

    pub fn verify_task(
        &mut self,
        task_id: Uuid,
        verifier_id: &str,
        tests_passed: u8,
        total_tests: u8,
        security_check: bool,
        performance_score: Option<f32>,
    ) -> Result<(), String> {
        let task = self.tasks.get_mut(&task_id).ok_or("Task not found")?;

        let verification = TaskVerification {
            verifier_id: verifier_id.to_string(),
            verified: tests_passed == total_tests && security_check,
            tests_passed,
            total_tests,
            security_check,
            performance_score,
            timestamp: Utc::now(),
        };

        task.verification = Some(verification);
        println!("✅ Task {} verified by {}", task_id, verifier_id);
        Ok(())
    }

    pub fn get_task(&self, task_id: Uuid) -> Option<&TaskItem> {
        self.tasks.get(&task_id)
    }

    pub fn list_tasks(&self) -> Vec<&TaskItem> {
        self.tasks.values().collect()
    }

    pub fn get_top_rated(&self, limit: usize) -> Vec<&TaskItem> {
        let mut tasks: Vec<&TaskItem> = self.tasks.values().collect();
        tasks.sort_by(|a, b| {
            let avg_a = self.calculate_average_rating(a);
            let avg_b = self.calculate_average_rating(b);
            avg_b
                .partial_cmp(&avg_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        tasks.into_iter().take(limit).collect()
    }

    pub fn search_by_tags(&self, tags: &[String]) -> Vec<&TaskItem> {
        self.tasks
            .values()
            .filter(|task| tags.iter().any(|tag| task.metadata.tags.contains(tag)))
            .collect()
    }

    fn assess_difficulty(&self, iterations: &[TaskIteration]) -> TaskDifficulty {
        match iterations.len() {
            1 => TaskDifficulty::Beginner,
            2..=3 => TaskDifficulty::Intermediate,
            4..=5 => TaskDifficulty::Advanced,
            _ => TaskDifficulty::Expert,
        }
    }

    fn calculate_average_rating(&self, task: &TaskItem) -> f32 {
        if task.ratings.is_empty() {
            return 0.0;
        }
        let sum: u32 = task.ratings.iter().map(|r| r.score as u32).sum();
        sum as f32 / task.ratings.len() as f32
    }

    pub fn export_to_json(&self, task_id: Uuid) -> Result<String, String> {
        let task = self.get_task(task_id).ok_or("Task not found")?;
        serde_json::to_string_pretty(task).map_err(|e| format!("Serialization error: {}", e))
    }

    pub fn import_from_json(&mut self, json: &str) -> Result<Uuid, String> {
        let task: TaskItem =
            serde_json::from_str(json).map_err(|e| format!("Deserialization error: {}", e))?;

        let task_id = task.id;
        self.tasks.insert(task_id, task);
        Ok(task_id)
    }
}
