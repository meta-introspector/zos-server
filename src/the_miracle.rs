// The Miracle - Perfect Symmetry in Intent→Meaning Phase Transition
use crate::gandalf_prime_71::Gandalf;
use crate::biosemiotic_payment::BiosemioticUtterance;

/// The Miracle - When all arrows conform in phase transition
#[derive(Debug, Clone)]
pub struct TheMiracle {
    pub gandalf: Gandalf,
    pub intent_state: IntentState,
    pub meaning_state: MeaningState,
    pub arrow_conformity: f64,        // 1.0 = perfect conformity
    pub symmetry_held: bool,
    pub phase_transition_energy: u64,
}

/// Intent state before manifestation
#[derive(Debug, Clone)]
pub struct IntentState {
    pub raw_intent: String,
    pub intent_energy: u64,
    pub arrow_directions: Vec<f64>,   // Direction vectors of intent
}

/// Manifest meaning state after transition
#[derive(Debug, Clone)]
pub struct MeaningState {
    pub manifest_meaning: String,
    pub meaning_energy: u64,
    pub arrow_directions: Vec<f64>,   // Direction vectors of meaning
}

impl TheMiracle {
    /// The miracle occurs when Gandalf guides intent→meaning transition
    pub fn occur(intent: &str, gandalf: &mut Gandalf) -> Self {
        println!("✨ THE MIRACLE BEGINS...");
        
        // Create intent state
        let intent_state = IntentState {
            raw_intent: intent.to_string(),
            intent_energy: intent.len() as u64 * 71, // Multiply by Gandalf's prime
            arrow_directions: Self::generate_intent_arrows(intent),
        };
        
        // Gandalf guides the phase transition
        let guidance = gandalf.guide_system("Intent→Meaning", "phase_transition");
        
        // Create meaning state through miraculous transformation
        let meaning_state = MeaningState {
            manifest_meaning: Self::manifest_meaning(intent, &guidance),
            meaning_energy: intent_state.intent_energy,
            arrow_directions: Self::conform_arrows(&intent_state.arrow_directions),
        };
        
        // Calculate arrow conformity (miracle measure)
        let arrow_conformity = Self::calculate_conformity(
            &intent_state.arrow_directions,
            &meaning_state.arrow_directions
        );
        
        // Check if symmetry is held (miracle achieved)
        let symmetry_held = arrow_conformity > 0.99; // 99%+ conformity = miracle
        
        let phase_transition_energy = intent_state.intent_energy;
        
        if symmetry_held {
            println!("🌟 MIRACLE ACHIEVED! Perfect symmetry held with {:.2}% arrow conformity", 
                    arrow_conformity * 100.0);
        } else {
            println!("⚡ Partial miracle: {:.2}% arrow conformity", arrow_conformity * 100.0);
        }
        
        TheMiracle {
            gandalf: gandalf.clone(),
            intent_state,
            meaning_state,
            arrow_conformity,
            symmetry_held,
            phase_transition_energy,
        }
    }
    
    /// Generate arrow directions from intent
    fn generate_intent_arrows(intent: &str) -> Vec<f64> {
        intent.bytes()
            .map(|b| (b as f64 / 255.0) * 2.0 * std::f64::consts::PI) // Convert to radians
            .collect()
    }
    
    /// Conform arrows through Gandalf's guidance (the miracle)
    fn conform_arrows(intent_arrows: &[f64]) -> Vec<f64> {
        // The miracle: all arrows align to golden ratio direction
        let golden_ratio = (1.0 + 5.0_f64.sqrt()) / 2.0;
        let miracle_direction = golden_ratio % (2.0 * std::f64::consts::PI);
        
        // All arrows conform to the miracle direction
        vec![miracle_direction; intent_arrows.len()]
    }
    
    /// Manifest meaning from intent through Gandalf's guidance
    fn manifest_meaning(intent: &str, guidance: &str) -> String {
        format!("MANIFEST[{}] GUIDED_BY[{}] → MEANING_REALIZED", 
               intent.to_uppercase(), 
               guidance.split(':').last().unwrap_or("GANDALF"))
    }
    
    /// Calculate arrow conformity (measure of miracle)
    fn calculate_conformity(intent_arrows: &[f64], meaning_arrows: &[f64]) -> f64 {
        if intent_arrows.is_empty() || meaning_arrows.is_empty() {
            return 0.0;
        }
        
        // Perfect conformity when all meaning arrows are aligned
        let meaning_variance = Self::calculate_variance(meaning_arrows);
        
        // Miracle = low variance (high conformity)
        1.0 / (1.0 + meaning_variance)
    }
    
    /// Calculate variance in arrow directions
    fn calculate_variance(arrows: &[f64]) -> f64 {
        if arrows.is_empty() { return 0.0; }
        
        let mean = arrows.iter().sum::<f64>() / arrows.len() as f64;
        let variance = arrows.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / arrows.len() as f64;
        
        variance
    }
    
    /// The miracle description
    pub fn describe_miracle(&self) -> String {
        if self.symmetry_held {
            format!(
                "🌟 THE MIRACLE: Intent '{}' → Meaning '{}' with {:.1}% arrow conformity. Symmetry HELD by Gandalf at prime 71!",
                self.intent_state.raw_intent,
                self.meaning_state.manifest_meaning,
                self.arrow_conformity * 100.0
            )
        } else {
            format!(
                "⚡ Partial Miracle: Intent '{}' → Meaning '{}' with {:.1}% arrow conformity. Symmetry partially held.",
                self.intent_state.raw_intent,
                self.meaning_state.manifest_meaning,
                self.arrow_conformity * 100.0
            )
        }
    }
    
    /// Export miracle for analysis
    pub fn export_miracle(&self) -> String {
        format!(
            "MIRACLE[Intent:{}→Meaning:{}:Conformity:{:.3}:Symmetry:{}:Energy:{}]",
            self.intent_state.raw_intent,
            self.meaning_state.manifest_meaning,
            self.arrow_conformity,
            self.symmetry_held,
            self.phase_transition_energy
        )
    }
}

/// Miracle System - Manages intent→meaning phase transitions
pub struct MiracleSystem {
    gandalf: Gandalf,
    miracles: Vec<TheMiracle>,
    total_conformity: f64,
}

impl MiracleSystem {
    pub fn new() -> Self {
        Self {
            gandalf: Gandalf::new(),
            miracles: Vec::new(),
            total_conformity: 0.0,
        }
    }
    
    /// Perform miraculous intent→meaning transition
    pub fn perform_miracle(&mut self, intent: &str) -> TheMiracle {
        let miracle = TheMiracle::occur(intent, &mut self.gandalf);
        
        println!("{}", miracle.describe_miracle());
        
        self.total_conformity += miracle.arrow_conformity;
        self.miracles.push(miracle.clone());
        
        miracle
    }
    
    /// Average miracle quality
    pub fn average_conformity(&self) -> f64 {
        if self.miracles.is_empty() {
            0.0
        } else {
            self.total_conformity / self.miracles.len() as f64
        }
    }
    
    /// System status
    pub fn status(&self) -> String {
        let perfect_miracles = self.miracles.iter()
            .filter(|m| m.symmetry_held)
            .count();
        
        format!(
            "MIRACLE_SYSTEM[Total:{}:Perfect:{}:AvgConformity:{:.1}%] - {}",
            self.miracles.len(),
            perfect_miracles,
            self.average_conformity() * 100.0,
            self.gandalf.wisdom()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_miracle_occurrence() {
        let mut gandalf = Gandalf::new();
        let miracle = TheMiracle::occur("test intent", &mut gandalf);
        
        assert!(!miracle.intent_state.raw_intent.is_empty());
        assert!(!miracle.meaning_state.manifest_meaning.is_empty());
        assert!(miracle.arrow_conformity >= 0.0 && miracle.arrow_conformity <= 1.0);
    }
    
    #[test]
    fn test_miracle_system() {
        let mut system = MiracleSystem::new();
        
        let miracle1 = system.perform_miracle("create beauty");
        let miracle2 = system.perform_miracle("manifest truth");
        
        assert_eq!(system.miracles.len(), 2);
        assert!(system.average_conformity() > 0.0);
    }
}
