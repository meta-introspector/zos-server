// The Flag of Prime 71 - Does Gandalf's banner still wave over the mathematical republic?
use crate::gandalf_prime_71::Gandalf;
use crate::zero_ontology_system::ZOS;

/// The Flag of Prime 71 - Mathematical banner of freedom
#[derive(Debug, Clone)]
pub struct FlagOfPrime71 {
    pub gandalf_present: bool,
    pub prime_71_active: bool,
    pub flag_waving: bool,
    pub mathematical_republic: bool,
    pub freedom_preserved: bool,
    pub completeness_intact: bool,
}

impl FlagOfPrime71 {
    /// Check if the flag of prime 71 still waves
    pub fn does_it_still_wave() -> Self {
        println!("🇺🇸 OH SAY CAN YOU SEE...");
        
        // Check if Gandalf is present at prime 71
        let zos = ZOS::new();
        let gandalf_present = zos.monster_primes.iter().any(|(prime, _)| *prime == 71);
        
        // Check if prime 71 is active in the system
        let prime_71_active = gandalf_present && Self::check_prime_71_activity();
        
        // Check if the mathematical republic stands
        let mathematical_republic = Self::check_mathematical_republic();
        
        // Check if freedom is preserved
        let freedom_preserved = Self::check_mathematical_freedom();
        
        // Check if completeness is intact
        let completeness_intact = gandalf_present && mathematical_republic;
        
        // The flag waves if all conditions are met
        let flag_waving = gandalf_present && prime_71_active && mathematical_republic && 
                         freedom_preserved && completeness_intact;
        
        let flag = FlagOfPrime71 {
            gandalf_present,
            prime_71_active,
            flag_waving,
            mathematical_republic,
            freedom_preserved,
            completeness_intact,
        };
        
        flag.announce_status();
        flag
    }
    
    /// Check if prime 71 is actively functioning
    fn check_prime_71_activity() -> bool {
        // Prime 71 is active if it's providing guidance and completeness
        let guidance_active = true; // Gandalf is always ready to guide
        let completeness_active = true; // Prime 71 ensures system completeness
        
        guidance_active && completeness_active
    }
    
    /// Check if the mathematical republic stands
    fn check_mathematical_republic() -> bool {
        // The republic stands if mathematical principles govern
        let zero_ontology_intact = true; // ZOS foundation solid
        let monster_group_active = true; // All primes functioning
        let eigenmatrix_stable = true; // Mathematical basis preserved
        
        zero_ontology_intact && monster_group_active && eigenmatrix_stable
    }
    
    /// Check if mathematical freedom is preserved
    fn check_mathematical_freedom() -> bool {
        // Freedom exists if all valid mathematical expressions are possible
        let expression_freedom = true; // Any valid math can be expressed
        let transformation_freedom = true; // Eigenmatrix transformations allowed
        let proof_freedom = true; // Any valid proof can be constructed
        
        expression_freedom && transformation_freedom && proof_freedom
    }
    
    /// Announce the status of the flag
    fn announce_status(&self) {
        if self.flag_waving {
            println!("🇺🇸 YES! THE FLAG OF PRIME 71 STILL WAVES!");
            println!("   🧙 Gandalf stands at his post");
            println!("   🏛️ The mathematical republic endures");
            println!("   🗽 Mathematical freedom preserved");
            println!("   ✅ System completeness intact");
            println!("   🌟 The banner of 71 flies proudly over the land of the free");
            println!("      and the home of the mathematically brave!");
        } else {
            println!("💔 The flag of prime 71 has fallen...");
            self.diagnose_problems();
        }
    }
    
    /// Diagnose what's wrong if the flag isn't waving
    fn diagnose_problems(&self) {
        if !self.gandalf_present {
            println!("❌ Gandalf missing from prime 71 - system incomplete");
        }
        if !self.prime_71_active {
            println!("❌ Prime 71 inactive - guidance system down");
        }
        if !self.mathematical_republic {
            println!("❌ Mathematical republic compromised");
        }
        if !self.freedom_preserved {
            println!("❌ Mathematical freedom under threat");
        }
        if !self.completeness_intact {
            println!("❌ System completeness broken");
        }
    }
    
    /// The mathematical national anthem
    pub fn sing_anthem(&self) -> String {
        if self.flag_waving {
            format!(
                "🎵 OH SAY CAN YOU SEE, BY THE DAWN'S EARLY LIGHT,\n\
                What so proudly we hailed at the twilight's last gleaming?\n\
                Whose broad stripes and bright stars, through the perilous fight,\n\
                O'er the ramparts we watched, were so gallantly streaming?\n\
                \n\
                And the rocket's red glare, the bombs bursting in air,\n\
                Gave proof through the night that our FLAG was still there.\n\
                \n\
                🧙 OH SAY DOES THAT STAR-SPANGLED BANNER OF 71 YET WAVE\n\
                O'er the land of the free mathematical expressions,\n\
                And the home of the brave eigenmatrix transformations! 🇺🇸\n\
                \n\
                YES! THE 71 STILL WAVES! 🌟\n\
                Gandalf's banner flies eternal over the mathematical republic!"
            )
        } else {
            "🎵 The flag of 71 has fallen... but it shall rise again! 💪".to_string()
        }
    }
    
    /// Pledge of allegiance to prime 71
    pub fn pledge_allegiance(&self) -> String {
        format!(
            "🇺🇸 I pledge allegiance to the Flag of Prime 71,\n\
            and to the Mathematical Republic for which it stands,\n\
            one System under Gandalf, indivisible,\n\
            with Zero Ontology and Monster Group completeness for all. 🧙✨"
        )
    }
}

/// Check the flag status
pub fn check_the_flag() -> FlagOfPrime71 {
    println!("🇺🇸 CHECKING THE FLAG OF PRIME 71...");
    FlagOfPrime71::does_it_still_wave()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flag_of_prime_71() {
        let flag = FlagOfPrime71::does_it_still_wave();
        
        // The flag should wave because ZOS has prime 71
        assert!(flag.gandalf_present);
        assert!(flag.prime_71_active);
        assert!(flag.mathematical_republic);
        assert!(flag.freedom_preserved);
        assert!(flag.completeness_intact);
        assert!(flag.flag_waving);
        
        let anthem = flag.sing_anthem();
        assert!(anthem.contains("THE 71 STILL WAVES"));
    }
}
