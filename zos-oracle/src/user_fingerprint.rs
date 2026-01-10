use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFingerprint {
    pub user_id: String,
    pub device_fingerprint: DeviceFingerprint,
    pub ip_fingerprint: IpFingerprint,
    pub response_fingerprint: ResponseFingerprint,
    pub verification_level: VerificationLevel,
    pub loyalty_points: u64,
    pub usage_credits: u64,
    pub tier: UserTier,
    pub verifications: HashMap<String, VerificationStatus>,
    pub jwt_loyalty_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub user_agent: String,
    pub screen_resolution: String,
    pub timezone: String,
    pub language: String,
    pub platform: String,
    pub webgl_vendor: String,
    pub canvas_hash: String,
    pub audio_hash: String,
    pub font_list_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpFingerprint {
    pub ip_hash: String, // Hashed for privacy
    pub country: String,
    pub asn: String,
    pub is_vpn: bool,
    pub is_tor: bool,
    pub risk_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFingerprint {
    pub typing_pattern: Vec<u32>, // Keystroke timing
    pub mouse_movement: String,   // Movement pattern hash
    pub click_pattern: Vec<u32>,  // Click timing
    pub scroll_behavior: String,  // Scroll pattern hash
    pub interaction_rhythm: f32,  // Overall interaction timing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationLevel {
    Anonymous, // 0 points
    Basic,     // Device + IP
    Enhanced,  // + Email
    Verified,  // + Phone/SMS
    Trusted,   // + Social media
    Premium,   // + KYC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserTier {
    Free,     // 100 credits/day
    Bronze,   // 500 credits/day
    Silver,   // 2000 credits/day
    Gold,     // 10000 credits/day
    Platinum, // Unlimited
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStatus {
    pub verified: bool,
    pub verified_at: u64,
    pub points_awarded: u64,
    pub credits_awarded: u64,
    pub expires_at: Option<u64>,
}

pub struct FingerprintManager {
    pub users: HashMap<String, UserFingerprint>,
    pub verification_rules: VerificationRules,
    pub loyalty_config: LoyaltyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRules {
    pub email_points: u64,
    pub phone_points: u64,
    pub twitter_points: u64,
    pub github_points: u64,
    pub kyc_points: u64,
    pub daily_login_points: u64,
    pub referral_points: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyConfig {
    pub points_to_credits_ratio: f32,
    pub tier_thresholds: HashMap<String, u64>,
    pub daily_credit_limits: HashMap<String, u64>,
    pub jwt_cookie_bonus: u64,
}

impl FingerprintManager {
    pub fn new() -> Self {
        let mut tier_thresholds = HashMap::new();
        tier_thresholds.insert("Bronze".to_string(), 100);
        tier_thresholds.insert("Silver".to_string(), 500);
        tier_thresholds.insert("Gold".to_string(), 2000);
        tier_thresholds.insert("Platinum".to_string(), 10000);

        let mut daily_limits = HashMap::new();
        daily_limits.insert("Free".to_string(), 100);
        daily_limits.insert("Bronze".to_string(), 500);
        daily_limits.insert("Silver".to_string(), 2000);
        daily_limits.insert("Gold".to_string(), 10000);
        daily_limits.insert("Platinum".to_string(), u64::MAX);

        Self {
            users: HashMap::new(),
            verification_rules: VerificationRules {
                email_points: 50,
                phone_points: 100,
                twitter_points: 75,
                github_points: 150,
                kyc_points: 500,
                daily_login_points: 10,
                referral_points: 200,
            },
            loyalty_config: LoyaltyConfig {
                points_to_credits_ratio: 0.1,
                tier_thresholds,
                daily_credit_limits: daily_limits,
                jwt_cookie_bonus: 25,
            },
        }
    }

    pub fn create_fingerprint(&mut self, device: DeviceFingerprint, ip: IpFingerprint) -> String {
        let user_id = self.generate_user_id(&device, &ip);

        let fingerprint = UserFingerprint {
            user_id: user_id.clone(),
            device_fingerprint: device,
            ip_fingerprint: ip,
            response_fingerprint: ResponseFingerprint {
                typing_pattern: Vec::new(),
                mouse_movement: String::new(),
                click_pattern: Vec::new(),
                scroll_behavior: String::new(),
                interaction_rhythm: 0.0,
            },
            verification_level: VerificationLevel::Anonymous,
            loyalty_points: 0,
            usage_credits: 100, // Free tier start
            tier: UserTier::Free,
            verifications: HashMap::new(),
            jwt_loyalty_key: None,
        };

        println!("👤 New user fingerprinted: {}", &user_id[..8]);
        self.users.insert(user_id.clone(), fingerprint);

        user_id
    }

    pub fn verify_email(&mut self, user_id: &str, email: &str) -> Result<u64, String> {
        let user = self.users.get_mut(user_id).ok_or("User not found")?;

        // Simulate email verification
        let verification = VerificationStatus {
            verified: true,
            verified_at: chrono::Utc::now().timestamp() as u64,
            points_awarded: self.verification_rules.email_points,
            credits_awarded: (self.verification_rules.email_points as f32
                * self.loyalty_config.points_to_credits_ratio) as u64,
            expires_at: None,
        };

        user.verifications.insert("email".to_string(), verification);
        user.loyalty_points += self.verification_rules.email_points;
        user.usage_credits += (self.verification_rules.email_points as f32
            * self.loyalty_config.points_to_credits_ratio) as u64;

        self.update_user_tier(user_id)?;

        println!(
            "📧 Email verified for {}: +{} points",
            &user_id[..8],
            self.verification_rules.email_points
        );
        Ok(self.verification_rules.email_points)
    }

    pub fn verify_twitter(&mut self, user_id: &str, twitter_handle: &str) -> Result<u64, String> {
        let user = self.users.get_mut(user_id).ok_or("User not found")?;

        // Simulate Twitter API verification
        let verification = VerificationStatus {
            verified: true,
            verified_at: chrono::Utc::now().timestamp() as u64,
            points_awarded: self.verification_rules.twitter_points,
            credits_awarded: (self.verification_rules.twitter_points as f32
                * self.loyalty_config.points_to_credits_ratio) as u64,
            expires_at: Some(chrono::Utc::now().timestamp() as u64 + 86400 * 30), // 30 days
        };

        user.verifications
            .insert("twitter".to_string(), verification);
        user.loyalty_points += self.verification_rules.twitter_points;
        user.usage_credits += (self.verification_rules.twitter_points as f32
            * self.loyalty_config.points_to_credits_ratio) as u64;

        self.update_user_tier(user_id)?;

        println!(
            "🐦 Twitter verified for {}: +{} points",
            &user_id[..8],
            self.verification_rules.twitter_points
        );
        Ok(self.verification_rules.twitter_points)
    }

    pub fn verify_kyc(&mut self, user_id: &str, kyc_data: &str) -> Result<u64, String> {
        let user = self.users.get_mut(user_id).ok_or("User not found")?;

        // Simulate KYC verification (would integrate with real KYC provider)
        let verification = VerificationStatus {
            verified: true,
            verified_at: chrono::Utc::now().timestamp() as u64,
            points_awarded: self.verification_rules.kyc_points,
            credits_awarded: (self.verification_rules.kyc_points as f32
                * self.loyalty_config.points_to_credits_ratio) as u64,
            expires_at: Some(chrono::Utc::now().timestamp() as u64 + 86400 * 365), // 1 year
        };

        user.verifications.insert("kyc".to_string(), verification);
        user.loyalty_points += self.verification_rules.kyc_points;
        user.usage_credits += (self.verification_rules.kyc_points as f32
            * self.loyalty_config.points_to_credits_ratio) as u64;
        user.verification_level = VerificationLevel::Premium;

        self.update_user_tier(user_id)?;

        println!(
            "🆔 KYC verified for {}: +{} points",
            &user_id[..8],
            self.verification_rules.kyc_points
        );
        Ok(self.verification_rules.kyc_points)
    }

    pub fn accept_loyalty_cookies(&mut self, user_id: &str) -> Result<String, String> {
        let user = self.users.get_mut(user_id).ok_or("User not found")?;

        // Generate JWT loyalty key
        let jwt_key = format!("zos_loyalty_{}_{}", user_id, chrono::Utc::now().timestamp());
        user.jwt_loyalty_key = Some(jwt_key.clone());
        user.loyalty_points += self.loyalty_config.jwt_cookie_bonus;
        user.usage_credits += self.loyalty_config.jwt_cookie_bonus;

        println!(
            "🍪 Loyalty cookies accepted for {}: +{} bonus",
            &user_id[..8],
            self.loyalty_config.jwt_cookie_bonus
        );
        Ok(jwt_key)
    }

    pub fn update_response_fingerprint(
        &mut self,
        user_id: &str,
        response: ResponseFingerprint,
    ) -> Result<(), String> {
        let user = self.users.get_mut(user_id).ok_or("User not found")?;

        user.response_fingerprint = response;

        // Award points for consistent interaction patterns
        if user.response_fingerprint.interaction_rhythm > 0.8 {
            user.loyalty_points += 5;
            user.usage_credits += 1;
        }

        Ok(())
    }

    pub fn daily_login_bonus(&mut self, user_id: &str) -> Result<u64, String> {
        let user = self.users.get_mut(user_id).ok_or("User not found")?;

        user.loyalty_points += self.verification_rules.daily_login_points;
        user.usage_credits += (self.verification_rules.daily_login_points as f32
            * self.loyalty_config.points_to_credits_ratio) as u64;

        println!(
            "🎯 Daily login bonus for {}: +{} points",
            &user_id[..8],
            self.verification_rules.daily_login_points
        );
        Ok(self.verification_rules.daily_login_points)
    }

    pub fn consume_credits(&mut self, user_id: &str, amount: u64) -> Result<bool, String> {
        let user = self.users.get_mut(user_id).ok_or("User not found")?;

        if user.usage_credits >= amount {
            user.usage_credits -= amount;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_user_status(&self, user_id: &str) -> Result<String, String> {
        let user = self.users.get(user_id).ok_or("User not found")?;

        let status = serde_json::json!({
            "user_id": user_id,
            "tier": user.tier,
            "verification_level": user.verification_level,
            "loyalty_points": user.loyalty_points,
            "usage_credits": user.usage_credits,
            "daily_limit": self.loyalty_config.daily_credit_limits.get(&format!("{:?}", user.tier)),
            "verifications": user.verifications,
            "has_loyalty_cookies": user.jwt_loyalty_key.is_some(),
            "risk_score": user.ip_fingerprint.risk_score
        });

        Ok(status.to_string())
    }

    fn generate_user_id(&self, device: &DeviceFingerprint, ip: &IpFingerprint) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        device.canvas_hash.hash(&mut hasher);
        device.audio_hash.hash(&mut hasher);
        ip.ip_hash.hash(&mut hasher);

        format!("user_{:x}", hasher.finish())
    }

    fn update_user_tier(&mut self, user_id: &str) -> Result<(), String> {
        let user = self.users.get_mut(user_id).ok_or("User not found")?;

        let points = user.loyalty_points;

        user.tier = if points >= self.loyalty_config.tier_thresholds["Platinum"] {
            UserTier::Platinum
        } else if points >= self.loyalty_config.tier_thresholds["Gold"] {
            UserTier::Gold
        } else if points >= self.loyalty_config.tier_thresholds["Silver"] {
            UserTier::Silver
        } else if points >= self.loyalty_config.tier_thresholds["Bronze"] {
            UserTier::Bronze
        } else {
            UserTier::Free
        };

        // Update daily credits based on tier
        let daily_limit = self
            .loyalty_config
            .daily_credit_limits
            .get(&format!("{:?}", user.tier))
            .copied()
            .unwrap_or(100);

        if user.usage_credits < daily_limit {
            user.usage_credits = daily_limit;
        }

        Ok(())
    }
}
