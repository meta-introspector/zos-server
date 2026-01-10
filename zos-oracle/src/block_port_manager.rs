use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPortManager {
    pub current_block: u64,
    pub block_duration_ms: u64,
    pub max_concurrent_users: u32,
    pub active_ports: HashMap<u16, UserPort>,
    pub port_marketplace: HashMap<String, PortListing>,
    pub user_sessions: HashMap<String, UserSession>,
    pub free_tier_services: Vec<FreeService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPort {
    pub port: u16,
    pub user_id: String,
    pub allocated_at_block: u64,
    pub expires_at_block: u64,
    pub service_type: ServiceType,
    pub shareable: bool,
    pub resellable: bool,
    pub shared_with: Vec<String>,
    pub usage_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortListing {
    pub listing_id: String,
    pub port: u16,
    pub seller_id: String,
    pub price_credits: u64,
    pub duration_blocks: u64,
    pub service_description: String,
    pub max_shares: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub current_port: Option<u16>,
    pub credits_spent: u64,
    pub services_used: Vec<String>,
    pub block_started: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    Compute,
    Storage,
    Network,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeService {
    pub name: String,
    pub code: String,
    pub description: String,
    pub max_execution_time_ms: u64,
    pub credits_cost: u64,
}

impl BlockPortManager {
    pub fn new(max_concurrent: u32) -> Self {
        let mut services = Vec::new();

        // Add fun free tier services
        services.push(FreeService {
            name: "pi_calculator".to_string(),
            code: r#"
fn calculate_pi(iterations: u32) -> f64 {
    let mut pi = 0.0;
    for i in 0..iterations {
        let term = 4.0 / (2.0 * i as f64 + 1.0);
        if i % 2 == 0 { pi += term; } else { pi -= term; }
    }
    pi
}
println!("π ≈ {:.10}", calculate_pi(1000000));
"#
            .to_string(),
            description: "Calculate π using Leibniz formula".to_string(),
            max_execution_time_ms: 5000,
            credits_cost: 1,
        });

        services.push(FreeService {
            name: "fibonacci_meme".to_string(),
            code: r#"
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0, 1 => 1,
        _ => fibonacci(n-1) + fibonacci(n-2)
    }
}
let fib = fibonacci(20);
println!("🐰 Fibonacci rabbits after 20 months: {} pairs", fib);
println!("📈 That's exponential growth! 🚀");
"#
            .to_string(),
            description: "Fibonacci sequence with rabbit meme".to_string(),
            max_execution_time_ms: 3000,
            credits_cost: 2,
        });

        services.push(FreeService {
            name: "prime_poetry".to_string(),
            code: r#"
fn is_prime(n: u32) -> bool {
    if n < 2 { return false; }
    for i in 2..=(n as f64).sqrt() as u32 {
        if n % i == 0 { return false; }
    }
    true
}
let primes: Vec<u32> = (2..100).filter(|&n| is_prime(n)).collect();
println!("🎭 Prime Poetry:");
println!("Two, three, five, seven,");
println!("Eleven, thirteen, seventeen,");
println!("Primes dance in mathematical heaven! ✨");
println!("Found {} primes under 100", primes.len());
"#
            .to_string(),
            description: "Prime numbers with poetic flair".to_string(),
            max_execution_time_ms: 2000,
            credits_cost: 1,
        });

        Self {
            current_block: 0,
            block_duration_ms: 400, // Solana block time
            max_concurrent_users,
            active_ports: HashMap::new(),
            port_marketplace: HashMap::new(),
            user_sessions: HashMap::new(),
            free_tier_services: services,
        }
    }

    pub fn advance_block(&mut self) {
        self.current_block += 1;

        // Expire old ports
        let expired_ports: Vec<u16> = self
            .active_ports
            .iter()
            .filter(|(_, port)| port.expires_at_block <= self.current_block)
            .map(|(port, _)| *port)
            .collect();

        for port in expired_ports {
            self.active_ports.remove(&port);
            println!("⏰ Port {} expired at block {}", port, self.current_block);
        }

        // Remove expired marketplace listings
        self.port_marketplace.retain(|_, listing| {
            let port_still_active = self.active_ports.contains_key(&listing.port);
            if !port_still_active {
                println!("🏪 Marketplace listing {} expired", listing.listing_id);
            }
            port_still_active
        });
    }

    pub fn allocate_user_port(
        &mut self,
        user_id: &str,
        service_type: ServiceType,
    ) -> Result<u16, String> {
        if self.active_ports.len() >= self.max_concurrent_users as usize {
            return Err("No available ports - server at capacity".to_string());
        }

        // Find available port in user range (20000-29999)
        let port = self.find_available_port(20000, 29999)?;

        let user_port = UserPort {
            port,
            user_id: user_id.to_string(),
            allocated_at_block: self.current_block,
            expires_at_block: self.current_block + 1, // One block duration
            service_type,
            shareable: true,
            resellable: true,
            shared_with: Vec::new(),
            usage_count: 0,
        };

        self.active_ports.insert(port, user_port);

        // Create user session
        let session = UserSession {
            user_id: user_id.to_string(),
            current_port: Some(port),
            credits_spent: 0,
            services_used: Vec::new(),
            block_started: self.current_block,
        };

        self.user_sessions.insert(user_id.to_string(), session);

        println!(
            "🔌 Port {} allocated to {} for block {}",
            port,
            &user_id[..8],
            self.current_block
        );
        Ok(port)
    }

    pub fn share_port(&mut self, owner_id: &str, share_with: &str) -> Result<(), String> {
        let session = self
            .user_sessions
            .get(owner_id)
            .ok_or("User session not found")?;

        let port = session.current_port.ok_or("No active port to share")?;

        let user_port = self.active_ports.get_mut(&port).ok_or("Port not found")?;

        if !user_port.shareable {
            return Err("Port is not shareable".to_string());
        }

        if user_port.shared_with.len() >= 5 {
            return Err("Maximum shares reached".to_string());
        }

        user_port.shared_with.push(share_with.to_string());

        println!(
            "🤝 Port {} shared with {} by {}",
            port,
            &share_with[..8],
            &owner_id[..8]
        );
        Ok(())
    }

    pub fn list_port_for_sale(
        &mut self,
        seller_id: &str,
        price_credits: u64,
        duration_blocks: u64,
        description: &str,
    ) -> Result<String, String> {
        let session = self
            .user_sessions
            .get(seller_id)
            .ok_or("User session not found")?;

        let port = session.current_port.ok_or("No active port to sell")?;

        let user_port = self.active_ports.get(port).ok_or("Port not found")?;

        if !user_port.resellable {
            return Err("Port is not resellable".to_string());
        }

        let listing_id = format!("listing_{}_{}", port, self.current_block);

        let listing = PortListing {
            listing_id: listing_id.clone(),
            port,
            seller_id: seller_id.to_string(),
            price_credits,
            duration_blocks,
            service_description: description.to_string(),
            max_shares: 3,
        };

        self.port_marketplace.insert(listing_id.clone(), listing);

        println!(
            "🏪 Port {} listed for sale: {} credits",
            port, price_credits
        );
        Ok(listing_id)
    }

    pub fn buy_port_access(&mut self, buyer_id: &str, listing_id: &str) -> Result<u16, String> {
        let listing = self
            .port_marketplace
            .get(listing_id)
            .ok_or("Listing not found")?
            .clone();

        // Check if buyer has enough credits (would integrate with user_fingerprint system)
        // For now, assume they do

        let port = listing.port;
        let user_port = self
            .active_ports
            .get_mut(&port)
            .ok_or("Port no longer available")?;

        if user_port.shared_with.len() >= listing.max_shares {
            return Err("Port at maximum capacity".to_string());
        }

        user_port.shared_with.push(buyer_id.to_string());

        println!(
            "💰 Port {} access sold to {} for {} credits",
            port,
            &buyer_id[..8],
            listing.price_credits
        );
        Ok(port)
    }

    pub fn execute_free_service(
        &mut self,
        user_id: &str,
        service_name: &str,
    ) -> Result<String, String> {
        let service = self
            .free_tier_services
            .iter()
            .find(|s| s.name == service_name)
            .ok_or("Service not found")?;

        // Check if user has active port or session
        let session = self.user_sessions.get_mut(user_id);
        if session.is_none() {
            return Err("No active session - allocate a port first".to_string());
        }

        let session = session.unwrap();

        // Simulate code execution
        println!("🚀 Executing {} for user {}", service_name, &user_id[..8]);
        println!("📝 Code:\n{}", service.code);

        // Simulate execution result
        let result = match service_name {
            "pi_calculator" => "π ≈ 3.1415926536".to_string(),
            "fibonacci_meme" => "🐰 Fibonacci rabbits after 20 months: 6765 pairs\n📈 That's exponential growth! 🚀".to_string(),
            "prime_poetry" => "🎭 Prime Poetry:\nTwo, three, five, seven,\nEleven, thirteen, seventeen,\nPrimes dance in mathematical heaven! ✨\nFound 25 primes under 100".to_string(),
            _ => "Service executed successfully".to_string(),
        };

        session.services_used.push(service_name.to_string());
        session.credits_spent += service.credits_cost;

        // Update port usage
        if let Some(port) = session.current_port {
            if let Some(user_port) = self.active_ports.get_mut(&port) {
                user_port.usage_count += 1;
            }
        }

        Ok(result)
    }

    pub fn get_marketplace_listings(&self) -> Vec<&PortListing> {
        self.port_marketplace.values().collect()
    }

    pub fn get_user_status(&self, user_id: &str) -> Result<String, String> {
        let session = self
            .user_sessions
            .get(user_id)
            .ok_or("User session not found")?;

        let port_info = if let Some(port) = session.current_port {
            if let Some(user_port) = self.active_ports.get(&port) {
                serde_json::json!({
                    "port": port,
                    "expires_at_block": user_port.expires_at_block,
                    "blocks_remaining": user_port.expires_at_block.saturating_sub(self.current_block),
                    "shared_with": user_port.shared_with.len(),
                    "usage_count": user_port.usage_count
                })
            } else {
                serde_json::json!(null)
            }
        } else {
            serde_json::json!(null)
        };

        let status = serde_json::json!({
            "current_block": self.current_block,
            "user_id": user_id,
            "active_port": port_info,
            "credits_spent": session.credits_spent,
            "services_used": session.services_used,
            "available_services": self.free_tier_services.iter().map(|s| &s.name).collect::<Vec<_>>(),
            "marketplace_listings": self.port_marketplace.len(),
            "server_capacity": format!("{}/{}", self.active_ports.len(), self.max_concurrent_users)
        });

        Ok(status.to_string())
    }

    fn find_available_port(&self, start: u16, end: u16) -> Result<u16, String> {
        for port in start..=end {
            if !self.active_ports.contains_key(&port) {
                return Ok(port);
            }
        }
        Err("No available ports in range".to_string())
    }
}
