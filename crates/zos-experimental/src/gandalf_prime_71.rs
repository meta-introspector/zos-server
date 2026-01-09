// Gandalf at Prime 71 - The Mentor Guide for System Completeness
use crate::zero_ontology_system::ZOS;

/// Gandalf - The mentor at prime 71 in Monster Group
#[derive(Debug, Clone)]
pub struct Gandalf {
    pub prime: u64,              // Always 71
    pub wisdom_level: u64,       // Accumulated guidance given
    pub guidance_history: Vec<String>,
    pub completeness_proofs: Vec<String>,
}

impl Gandalf {
    /// Gandalf exists at prime 71 in Monster Group
    pub fn new() -> Self {
        Self {
            prime: 71,           // The mentor prime
            wisdom_level: 0,
            guidance_history: Vec::new(),
            completeness_proofs: Vec::new(),
        }
    }

    /// Gandalf provides guidance to make system complete
    pub fn guide_system(&mut self, system_name: &str, current_state: &str) -> String {
        let guidance = match current_state {
            "incomplete" => format!("You shall not pass without completeness! System '{}' needs the 71st element.", system_name),
            "unstable" => format!("A system, like a ring, must be forged in the fires of prime 71 to be stable, {}.", system_name),
            "asymmetric" => format!("Balance you seek, {}. The 71st prime brings harmony to chaos.", system_name),
            "symmetric" => format!("Perfect symmetry is death, {}. Prime 71 breaks the spell.", system_name),
            "lost" => format!("Not all who wander are lost, but {} needs the 71st guide.", system_name),
            _ => format!("I am Gandalf at prime 71. {} shall be complete.", system_name),
        };

        self.wisdom_level += 71; // Each guidance adds prime 71 wisdom
        self.guidance_history.push(guidance.clone());

        println!("🧙 Gandalf (Prime 71): {}", guidance);
        guidance
    }

    /// Prove system completeness with Gandalf's presence
    pub fn prove_completeness(&mut self, system: &str) -> String {
        let proof = format!(
            "COMPLETENESS_PROOF[{}]: ∃ Gandalf @ Prime 71 → System Complete ∎",
            system
        );

        self.completeness_proofs.push(proof.clone());

        println!("✨ Completeness Proven: {}", proof);
        proof
    }

    /// Check if system has Gandalf (prime 71)
    pub fn system_has_mentor(monster_primes: &[(u64, u32)]) -> bool {
        monster_primes.iter().any(|(prime, _)| *prime == 71)
    }

    /// Gandalf's wisdom about the system
    pub fn wisdom(&self) -> String {
        format!(
            "🧙 Gandalf's Wisdom: {} guidance given, {} systems proven complete. Prime 71 is the key to all completeness.",
            self.guidance_history.len(),
            self.completeness_proofs.len()
        )
    }

    /// The famous quote adapted for systems
    pub fn famous_quote(&self, system: &str) -> String {
        format!("A {} is never late, nor is it early. It arrives precisely when it has prime 71.", system)
    }
}

/// System Completeness Checker with Gandalf
pub struct SystemCompletenessChecker {
    gandalf: Gandalf,
    incomplete_systems: Vec<String>,
    complete_systems: Vec<String>,
}

impl SystemCompletenessChecker {
    pub fn new() -> Self {
        Self {
            gandalf: Gandalf::new(),
            incomplete_systems: Vec::new(),
            complete_systems: Vec::new(),
        }
    }

    /// Check if system is complete (has prime 71)
    pub fn check_completeness(&mut self, system_name: &str, has_prime_71: bool) -> bool {
        if has_prime_71 {
            let proof = self.gandalf.prove_completeness(system_name);
            self.complete_systems.push(system_name.to_string());
            println!("✅ System '{}' is COMPLETE (has Gandalf at prime 71)", system_name);
            true
        } else {
            let guidance = self.gandalf.guide_system(system_name, "incomplete");
            self.incomplete_systems.push(system_name.to_string());
            println!("❌ System '{}' is INCOMPLETE (missing prime 71)", system_name);
            false
        }
    }

    /// Add Gandalf (prime 71) to make system complete
    pub fn add_gandalf(&mut self, system_name: &str) -> String {
        let guidance = format!(
            "🧙 Adding Gandalf (Prime 71) to system '{}'. Now it shall be complete!",
            system_name
        );

        // Move from incomplete to complete
        if let Some(pos) = self.incomplete_systems.iter().position(|s| s == system_name) {
            self.incomplete_systems.remove(pos);
            self.complete_systems.push(system_name.to_string());
        }

        self.gandalf.prove_completeness(system_name);

        println!("{}", guidance);
        guidance
    }

    /// Status of all systems
    pub fn status(&self) -> String {
        format!(
            "SYSTEM_STATUS[Complete:{}:Incomplete:{}] - {}",
            self.complete_systems.len(),
            self.incomplete_systems.len(),
            self.gandalf.wisdom()
        )
    }
}

/// Check if ZOS has Gandalf
pub fn zos_has_gandalf() -> bool {
    let zos = ZOS::new();
    Gandalf::system_has_mentor(&zos.monster_primes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gandalf_at_prime_71() {
        let gandalf = Gandalf::new();
        assert_eq!(gandalf.prime, 71);
    }

    #[test]
    fn test_system_completeness() {
        let mut checker = SystemCompletenessChecker::new();

        // System without prime 71 is incomplete
        assert!(!checker.check_completeness("TestSystem", false));

        // Add Gandalf to make it complete
        checker.add_gandalf("TestSystem");

        // Now system should be complete
        assert_eq!(checker.complete_systems.len(), 1);
        assert_eq!(checker.incomplete_systems.len(), 0);
    }

    #[test]
    fn test_zos_has_gandalf() {
        // ZOS should have prime 71 (Gandalf)
        assert!(zos_has_gandalf());
    }
}
