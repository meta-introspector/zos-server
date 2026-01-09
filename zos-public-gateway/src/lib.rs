use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionSystem {
    pub referral_tracking: HashMap<String, ReferralRecord>,
    pub commission_rates: CommissionRates,
    pub earnings_ledger: HashMap<String, EarningsAccount>,
    pub referral_links: HashMap<String, ReferralLink>,
    pub commission_history: HashMap<String, Vec<CommissionPayment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralRecord {
    pub referrer_wallet: String,
    pub referee_wallet: String,
    pub referral_code: String,
    pub first_transaction_at: u64,
    pub total_volume: f64,
    pub total_commissions_earned: f64,
    pub status: ReferralStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionRates {
    pub swap_commission_percentage: f64,      // % of swap fees
    pub referral_commission_percentage: f64, // % of referee's fees
    pub service_commission_percentage: f64,   // % of service payments
    pub tier_multipliers: HashMap<String, f64>, // Tier-based multipliers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsAccount {
    pub wallet_address: String,
    pub total_earned_usdc: f64,
    pub total_earned_solfunmeme: f64,
    pub pending_withdrawals: f64,
    pub lifetime_volume: f64,
    pub referral_count: u32,
    pub tier: EarningsTier,
    pub last_payout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralLink {
    pub link_id: String,
    pub referrer_wallet: String,
    pub service_endpoint: String,
    pub custom_params: HashMap<String, String>,
    pub click_count: u32,
    pub conversion_count: u32,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionPayment {
    pub payment_id: String,
    pub recipient_wallet: String,
    pub amount: f64,
    pub token: String,
    pub commission_type: CommissionType,
    pub source_transaction: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferralStatus {
    Active,
    Inactive,
    Suspended,
    Graduated, // No longer needs referrer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EarningsTier {
    Bronze,   // 0-10 referrals
    Silver,   // 11-50 referrals
    Gold,     // 51-200 referrals
    Platinum, // 201+ referrals
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommissionType {
    SwapFee,
    ReferralBonus,
    ServiceFee,
    VolumeBonus,
}

impl PublicGateway {
    pub fn initialize_commission_system(&mut self) {
        self.commission_system = Some(CommissionSystem {
            referral_tracking: HashMap::new(),
            commission_rates: CommissionRates {
                swap_commission_percentage: 20.0,    // 20% of swap fees
                referral_commission_percentage: 10.0, // 10% of referee's fees
                service_commission_percentage: 5.0,   // 5% of service payments
                tier_multipliers: HashMap::from([
                    ("Bronze".to_string(), 1.0),
                    ("Silver".to_string(), 1.2),
                    ("Gold".to_string(), 1.5),
                    ("Platinum".to_string(), 2.0),
                ]),
            },
            earnings_ledger: HashMap::new(),
            referral_links: HashMap::new(),
            commission_history: HashMap::new(),
        });
    }

    pub fn create_referral_link(&mut self, referrer_wallet: &str, service_endpoint: &str,
                               custom_params: HashMap<String, String>) -> Result<String, String> {

        let link_id = format!("ref_{}_{}", referrer_wallet, chrono::Utc::now().timestamp());

        let referral_link = ReferralLink {
            link_id: link_id.clone(),
            referrer_wallet: referrer_wallet.to_string(),
            service_endpoint: service_endpoint.to_string(),
            custom_params,
            click_count: 0,
            conversion_count: 0,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        let commission_system = self.commission_system.as_mut()
            .ok_or("Commission system not initialized")?;

        commission_system.referral_links.insert(link_id.clone(), referral_link);

        // Generate referral URL
        let referral_url = format!("https://{}/{}?ref={}",
                                  self.domain, service_endpoint, link_id);

        println!("🔗 Referral link created: {} → {}", &referrer_wallet[..8], referral_url);

        Ok(referral_url)
    }

    pub fn track_referral(&mut self, referral_code: &str, referee_wallet: &str) -> Result<(), String> {
        let commission_system = self.commission_system.as_mut()
            .ok_or("Commission system not initialized")?;

        // Find referral link
        let referral_link = commission_system.referral_links.get_mut(referral_code)
            .ok_or("Invalid referral code")?;

        referral_link.click_count += 1;

        // Check if this is a new referral
        let referral_key = format!("{}_{}", referral_link.referrer_wallet, referee_wallet);

        if !commission_system.referral_tracking.contains_key(&referral_key) {
            let referral_record = ReferralRecord {
                referrer_wallet: referral_link.referrer_wallet.clone(),
                referee_wallet: referee_wallet.to_string(),
                referral_code: referral_code.to_string(),
                first_transaction_at: chrono::Utc::now().timestamp() as u64,
                total_volume: 0.0,
                total_commissions_earned: 0.0,
                status: ReferralStatus::Active,
            };

            commission_system.referral_tracking.insert(referral_key, referral_record);
            referral_link.conversion_count += 1;

            // Update referrer's earnings account
            self.update_earnings_account(&referral_link.referrer_wallet, 0.0, CommissionType::ReferralBonus)?;

            println!("👥 New referral tracked: {} → {}",
                     &referral_link.referrer_wallet[..8], &referee_wallet[..8]);
        }

        Ok(())
    }

    pub fn calculate_and_pay_commissions(&mut self, transaction_type: &str,
                                       transaction_amount: f64, fee_amount: f64,
                                       payer_wallet: &str, service_endpoint: &str) -> Result<(), String> {

        let commission_system = self.commission_system.as_mut()
            .ok_or("Commission system not initialized")?;

        // 1. Pay service endpoint owner (swap commission)
        if let Some(service) = self.service_registry.get(service_endpoint) {
            let swap_commission = fee_amount * commission_system.commission_rates.swap_commission_percentage / 100.0;

            self.pay_commission(&service.wallet_address, swap_commission,
                              CommissionType::SwapFee, transaction_type)?;
        }

        // 2. Pay referrer commission (if payer was referred)
        let referral_key_pattern = format!("_{}", payer_wallet);

        for (key, referral) in &mut commission_system.referral_tracking {
            if key.ends_with(&referral_key_pattern) && matches!(referral.status, ReferralStatus::Active) {
                let referral_commission = fee_amount * commission_system.commission_rates.referral_commission_percentage / 100.0;

                // Apply tier multiplier
                let earnings_account = commission_system.earnings_ledger
                    .get(&referral.referrer_wallet)
                    .cloned()
                    .unwrap_or_else(|| self.create_default_earnings_account(&referral.referrer_wallet));

                let tier_multiplier = commission_system.commission_rates.tier_multipliers
                    .get(&format!("{:?}", earnings_account.tier))
                    .unwrap_or(&1.0);

                let final_commission = referral_commission * tier_multiplier;

                self.pay_commission(&referral.referrer_wallet, final_commission,
                                  CommissionType::ReferralBonus, transaction_type)?;

                // Update referral stats
                referral.total_volume += transaction_amount;
                referral.total_commissions_earned += final_commission;

                break;
            }
        }

        // 3. Pay service usage commission (if different from swap)
        if transaction_type == "service_call" {
            if let Some(service) = self.service_registry.get(service_endpoint) {
                let service_commission = transaction_amount * commission_system.commission_rates.service_commission_percentage / 100.0;

                self.pay_commission(&service.wallet_address, service_commission,
                                  CommissionType::ServiceFee, transaction_type)?;
            }
        }

        Ok(())
    }

    fn pay_commission(&mut self, recipient_wallet: &str, amount: f64,
                     commission_type: CommissionType, source_tx: &str) -> Result<(), String> {

        let commission_system = self.commission_system.as_mut()
            .ok_or("Commission system not initialized")?;

        // Update earnings account
        self.update_earnings_account(recipient_wallet, amount, commission_type.clone())?;

        // Record commission payment
        let payment = CommissionPayment {
            payment_id: format!("comm_{}_{}", recipient_wallet, chrono::Utc::now().timestamp()),
            recipient_wallet: recipient_wallet.to_string(),
            amount,
            token: "USDC".to_string(), // Default to USDC
            commission_type,
            source_transaction: source_tx.to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        commission_system.commission_history
            .entry(recipient_wallet.to_string())
            .or_insert_with(Vec::new)
            .push(payment);

        println!("💰 Commission paid: {} USDC to {}", amount, &recipient_wallet[..8]);

        Ok(())
    }

    fn update_earnings_account(&mut self, wallet_address: &str, amount: f64,
                              commission_type: CommissionType) -> Result<(), String> {

        let commission_system = self.commission_system.as_mut()
            .ok_or("Commission system not initialized")?;

        let account = commission_system.earnings_ledger
            .entry(wallet_address.to_string())
            .or_insert_with(|| self.create_default_earnings_account(wallet_address));

        // Update earnings
        account.total_earned_usdc += amount;
        account.lifetime_volume += amount;

        // Update referral count and tier
        if matches!(commission_type, CommissionType::ReferralBonus) {
            account.referral_count += 1;
            account.tier = self.calculate_earnings_tier(account.referral_count);
        }

        account.last_payout = chrono::Utc::now().timestamp() as u64;

        Ok(())
    }

    fn create_default_earnings_account(&self, wallet_address: &str) -> EarningsAccount {
        EarningsAccount {
            wallet_address: wallet_address.to_string(),
            total_earned_usdc: 0.0,
            total_earned_solfunmeme: 0.0,
            pending_withdrawals: 0.0,
            lifetime_volume: 0.0,
            referral_count: 0,
            tier: EarningsTier::Bronze,
            last_payout: chrono::Utc::now().timestamp() as u64,
        }
    }

    fn calculate_earnings_tier(&self, referral_count: u32) -> EarningsTier {
        match referral_count {
            0..=10 => EarningsTier::Bronze,
            11..=50 => EarningsTier::Silver,
            51..=200 => EarningsTier::Gold,
            _ => EarningsTier::Platinum,
        }
    }

    pub fn get_earnings_dashboard(&self, wallet_address: &str) -> Result<String, String> {
        let commission_system = self.commission_system.as_ref()
            .ok_or("Commission system not initialized")?;

        let account = commission_system.earnings_ledger.get(wallet_address)
            .ok_or("Earnings account not found")?;

        let recent_payments = commission_system.commission_history
            .get(wallet_address)
            .map(|payments| payments.iter().rev().take(10).collect::<Vec<_>>())
            .unwrap_or_default();

        let referral_links = commission_system.referral_links.values()
            .filter(|link| link.referrer_wallet == wallet_address)
            .collect::<Vec<_>>();

        let dashboard = serde_json::json!({
            "wallet_address": wallet_address,
            "earnings": {
                "total_earned_usdc": account.total_earned_usdc,
                "total_earned_solfunmeme": account.total_earned_solfunmeme,
                "pending_withdrawals": account.pending_withdrawals,
                "lifetime_volume": account.lifetime_volume
            },
            "referrals": {
                "total_referrals": account.referral_count,
                "current_tier": account.tier,
                "tier_multiplier": commission_system.commission_rates.tier_multipliers
                    .get(&format!("{:?}", account.tier)).unwrap_or(&1.0)
            },
            "referral_links": referral_links.iter().map(|link| serde_json::json!({
                "link_id": link.link_id,
                "service_endpoint": link.service_endpoint,
                "clicks": link.click_count,
                "conversions": link.conversion_count,
                "conversion_rate": if link.click_count > 0 {
                    link.conversion_count as f64 / link.click_count as f64 * 100.0
                } else { 0.0 }
            })).collect::<Vec<_>>(),
            "recent_payments": recent_payments,
            "commission_rates": {
                "swap_commission": commission_system.commission_rates.swap_commission_percentage,
                "referral_commission": commission_system.commission_rates.referral_commission_percentage,
                "service_commission": commission_system.commission_rates.service_commission_percentage
            }
        });

        Ok(dashboard.to_string())
    }
}

// Add commission_system field to PublicGateway struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicGateway {
    pub domain: String,
    pub wallet_endpoints: HashMap<String, WalletEndpoint>,
    pub service_registry: HashMap<String, ServiceEndpoint>,
    pub payment_processor: PaymentProcessor,
    pub libp2p_bridge: LibP2PBridge,
    pub rate_limiter: RateLimiter,
    pub commission_system: Option<CommissionSystem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicGateway {
    pub domain: String,
    pub wallet_endpoints: HashMap<String, WalletEndpoint>,
    pub service_registry: HashMap<String, ServiceEndpoint>,
    pub payment_processor: PaymentProcessor,
    pub libp2p_bridge: LibP2PBridge,
    pub rate_limiter: RateLimiter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEndpoint {
    pub wallet_address: String,
    pub user_id: String,
    pub allocated_ports: Vec<u16>,
    pub services: HashMap<String, ServiceConfig>,
    pub payment_methods: Vec<PaymentMethod>,
    pub rate_limits: RateLimit,
    pub custom_domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub service_name: String,
    pub wallet_address: String,
    pub libp2p_port: u16,
    pub http_path: String,
    pub pricing: PricingConfig,
    pub payment_required: bool,
    pub cors_enabled: bool,
    pub auth_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProcessor {
    pub supported_tokens: Vec<TokenConfig>,
    pub swap_pools: HashMap<String, SwapPool>,
    pub payment_history: HashMap<String, Vec<PaymentRecord>>,
    pub quote_cache: HashMap<String, QuoteCache>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibP2PBridge {
    pub peer_connections: HashMap<String, PeerConnection>,
    pub protocol_handlers: HashMap<String, ProtocolHandler>,
    pub connection_pool: ConnectionPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub service_name: String,
    pub port: u16,
    pub pricing_tier: PricingTier,
    pub payment_required: bool,
    pub description: String,
    pub api_spec: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingConfig {
    pub base_price_usdc: f64,
    pub per_request_price: f64,
    pub per_mb_price: f64,
    pub per_second_price: f64,
    pub bulk_discounts: Vec<BulkDiscount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub symbol: String,
    pub contract_address: String,
    pub decimals: u8,
    pub is_stablecoin: bool,
    pub swap_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapPool {
    pub pool_id: String,
    pub token_a: String,
    pub token_b: String,
    pub liquidity: f64,
    pub fee_percentage: f64,
    pub price_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub payment_id: String,
    pub payer_wallet: String,
    pub amount: f64,
    pub token: String,
    pub service_endpoint: String,
    pub timestamp: u64,
    pub status: PaymentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteCache {
    pub from_token: String,
    pub to_token: String,
    pub amount: f64,
    pub quoted_price: f64,
    pub expires_at: u64,
    pub slippage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    USDC,
    USDT,
    SOL,
    SOLFUNMEME,
    Lightning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingTier {
    Free,
    Basic,      // $0.01 per request
    Premium,    // $0.10 per request
    Enterprise, // $1.00 per request
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub bandwidth_limit_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkDiscount {
    pub min_requests: u32,
    pub discount_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnection {
    pub peer_id: String,
    pub multiaddr: String,
    pub protocols: Vec<String>,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolHandler {
    pub protocol: String,
    pub handler_type: String,
    pub port_mapping: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPool {
    pub max_connections: u32,
    pub active_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiter {
    pub global_limits: RateLimit,
    pub per_wallet_limits: HashMap<String, RateLimit>,
    pub current_usage: HashMap<String, UsageStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub requests_this_minute: u32,
    pub requests_this_hour: u32,
    pub bandwidth_used_mb: f64,
    pub last_reset: u64,
}

impl PublicGateway {
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            wallet_endpoints: HashMap::new(),
            service_registry: HashMap::new(),
            payment_processor: PaymentProcessor {
                supported_tokens: vec![
                    TokenConfig {
                        symbol: "USDC".to_string(),
                        contract_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                        decimals: 6,
                        is_stablecoin: true,
                        swap_enabled: true,
                    },
                    TokenConfig {
                        symbol: "SOLFUNMEME".to_string(),
                        contract_address: "SoLFuNMeMeTokenAddress123456789".to_string(),
                        decimals: 9,
                        is_stablecoin: false,
                        swap_enabled: true,
                    },
                ],
                swap_pools: HashMap::new(),
                payment_history: HashMap::new(),
                quote_cache: HashMap::new(),
            },
            libp2p_bridge: LibP2PBridge {
                peer_connections: HashMap::new(),
                protocol_handlers: HashMap::new(),
                connection_pool: ConnectionPool {
                    max_connections: 1000,
                    active_connections: 0,
                    connection_timeout: 30000,
                },
            },
            rate_limiter: RateLimiter {
                global_limits: RateLimit {
                    requests_per_minute: 1000,
                    requests_per_hour: 10000,
                    bandwidth_limit_mbps: 100.0,
                },
                per_wallet_limits: HashMap::new(),
                current_usage: HashMap::new(),
            },
        }
    }

    pub fn register_wallet_endpoint(&mut self, wallet_address: &str, user_id: &str,
                                  allocated_ports: Vec<u16>) -> Result<String, String> {

        let endpoint = WalletEndpoint {
            wallet_address: wallet_address.to_string(),
            user_id: user_id.to_string(),
            allocated_ports,
            services: HashMap::new(),
            payment_methods: vec![PaymentMethod::USDC, PaymentMethod::SOLFUNMEME],
            rate_limits: RateLimit {
                requests_per_minute: 100,
                requests_per_hour: 1000,
                bandwidth_limit_mbps: 10.0,
            },
            custom_domain: None,
        };

        self.wallet_endpoints.insert(wallet_address.to_string(), endpoint);

        let public_url = format!("https://{}/{}", self.domain, wallet_address);

        println!("🌐 Public endpoint registered: {}", public_url);
        Ok(public_url)
    }

    pub fn add_service(&mut self, wallet_address: &str, service_name: &str,
                      libp2p_port: u16, pricing_tier: PricingTier) -> Result<String, String> {

        let wallet_endpoint = self.wallet_endpoints.get_mut(wallet_address)
            .ok_or("Wallet endpoint not found")?;

        if !wallet_endpoint.allocated_ports.contains(&libp2p_port) {
            return Err("Port not allocated to this wallet".to_string());
        }

        let pricing = match pricing_tier {
            PricingTier::Free => PricingConfig {
                base_price_usdc: 0.0,
                per_request_price: 0.0,
                per_mb_price: 0.0,
                per_second_price: 0.0,
                bulk_discounts: Vec::new(),
            },
            PricingTier::Basic => PricingConfig {
                base_price_usdc: 0.01,
                per_request_price: 0.01,
                per_mb_price: 0.001,
                per_second_price: 0.001,
                bulk_discounts: vec![
                    BulkDiscount { min_requests: 100, discount_percentage: 10.0 },
                    BulkDiscount { min_requests: 1000, discount_percentage: 25.0 },
                ],
            },
            PricingTier::Premium => PricingConfig {
                base_price_usdc: 0.10,
                per_request_price: 0.10,
                per_mb_price: 0.01,
                per_second_price: 0.01,
                bulk_discounts: vec![
                    BulkDiscount { min_requests: 50, discount_percentage: 15.0 },
                    BulkDiscount { min_requests: 500, discount_percentage: 30.0 },
                ],
            },
            PricingTier::Enterprise => PricingConfig {
                base_price_usdc: 1.00,
                per_request_price: 1.00,
                per_mb_price: 0.10,
                per_second_price: 0.10,
                bulk_discounts: vec![
                    BulkDiscount { min_requests: 10, discount_percentage: 20.0 },
                    BulkDiscount { min_requests: 100, discount_percentage: 40.0 },
                ],
            },
        };

        let service_endpoint = ServiceEndpoint {
            service_name: service_name.to_string(),
            wallet_address: wallet_address.to_string(),
            libp2p_port,
            http_path: format!("/{}/{}", wallet_address, service_name),
            pricing,
            payment_required: !matches!(pricing_tier, PricingTier::Free),
            cors_enabled: true,
            auth_required: false,
        };

        let service_config = ServiceConfig {
            service_name: service_name.to_string(),
            port: libp2p_port,
            pricing_tier,
            payment_required: service_endpoint.payment_required,
            description: format!("Service {} on port {}", service_name, libp2p_port),
            api_spec: None,
        };

        wallet_endpoint.services.insert(service_name.to_string(), service_config);

        let service_key = format!("{}_{}", wallet_address, service_name);
        self.service_registry.insert(service_key, service_endpoint);

        let service_url = format!("https://{}/{}/{}", self.domain, wallet_address, service_name);

        println!("🔗 Service endpoint created: {}", service_url);
        Ok(service_url)
    }

    pub fn handle_http_request(&mut self, path: &str, method: &str,
                              headers: &HashMap<String, String>,
                              body: &[u8]) -> Result<HttpResponse, String> {

        // Parse path: /{wallet}/{service} or /{wallet}/{service}/swap or /{wallet}/{service}/quote
        let path_parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        if path_parts.len() < 2 {
            return Err("Invalid path format. Expected: /{wallet}/{service}".to_string());
        }

        let wallet_address = path_parts[0];
        let service_name = path_parts[1];
        let action = path_parts.get(2).unwrap_or(&"");

        // Handle special endpoints
        match action {
            "swap" => return self.handle_swap_request(wallet_address, service_name, body),
            "quote" => return self.handle_quote_request(wallet_address, service_name, body),
            _ => {}
        }

        // Check rate limits
        self.check_rate_limits(wallet_address)?;

        // Find service
        let service_key = format!("{}_{}", wallet_address, service_name);
        let service = self.service_registry.get(&service_key)
            .ok_or("Service not found")?;

        // Check payment requirement
        if service.payment_required {
            let payment_header = headers.get("X-Payment-Token")
                .ok_or("Payment required. Include X-Payment-Token header")?;

            self.verify_payment(payment_header, &service.pricing)?;
        }

        // Forward to libp2p service
        let response = self.forward_to_libp2p(service, method, body)?;

        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::from([
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
            ]),
            body: response,
        })
    }

    pub fn handle_swap_request(&mut self, wallet_address: &str, service_name: &str,
                              body: &[u8]) -> Result<HttpResponse, String> {

        let swap_request: SwapRequest = serde_json::from_slice(body)
            .map_err(|e| format!("Invalid swap request: {}", e))?;

        // Find best swap pool
        let pool = self.find_best_swap_pool(&swap_request.from_token, &swap_request.to_token)?;

        // Calculate swap
        let output_amount = self.calculate_swap_output(&pool, swap_request.amount)?;

        // Execute swap (simplified)
        let swap_result = SwapResult {
            transaction_id: format!("tx_{}", chrono::Utc::now().timestamp()),
            input_amount: swap_request.amount,
            output_amount,
            price_impact: pool.price_impact,
            fee: swap_request.amount * pool.fee_percentage / 100.0,
            status: "completed".to_string(),
        };

        let response_body = serde_json::to_vec(&swap_result)
            .map_err(|e| format!("Failed to serialize response: {}", e))?;

        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::from([
                ("Content-Type".to_string(), "application/json".to_string()),
            ]),
            body: response_body,
        })
    }

    pub fn handle_quote_request(&mut self, wallet_address: &str, service_name: &str,
                               body: &[u8]) -> Result<HttpResponse, String> {

        let quote_request: QuoteRequest = serde_json::from_slice(body)
            .map_err(|e| format!("Invalid quote request: {}", e))?;

        // Check cache first
        let cache_key = format!("{}_{}_{}_{}",
                               quote_request.from_token, quote_request.to_token,
                               quote_request.amount, wallet_address);

        if let Some(cached_quote) = self.payment_processor.quote_cache.get(&cache_key) {
            if cached_quote.expires_at > chrono::Utc::now().timestamp() as u64 {
                let response_body = serde_json::to_vec(cached_quote)
                    .map_err(|e| format!("Failed to serialize cached quote: {}", e))?;

                return Ok(HttpResponse {
                    status_code: 200,
                    headers: HashMap::from([
                        ("Content-Type".to_string(), "application/json".to_string()),
                        ("X-Cache".to_string(), "HIT".to_string()),
                    ]),
                    body: response_body,
                });
            }
        }

        // Calculate fresh quote
        let pool = self.find_best_swap_pool(&quote_request.from_token, &quote_request.to_token)?;
        let output_amount = self.calculate_swap_output(&pool, quote_request.amount)?;

        let quote = QuoteCache {
            from_token: quote_request.from_token.clone(),
            to_token: quote_request.to_token.clone(),
            amount: quote_request.amount,
            quoted_price: output_amount,
            expires_at: chrono::Utc::now().timestamp() as u64 + 30, // 30 second expiry
            slippage: pool.price_impact,
        };

        // Cache the quote
        self.payment_processor.quote_cache.insert(cache_key, quote.clone());

        let response_body = serde_json::to_vec(&quote)
            .map_err(|e| format!("Failed to serialize quote: {}", e))?;

        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::from([
                ("Content-Type".to_string(), "application/json".to_string()),
                ("X-Cache".to_string(), "MISS".to_string()),
            ]),
            body: response_body,
        })
    }

    fn check_rate_limits(&mut self, wallet_address: &str) -> Result<(), String> {
        let current_time = chrono::Utc::now().timestamp() as u64;

        let usage = self.rate_limiter.current_usage
            .entry(wallet_address.to_string())
            .or_insert(UsageStats {
                requests_this_minute: 0,
                requests_this_hour: 0,
                bandwidth_used_mb: 0.0,
                last_reset: current_time,
            });

        // Reset counters if needed
        if current_time - usage.last_reset > 60 {
            usage.requests_this_minute = 0;
            usage.last_reset = current_time;
        }

        if current_time - usage.last_reset > 3600 {
            usage.requests_this_hour = 0;
        }

        // Check limits
        let limits = self.rate_limiter.per_wallet_limits
            .get(wallet_address)
            .unwrap_or(&self.rate_limiter.global_limits);

        if usage.requests_this_minute >= limits.requests_per_minute {
            return Err("Rate limit exceeded: too many requests per minute".to_string());
        }

        if usage.requests_this_hour >= limits.requests_per_hour {
            return Err("Rate limit exceeded: too many requests per hour".to_string());
        }

        // Increment counters
        usage.requests_this_minute += 1;
        usage.requests_this_hour += 1;

        Ok(())
    }

    fn verify_payment(&self, payment_token: &str, pricing: &PricingConfig) -> Result<(), String> {
        // Simplified payment verification
        // In real implementation, would verify blockchain transaction
        if payment_token.starts_with("pay_") && payment_token.len() > 10 {
            Ok(())
        } else {
            Err("Invalid payment token".to_string())
        }
    }

    fn forward_to_libp2p(&self, service: &ServiceEndpoint, method: &str, body: &[u8]) -> Result<Vec<u8>, String> {
        // Simplified libp2p forwarding
        // In real implementation, would use libp2p client to forward request
        let response = serde_json::json!({
            "service": service.service_name,
            "port": service.libp2p_port,
            "method": method,
            "response": "Service response from libp2p",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        serde_json::to_vec(&response)
            .map_err(|e| format!("Failed to serialize response: {}", e))
    }

    fn find_best_swap_pool(&self, from_token: &str, to_token: &str) -> Result<&SwapPool, String> {
        // Find pool with best liquidity and lowest fees
        self.payment_processor.swap_pools
            .values()
            .find(|pool| {
                (pool.token_a == from_token && pool.token_b == to_token) ||
                (pool.token_a == to_token && pool.token_b == from_token)
            })
            .ok_or("No swap pool found for token pair".to_string())
    }

    fn calculate_swap_output(&self, pool: &SwapPool, input_amount: f64) -> Result<f64, String> {
        // Simplified AMM calculation
        let fee = input_amount * pool.fee_percentage / 100.0;
        let amount_after_fee = input_amount - fee;
        let output = amount_after_fee * 0.98; // 2% slippage

        Ok(output)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct SwapRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: f64,
    pub slippage_tolerance: f64,
}

#[derive(Debug, Serialize)]
pub struct SwapResult {
    pub transaction_id: String,
    pub input_amount: f64,
    pub output_amount: f64,
    pub price_impact: f64,
    pub fee: f64,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct QuoteRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: f64,
}

// Example usage and routing
pub fn create_gateway_routes() -> String {
    r#"
🌐 ZOS Public Gateway Routes:

Base URL: https://node1.solfunmeme.com

Wallet Endpoints:
  GET  /{wallet}                    → Wallet info and services
  POST /{wallet}/register           → Register new service

Service Endpoints:
  GET    /{wallet}/{service}        → Call service (GET)
  POST   /{wallet}/{service}        → Call service (POST)
  PUT    /{wallet}/{service}        → Call service (PUT)
  DELETE /{wallet}/{service}        → Call service (DELETE)

Payment Endpoints:
  POST /{wallet}/{service}/swap     → Swap tokens
  GET  /{wallet}/{service}/quote    → Get swap quote
  POST /{wallet}/{service}/pay      → Process payment

Headers:
  X-Payment-Token: pay_abc123...    → Payment authorization
  X-Wallet-Address: 0x123...        → Caller wallet
  Content-Type: application/json    → Request format

HTTP Status Codes:
  200 OK                           → Success
  402 Payment Required             → Need payment
  429 Too Many Requests            → Rate limited
  404 Not Found                    → Service not found
  500 Internal Server Error        → Server error

Example Requests:

# Get service quote
curl https://node1.solfunmeme.com/0xABC.../ai-service/quote \
  -d '{"from_token":"SOLFUNMEME","to_token":"USDC","amount":100}'

# Execute paid service
curl https://node1.solfunmeme.com/0xABC.../ai-service \
  -H "X-Payment-Token: pay_xyz789..." \
  -d '{"prompt":"Generate code"}'

# Swap tokens
curl https://node1.solfunmeme.com/0xABC.../ai-service/swap \
  -d '{"from_token":"SOLFUNMEME","to_token":"USDC","amount":100,"slippage_tolerance":0.5}'
"#.to_string()
}
