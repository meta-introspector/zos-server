use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityResourceEconomy {
    pub servers: HashMap<String, CommunityServer>,
    pub resource_pools: HashMap<String, ResourcePool>,
    pub token_distribution: HashMap<String, TokenAllocation>,
    pub governance_proposals: HashMap<String, ResourceProposal>,
    pub community_metrics: CommunityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityServer {
    pub server_id: String,
    pub operator_id: String,
    pub server_name: String,
    pub location: String,
    pub contributed_resources: ContributedResources,
    pub token_allocation: u64,
    pub distribution_policy: DistributionPolicy,
    pub active_users: u32,
    pub reputation_score: f32,
    pub uptime_percentage: f32,
    pub community_benefits: Vec<CommunityBenefit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributedResources {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub storage_gb: u32,
    pub bandwidth_mbps: u32,
    pub gpu_units: u32,
    pub specialized_hardware: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    pub pool_id: String,
    pub server_id: String,
    pub pool_type: PoolType,
    pub total_capacity: u64,
    pub allocated_capacity: u64,
    pub token_cost_per_unit: f64,
    pub allocation_rules: AllocationRules,
    pub beneficiaries: Vec<Beneficiary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAllocation {
    pub recipient_id: String,
    pub allocation_type: AllocationType,
    pub amount: u64,
    pub vesting_schedule: Option<VestingSchedule>,
    pub conditions: Vec<String>,
    pub allocated_by: String,
    pub allocated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionPolicy {
    pub policy_name: String,
    pub free_tier_percentage: f32,      // % for free users
    pub community_percentage: f32,      // % for community projects
    pub staking_percentage: f32,        // % for stakers
    pub developer_percentage: f32,      // % for developers
    pub reserve_percentage: f32,        // % held in reserve
    pub distribution_criteria: Vec<DistributionCriteria>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityBenefit {
    pub benefit_type: BenefitType,
    pub description: String,
    pub token_reward: u64,
    pub resource_allocation: Option<u64>,
    pub eligibility_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityMetrics {
    pub total_servers: u32,
    pub total_contributed_resources: ContributedResources,
    pub total_active_users: u32,
    pub token_circulation: u64,
    pub community_projects: u32,
    pub average_server_uptime: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolType {
    Compute,
    Storage,
    Network,
    GPU,
    Development,
    Gaming,
    AI_Training,
    Community,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationType {
    ServerContribution,    // Rewards for running servers
    CommunityProject,      // Funding community initiatives
    UserReward,           // Individual user rewards
    DeveloperGrant,       // Developer funding
    StakingReward,        // Staking pool rewards
    GovernanceReward,     // DAO participation rewards
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenefitType {
    FreeAccounts,         // Free Unix accounts
    PriorityAccess,       // Queue jumping
    ExtraResources,       // Bonus CPU/RAM/disk
    DeveloperTools,       // Compiler access, etc.
    CommunityStatus,      // Special recognition
    TokenRewards,         // Direct token payments
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRules {
    pub max_per_user: u64,
    pub cooldown_period: u64,
    pub reputation_requirement: f32,
    pub community_priority: bool,
    pub staking_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beneficiary {
    pub user_id: String,
    pub allocation_amount: u64,
    pub allocation_reason: String,
    pub granted_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionCriteria {
    pub criteria_name: String,
    pub weight: f32,
    pub measurement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    pub total_amount: u64,
    pub cliff_period: u64,
    pub vesting_period: u64,
    pub released_amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceProposal {
    pub proposal_id: String,
    pub proposer_id: String,
    pub title: String,
    pub description: String,
    pub requested_tokens: u64,
    pub requested_resources: ContributedResources,
    pub community_benefit: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Draft,
    Voting,
    Approved,
    Rejected,
    Implemented,
}

impl CommunityResourceEconomy {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            resource_pools: HashMap::new(),
            token_distribution: HashMap::new(),
            governance_proposals: HashMap::new(),
            community_metrics: CommunityMetrics {
                total_servers: 0,
                total_contributed_resources: ContributedResources {
                    cpu_cores: 0,
                    memory_gb: 0,
                    storage_gb: 0,
                    bandwidth_mbps: 0,
                    gpu_units: 0,
                    specialized_hardware: Vec::new(),
                },
                total_active_users: 0,
                token_circulation: 0,
                community_projects: 0,
                average_server_uptime: 0.0,
            },
        }
    }

    pub fn register_community_server(&mut self, operator_id: &str, server_name: &str,
                                   location: &str, resources: ContributedResources) -> Result<String, String> {

        let server_id = format!("server_{}_{}", operator_id, chrono::Utc::now().timestamp());

        // Calculate token allocation based on contributed resources
        let token_allocation = self.calculate_server_token_allocation(&resources);

        let server = CommunityServer {
            server_id: server_id.clone(),
            operator_id: operator_id.to_string(),
            server_name: server_name.to_string(),
            location: location.to_string(),
            contributed_resources: resources.clone(),
            token_allocation,
            distribution_policy: DistributionPolicy {
                policy_name: "Default Community Policy".to_string(),
                free_tier_percentage: 30.0,      // 30% for free users
                community_percentage: 25.0,      // 25% for community projects
                staking_percentage: 20.0,        // 20% for stakers
                developer_percentage: 15.0,      // 15% for developers
                reserve_percentage: 10.0,        // 10% reserve
                distribution_criteria: vec![
                    DistributionCriteria {
                        criteria_name: "Reputation Score".to_string(),
                        weight: 0.4,
                        measurement: "user_reputation".to_string(),
                    },
                    DistributionCriteria {
                        criteria_name: "Community Contribution".to_string(),
                        weight: 0.3,
                        measurement: "community_activity".to_string(),
                    },
                    DistributionCriteria {
                        criteria_name: "Resource Usage".to_string(),
                        weight: 0.3,
                        measurement: "efficient_usage".to_string(),
                    },
                ],
            },
            active_users: 0,
            reputation_score: 50.0,
            uptime_percentage: 100.0,
            community_benefits: vec![
                CommunityBenefit {
                    benefit_type: BenefitType::FreeAccounts,
                    description: "Free Unix accounts for community members".to_string(),
                    token_reward: 0,
                    resource_allocation: Some(resources.cpu_cores as u64 * 10), // 10 accounts per core
                    eligibility_criteria: vec!["good_standing".to_string(), "community_member".to_string()],
                },
                CommunityBenefit {
                    benefit_type: BenefitType::DeveloperTools,
                    description: "Compiler and development tool access".to_string(),
                    token_reward: 50,
                    resource_allocation: Some(resources.cpu_cores as u64 * 5), // 5 dev accounts per core
                    eligibility_criteria: vec!["developer_verified".to_string(), "project_contribution".to_string()],
                },
            ],
        };

        // Create resource pools for this server
        self.create_server_resource_pools(&server_id, &resources)?;

        // Distribute initial tokens to server operator
        self.distribute_tokens(&server_id, operator_id, AllocationType::ServerContribution, token_allocation)?;

        self.servers.insert(server_id.clone(), server);
        self.update_community_metrics();

        println!("🖥️  Community server registered: {} by {} ({} tokens allocated)",
                 server_name, &operator_id[..8], token_allocation);

        Ok(server_id)
    }

    pub fn allocate_resources(&mut self, server_id: &str, user_id: &str,
                            resource_type: PoolType, amount: u64) -> Result<String, String> {

        let server = self.servers.get_mut(server_id)
            .ok_or("Server not found")?;

        // Check if server has available resources
        let pool_id = format!("{}_{:?}", server_id, resource_type);
        let pool = self.resource_pools.get_mut(&pool_id)
            .ok_or("Resource pool not found")?;

        if pool.allocated_capacity + amount > pool.total_capacity {
            return Err("Insufficient resources available".to_string());
        }

        // Check allocation rules
        if amount > pool.allocation_rules.max_per_user {
            return Err("Exceeds maximum allocation per user".to_string());
        }

        // Apply server's distribution policy
        let allocation_approved = self.check_distribution_policy(server, user_id, amount)?;
        if !allocation_approved {
            return Err("Allocation denied by server policy".to_string());
        }

        // Allocate resources
        pool.allocated_capacity += amount;
        pool.beneficiaries.push(Beneficiary {
            user_id: user_id.to_string(),
            allocation_amount: amount,
            allocation_reason: format!("{:?} allocation", resource_type),
            granted_at: chrono::Utc::now().timestamp() as u64,
        });

        // Calculate token cost
        let token_cost = (amount as f64 * pool.token_cost_per_unit) as u64;

        println!("📦 Resources allocated: {} units of {:?} to {} (cost: {} tokens)",
                 amount, resource_type, &user_id[..8], token_cost);

        Ok(format!("Allocated {} units for {} tokens", amount, token_cost))
    }

    pub fn propose_community_project(&mut self, proposer_id: &str, title: &str,
                                   description: &str, requested_tokens: u64,
                                   requested_resources: ContributedResources) -> Result<String, String> {

        let proposal_id = format!("prop_{}_{}", proposer_id, chrono::Utc::now().timestamp());

        let proposal = ResourceProposal {
            proposal_id: proposal_id.clone(),
            proposer_id: proposer_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            requested_tokens,
            requested_resources,
            community_benefit: "Community infrastructure improvement".to_string(),
            votes_for: 0,
            votes_against: 0,
            status: ProposalStatus::Voting,
        };

        self.governance_proposals.insert(proposal_id.clone(), proposal);

        println!("📋 Community proposal created: {} requesting {} tokens", title, requested_tokens);

        Ok(proposal_id)
    }

    pub fn distribute_server_rewards(&mut self, server_id: &str) -> Result<u64, String> {
        let server = self.servers.get(server_id)
            .ok_or("Server not found")?;

        // Calculate rewards based on uptime, users, and community contribution
        let base_reward = server.token_allocation / 100; // 1% of allocation per period
        let uptime_multiplier = server.uptime_percentage / 100.0;
        let user_multiplier = (server.active_users as f32 / 10.0).min(2.0); // Max 2x for user activity
        let reputation_multiplier = server.reputation_score / 50.0; // Normalized to 1.0 at 50 reputation

        let total_reward = (base_reward as f32 * uptime_multiplier * user_multiplier * reputation_multiplier) as u64;

        // Distribute according to server policy
        let free_tier_tokens = (total_reward as f32 * server.distribution_policy.free_tier_percentage / 100.0) as u64;
        let community_tokens = (total_reward as f32 * server.distribution_policy.community_percentage / 100.0) as u64;
        let staking_tokens = (total_reward as f32 * server.distribution_policy.staking_percentage / 100.0) as u64;
        let developer_tokens = (total_reward as f32 * server.distribution_policy.developer_percentage / 100.0) as u64;

        println!("💰 Server {} rewards distributed: {} total tokens", &server_id[..12], total_reward);
        println!("   Free tier: {}, Community: {}, Staking: {}, Developers: {}",
                 free_tier_tokens, community_tokens, staking_tokens, developer_tokens);

        Ok(total_reward)
    }

    pub fn get_community_status(&self) -> String {
        let status = serde_json::json!({
            "total_servers": self.community_metrics.total_servers,
            "total_resources": {
                "cpu_cores": self.community_metrics.total_contributed_resources.cpu_cores,
                "memory_gb": self.community_metrics.total_contributed_resources.memory_gb,
                "storage_gb": self.community_metrics.total_contributed_resources.storage_gb,
                "bandwidth_mbps": self.community_metrics.total_contributed_resources.bandwidth_mbps
            },
            "active_users": self.community_metrics.total_active_users,
            "token_circulation": self.community_metrics.token_circulation,
            "average_uptime": self.community_metrics.average_server_uptime,
            "community_projects": self.community_metrics.community_projects,
            "top_servers": self.get_top_servers(5)
        });

        status.to_string()
    }

    fn calculate_server_token_allocation(&self, resources: &ContributedResources) -> u64 {
        // Token allocation formula based on contributed resources
        let cpu_value = resources.cpu_cores as u64 * 1000;
        let memory_value = resources.memory_gb as u64 * 100;
        let storage_value = resources.storage_gb as u64 * 10;
        let bandwidth_value = resources.bandwidth_mbps as u64 * 50;
        let gpu_value = resources.gpu_units as u64 * 5000;

        cpu_value + memory_value + storage_value + bandwidth_value + gpu_value
    }

    fn create_server_resource_pools(&mut self, server_id: &str, resources: &ContributedResources) -> Result<(), String> {
        // Create compute pool
        let compute_pool = ResourcePool {
            pool_id: format!("{}_Compute", server_id),
            server_id: server_id.to_string(),
            pool_type: PoolType::Compute,
            total_capacity: resources.cpu_cores as u64 * 100, // 100 units per core
            allocated_capacity: 0,
            token_cost_per_unit: 1.0,
            allocation_rules: AllocationRules {
                max_per_user: resources.cpu_cores as u64 * 10,
                cooldown_period: 3600, // 1 hour
                reputation_requirement: 25.0,
                community_priority: true,
                staking_multiplier: 1.5,
            },
            beneficiaries: Vec::new(),
        };

        self.resource_pools.insert(compute_pool.pool_id.clone(), compute_pool);

        // Create storage pool
        let storage_pool = ResourcePool {
            pool_id: format!("{}_Storage", server_id),
            server_id: server_id.to_string(),
            pool_type: PoolType::Storage,
            total_capacity: resources.storage_gb as u64 * 1024, // MB units
            allocated_capacity: 0,
            token_cost_per_unit: 0.1,
            allocation_rules: AllocationRules {
                max_per_user: resources.storage_gb as u64 * 100,
                cooldown_period: 86400, // 24 hours
                reputation_requirement: 10.0,
                community_priority: true,
                staking_multiplier: 1.2,
            },
            beneficiaries: Vec::new(),
        };

        self.resource_pools.insert(storage_pool.pool_id.clone(), storage_pool);

        Ok(())
    }

    fn check_distribution_policy(&self, server: &CommunityServer, user_id: &str, amount: u64) -> Result<bool, String> {
        // Simplified policy check - in real implementation would check user reputation,
        // community standing, staking status, etc.
        Ok(true)
    }

    fn distribute_tokens(&mut self, server_id: &str, recipient_id: &str,
                        allocation_type: AllocationType, amount: u64) -> Result<(), String> {

        let allocation_id = format!("alloc_{}_{}", recipient_id, chrono::Utc::now().timestamp());

        let allocation = TokenAllocation {
            recipient_id: recipient_id.to_string(),
            allocation_type,
            amount,
            vesting_schedule: None,
            conditions: Vec::new(),
            allocated_by: server_id.to_string(),
            allocated_at: chrono::Utc::now().timestamp() as u64,
        };

        self.token_distribution.insert(allocation_id, allocation);
        Ok(())
    }

    fn update_community_metrics(&mut self) {
        self.community_metrics.total_servers = self.servers.len() as u32;

        // Aggregate resources
        let mut total_resources = ContributedResources {
            cpu_cores: 0,
            memory_gb: 0,
            storage_gb: 0,
            bandwidth_mbps: 0,
            gpu_units: 0,
            specialized_hardware: Vec::new(),
        };

        let mut total_uptime = 0.0;
        let mut total_users = 0;

        for server in self.servers.values() {
            total_resources.cpu_cores += server.contributed_resources.cpu_cores;
            total_resources.memory_gb += server.contributed_resources.memory_gb;
            total_resources.storage_gb += server.contributed_resources.storage_gb;
            total_resources.bandwidth_mbps += server.contributed_resources.bandwidth_mbps;
            total_resources.gpu_units += server.contributed_resources.gpu_units;

            total_uptime += server.uptime_percentage;
            total_users += server.active_users;
        }

        self.community_metrics.total_contributed_resources = total_resources;
        self.community_metrics.total_active_users = total_users;
        self.community_metrics.average_server_uptime = if self.servers.len() > 0 {
            total_uptime / self.servers.len() as f32
        } else {
            0.0
        };
    }

    fn get_top_servers(&self, limit: usize) -> Vec<String> {
        let mut servers: Vec<_> = self.servers.values().collect();
        servers.sort_by(|a, b| {
            let a_score = a.reputation_score * a.uptime_percentage * (a.active_users as f32);
            let b_score = b.reputation_score * b.uptime_percentage * (b.active_users as f32);
            b_score.partial_cmp(&a_score).unwrap()
        });

        servers.into_iter()
            .take(limit)
            .map(|s| format!("{} ({})", s.server_name, s.location))
            .collect()
    }
}
