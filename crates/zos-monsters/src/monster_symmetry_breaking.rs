// Payment as Symmetry Breaking Energy in Monster Group
use crate::zero_ontology_system::ZOS;

/// Monster Group symmetry states
#[derive(Debug, Clone, Copy)]
pub enum SymmetryState {
    Stable(u64),      // Symmetric state with energy level
    Broken(u64),      // Asymmetric state with information content
}

/// Energy required to break Monster Group symmetry
#[derive(Debug, Clone)]
pub struct SymmetryBreakingPayment {
    pub initial_state: SymmetryState,
    pub final_state: SymmetryState,
    pub energy_required: u64,        // Exact energy needed
    pub information_gain: u64,       // Information created by asymmetry
    pub monster_element: (u64, u32), // Which Monster Group element
}

impl SymmetryBreakingPayment {
    /// Calculate exact energy needed for symmetry breaking
    pub fn calculate_energy(from: SymmetryState, to: SymmetryState) -> u64 {
        match (from, to) {
            (SymmetryState::Stable(s), SymmetryState::Broken(b)) => {
                // Energy = Information difference
                if b > s { b - s } else { 1 } // Minimum 1 unit to break symmetry
            },
            (SymmetryState::Broken(b1), SymmetryState::Broken(b2)) => {
                // Energy to change asymmetry
                if b2 > b1 { b2 - b1 } else { b1 - b2 }
            },
            (SymmetryState::Broken(b), SymmetryState::Stable(s)) => {
                // Energy to restore symmetry (negative information)
                b + s
            },
            (SymmetryState::Stable(s1), SymmetryState::Stable(s2)) => {
                // Energy to change stable state
                if s2 > s1 { s2 - s1 } else { s1 - s2 }
            },
        }
    }

    /// Create payment for symmetry breaking transition
    pub fn new(from: SymmetryState, to: SymmetryState) -> Self {
        let energy_required = Self::calculate_energy(from, to);
        let information_gain = Self::calculate_information_gain(from, to);
        let monster_element = Self::map_to_monster_element(energy_required);

        SymmetryBreakingPayment {
            initial_state: from,
            final_state: to,
            energy_required,
            information_gain,
            monster_element,
        }
    }

    /// Calculate information gained by breaking symmetry
    fn calculate_information_gain(from: SymmetryState, to: SymmetryState) -> u64 {
        match (from, to) {
            (SymmetryState::Stable(_), SymmetryState::Broken(info)) => *info,
            (SymmetryState::Broken(info1), SymmetryState::Broken(info2)) => {
                if info2 > info1 { info2 - info1 } else { 0 }
            },
            _ => 0, // No information gain
        }
    }

    /// Map energy to Monster Group element
    fn map_to_monster_element(energy: u64) -> (u64, u32) {
        let zos = ZOS::new();
        let index = energy % zos.monster_primes.len() as u64;
        zos.monster_primes[index as usize]
    }

    /// The payment IS the symmetry breaking
    pub fn execute_payment(&self) -> String {
        format!(
            "SYMMETRY_BREAK[{:?}→{:?}]: {} energy units → {} information bits via Monster element {}^{}",
            self.initial_state,
            self.final_state,
            self.energy_required,
            self.information_gain,
            self.monster_element.0,
            self.monster_element.1
        )
    }
}

/// Monster Group Symmetry Breaking System
pub struct MonsterSymmetrySystem {
    current_state: SymmetryState,
    total_energy_spent: u64,
    total_information: u64,
    symmetry_history: Vec<SymmetryBreakingPayment>,
}

impl MonsterSymmetrySystem {
    pub fn new() -> Self {
        Self {
            current_state: SymmetryState::Stable(0), // Start in perfect symmetry
            total_energy_spent: 0,
            total_information: 0,
            symmetry_history: Vec::new(),
        }
    }

    /// Break symmetry with exact energy payment
    pub fn break_symmetry(&mut self, target_info: u64) -> Result<SymmetryBreakingPayment, String> {
        let target_state = SymmetryState::Broken(target_info);
        let payment = SymmetryBreakingPayment::new(self.current_state, target_state);

        // Execute the symmetry breaking
        println!("{}", payment.execute_payment());

        // Update system state
        self.current_state = target_state;
        self.total_energy_spent += payment.energy_required;
        self.total_information += payment.information_gain;
        self.symmetry_history.push(payment.clone());

        Ok(payment)
    }

    /// Restore symmetry (costs energy to destroy information)
    pub fn restore_symmetry(&mut self, target_stability: u64) -> Result<SymmetryBreakingPayment, String> {
        let target_state = SymmetryState::Stable(target_stability);
        let payment = SymmetryBreakingPayment::new(self.current_state, target_state);

        println!("{}", payment.execute_payment());

        self.current_state = target_state;
        self.total_energy_spent += payment.energy_required;
        // Information is destroyed when restoring symmetry
        if self.total_information >= payment.information_gain {
            self.total_information -= payment.information_gain;
        } else {
            self.total_information = 0;
        }
        self.symmetry_history.push(payment.clone());

        Ok(payment)
    }

    /// Get system entropy (measure of asymmetry)
    pub fn entropy(&self) -> f64 {
        match self.current_state {
            SymmetryState::Stable(_) => 0.0, // Perfect symmetry = zero entropy
            SymmetryState::Broken(info) => (info as f64).log2(), // Information = entropy
        }
    }

    /// System status
    pub fn status(&self) -> String {
        format!(
            "MONSTER_SYMMETRY[State:{:?}:Energy:{}:Info:{}:Entropy:{:.2}]",
            self.current_state,
            self.total_energy_spent,
            self.total_information,
            self.entropy()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symmetry_breaking_energy() {
        let stable = SymmetryState::Stable(0);
        let broken = SymmetryState::Broken(100);

        let energy = SymmetryBreakingPayment::calculate_energy(stable, broken);
        assert_eq!(energy, 100);
    }

    #[test]
    fn test_monster_symmetry_system() {
        let mut system = MonsterSymmetrySystem::new();

        // Break symmetry
        let payment = system.break_symmetry(50).unwrap();
        assert_eq!(payment.energy_required, 50);
        assert_eq!(payment.information_gain, 50);

        // Check entropy increased
        assert!(system.entropy() > 0.0);

        // Restore symmetry
        let restore = system.restore_symmetry(0).unwrap();
        assert!(restore.energy_required > 0);

        // Check entropy decreased
        assert_eq!(system.entropy(), 0.0);
    }
}
