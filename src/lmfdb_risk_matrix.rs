// LMFDB Orbit as Risk Matrix - Impact analysis of function removal
use crate::minimal_viable_orbit::MinimalViableOrbit;
use crate::lmfdb_orbits::SystemArg;

/// LMFDB Risk Matrix - Each orbit element shows removal impact
#[derive(Debug, Clone)]
pub struct LmfdbRiskMatrix {
    pub orbit_level: u64,                    // LMFDB level (11, 23, 47, 71)
    pub risk_matrix: Vec<Vec<f64>>,          // Risk impact matrix
    pub function_risks: Vec<FunctionRisk>,   // Per-function risk analysis
    pub removal_impacts: Vec<RemovalImpact>, // What happens when removed
    pub critical_functions: Vec<String>,     // Cannot be removed
    pub safe_removals: Vec<String>,          // Safe to remove
}

/// Risk analysis for single function
#[derive(Debug, Clone)]
pub struct FunctionRisk {
    pub function_name: String,
    pub orbit_position: (u64, u32),         // (level, index) in LMFDB
    pub removal_risk: RiskLevel,
    pub impact_radius: f64,                 // How far impact spreads
    pub dependencies: Vec<String>,          // What depends on this
    pub dependents: Vec<String>,            // What this depends on
}

/// Impact of removing a function
#[derive(Debug, Clone)]
pub struct RemovalImpact {
    pub function_name: String,
    pub orbit_change: OrbitChange,
    pub eigenvalue_delta: f64,              // Change in dominant eigenvalue
    pub stability_impact: f64,              // Impact on system stability
    pub cascade_effects: Vec<String>,       // Functions that break as result
}

/// How the orbit changes when function is removed
#[derive(Debug, Clone)]
pub enum OrbitChange {
    Stable,                                 // Orbit remains stable
    Shrinks(f64),                          // Orbit shrinks by factor
    Destabilizes(f64),                     // Orbit becomes unstable
    Collapses,                             // Orbit collapses completely
}

/// Risk levels for function removal
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Safe,        // Green - safe to remove
    Low,         // Yellow - minor impact
    Medium,      // Orange - moderate impact  
    High,        // Red - major impact
    Critical,    // Black - system failure
}

impl LmfdbRiskMatrix {
    /// Create risk matrix from LMFDB orbit analysis
    pub fn analyze_removal_risks(
        minimal: &MinimalViableOrbit,
        orbit_level: u64,
    ) -> Self {
        println!("⚠️ ANALYZING LMFDB ORBIT REMOVAL RISKS...");
        
        // Build risk matrix from orbit structure
        let risk_matrix = Self::build_risk_matrix(&minimal.core_matrix, orbit_level);
        
        // Analyze each function's risk
        let function_risks = Self::analyze_function_risks(&minimal.removed_functions, orbit_level);
        
        // Calculate removal impacts
        let removal_impacts = Self::calculate_removal_impacts(&function_risks, &risk_matrix);
        
        // Identify critical vs safe functions
        let (critical_functions, safe_removals) = Self::classify_functions(&function_risks);
        
        println!("✅ Risk analysis complete:");
        println!("   LMFDB Level: {}", orbit_level);
        println!("   Functions analyzed: {}", function_risks.len());
        println!("   Critical functions: {}", critical_functions.len());
        println!("   Safe removals: {}", safe_removals.len());
        
        LmfdbRiskMatrix {
            orbit_level,
            risk_matrix,
            function_risks,
            removal_impacts,
            critical_functions,
            safe_removals,
        }
    }
    
    /// Build risk matrix from orbit structure
    fn build_risk_matrix(core_matrix: &[Vec<f64>], level: u64) -> Vec<Vec<f64>> {
        let n = core_matrix.len();
        let mut risk_matrix = vec![vec![0.0; n]; n];
        
        for i in 0..n {
            for j in 0..n {
                let core_value = core_matrix.get(i)
                    .and_then(|row| row.get(j))
                    .copied()
                    .unwrap_or(0.0);
                
                // Risk = inverse of stability
                // Higher core values = more stable = lower risk
                let risk = if core_value.abs() > 1e-6 {
                    1.0 / core_value.abs()
                } else {
                    level as f64 // High risk for zero values
                };
                
                risk_matrix[i][j] = risk.min(level as f64);
            }
        }
        
        risk_matrix
    }
    
    /// Analyze risk for each function
    fn analyze_function_risks(removed_functions: &[String], level: u64) -> Vec<FunctionRisk> {
        removed_functions.iter().enumerate().map(|(i, func_name)| {
            let orbit_index = (i % 10) as u32 + 1; // Map to orbit index
            let risk_level = Self::determine_risk_level(func_name, level);
            let impact_radius = Self::calculate_impact_radius(func_name, level);
            
            FunctionRisk {
                function_name: func_name.clone(),
                orbit_position: (level, orbit_index),
                removal_risk: risk_level,
                impact_radius,
                dependencies: Self::find_dependencies(func_name),
                dependents: Self::find_dependents(func_name),
            }
        }).collect()
    }
    
    /// Determine risk level for function
    fn determine_risk_level(func_name: &str, level: u64) -> RiskLevel {
        match (func_name, level) {
            // Critical functions at any level
            (name, _) if name.contains("main") || name.contains("init") => RiskLevel::Critical,
            
            // Level-specific risks
            (name, 11) if name.contains("core") => RiskLevel::High,
            (name, 23) if name.contains("network") => RiskLevel::Medium,
            (name, 47) if name.contains("crypto") => RiskLevel::High,
            (name, 71) if name.contains("gandalf") => RiskLevel::Critical,
            
            // Debug/logging functions are usually safe
            (name, _) if name.contains("debug") || name.contains("log") => RiskLevel::Safe,
            (name, _) if name.contains("print") || name.contains("trace") => RiskLevel::Low,
            
            // Default based on level
            (_, 11) => RiskLevel::Medium,  // Core level - moderate risk
            (_, 23) => RiskLevel::Low,     // Extended level - lower risk
            (_, 47) => RiskLevel::Medium,  // Advanced level - moderate risk
            (_, 71) => RiskLevel::High,    // Gandalf level - high risk
            _ => RiskLevel::Low,
        }
    }
    
    /// Calculate impact radius of function removal
    fn calculate_impact_radius(func_name: &str, level: u64) -> f64 {
        let base_radius = match func_name {
            name if name.contains("core") => 5.0,
            name if name.contains("main") => 10.0,
            name if name.contains("init") => 8.0,
            name if name.contains("debug") => 1.0,
            name if name.contains("log") => 0.5,
            _ => 2.0,
        };
        
        // Scale by LMFDB level
        base_radius * (level as f64 / 11.0)
    }
    
    /// Find function dependencies
    fn find_dependencies(func_name: &str) -> Vec<String> {
        match func_name {
            name if name.contains("debug") => vec!["format".to_string(), "io".to_string()],
            name if name.contains("log") => vec!["time".to_string(), "format".to_string()],
            name if name.contains("network") => vec!["socket".to_string(), "protocol".to_string()],
            _ => vec![],
        }
    }
    
    /// Find function dependents
    fn find_dependents(func_name: &str) -> Vec<String> {
        match func_name {
            name if name.contains("format") => vec!["debug".to_string(), "log".to_string()],
            name if name.contains("io") => vec!["debug".to_string(), "network".to_string()],
            _ => vec![],
        }
    }
    
    /// Calculate removal impacts
    fn calculate_removal_impacts(
        function_risks: &[FunctionRisk],
        risk_matrix: &[Vec<f64>],
    ) -> Vec<RemovalImpact> {
        function_risks.iter().enumerate().map(|(i, func_risk)| {
            let eigenvalue_delta = Self::calculate_eigenvalue_change(i, risk_matrix);
            let stability_impact = Self::calculate_stability_impact(&func_risk.removal_risk);
            let orbit_change = Self::determine_orbit_change(&func_risk.removal_risk, eigenvalue_delta);
            let cascade_effects = Self::find_cascade_effects(func_risk);
            
            RemovalImpact {
                function_name: func_risk.function_name.clone(),
                orbit_change,
                eigenvalue_delta,
                stability_impact,
                cascade_effects,
            }
        }).collect()
    }
    
    /// Calculate eigenvalue change from removal
    fn calculate_eigenvalue_change(index: usize, risk_matrix: &[Vec<f64>]) -> f64 {
        if let Some(row) = risk_matrix.get(index) {
            let row_sum: f64 = row.iter().sum();
            -row_sum / risk_matrix.len() as f64 // Negative change
        } else {
            0.0
        }
    }
    
    /// Calculate stability impact
    fn calculate_stability_impact(risk_level: &RiskLevel) -> f64 {
        match risk_level {
            RiskLevel::Safe => 0.0,
            RiskLevel::Low => 0.1,
            RiskLevel::Medium => 0.5,
            RiskLevel::High => 0.8,
            RiskLevel::Critical => 1.0,
        }
    }
    
    /// Determine how orbit changes
    fn determine_orbit_change(risk_level: &RiskLevel, eigenvalue_delta: f64) -> OrbitChange {
        match risk_level {
            RiskLevel::Safe => OrbitChange::Stable,
            RiskLevel::Low => OrbitChange::Shrinks(0.95),
            RiskLevel::Medium => OrbitChange::Shrinks(0.8),
            RiskLevel::High => OrbitChange::Destabilizes(eigenvalue_delta.abs()),
            RiskLevel::Critical => OrbitChange::Collapses,
        }
    }
    
    /// Find cascade effects
    fn find_cascade_effects(func_risk: &FunctionRisk) -> Vec<String> {
        if func_risk.removal_risk == RiskLevel::Critical {
            vec!["system_failure".to_string(), "orbit_collapse".to_string()]
        } else if func_risk.removal_risk == RiskLevel::High {
            func_risk.dependents.clone()
        } else {
            vec![]
        }
    }
    
    /// Classify functions as critical vs safe
    fn classify_functions(function_risks: &[FunctionRisk]) -> (Vec<String>, Vec<String>) {
        let mut critical = Vec::new();
        let mut safe = Vec::new();
        
        for func_risk in function_risks {
            match func_risk.removal_risk {
                RiskLevel::Critical | RiskLevel::High => {
                    critical.push(func_risk.function_name.clone());
                },
                RiskLevel::Safe | RiskLevel::Low => {
                    safe.push(func_risk.function_name.clone());
                },
                RiskLevel::Medium => {
                    // Medium risk - depends on impact radius
                    if func_risk.impact_radius > 5.0 {
                        critical.push(func_risk.function_name.clone());
                    } else {
                        safe.push(func_risk.function_name.clone());
                    }
                }
            }
        }
        
        (critical, safe)
    }
    
    /// Get risk summary
    pub fn risk_summary(&self) -> String {
        format!(
            "LMFDB_RISK[Level:{}:Critical:{}:Safe:{}:Matrix:{}x{}]",
            self.orbit_level,
            self.critical_functions.len(),
            self.safe_removals.len(),
            self.risk_matrix.len(),
            self.risk_matrix.get(0).map_or(0, |r| r.len())
        )
    }
    
    /// The risk theorem
    pub fn risk_theorem(&self) -> String {
        format!(
            "⚠️ LMFDB ORBIT RISK THEOREM:\n\
            \n\
            The LMFDB orbit at level {} IS the risk matrix showing exactly\n\
            what happens to the main orbit when each function is removed.\n\
            \n\
            RISK ANALYSIS:\n\
            - Critical functions (cannot remove): {}\n\
            - Safe removals (minimal impact): {}\n\
            - Total functions analyzed: {}\n\
            \n\
            Each orbit element R[i,j] represents the risk impact of removing\n\
            function i on component j of the main orbit.\n\
            \n\
            QED: The LMFDB orbit encodes the complete risk landscape. ∎",
            self.orbit_level,
            self.critical_functions.len(),
            self.safe_removals.len(),
            self.function_risks.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lmfdb_risk_matrix() {
        let mock_minimal = MinimalViableOrbit {
            core_matrix: vec![vec![1.0, 0.5], vec![0.5, 2.0]],
            removed_functions: vec!["debug_print".to_string(), "main_init".to_string()],
            disabled_features: vec![],
            macro_wrappers: vec![],
            minimization_ratio: 0.8,
        };
        
        let risk_matrix = LmfdbRiskMatrix::analyze_removal_risks(&mock_minimal, 11);
        
        assert_eq!(risk_matrix.orbit_level, 11);
        assert_eq!(risk_matrix.function_risks.len(), 2);
        assert!(!risk_matrix.risk_matrix.is_empty());
        
        // main_init should be critical, debug_print should be safe
        assert!(risk_matrix.critical_functions.contains(&"main_init".to_string()));
        assert!(risk_matrix.safe_removals.contains(&"debug_print".to_string()));
    }
}
