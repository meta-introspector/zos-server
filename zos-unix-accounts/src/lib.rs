use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnixAccountManager {
    pub user_accounts: HashMap<String, UnixAccount>,
    pub vouching_system: HashMap<String, VouchRecord>,
    pub staking_pools: HashMap<String, StakingPool>,
    pub account_tiers: HashMap<String, AccountTier>,
    pub system_resources: SystemResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnixAccount {
    pub username: String,
    pub user_id: u32,
    pub group_id: u32,
    pub home_directory: String,
    pub shell: String,
    pub account_type: AccountType,
    pub balance_requirement: u64,
    pub current_balance: u64,
    pub vouched_by: Option<String>,
    pub staked_by: Vec<String>,
    pub total_stake: u64,
    pub created_at: u64,
    pub last_login: u64,
    pub resource_limits: ResourceLimits,
    pub permissions: Vec<String>,
    pub good_standing: bool,
    pub reputation_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VouchRecord {
    pub voucher_id: String,
    pub vouched_user: String,
    pub vouch_type: VouchType,
    pub stake_amount: u64,
    pub conditions: Vec<String>,
    pub expires_at: Option<u64>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    pub pool_id: String,
    pub staker_id: String,
    pub total_staked: u64,
    pub backed_users: Vec<String>,
    pub risk_level: RiskLevel,
    pub rewards_earned: u64,
    pub slashing_history: Vec<SlashEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTier {
    pub tier_name: String,
    pub balance_requirement: u64,
    pub resource_limits: ResourceLimits,
    pub permissions: Vec<String>,
    pub max_vouched_users: u32,
    pub staking_power: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_quota: f32,              // CPU percentage
    pub memory_limit_mb: u64,        // RAM limit
    pub disk_quota_mb: u64,          // Disk space
    pub network_bandwidth_kbps: u64, // Network limit
    pub process_limit: u32,          // Max processes
    pub file_descriptors: u32,       // Max open files
    pub cron_jobs: u32,              // Max cron jobs
    pub login_sessions: u32,         // Concurrent logins
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub total_users: u32,
    pub active_users: u32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Free,     // Vouched or good standing
    Balanced, // Maintains balance requirement
    Staked,   // Backed by stakers
    Premium,  // High balance + good standing
    Admin,    // System administrators
    Guest,    // Temporary access
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VouchType {
    FullAccess,   // Complete vouching
    Limited,      // Restricted access
    Probationary, // Trial period
    Educational,  // Learning/demo access
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashEvent {
    pub user_id: String,
    pub reason: String,
    pub amount_slashed: u64,
    pub timestamp: u64,
}

impl UnixAccountManager {
    pub fn new() -> Self {
        let mut manager = Self {
            user_accounts: HashMap::new(),
            vouching_system: HashMap::new(),
            staking_pools: HashMap::new(),
            account_tiers: HashMap::new(),
            system_resources: SystemResources {
                total_users: 0,
                active_users: 0,
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.0,
                network_usage: 0.0,
            },
        };

        manager.initialize_account_tiers();
        manager
    }

    fn initialize_account_tiers(&mut self) {
        // Free tier - vouched users
        self.account_tiers.insert(
            "free".to_string(),
            AccountTier {
                tier_name: "Free".to_string(),
                balance_requirement: 0,
                resource_limits: ResourceLimits {
                    cpu_quota: 5.0,              // 5% CPU
                    memory_limit_mb: 128,        // 128MB RAM
                    disk_quota_mb: 100,          // 100MB disk
                    network_bandwidth_kbps: 100, // 100 Kbps
                    process_limit: 10,
                    file_descriptors: 100,
                    cron_jobs: 2,
                    login_sessions: 1,
                },
                permissions: vec!["ssh".to_string(), "basic_shell".to_string()],
                max_vouched_users: 0,
                staking_power: 0.0,
            },
        );

        // Balanced tier - maintains balance
        self.account_tiers.insert(
            "balanced".to_string(),
            AccountTier {
                tier_name: "Balanced".to_string(),
                balance_requirement: 1000,
                resource_limits: ResourceLimits {
                    cpu_quota: 15.0,              // 15% CPU
                    memory_limit_mb: 512,         // 512MB RAM
                    disk_quota_mb: 1000,          // 1GB disk
                    network_bandwidth_kbps: 1000, // 1 Mbps
                    process_limit: 25,
                    file_descriptors: 500,
                    cron_jobs: 5,
                    login_sessions: 2,
                },
                permissions: vec![
                    "ssh".to_string(),
                    "shell".to_string(),
                    "compiler".to_string(),
                ],
                max_vouched_users: 2,
                staking_power: 1.0,
            },
        );

        // Premium tier - high balance
        self.account_tiers.insert(
            "premium".to_string(),
            AccountTier {
                tier_name: "Premium".to_string(),
                balance_requirement: 10000,
                resource_limits: ResourceLimits {
                    cpu_quota: 50.0,               // 50% CPU
                    memory_limit_mb: 2048,         // 2GB RAM
                    disk_quota_mb: 10000,          // 10GB disk
                    network_bandwidth_kbps: 10000, // 10 Mbps
                    process_limit: 100,
                    file_descriptors: 2000,
                    cron_jobs: 20,
                    login_sessions: 5,
                },
                permissions: vec![
                    "ssh".to_string(),
                    "shell".to_string(),
                    "compiler".to_string(),
                    "docker".to_string(),
                    "admin_tools".to_string(),
                ],
                max_vouched_users: 10,
                staking_power: 5.0,
            },
        );
    }

    pub fn create_account(
        &mut self,
        username: &str,
        account_type: AccountType,
        voucher: Option<&str>,
        initial_balance: u64,
    ) -> Result<UnixAccount, String> {
        if self.user_accounts.contains_key(username) {
            return Err("Username already exists".to_string());
        }

        // Determine tier and requirements
        let (tier, balance_req) = match account_type {
            AccountType::Free => ("free", 0),
            AccountType::Balanced => ("balanced", 1000),
            AccountType::Premium => ("premium", 10000),
            _ => ("free", 0),
        };

        let account_tier = self.account_tiers.get(tier).ok_or("Invalid account tier")?;

        // Check balance requirement
        if initial_balance < balance_req && voucher.is_none() {
            return Err(format!(
                "Insufficient balance. Need {} credits or voucher",
                balance_req
            ));
        }

        // Validate voucher if provided
        if let Some(voucher_id) = voucher {
            self.validate_voucher(voucher_id, username)?;
        }

        // Assign UID/GID
        let user_id = 1000 + self.user_accounts.len() as u32;
        let group_id = user_id;

        let account = UnixAccount {
            username: username.to_string(),
            user_id,
            group_id,
            home_directory: format!("/home/{}", username),
            shell: "/bin/bash".to_string(),
            account_type,
            balance_requirement: balance_req,
            current_balance: initial_balance,
            vouched_by: voucher.map(|v| v.to_string()),
            staked_by: Vec::new(),
            total_stake: 0,
            created_at: chrono::Utc::now().timestamp() as u64,
            last_login: 0,
            resource_limits: account_tier.resource_limits.clone(),
            permissions: account_tier.permissions.clone(),
            good_standing: true,
            reputation_score: 50.0, // Start neutral
        };

        // Create Unix account
        self.create_unix_user(&account)?;

        println!(
            "👤 Unix account created: {} (UID: {}, Tier: {})",
            username, user_id, tier
        );

        self.user_accounts
            .insert(username.to_string(), account.clone());
        self.system_resources.total_users += 1;

        Ok(account)
    }

    pub fn vouch_for_user(
        &mut self,
        voucher_id: &str,
        username: &str,
        vouch_type: VouchType,
        stake_amount: u64,
    ) -> Result<String, String> {
        // Check if voucher has permission
        let voucher = self
            .user_accounts
            .get(voucher_id)
            .ok_or("Voucher not found")?;

        let voucher_tier = self.get_user_tier(voucher_id)?;
        if voucher_tier.max_vouched_users == 0 {
            return Err("Voucher tier cannot vouch for users".to_string());
        }

        // Count existing vouches
        let current_vouches = self
            .vouching_system
            .values()
            .filter(|v| v.voucher_id == voucher_id && v.active)
            .count() as u32;

        if current_vouches >= voucher_tier.max_vouched_users {
            return Err("Voucher has reached maximum vouched users".to_string());
        }

        // Check stake amount
        if voucher.current_balance < stake_amount {
            return Err("Insufficient balance for stake".to_string());
        }

        let vouch_id = format!("vouch_{}_{}", voucher_id, username);

        let vouch_record = VouchRecord {
            voucher_id: voucher_id.to_string(),
            vouched_user: username.to_string(),
            vouch_type,
            stake_amount,
            conditions: vec!["maintain_good_standing".to_string()],
            expires_at: None, // Permanent unless revoked
            active: true,
        };

        self.vouching_system.insert(vouch_id.clone(), vouch_record);

        // Update voucher balance
        if let Some(voucher_account) = self.user_accounts.get_mut(voucher_id) {
            voucher_account.current_balance -= stake_amount;
        }

        println!(
            "🤝 User {} vouched by {} (stake: {} credits)",
            username,
            &voucher_id[..8],
            stake_amount
        );

        Ok(vouch_id)
    }

    pub fn create_staking_pool(
        &mut self,
        staker_id: &str,
        initial_stake: u64,
    ) -> Result<String, String> {
        let staker = self
            .user_accounts
            .get_mut(staker_id)
            .ok_or("Staker not found")?;

        if staker.current_balance < initial_stake {
            return Err("Insufficient balance for staking pool".to_string());
        }

        let pool_id = format!("pool_{}_{}", staker_id, chrono::Utc::now().timestamp());

        let pool = StakingPool {
            pool_id: pool_id.clone(),
            staker_id: staker_id.to_string(),
            total_staked: initial_stake,
            backed_users: Vec::new(),
            risk_level: RiskLevel::Low,
            rewards_earned: 0,
            slashing_history: Vec::new(),
        };

        staker.current_balance -= initial_stake;
        self.staking_pools.insert(pool_id.clone(), pool);

        println!(
            "💰 Staking pool created: {} with {} credits",
            &pool_id[..12],
            initial_stake
        );

        Ok(pool_id)
    }

    pub fn check_balance_requirements(&mut self) -> Vec<String> {
        let mut violations = Vec::new();

        for (username, account) in &mut self.user_accounts {
            if matches!(
                account.account_type,
                AccountType::Balanced | AccountType::Premium
            ) {
                if account.current_balance < account.balance_requirement {
                    // Grace period or find staking
                    if account.total_stake < account.balance_requirement {
                        account.good_standing = false;
                        violations.push(username.clone());
                        println!(
                            "⚠️  Balance violation: {} (need: {}, have: {})",
                            username, account.balance_requirement, account.current_balance
                        );
                    }
                }
            }
        }

        violations
    }

    pub fn get_account_status(&self, username: &str) -> Option<String> {
        if let Some(account) = self.user_accounts.get(username) {
            let status = serde_json::json!({
                "username": username,
                "uid": account.user_id,
                "account_type": account.account_type,
                "balance": account.current_balance,
                "balance_requirement": account.balance_requirement,
                "good_standing": account.good_standing,
                "reputation": account.reputation_score,
                "vouched_by": account.vouched_by,
                "total_stake": account.total_stake,
                "resource_limits": account.resource_limits,
                "permissions": account.permissions,
                "last_login": account.last_login,
                "home_directory": account.home_directory
            });
            Some(status.to_string())
        } else {
            None
        }
    }

    fn create_unix_user(&self, account: &UnixAccount) -> Result<(), String> {
        // Create user with useradd
        let useradd_cmd = format!(
            "useradd -u {} -g {} -d {} -s {} -c 'ZOS User' {}",
            account.user_id,
            account.group_id,
            account.home_directory,
            account.shell,
            account.username
        );

        // Set resource limits in /etc/security/limits.conf
        let limits = format!(
            "{} soft cpu {}\n{} hard cpu {}\n{} soft memlock {}\n{} hard memlock {}",
            account.username,
            account.resource_limits.cpu_quota as u32,
            account.username,
            account.resource_limits.cpu_quota as u32,
            account.username,
            account.resource_limits.memory_limit_mb,
            account.username,
            account.resource_limits.memory_limit_mb
        );

        // Set disk quota
        let quota_cmd = format!(
            "setquota -u {} {} {} 0 0 /",
            account.username,
            account.resource_limits.disk_quota_mb * 1024, // Convert to KB
            account.resource_limits.disk_quota_mb * 1024
        );

        println!("🔧 Unix user created: {}", useradd_cmd);
        println!("📊 Resource limits: {}", limits);
        println!("💾 Disk quota: {}", quota_cmd);

        // In real implementation, would execute these commands
        Ok(())
    }

    fn validate_voucher(&self, voucher_id: &str, username: &str) -> Result<(), String> {
        if let Some(vouch) = self
            .vouching_system
            .get(&format!("vouch_{}_{}", voucher_id, username))
        {
            if vouch.active {
                Ok(())
            } else {
                Err("Vouch is not active".to_string())
            }
        } else {
            Err("No valid vouch found".to_string())
        }
    }

    fn get_user_tier(&self, user_id: &str) -> Result<&AccountTier, String> {
        let account = self.user_accounts.get(user_id).ok_or("User not found")?;

        let tier_name = match account.account_type {
            AccountType::Free => "free",
            AccountType::Balanced => "balanced",
            AccountType::Premium => "premium",
            _ => "free",
        };

        self.account_tiers.get(tier_name).ok_or("Tier not found")
    }
}
