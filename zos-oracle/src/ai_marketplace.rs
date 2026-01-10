use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPromptMarketplace {
    pub published_prompts: HashMap<String, PublishedPrompt>,
    pub llm_endpoints: HashMap<String, LLMEndpoint>,
    pub usage_quotas: HashMap<String, UsageQuota>,
    pub request_history: HashMap<String, Vec<AIRequest>>,
    pub revenue_sharing: HashMap<String, RevenueShare>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedPrompt {
    pub prompt_id: String,
    pub publisher_id: String,
    pub title: String,
    pub description: String,
    pub category: PromptCategory,
    pub prompt_template: String,
    pub input_schema: serde_json::Value,
    pub output_format: String,
    pub price_per_request: u64,
    pub model_requirements: ModelRequirements,
    pub usage_count: u64,
    pub rating: f32,
    pub reviews: Vec<PromptReview>,
    pub published_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMEndpoint {
    pub endpoint_id: String,
    pub owner_id: String,
    pub port: u16,
    pub model_name: String,
    pub model_type: ModelType,
    pub max_tokens: u32,
    pub requests_per_block: u32,
    pub cost_per_token: f64,
    pub available_quota: u64,
    pub total_requests: u64,
    pub uptime_percentage: f32,
    pub endpoint_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageQuota {
    pub user_id: String,
    pub tier: String,
    pub daily_requests: u32,
    pub requests_used_today: u32,
    pub monthly_tokens: u64,
    pub tokens_used_month: u64,
    pub premium_credits: u64,
    pub reset_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub request_id: String,
    pub user_id: String,
    pub prompt_id: Option<String>,
    pub endpoint_id: String,
    pub input_data: serde_json::Value,
    pub output_data: Option<serde_json::Value>,
    pub tokens_used: u32,
    pub cost_credits: u64,
    pub processing_time_ms: u64,
    pub timestamp: u64,
    pub status: RequestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueShare {
    pub prompt_id: String,
    pub publisher_earnings: u64,
    pub platform_fee: u64,
    pub endpoint_provider_share: u64,
    pub total_revenue: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptCategory {
    CodeGeneration,
    ContentWriting,
    DataAnalysis,
    ImageGeneration,
    Translation,
    Summarization,
    QA,
    Creative,
    Business,
    Educational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    TextGeneration,
    CodeGeneration,
    ImageGeneration,
    Embedding,
    Classification,
    Translation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    pub min_context_length: u32,
    pub required_capabilities: Vec<String>,
    pub preferred_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    RateLimited,
    InsufficientCredits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptReview {
    pub reviewer_id: String,
    pub rating: u8,
    pub comment: String,
    pub timestamp: u64,
}

impl AIPromptMarketplace {
    pub fn new() -> Self {
        Self {
            published_prompts: HashMap::new(),
            llm_endpoints: HashMap::new(),
            usage_quotas: HashMap::new(),
            request_history: HashMap::new(),
            revenue_sharing: HashMap::new(),
        }
    }

    pub fn publish_prompt(
        &mut self,
        publisher_id: &str,
        title: &str,
        description: &str,
        prompt_template: &str,
        category: PromptCategory,
        price_per_request: u64,
    ) -> Result<String, String> {
        let prompt_id = format!("prompt_{}_{}", publisher_id, chrono::Utc::now().timestamp());

        let prompt = PublishedPrompt {
            prompt_id: prompt_id.clone(),
            publisher_id: publisher_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            category,
            prompt_template: prompt_template.to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string", "description": "User input"}
                }
            }),
            output_format: "text".to_string(),
            price_per_request,
            model_requirements: ModelRequirements {
                min_context_length: 2048,
                required_capabilities: vec!["text-generation".to_string()],
                preferred_models: vec!["gpt-4".to_string(), "claude-3".to_string()],
            },
            usage_count: 0,
            rating: 0.0,
            reviews: Vec::new(),
            published_at: chrono::Utc::now().timestamp() as u64,
        };

        self.published_prompts.insert(prompt_id.clone(), prompt);

        println!("📝 Prompt published: {} by {}", title, &publisher_id[..8]);
        Ok(prompt_id)
    }

    pub fn register_llm_endpoint(
        &mut self,
        owner_id: &str,
        port: u16,
        model_name: &str,
        model_type: ModelType,
        max_tokens: u32,
    ) -> Result<String, String> {
        let endpoint_id = format!("endpoint_{}_{}", owner_id, port);

        let endpoint = LLMEndpoint {
            endpoint_id: endpoint_id.clone(),
            owner_id: owner_id.to_string(),
            port,
            model_name: model_name.to_string(),
            model_type,
            max_tokens,
            requests_per_block: 10, // Based on user tier
            cost_per_token: 0.001,
            available_quota: 1000000, // 1M tokens
            total_requests: 0,
            uptime_percentage: 100.0,
            endpoint_url: format!("http://localhost:{}/ai", port),
        };

        self.llm_endpoints.insert(endpoint_id.clone(), endpoint);

        println!(
            "🤖 LLM endpoint registered: {} on port {}",
            model_name, port
        );
        Ok(endpoint_id)
    }

    pub fn execute_prompt(
        &mut self,
        user_id: &str,
        prompt_id: &str,
        input_data: serde_json::Value,
    ) -> Result<AIRequest, String> {
        // Check user quota
        let quota = self
            .usage_quotas
            .get_mut(user_id)
            .ok_or("User quota not found")?;

        if quota.requests_used_today >= quota.daily_requests {
            return Err("Daily request limit exceeded".to_string());
        }

        // Get prompt
        let prompt = self
            .published_prompts
            .get_mut(prompt_id)
            .ok_or("Prompt not found")?;

        // Find available endpoint
        let endpoint_id = self.find_best_endpoint(&prompt.model_requirements)?;
        let endpoint = self
            .llm_endpoints
            .get_mut(&endpoint_id)
            .ok_or("No available endpoints")?;

        // Check credits
        if quota.premium_credits < prompt.price_per_request {
            return Err("Insufficient credits".to_string());
        }

        // Create request
        let request_id = format!("req_{}_{}", user_id, chrono::Utc::now().timestamp());

        // Execute prompt (simulate)
        let processed_prompt =
            self.process_prompt_template(&prompt.prompt_template, &input_data)?;
        let output = self.call_llm_endpoint(&endpoint.endpoint_url, &processed_prompt)?;

        let tokens_used = (processed_prompt.len() + output.len()) / 4; // Rough estimate
        let cost = prompt.price_per_request;

        // Update usage
        quota.requests_used_today += 1;
        quota.tokens_used_month += tokens_used as u64;
        quota.premium_credits -= cost;

        prompt.usage_count += 1;
        endpoint.total_requests += 1;
        endpoint.available_quota -= tokens_used as u64;

        // Create revenue share
        let revenue = RevenueShare {
            prompt_id: prompt_id.to_string(),
            publisher_earnings: (cost as f64 * 0.7) as u64, // 70% to prompt creator
            platform_fee: (cost as f64 * 0.2) as u64,       // 20% platform fee
            endpoint_provider_share: (cost as f64 * 0.1) as u64, // 10% to endpoint provider
            total_revenue: cost,
        };

        self.revenue_sharing.insert(request_id.clone(), revenue);

        let ai_request = AIRequest {
            request_id: request_id.clone(),
            user_id: user_id.to_string(),
            prompt_id: Some(prompt_id.to_string()),
            endpoint_id,
            input_data,
            output_data: Some(serde_json::json!({"result": output})),
            tokens_used: tokens_used as u32,
            cost_credits: cost,
            processing_time_ms: 1500, // Simulated
            timestamp: chrono::Utc::now().timestamp() as u64,
            status: RequestStatus::Completed,
        };

        // Store request history
        self.request_history
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(ai_request.clone());

        println!(
            "🧠 AI request executed: {} tokens, {} credits",
            tokens_used, cost
        );
        Ok(ai_request)
    }

    pub fn direct_llm_request(
        &mut self,
        user_id: &str,
        endpoint_id: &str,
        prompt: &str,
    ) -> Result<AIRequest, String> {
        let quota = self
            .usage_quotas
            .get_mut(user_id)
            .ok_or("User quota not found")?;

        let endpoint = self
            .llm_endpoints
            .get_mut(endpoint_id)
            .ok_or("Endpoint not found")?;

        if quota.requests_used_today >= quota.daily_requests {
            return Err("Daily request limit exceeded".to_string());
        }

        let tokens_estimate = prompt.len() / 4;
        let cost = (tokens_estimate as f64 * endpoint.cost_per_token) as u64;

        if quota.premium_credits < cost {
            return Err("Insufficient credits".to_string());
        }

        // Execute direct request
        let output = self.call_llm_endpoint(&endpoint.endpoint_url, prompt)?;
        let actual_tokens = (prompt.len() + output.len()) / 4;

        // Update usage
        quota.requests_used_today += 1;
        quota.tokens_used_month += actual_tokens as u64;
        quota.premium_credits -= cost;

        endpoint.total_requests += 1;
        endpoint.available_quota -= actual_tokens as u64;

        let request_id = format!("direct_{}_{}", user_id, chrono::Utc::now().timestamp());

        let ai_request = AIRequest {
            request_id: request_id.clone(),
            user_id: user_id.to_string(),
            prompt_id: None,
            endpoint_id: endpoint_id.to_string(),
            input_data: serde_json::json!({"prompt": prompt}),
            output_data: Some(serde_json::json!({"result": output})),
            tokens_used: actual_tokens as u32,
            cost_credits: cost,
            processing_time_ms: 2000,
            timestamp: chrono::Utc::now().timestamp() as u64,
            status: RequestStatus::Completed,
        };

        self.request_history
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(ai_request.clone());

        println!(
            "🎯 Direct LLM request: {} tokens, {} credits",
            actual_tokens, cost
        );
        Ok(ai_request)
    }

    pub fn get_marketplace_prompts(
        &self,
        category: Option<PromptCategory>,
    ) -> Vec<&PublishedPrompt> {
        let mut prompts: Vec<_> = self.published_prompts.values().collect();

        if let Some(cat) = category {
            prompts.retain(|p| matches!(p.category, cat));
        }

        // Sort by rating and usage
        prompts.sort_by(|a, b| {
            let a_score = a.rating * (a.usage_count as f32).log10().max(1.0);
            let b_score = b.rating * (b.usage_count as f32).log10().max(1.0);
            b_score.partial_cmp(&a_score).unwrap()
        });

        prompts
    }

    pub fn get_available_endpoints(&self, model_type: Option<ModelType>) -> Vec<&LLMEndpoint> {
        let mut endpoints: Vec<_> = self
            .llm_endpoints
            .values()
            .filter(|e| e.available_quota > 1000) // Has quota available
            .collect();

        if let Some(mt) = model_type {
            endpoints.retain(|e| matches!(e.model_type, mt));
        }

        // Sort by uptime and cost
        endpoints.sort_by(|a, b| {
            let a_score = a.uptime_percentage / a.cost_per_token;
            let b_score = b.uptime_percentage / b.cost_per_token;
            b_score.partial_cmp(&a_score).unwrap()
        });

        endpoints
    }

    pub fn get_user_ai_stats(&self, user_id: &str) -> Option<String> {
        let quota = self.usage_quotas.get(user_id)?;
        let history = self.request_history.get(user_id)?;

        let total_requests = history.len();
        let total_tokens = history.iter().map(|r| r.tokens_used as u64).sum::<u64>();
        let total_spent = history.iter().map(|r| r.cost_credits).sum::<u64>();

        let stats = serde_json::json!({
            "user_id": user_id,
            "quota": {
                "daily_requests": quota.daily_requests,
                "requests_used_today": quota.requests_used_today,
                "monthly_tokens": quota.monthly_tokens,
                "tokens_used_month": quota.tokens_used_month,
                "premium_credits": quota.premium_credits
            },
            "usage": {
                "total_requests": total_requests,
                "total_tokens": total_tokens,
                "total_spent": total_spent,
                "average_cost_per_request": if total_requests > 0 { total_spent / total_requests as u64 } else { 0 }
            },
            "recent_requests": history.iter().rev().take(5).collect::<Vec<_>>()
        });

        Some(stats.to_string())
    }

    fn find_best_endpoint(&self, requirements: &ModelRequirements) -> Result<String, String> {
        let mut candidates: Vec<_> = self
            .llm_endpoints
            .values()
            .filter(|e| e.available_quota > 1000)
            .filter(|e| e.max_tokens >= requirements.min_context_length)
            .collect();

        if candidates.is_empty() {
            return Err("No available endpoints meet requirements".to_string());
        }

        // Prefer endpoints with required capabilities
        candidates.sort_by(|a, b| {
            let a_score = a.uptime_percentage / a.cost_per_token;
            let b_score = b.uptime_percentage / b.cost_per_token;
            b_score.partial_cmp(&a_score).unwrap()
        });

        Ok(candidates[0].endpoint_id.clone())
    }

    fn process_prompt_template(
        &self,
        template: &str,
        input: &serde_json::Value,
    ) -> Result<String, String> {
        let mut processed = template.to_string();

        // Simple template variable replacement
        if let Some(input_str) = input.get("input").and_then(|v| v.as_str()) {
            processed = processed.replace("{input}", input_str);
        }

        // Add more template processing as needed
        processed = processed.replace("{timestamp}", &chrono::Utc::now().to_rfc3339());

        Ok(processed)
    }

    fn call_llm_endpoint(&self, endpoint_url: &str, prompt: &str) -> Result<String, String> {
        // Simulate LLM API call
        // In real implementation, would make HTTP request to the endpoint

        let simulated_responses = vec![
            "Here's a helpful response to your query.",
            "Based on the input, I can provide this analysis.",
            "The solution to your problem is as follows:",
            "After processing your request, here's what I found:",
        ];

        let response_idx = prompt.len() % simulated_responses.len();
        Ok(format!(
            "{} {}",
            simulated_responses[response_idx],
            prompt.chars().take(50).collect::<String>()
        ))
    }

    pub fn initialize_user_quota(&mut self, user_id: &str, tier: &str) {
        let (daily_requests, monthly_tokens, premium_credits) = match tier {
            "Free" => (10, 10000, 100),
            "Bronze" => (50, 50000, 500),
            "Silver" => (200, 200000, 2000),
            "Gold" => (1000, 1000000, 10000),
            "Platinum" => (u32::MAX, u64::MAX, u64::MAX),
            _ => (10, 10000, 100),
        };

        let quota = UsageQuota {
            user_id: user_id.to_string(),
            tier: tier.to_string(),
            daily_requests,
            requests_used_today: 0,
            monthly_tokens,
            tokens_used_month: 0,
            premium_credits,
            reset_at: chrono::Utc::now().timestamp() as u64 + 86400, // 24 hours
        };

        self.usage_quotas.insert(user_id.to_string(), quota);
    }
}
