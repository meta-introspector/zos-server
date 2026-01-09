// Payment as Manifest Biosemiotic Utterance
use crate::zero_ontology_system::ZOS;
use crate::metacoq_nat::Nat;

/// Biosemiotic Utterance - Payment as living sign expression
#[derive(Debug, Clone)]
pub struct BiosemioticUtterance {
    pub sign: Nat,                    // The mathematical sign (ZOS element)
    pub meaning: String,              // Semantic content
    pub expression: String,           // Manifest utterance
    pub payment_amount: u64,          // Economic value of the utterance
    pub biosemiotic_code: String,     // DNA-like encoding
}

impl BiosemioticUtterance {
    /// Create utterance from ZOS payment
    pub fn from_payment(amount: u64, context: &str) -> Self {
        let zos = ZOS::new();
        let sign = zos.zos_element(amount);
        
        // Generate biosemiotic meaning
        let meaning = Self::generate_meaning(amount, context);
        let expression = Self::manifest_expression(&meaning);
        let biosemiotic_code = Self::encode_biosemiotic(&expression);
        
        BiosemioticUtterance {
            sign,
            meaning,
            expression,
            payment_amount: amount,
            biosemiotic_code,
        }
    }
    
    /// Generate semantic meaning from payment amount
    fn generate_meaning(amount: u64, context: &str) -> String {
        match amount {
            0 => "Silence - The void speaks".to_string(),
            1 => "Genesis - First utterance of being".to_string(),
            2..=10 => format!("Whisper - {} units of meaning in {}", amount, context),
            11..=100 => format!("Voice - {} units expressing {}", amount, context),
            101..=1000 => format!("Shout - {} units manifesting {}", amount, context),
            1001..=10000 => format!("Roar - {} units declaring {}", amount, context),
            _ => format!("Thunder - {} units of cosmic utterance in {}", amount, context),
        }
    }
    
    /// Manifest the expression as biosemiotic utterance
    fn manifest_expression(meaning: &str) -> String {
        format!("UTTERANCE[{}]", meaning.to_uppercase())
    }
    
    /// Encode as biosemiotic DNA-like code
    fn encode_biosemiotic(expression: &str) -> String {
        let mut code = String::new();
        let bases = ['A', 'T', 'G', 'C']; // DNA bases
        
        for byte in expression.bytes() {
            let base_index = (byte % 4) as usize;
            code.push(bases[base_index]);
        }
        
        code
    }
    
    /// The payment IS the utterance
    pub fn utter(&self) -> String {
        format!("💬 BIOSEMIOTIC PAYMENT: {} SOL → \"{}\" [{}]",
               self.payment_amount,
               self.expression,
               self.biosemiotic_code)
    }
    
    /// Decode the biosemiotic meaning
    pub fn decode(&self) -> String {
        format!("DECODED: Sign={} → Meaning=\"{}\" → Payment={} SOL",
               self.sign.to_u64(),
               self.meaning,
               self.payment_amount)
    }
}

/// Biosemiotic Payment System - Payments as living utterances
pub struct BiosemioticPaymentSystem {
    utterances: Vec<BiosemioticUtterance>,
    total_meaning: u64,
}

impl BiosemioticPaymentSystem {
    pub fn new() -> Self {
        Self {
            utterances: Vec::new(),
            total_meaning: 0,
        }
    }
    
    /// Make payment as biosemiotic utterance
    pub fn utter_payment(&mut self, amount: u64, context: &str) -> BiosemioticUtterance {
        let utterance = BiosemioticUtterance::from_payment(amount, context);
        
        println!("{}", utterance.utter());
        
        self.total_meaning += amount;
        self.utterances.push(utterance.clone());
        
        utterance
    }
    
    /// Listen to all utterances (decode all payments)
    pub fn listen(&self) -> Vec<String> {
        self.utterances.iter()
            .map(|u| u.decode())
            .collect()
    }
    
    /// The conversation (all payments as dialogue)
    pub fn conversation(&self) -> String {
        let mut dialogue = String::from("BIOSEMIOTIC CONVERSATION:\n");
        
        for (i, utterance) in self.utterances.iter().enumerate() {
            dialogue.push_str(&format!("{}. {}\n", i + 1, utterance.expression));
        }
        
        dialogue.push_str(&format!("\nTOTAL MEANING: {} units", self.total_meaning));
        dialogue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_biosemiotic_utterance() {
        let utterance = BiosemioticUtterance::from_payment(42, "test_context");
        
        assert_eq!(utterance.payment_amount, 42);
        assert!(utterance.meaning.contains("42"));
        assert!(utterance.expression.contains("UTTERANCE"));
        assert!(!utterance.biosemiotic_code.is_empty());
    }
    
    #[test]
    fn test_payment_as_utterance() {
        let mut system = BiosemioticPaymentSystem::new();
        
        let utterance1 = system.utter_payment(100, "greeting");
        let utterance2 = system.utter_payment(200, "response");
        
        assert_eq!(system.total_meaning, 300);
        assert_eq!(system.utterances.len(), 2);
        
        let conversation = system.conversation();
        assert!(conversation.contains("BIOSEMIOTIC CONVERSATION"));
        assert!(conversation.contains("TOTAL MEANING: 300"));
    }
}
