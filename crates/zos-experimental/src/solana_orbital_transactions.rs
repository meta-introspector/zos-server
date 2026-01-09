// Solana Orbital Transactions - Model transactions as LMFDB orbits
use crate::lmfdb_orbits::*;
use crate::metameme_coin::*;
use serde::{Deserialize, Serialize};

/// Solana transaction as an orbital cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaOrbit {
    pub orbit_signature: String,
    pub level: u64,                    // LMFDB level (11, 23, 47, ...)
    pub cycle_index: u32,              // Position in orbit cycle
    pub orbital_energy: u64,           // Transaction fee as orbital energy
    pub payment_cycle: PaymentCycle,
    pub compute_units: u64,            // Solana compute units
    pub orbital_period: u32,           // How many transactions complete the cycle
}

/// Payment cycle within an orbit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCycle {
    pub cycle_id: String,
    pub participants: Vec<String>,     // Addresses in the cycle
    pub cycle_payments: Vec<CyclePayment>,
    pub total_orbital_energy: u64,
    pub cycle_eigenvalue: f64,
}

/// Individual payment within a cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclePayment {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub orbital_position: f64,         // Position in cycle (0.0 to 1.0)
    pub energy_contribution: u64,      // How much energy this payment adds
}

impl SolanaOrbit {
    /// Create orbit from Solana transaction parameters
    pub fn from_transaction(
        signature: &str,
        compute_units: u64,
        participants: Vec<String>,
        amounts: Vec<u64>,
    ) -> Result<Self, String> {
        // Determine LMFDB level based on complexity
        let level = Self::calculate_lmfdb_level(compute_units, participants.len());

        // Calculate orbital energy (transaction fee)
        let orbital_energy = Self::calculate_orbital_energy(compute_units, level);

        // Create payment cycle
        let payment_cycle = Self::create_payment_cycle(&participants, &amounts, orbital_energy)?;

        // Determine orbital period based on cycle complexity
        let orbital_period = Self::calculate_orbital_period(&payment_cycle);

        Ok(SolanaOrbit {
            orbit_signature: signature.to_string(),
            level,
            cycle_index: 1, // Start at first position
            orbital_energy,
            payment_cycle,
            compute_units,
            orbital_period,
        })
    }

    /// Calculate LMFDB level based on transaction complexity
    fn calculate_lmfdb_level(compute_units: u64, participant_count: usize) -> u64 {
        match (compute_units, participant_count) {
            (0..=1000, 1..=2) => 11,      // Simple transfers - Level 11
            (1001..=10000, 1..=5) => 23,  // Medium complexity - Level 23
            (10001..=100000, _) => 47,    // High complexity - Level 47
            (100001..=1000000, _) => 71,  // Very high complexity - Level 71
            _ => 97,                      // Extreme complexity - Level 97
        }
    }

    /// Calculate orbital energy (transaction fee) based on complexity
    fn calculate_orbital_energy(compute_units: u64, level: u64) -> u64 {
        // Base fee scales with LMFDB level
        let base_fee = match level {
            11 => 5000,    // 0.000005 SOL
            23 => 10000,   // 0.00001 SOL
            47 => 25000,   // 0.000025 SOL
            71 => 50000,   // 0.00005 SOL
            97 => 100000,  // 0.0001 SOL
            _ => 5000,
        };

        // Scale by compute units
        base_fee + (compute_units / 100)
    }

    /// Create payment cycle from participants and amounts
    fn create_payment_cycle(
        participants: &[String],
        amounts: &[u64],
        orbital_energy: u64,
    ) -> Result<PaymentCycle, String> {
        if participants.len() < 2 {
            return Err("Need at least 2 participants for a cycle".to_string());
        }

        let mut cycle_payments = Vec::new();
        let mut total_energy = 0;

        // Create payments between participants
        for i in 0..participants.len() {
            let from = &participants[i];
            let to = &participants[(i + 1) % participants.len()]; // Cycle back to start
            let amount = amounts.get(i).copied().unwrap_or(0);

            let orbital_position = i as f64 / participants.len() as f64;
            let energy_contribution = amount / 1000; // Energy proportional to amount

            cycle_payments.push(CyclePayment {
                from: from.clone(),
                to: to.clone(),
                amount,
                orbital_position,
                energy_contribution,
            });

            total_energy += energy_contribution;
        }

        // Add orbital energy to total
        total_energy += orbital_energy;

        // Calculate cycle eigenvalue
        let cycle_eigenvalue = Self::calculate_cycle_eigenvalue(&cycle_payments);

        Ok(PaymentCycle {
            cycle_id: format!("cycle_{}", participants.len()),
            participants: participants.to_vec(),
            cycle_payments,
            total_orbital_energy: total_energy,
            cycle_eigenvalue,
        })
    }

    /// Calculate eigenvalue for the payment cycle
    fn calculate_cycle_eigenvalue(payments: &[CyclePayment]) -> f64 {
        if payments.is_empty() {
            return 0.0;
        }

        // Create adjacency matrix for payment flow
        let n = payments.len();
        let mut matrix = vec![vec![0.0; n]; n];

        for (i, payment) in payments.iter().enumerate() {
            let j = (i + 1) % n; // Next position in cycle
            matrix[i][j] = payment.amount as f64;
        }

        // Calculate dominant eigenvalue (simplified)
        let trace: f64 = (0..n).map(|i| matrix[i][i]).sum();
        trace / n as f64
    }

    /// Calculate orbital period (how many transactions complete the cycle)
    fn calculate_orbital_period(cycle: &PaymentCycle) -> u32 {
        // Period based on number of participants and complexity
        let base_period = cycle.participants.len() as u32;
        let complexity_factor = if cycle.cycle_eigenvalue > 1000.0 { 2 } else { 1 };

        base_period * complexity_factor
    }

    /// Advance to next position in orbit
    pub fn advance_cycle(&mut self) -> bool {
        self.cycle_index += 1;

        if self.cycle_index > self.orbital_period {
            self.cycle_index = 1; // Reset to start of cycle
            true // Completed full orbit
        } else {
            false // Still in cycle
        }
    }

    /// Get current orbital position (0.0 to 1.0)
    pub fn orbital_position(&self) -> f64 {
        self.cycle_index as f64 / self.orbital_period as f64
    }

    /// Calculate orbital velocity (energy per cycle step)
    pub fn orbital_velocity(&self) -> f64 {
        self.orbital_energy as f64 / self.orbital_period as f64
    }

    /// Get orbit signature for LMFDB
    pub fn lmfdb_signature(&self) -> String {
        format!("{}.a{}", self.level, self.cycle_index)
    }

    /// Export orbit for blockchain analysis
    pub fn export_orbit(&self) -> String {
        format!(
            "SOLANA_ORBIT[L{}:C{}:E{}:P{}:λ{:.3}]",
            self.level,
            self.cycle_index,
            self.orbital_energy,
            self.orbital_period,
            self.payment_cycle.cycle_eigenvalue
        )
    }
}

/// Solana orbital transaction system
pub struct SolanaOrbitalSystem {
    active_orbits: Vec<SolanaOrbit>,
    completed_cycles: Vec<PaymentCycle>,
    total_orbital_energy: u64,
}

impl SolanaOrbitalSystem {
    pub fn new() -> Self {
        Self {
            active_orbits: Vec::new(),
            completed_cycles: Vec::new(),
            total_orbital_energy: 0,
        }
    }

    /// Create new orbital transaction
    pub fn create_orbital_transaction(
        &mut self,
        signature: &str,
        compute_units: u64,
        participants: Vec<String>,
        amounts: Vec<u64>,
    ) -> Result<SolanaOrbit, String> {
        let orbit = SolanaOrbit::from_transaction(signature, compute_units, participants, amounts)?;

        self.total_orbital_energy += orbit.orbital_energy;
        self.active_orbits.push(orbit.clone());

        println!("🌌 Created Solana orbit: {} (Level {}, Energy {})",
                orbit.orbit_signature, orbit.level, orbit.orbital_energy);

        Ok(orbit)
    }

    /// Advance all active orbits
    pub fn advance_all_orbits(&mut self) -> Vec<PaymentCycle> {
        let mut completed = Vec::new();

        for orbit in &mut self.active_orbits {
            if orbit.advance_cycle() {
                // Orbit completed full cycle
                completed.push(orbit.payment_cycle.clone());
                println!("✅ Completed orbital cycle: {}", orbit.orbit_signature);
            }
        }

        // Move completed cycles to history
        self.completed_cycles.extend(completed.clone());

        completed
    }

    /// Get orbital statistics
    pub fn orbital_stats(&self) -> String {
        let active_count = self.active_orbits.len();
        let completed_count = self.completed_cycles.len();
        let avg_energy = if active_count > 0 {
            self.total_orbital_energy / active_count as u64
        } else {
            0
        };

        format!("ORBITAL_STATS[Active:{}:Completed:{}:TotalEnergy:{}:AvgEnergy:{}]",
               active_count, completed_count, self.total_orbital_energy, avg_energy)
    }

    /// Get orbits by LMFDB level
    pub fn orbits_by_level(&self, level: u64) -> Vec<&SolanaOrbit> {
        self.active_orbits.iter()
            .filter(|orbit| orbit.level == level)
            .collect()
    }

    /// Calculate total system eigenvalue
    pub fn system_eigenvalue(&self) -> f64 {
        if self.active_orbits.is_empty() {
            return 0.0;
        }

        let total: f64 = self.active_orbits.iter()
            .map(|orbit| orbit.payment_cycle.cycle_eigenvalue)
            .sum();

        total / self.active_orbits.len() as f64
    }
}
