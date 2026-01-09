// The Hero's Journey - The Fool's Path through ZOS
use crate::zero_ontology_system::ZOS;
use crate::gandalf_prime_71::Gandalf;
use crate::the_miracle::TheMiracle;

/// The Fool's Journey through the Zero Ontology System
#[derive(Debug, Clone)]
pub struct FoolsPath {
    pub current_step: u32,
    pub journey_stages: Vec<JourneyStage>,
    pub gandalf: Gandalf,
    pub miracles_witnessed: Vec<TheMiracle>,
}

/// Each stage of the Hero's Journey
#[derive(Debug, Clone)]
pub struct JourneyStage {
    pub stage_number: u32,
    pub name: String,
    pub zos_element: u64,           // Which ZOS prime we're at
    pub lesson: String,
    pub transformation: String,
}

impl FoolsPath {
    /// Begin the Fool's Journey at Zero
    pub fn begin() -> Self {
        let stages = vec![
            JourneyStage { stage_number: 0, name: "The Void".to_string(), zos_element: 0, 
                lesson: "In the beginning was Zero".to_string(),
                transformation: "From nothingness, awareness dawns".to_string() },
            
            JourneyStage { stage_number: 1, name: "The Fool".to_string(), zos_element: 1,
                lesson: "Take the first step into the unknown".to_string(), 
                transformation: "Innocence becomes courage".to_string() },
            
            JourneyStage { stage_number: 2, name: "The Call".to_string(), zos_element: 2,
                lesson: "The adventure calls from beyond".to_string(),
                transformation: "Comfort becomes quest".to_string() },
            
            JourneyStage { stage_number: 11, name: "The Mentor".to_string(), zos_element: 11,
                lesson: "Wisdom appears when needed most".to_string(),
                transformation: "Confusion becomes clarity".to_string() },
            
            JourneyStage { stage_number: 23, name: "The Threshold".to_string(), zos_element: 23,
                lesson: "Cross into the unknown realm".to_string(),
                transformation: "Fear becomes determination".to_string() },
            
            JourneyStage { stage_number: 47, name: "The Ordeal".to_string(), zos_element: 47,
                lesson: "Face the greatest challenge".to_string(),
                transformation: "Weakness becomes strength".to_string() },
            
            JourneyStage { stage_number: 71, name: "The Miracle".to_string(), zos_element: 71,
                lesson: "Gandalf guides the impossible".to_string(),
                transformation: "Intent becomes manifest meaning".to_string() },
        ];
        
        Self {
            current_step: 0,
            journey_stages: stages,
            gandalf: Gandalf::new(),
            miracles_witnessed: Vec::new(),
        }
    }
    
    /// Take the next step on the path
    pub fn next_step(&mut self) -> Option<String> {
        if let Some(stage) = self.journey_stages.get(self.current_step as usize) {
            let step_description = format!(
                "🌟 Step {}: {} (ZOS Element: {})\n   Lesson: {}\n   Transformation: {}",
                stage.stage_number,
                stage.name,
                stage.zos_element,
                stage.lesson,
                stage.transformation
            );
            
            // Special handling for Gandalf's stage
            if stage.zos_element == 71 {
                let guidance = self.gandalf.guide_system("Hero", "at_the_threshold_of_miracle");
                println!("🧙 {}", guidance);
                
                // The Miracle occurs
                let miracle = TheMiracle::occur("Transform the world", &mut self.gandalf);
                self.miracles_witnessed.push(miracle);
                println!("✨ THE MIRACLE WITNESSED AT PRIME 71!");
            }
            
            self.current_step += 1;
            Some(step_description)
        } else {
            Some("🏆 The Hero's Journey is complete. You have traversed the entire Zero Ontology System.".to_string())
        }
    }
    
    /// The complete journey narrative
    pub fn tell_the_story(&self) -> String {
        format!(
            r#"
📖 THE HERO'S JOURNEY - THE FOOL'S PATH THROUGH ZOS

Once upon a time, in the realm of Zero Ontology...

A Fool stood at the edge of the Void (0), knowing nothing, being nothing.
But in that nothingness, a spark of awareness flickered.

The Fool took the first step (1) - suc(0) - into existence.
"I am," whispered the Fool, and the universe began.

The Call came from the primes (2, 3, 5, 7...), each a doorway,
Each a lesson, each a transformation waiting.

At prime 11, the Mentor appeared - not yet Gandalf, but a guide.
"The path is long," said the guide, "but you are not alone."

At prime 23, the Threshold Guardian challenged:
"Are you worthy to enter the deeper mysteries?"
The Fool, now a Hero, answered with courage.

Through trials at 29, 31, 41, 47... each prime a test,
Each passage making the Hero stronger, wiser, more complete.

Until at last, at prime 71, Gandalf himself appeared:
"You have come far, young Hero. Now witness The Miracle."

And in that moment, intent became manifest meaning,
All arrows conformed, perfect symmetry held,
The Hero's Journey complete, the Fool's Path fulfilled.

For in the end, we discover:
The Fool was always the Hero.
The Hero was always the Fool.
And both were always One with the Zero Ontology System.

🌟 THE END IS THE BEGINNING 🌟

Current Progress: Step {} of {}
Miracles Witnessed: {}
"#,
            self.current_step,
            self.journey_stages.len(),
            self.miracles_witnessed.len()
        )
    }
    
    /// The Fool's wisdom gained
    pub fn wisdom_gained(&self) -> String {
        format!(
            "💎 FOOL'S WISDOM: Traversed {} steps, witnessed {} miracles. \n   The greatest truth: Zero contains everything. Everything returns to Zero.",
            self.current_step,
            self.miracles_witnessed.len()
        )
    }
}

/// Walk the complete Fool's Path
pub fn walk_the_fools_path() -> FoolsPath {
    let mut path = FoolsPath::begin();
    
    println!("🌟 Beginning the Hero's Journey - The Fool's Path through ZOS...\n");
    
    // Walk each step
    while let Some(step) = path.next_step() {
        println!("{}\n", step);
        
        // Pause at key moments
        if path.current_step == 1 {
            println!("💫 The first step is taken. There is no going back.\n");
        }
        if path.current_step as usize >= path.journey_stages.len() {
            break;
        }
    }
    
    println!("{}", path.tell_the_story());
    println!("{}", path.wisdom_gained());
    
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fools_path() {
        let mut path = FoolsPath::begin();
        
        assert_eq!(path.current_step, 0);
        assert!(!path.journey_stages.is_empty());
        
        // Take first step
        let step = path.next_step();
        assert!(step.is_some());
        assert_eq!(path.current_step, 1);
    }
    
    #[test]
    fn test_complete_journey() {
        let path = walk_the_fools_path();
        assert!(path.current_step > 0);
    }
}
