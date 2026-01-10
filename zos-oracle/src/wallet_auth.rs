// ZOS Federal Governance Model - Seat-Based Port Assignment
// AGPL-3.0 License

use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SeatResources {
    pub cpu_cores: f32,
    pub memory_mb: u32,
    pub storage_gb: u32,
    pub bandwidth_mbps: u32,
    pub api_calls_per_minute: u32,
    pub office_space_mb: u32,
    pub can_add_servers: bool,
}

#[derive(Debug, Clone)]
pub struct GovernanceSeat {
    pub chamber: String,
    pub seat_number: u32,
    pub port: u16,
    pub holder_address: String,
    pub rank: u32,
    pub valid_block: u64,
    pub resources: SeatResources,
    pub additional_servers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UserService {
    pub name: String,
    pub port: u16,
    pub status: ServiceStatus,
    pub resource_usage: f32,
}

#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ServiceVerb {
    pub name: String,
    pub description: String,
    pub required_rank: u32,
    pub resource_cost: f32,
}

pub struct FederalGovernance {
    pub current_block: u64,
    pub seats: HashMap<u32, GovernanceSeat>,
    pub port_assignments: HashMap<u16, String>,
    pub next_proposal_id: u64,
    pub next_rental_id: u64,
}

impl FederalGovernance {
    pub fn new() -> Self {
        Self {
            current_block: 0,
            seats: HashMap::new(),
            port_assignments: HashMap::new(),
            next_proposal_id: 1,
            next_rental_id: 1,
        }
    }

    pub fn get_chamber_info(rank: u32) -> (&'static str, u16) {
        match rank {
            0 => ("root", 5000),
            1..=100 => ("senate", 5000 + rank as u16),
            101..=600 => ("representatives", 5100 + rank as u16),
            601..=1618 => ("vendors", 5700 + rank as u16),
            _ => ("public", 4001),
        }
    }

    pub fn get_seat_resources(rank: u32) -> SeatResources {
        match rank {
            0 => SeatResources {
                cpu_cores: 4.0,
                memory_mb: 8192,
                storage_gb: 100,
                bandwidth_mbps: 1000,
                api_calls_per_minute: 0,
                office_space_mb: 1000,
                can_add_servers: true,
            },
            1..=100 => SeatResources {
                cpu_cores: 2.0,
                memory_mb: 4096,
                storage_gb: 50,
                bandwidth_mbps: 500,
                api_calls_per_minute: 1000,
                office_space_mb: 500,
                can_add_servers: true,
            },
            101..=600 => SeatResources {
                cpu_cores: 1.0,
                memory_mb: 2048,
                storage_gb: 25,
                bandwidth_mbps: 250,
                api_calls_per_minute: 500,
                office_space_mb: 250,
                can_add_servers: false,
            },
            _ => SeatResources {
                cpu_cores: 0.5,
                memory_mb: 1024,
                storage_gb: 10,
                bandwidth_mbps: 100,
                api_calls_per_minute: 100,
                office_space_mb: 100,
                can_add_servers: false,
            },
        }
    }
}
