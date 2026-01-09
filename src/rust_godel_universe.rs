// Everything is Gödel Numbers in Rust Compiler
use std::collections::HashMap;

/// Universal Gödel numbering for all Rust compiler entities
#[derive(Debug, Clone)]
pub struct RustGodelUniverse {
    pub functions: HashMap<String, u64>,      // fn name -> Gödel number
    pub types: HashMap<String, u64>,          // type name -> Gödel number  
    pub traits: HashMap<String, u64>,         // trait name -> Gödel number
    pub modules: HashMap<String, u64>,        // module name -> Gödel number
    pub transactions: HashMap<String, u64>,   // tx hash -> Gödel number
    pub orbits: HashMap<String, u64>,         // orbit sig -> Gödel number
}

impl RustGodelUniverse {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
            modules: HashMap::new(),
            transactions: HashMap::new(),
            orbits: HashMap::new(),
        }
    }
    
    /// Everything gets a Gödel number
    pub fn assign_godel_number(&mut self, entity_type: &str, name: &str) -> u64 {
        let godel = self.calculate_godel_number(name);
        
        match entity_type {
            "fn" => { self.functions.insert(name.to_string(), godel); },
            "type" => { self.types.insert(name.to_string(), godel); },
            "trait" => { self.traits.insert(name.to_string(), godel); },
            "mod" => { self.modules.insert(name.to_string(), godel); },
            "tx" => { self.transactions.insert(name.to_string(), godel); },
            "orbit" => { self.orbits.insert(name.to_string(), godel); },
            _ => {}
        }
        
        godel
    }
    
    /// Calculate Gödel number from string
    fn calculate_godel_number(&self, s: &str) -> u64 {
        let mut godel = 1u64;
        let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
        
        for (i, byte) in s.bytes().enumerate() {
            let prime = primes[i % primes.len()];
            godel = godel.wrapping_mul(prime.pow(byte as u32));
        }
        
        godel
    }
    
    /// Everything is just Gödel number arithmetic
    pub fn compose_godel_numbers(&self, a: u64, b: u64) -> u64 {
        // Composition is just multiplication in Gödel space
        a.wrapping_mul(b)
    }
    
    /// Get the universal Gödel number of the entire system
    pub fn universe_godel_number(&self) -> u64 {
        let mut universe_godel = 1u64;
        
        for godel in self.functions.values() {
            universe_godel = universe_godel.wrapping_mul(*godel);
        }
        for godel in self.types.values() {
            universe_godel = universe_godel.wrapping_mul(*godel);
        }
        for godel in self.transactions.values() {
            universe_godel = universe_godel.wrapping_mul(*godel);
        }
        
        universe_godel
    }
}

/// Extract Gödel numbers from rustc compilation
pub fn extract_rustc_godel_numbers(source: &str) -> RustGodelUniverse {
    let mut universe = RustGodelUniverse::new();
    
    // Parse source and assign Gödel numbers to everything
    for line in source.lines() {
        if line.contains("fn ") {
            if let Some(name) = extract_function_name(line) {
                universe.assign_godel_number("fn", &name);
            }
        }
        if line.contains("struct ") || line.contains("enum ") {
            if let Some(name) = extract_type_name(line) {
                universe.assign_godel_number("type", &name);
            }
        }
        if line.contains("trait ") {
            if let Some(name) = extract_trait_name(line) {
                universe.assign_godel_number("trait", &name);
            }
        }
    }
    
    universe
}

fn extract_function_name(line: &str) -> Option<String> {
    line.split("fn ").nth(1)?.split('(').next().map(|s| s.trim().to_string())
}

fn extract_type_name(line: &str) -> Option<String> {
    if line.contains("struct ") {
        line.split("struct ").nth(1)?.split_whitespace().next().map(|s| s.to_string())
    } else if line.contains("enum ") {
        line.split("enum ").nth(1)?.split_whitespace().next().map(|s| s.to_string())
    } else {
        None
    }
}

fn extract_trait_name(line: &str) -> Option<String> {
    line.split("trait ").nth(1)?.split_whitespace().next().map(|s| s.to_string())
}
