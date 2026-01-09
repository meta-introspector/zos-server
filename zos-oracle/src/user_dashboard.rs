use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDashboard {
    pub user_id: String,
    pub current_status: UserStatus,
    pub allocations: AllocationInfo,
    pub usage_stats: UsageStats,
    pub rewards: RewardInfo,
    pub competitive_status: CompetitiveStatus,
    pub marketplace: MarketplaceInfo,
    pub notifications: Vec<Notification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatus {
    pub tier: String,
    pub verification_level: String,
    pub loyalty_points: u64,
    pub usage_credits: u64,
    pub daily_credits_remaining: u64,
    pub streak_days: u32,
    pub last_active: u64,
    pub fingerprint_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    pub current_port: Option<PortAllocation>,
    pub seat_assignment: Option<SeatAssignment>,
    pub shared_ports: Vec<SharedPort>,
    pub port_history: Vec<PortHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortAllocation {
    pub port: u16,
    pub allocated_at_block: u64,
    pub expires_at_block: u64,
    pub blocks_remaining: u64,
    pub service_type: String,
    pub shareable: bool,
    pub shared_count: u32,
    pub usage_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatAssignment {
    pub seat_number: u32,
    pub chamber: String,
    pub held_since_block: u64,
    pub total_blocks_held: u64,
    pub challenges_faced: u32,
    pub challenges_won: u32,
    pub next_challenger: Option<String>,
    pub threat_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedPort {
    pub port: u16,
    pub owner: String,
    pub shared_at: u64,
    pub expires_at: u64,
    pub usage_allowed: u32,
    pub usage_consumed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortHistory {
    pub port: u16,
    pub allocated_at: u64,
    pub duration_blocks: u64,
    pub services_used: Vec<String>,
    pub credits_spent: u64,
    pub shared_with: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub today: DailyUsage,
    pub this_week: WeeklyUsage,
    pub this_month: MonthlyUsage,
    pub all_time: AllTimeUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub ports_allocated: u32,
    pub services_executed: u32,
    pub credits_spent: u64,
    pub credits_earned: u64,
    pub shares_given: u32,
    pub shares_received: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyUsage {
    pub total_blocks_active: u64,
    pub average_daily_credits: u64,
    pub most_used_service: String,
    pub marketplace_transactions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyUsage {
    pub tier_progression: Vec<String>,
    pub verification_milestones: Vec<String>,
    pub peak_daily_usage: u64,
    pub consistency_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllTimeUsage {
    pub total_ports_allocated: u64,
    pub total_services_executed: u64,
    pub total_credits_earned: u64,
    pub total_credits_spent: u64,
    pub highest_rank_achieved: u32,
    pub longest_streak: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardInfo {
    pub pending_rewards: Vec<PendingReward>,
    pub recent_rewards: Vec<CompletedReward>,
    pub milestone_progress: Vec<MilestoneProgress>,
    pub loyalty_bonuses: LoyaltyBonuses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingReward {
    pub reward_type: String,
    pub amount: u64,
    pub unlock_condition: String,
    pub progress_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedReward {
    pub reward_type: String,
    pub amount: u64,
    pub earned_at: u64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneProgress {
    pub milestone_name: String,
    pub current_progress: u64,
    pub target: u64,
    pub reward: u64,
    pub estimated_completion: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyBonuses {
    pub streak_multiplier: f32,
    pub tier_bonus: f32,
    pub verification_bonus: f32,
    pub social_bonus: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveStatus {
    pub current_rank: u32,
    pub rank_change: i32,
    pub cumulative_value: f64,
    pub threat_level: String,
    pub next_challenger: Option<String>,
    pub challenge_target: Option<String>,
    pub leaderboard_position: LeaderboardPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardPosition {
    pub global_rank: u32,
    pub chamber_rank: Option<u32>,
    pub tier_rank: u32,
    pub percentile: f32,
    pub points_to_next_rank: f64,
    pub points_lead_over_next: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceInfo {
    pub active_listings: Vec<ActiveListing>,
    pub purchase_history: Vec<PurchaseHistory>,
    pub sales_history: Vec<SalesHistory>,
    pub market_stats: MarketStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveListing {
    pub listing_id: String,
    pub port: u16,
    pub price: u64,
    pub duration: u64,
    pub views: u32,
    pub interested_buyers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseHistory {
    pub port: u16,
    pub seller: String,
    pub price: u64,
    pub purchased_at: u64,
    pub usage_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesHistory {
    pub port: u16,
    pub buyer: String,
    pub price: u64,
    pub sold_at: u64,
    pub profit_margin: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStats {
    pub total_sales: u64,
    pub total_purchases: u64,
    pub net_profit: i64,
    pub success_rate: f32,
    pub average_sale_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub timestamp: u64,
    pub read: bool,
    pub action_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    PortExpiring,
    SeatChallenged,
    RankChanged,
    RewardEarned,
    MarketplaceSale,
    VerificationRequired,
    SystemUpdate,
}

pub struct DashboardManager {
    pub dashboards: HashMap<String, UserDashboard>,
}

impl DashboardManager {
    pub fn new() -> Self {
        Self {
            dashboards: HashMap::new(),
        }
    }

    pub fn get_dashboard(&mut self, user_id: &str) -> UserDashboard {
        if let Some(dashboard) = self.dashboards.get(user_id) {
            dashboard.clone()
        } else {
            let dashboard = self.create_dashboard(user_id);
            self.dashboards.insert(user_id.to_string(), dashboard.clone());
            dashboard
        }
    }

    pub fn update_dashboard(&mut self, user_id: &str, updates: DashboardUpdate) {
        if let Some(dashboard) = self.dashboards.get_mut(user_id) {
            self.apply_updates(dashboard, updates);
        }
    }

    pub fn get_dashboard_json(&mut self, user_id: &str) -> String {
        let dashboard = self.get_dashboard(user_id);
        serde_json::to_string_pretty(&dashboard).unwrap_or_default()
    }

    pub fn get_quick_stats(&self, user_id: &str) -> Option<String> {
        if let Some(dashboard) = self.dashboards.get(user_id) {
            let stats = serde_json::json!({
                "user_id": user_id,
                "tier": dashboard.current_status.tier,
                "rank": dashboard.competitive_status.current_rank,
                "credits": dashboard.current_status.usage_credits,
                "current_port": dashboard.allocations.current_port.as_ref().map(|p| p.port),
                "seat": dashboard.allocations.seat_assignment.as_ref().map(|s| s.seat_number),
                "streak": dashboard.current_status.streak_days,
                "notifications": dashboard.notifications.iter().filter(|n| !n.read).count()
            });
            Some(stats.to_string())
        } else {
            None
        }
    }

    fn create_dashboard(&self, user_id: &str) -> UserDashboard {
        UserDashboard {
            user_id: user_id.to_string(),
            current_status: UserStatus {
                tier: "Free".to_string(),
                verification_level: "Anonymous".to_string(),
                loyalty_points: 0,
                usage_credits: 100,
                daily_credits_remaining: 100,
                streak_days: 0,
                last_active: 0,
                fingerprint_score: 0.0,
            },
            allocations: AllocationInfo {
                current_port: None,
                seat_assignment: None,
                shared_ports: Vec::new(),
                port_history: Vec::new(),
            },
            usage_stats: UsageStats {
                today: DailyUsage {
                    ports_allocated: 0,
                    services_executed: 0,
                    credits_spent: 0,
                    credits_earned: 0,
                    shares_given: 0,
                    shares_received: 0,
                },
                this_week: WeeklyUsage {
                    total_blocks_active: 0,
                    average_daily_credits: 0,
                    most_used_service: "None".to_string(),
                    marketplace_transactions: 0,
                },
                this_month: MonthlyUsage {
                    tier_progression: vec!["Free".to_string()],
                    verification_milestones: Vec::new(),
                    peak_daily_usage: 0,
                    consistency_score: 0.0,
                },
                all_time: AllTimeUsage {
                    total_ports_allocated: 0,
                    total_services_executed: 0,
                    total_credits_earned: 0,
                    total_credits_spent: 0,
                    highest_rank_achieved: 0,
                    longest_streak: 0,
                },
            },
            rewards: RewardInfo {
                pending_rewards: vec![
                    PendingReward {
                        reward_type: "Email Verification".to_string(),
                        amount: 50,
                        unlock_condition: "Verify your email address".to_string(),
                        progress_percentage: 0.0,
                    },
                    PendingReward {
                        reward_type: "First Port".to_string(),
                        amount: 25,
                        unlock_condition: "Allocate your first port".to_string(),
                        progress_percentage: 0.0,
                    },
                ],
                recent_rewards: Vec::new(),
                milestone_progress: vec![
                    MilestoneProgress {
                        milestone_name: "Bronze Tier".to_string(),
                        current_progress: 0,
                        target: 100,
                        reward: 100,
                        estimated_completion: None,
                    },
                ],
                loyalty_bonuses: LoyaltyBonuses {
                    streak_multiplier: 1.0,
                    tier_bonus: 1.0,
                    verification_bonus: 1.0,
                    social_bonus: 1.0,
                },
            },
            competitive_status: CompetitiveStatus {
                current_rank: 0,
                rank_change: 0,
                cumulative_value: 0.0,
                threat_level: "Safe".to_string(),
                next_challenger: None,
                challenge_target: None,
                leaderboard_position: LeaderboardPosition {
                    global_rank: 0,
                    chamber_rank: None,
                    tier_rank: 0,
                    percentile: 0.0,
                    points_to_next_rank: 0.0,
                    points_lead_over_next: 0.0,
                },
            },
            marketplace: MarketplaceInfo {
                active_listings: Vec::new(),
                purchase_history: Vec::new(),
                sales_history: Vec::new(),
                market_stats: MarketStats {
                    total_sales: 0,
                    total_purchases: 0,
                    net_profit: 0,
                    success_rate: 0.0,
                    average_sale_price: 0.0,
                },
            },
            notifications: vec![
                Notification {
                    id: "welcome".to_string(),
                    notification_type: NotificationType::SystemUpdate,
                    title: "Welcome to ZOS!".to_string(),
                    message: "Complete your profile to earn bonus credits".to_string(),
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    read: false,
                    action_url: Some("/verify".to_string()),
                },
            ],
        }
    }

    fn apply_updates(&self, dashboard: &mut UserDashboard, updates: DashboardUpdate) {
        // Apply updates to dashboard
        // This would be called when user actions occur
    }
}

#[derive(Debug, Clone)]
pub struct DashboardUpdate {
    // Fields for updating dashboard state
    pub port_allocated: Option<u16>,
    pub service_executed: Option<String>,
    pub credits_changed: Option<i64>,
    pub rank_changed: Option<u32>,
}

// Dashboard API endpoints
pub fn dashboard_routes() -> String {
    r#"
🎯 ZOS Dashboard API Endpoints:

GET /dashboard/{user_id}
  → Full dashboard JSON

GET /dashboard/{user_id}/quick
  → Quick stats summary

GET /dashboard/{user_id}/allocations
  → Current port and seat allocations

GET /dashboard/{user_id}/usage
  → Usage statistics and history

GET /dashboard/{user_id}/rewards
  → Rewards and milestone progress

GET /dashboard/{user_id}/competitive
  → Ranking and competitive status

GET /dashboard/{user_id}/marketplace
  → Marketplace activity and stats

GET /dashboard/{user_id}/notifications
  → Unread notifications

POST /dashboard/{user_id}/notifications/{id}/read
  → Mark notification as read

WebSocket /dashboard/{user_id}/live
  → Real-time dashboard updates
"#.to_string()
}
