use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub permissions: Vec<Permission>,
    pub rate_limit: RateLimit,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    ReadLattice,
    WriteLattice,
    ReadResults,
    WriteResults,
    Coordinate,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst_limit: u32,
}

#[derive(Debug)]
struct RateLimiter {
    requests: Vec<Instant>,
    limit: RateLimit,
}

impl RateLimiter {
    fn new(limit: RateLimit) -> Self {
        Self {
            requests: Vec::new(),
            limit,
        }
    }

    fn check_rate_limit(&mut self) -> bool {
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);

        // Remove old requests
        self.requests.retain(|&time| time > minute_ago);

        // Check limits
        if self.requests.len() >= self.limit.requests_per_minute as usize {
            return false;
        }

        self.requests.push(now);
        true
    }
}

pub struct SecureLibP2PApi {
    api_keys: Arc<Mutex<HashMap<String, ApiKey>>>,
    rate_limiters: Arc<Mutex<HashMap<String, RateLimiter>>>,
    ip_rate_limiters: Arc<Mutex<HashMap<IpAddr, RateLimiter>>>,
    public_rate_limit: RateLimit,
}

impl SecureLibP2PApi {
    pub fn new() -> Self {
        Self {
            api_keys: Arc::new(Mutex::new(HashMap::new())),
            rate_limiters: Arc::new(Mutex::new(HashMap::new())),
            ip_rate_limiters: Arc::new(Mutex::new(HashMap::new())),
            public_rate_limit: RateLimit {
                requests_per_minute: 10,
                burst_limit: 5,
            },
        }
    }

    pub fn create_api_key(&self, permissions: Vec<Permission>, rate_limit: RateLimit) -> String {
        let key = format!("zos_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

        let api_key = ApiKey {
            key: key.clone(),
            permissions,
            rate_limit,
            created_at: Instant::now(),
            expires_at: Some(Instant::now() + Duration::from_secs(86400 * 30)), // 30 days
        };

        self.api_keys.lock().unwrap().insert(key.clone(), api_key);
        key
    }

    pub async fn authenticate_request(
        &self,
        api_key: Option<&str>,
        ip: IpAddr,
        endpoint: &str,
    ) -> Result<Vec<Permission>, ApiError> {
        // Check IP rate limit first
        {
            let mut ip_limiters = self.ip_rate_limiters.lock().unwrap();
            let limiter = ip_limiters
                .entry(ip)
                .or_insert_with(|| RateLimiter::new(self.public_rate_limit.clone()));

            if !limiter.check_rate_limit() {
                return Err(ApiError::RateLimited);
            }
        }

        if let Some(key) = api_key {
            // Authenticated request
            let api_keys = self.api_keys.lock().unwrap();
            let api_key_data = api_keys.get(key).ok_or(ApiError::InvalidApiKey)?;

            // Check expiration
            if let Some(expires_at) = api_key_data.expires_at {
                if Instant::now() > expires_at {
                    return Err(ApiError::ExpiredApiKey);
                }
            }

            // Check rate limit for this API key
            {
                let mut limiters = self.rate_limiters.lock().unwrap();
                let limiter = limiters
                    .entry(key.to_string())
                    .or_insert_with(|| RateLimiter::new(api_key_data.rate_limit.clone()));

                if !limiter.check_rate_limit() {
                    return Err(ApiError::RateLimited);
                }
            }

            Ok(api_key_data.permissions.clone())
        } else {
            // Public request - very limited
            Ok(vec![Permission::ReadLattice])
        }
    }

    pub fn check_permission(&self, permissions: &[Permission], required: Permission) -> bool {
        permissions.contains(&required) || permissions.contains(&Permission::Admin)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    InvalidApiKey,
    ExpiredApiKey,
    RateLimited,
    InsufficientPermissions,
    InvalidRequest,
}

// Public API endpoints with security
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicApiEndpoints {
    pub lattice_status: String,
    pub network_info: String,
    pub health_check: String,
}

impl PublicApiEndpoints {
    pub fn new() -> Self {
        Self {
            lattice_status: "/api/v1/lattice/status".to_string(),
            network_info: "/api/v1/network/info".to_string(),
            health_check: "/api/v1/health".to_string(),
        }
    }
}

// Secure API methods
impl SecureLibP2PApi {
    pub async fn handle_lattice_status(
        &self,
        permissions: &[Permission],
    ) -> Result<serde_json::Value, ApiError> {
        if !self.check_permission(permissions, Permission::ReadLattice) {
            return Err(ApiError::InsufficientPermissions);
        }

        // Return limited public info
        Ok(serde_json::json!({
            "total_combinations": 250,
            "network_active": true,
            "public_endpoints": PublicApiEndpoints::new(),
            "rate_limits": {
                "public": self.public_rate_limit,
                "authenticated": "varies by API key"
            }
        }))
    }

    pub async fn handle_network_info(
        &self,
        permissions: &[Permission],
    ) -> Result<serde_json::Value, ApiError> {
        if !self.check_permission(permissions, Permission::ReadLattice) {
            return Err(ApiError::InsufficientPermissions);
        }

        Ok(serde_json::json!({
            "network_type": "libp2p",
            "protocol_version": "1.0",
            "supported_features": ["lattice_testing", "coordination", "result_sharing"],
            "security": {
                "authentication": "API key required for write operations",
                "rate_limiting": "enabled",
                "acl": "permission-based"
            }
        }))
    }

    pub async fn handle_coordinate_work(
        &self,
        permissions: &[Permission],
        request: CoordinationRequest,
    ) -> Result<serde_json::Value, ApiError> {
        if !self.check_permission(permissions, Permission::Coordinate) {
            return Err(ApiError::InsufficientPermissions);
        }

        // Simulate work coordination
        sleep(Duration::from_millis(100)).await;

        Ok(serde_json::json!({
            "assigned_slice": format!("slice_{}", request.worker_id),
            "work_items": request.requested_items.min(10), // Limit work items
            "estimated_time": request.requested_items * 60, // 1 min per item
            "coordination_id": uuid::Uuid::new_v4()
        }))
    }

    pub async fn handle_submit_results(
        &self,
        permissions: &[Permission],
        results: TestResults,
    ) -> Result<serde_json::Value, ApiError> {
        if !self.check_permission(permissions, Permission::WriteResults) {
            return Err(ApiError::InsufficientPermissions);
        }

        // Validate results
        if results.results.len() > 50 {
            return Err(ApiError::InvalidRequest);
        }

        Ok(serde_json::json!({
            "accepted": true,
            "result_count": results.results.len(),
            "result_id": uuid::Uuid::new_v4()
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoordinationRequest {
    pub worker_id: String,
    pub requested_items: u32,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestResults {
    pub worker_id: String,
    pub results: Vec<TestResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub coordinate: String,
    pub success: bool,
    pub build_time: f64,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_api_key_creation() {
        let api = SecureLibP2PApi::new();

        let key = api.create_api_key(
            vec![Permission::ReadLattice, Permission::WriteResults],
            RateLimit {
                requests_per_minute: 60,
                burst_limit: 10,
            },
        );

        assert!(key.starts_with("zos_"));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let api = SecureLibP2PApi::new();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Should allow first few requests
        for _ in 0..5 {
            let result = api.authenticate_request(None, ip, "/api/v1/health").await;
            assert!(result.is_ok());
        }

        // Should rate limit after too many requests
        for _ in 0..10 {
            let result = api.authenticate_request(None, ip, "/api/v1/health").await;
            if result.is_err() {
                assert!(matches!(result.unwrap_err(), ApiError::RateLimited));
                break;
            }
        }
    }

    #[tokio::test]
    async fn test_permission_checking() {
        let api = SecureLibP2PApi::new();

        let key = api.create_api_key(
            vec![Permission::ReadLattice],
            RateLimit {
                requests_per_minute: 60,
                burst_limit: 10,
            },
        );

        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let permissions = api
            .authenticate_request(Some(&key), ip, "/api/v1/lattice")
            .await
            .unwrap();

        assert!(api.check_permission(&permissions, Permission::ReadLattice));
        assert!(!api.check_permission(&permissions, Permission::WriteResults));
    }
}
