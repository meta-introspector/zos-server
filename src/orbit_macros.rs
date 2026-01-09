// Orbit-based Macro System - Generate orbit classes from LMFDB specifications
use crate::lmfdb_orbits::*;

/// Generate complete orbit class from LMFDB level
#[macro_export]
macro_rules! mkorbit_class {
    ($class_name:ident {
        level: $level:literal,
        weight: $weight:literal,
        character: $char:literal,
        orbits: {
            $($orbit_idx:literal => $orbit_name:ident($($field:ident: $field_type:ty),*)),*
        },
        operations: {
            $($op_name:ident => $op_impl:expr),*
        }
    }) => {
        // Generate the orbit class structure
        pub struct $class_name {
            pub level: u64,
            pub weight: u32,
            pub character: u32,
            pub orbits: std::collections::HashMap<u32, SystemArg>,
        }

        impl $class_name {
            pub fn new() -> Result<Self, String> {
                let mut orbits = std::collections::HashMap::new();
                
                $(
                    let orbit_label = format!("{}.a{}", $level, $orbit_idx);
                    let orbit = SystemArg::from_lmfdb(&orbit_label)?;
                    orbits.insert($orbit_idx, orbit);
                )*
                
                Ok($class_name {
                    level: $level,
                    weight: $weight,
                    character: $char,
                    orbits,
                })
            }
            
            pub fn get_orbit(&self, index: u32) -> Option<&SystemArg> {
                self.orbits.get(&index)
            }
            
            pub fn execute_orbit(&self, index: u32, input: &[u8]) -> Result<Vec<u8>, String> {
                match self.get_orbit(index) {
                    Some(orbit) => orbit.execute(input),
                    None => Err(format!("Orbit {} not found in level {}", index, self.level)),
                }
            }
            
            pub fn class_signature(&self) -> String {
                format!("L{}W{}C{}[{}]", 
                       self.level, self.weight, self.character, self.orbits.len())
            }
            
            $(
                pub fn $op_name(&self, input: &[u8]) -> Vec<u8> {
                    $op_impl(input, self)
                }
            )*
        }
        
        // Generate individual orbit accessors
        $(
            impl $class_name {
                pub fn $orbit_name(&self) -> Option<&SystemArg> {
                    self.get_orbit($orbit_idx)
                }
            }
        )*
    };
}

/// Generate orbit transformation between levels
#[macro_export]
macro_rules! mkorbit_transform {
    ($name:ident: $from_level:literal -> $to_level:literal {
        $($transform_rule:ident: $from_idx:literal => $to_idx:literal),*
    }) => {
        pub struct $name;
        
        impl $name {
            $(
                pub fn $transform_rule(input: &SystemArg) -> Result<SystemArg, String> {
                    if input.orbit().level != $from_level {
                        return Err(format!("Expected level {}, got {}", 
                                         $from_level, input.orbit().level));
                    }
                    
                    let target_label = format!("{}.a{}", $to_level, $to_idx);
                    SystemArg::from_lmfdb(&target_label)
                }
            )*
            
            pub fn transform_class(from_class: &[SystemArg]) -> Result<Vec<SystemArg>, String> {
                let mut result = Vec::new();
                for orbit in from_class {
                    // Apply appropriate transformation based on orbit index
                    match orbit.orbit().orbit_index {
                        $($from_idx => result.push(Self::$transform_rule(orbit)?),)*
                        _ => return Err(format!("No transformation for orbit {}", 
                                              orbit.orbit().orbit_index)),
                    }
                }
                Ok(result)
            }
        }
    };
}

/// Generate orbit composition rules
#[macro_export]
macro_rules! mkorbit_compose {
    ($name:ident {
        $(($left_level:literal, $left_idx:literal) + ($right_level:literal, $right_idx:literal) 
          => ($result_level:literal, $result_idx:literal)),*
    }) => {
        pub struct $name;
        
        impl $name {
            pub fn compose(left: &SystemArg, right: &SystemArg) -> Result<SystemArg, String> {
                let left_key = (left.orbit().level, left.orbit().orbit_index);
                let right_key = (right.orbit().level, right.orbit().orbit_index);
                
                match (left_key, right_key) {
                    $(
                        (($left_level, $left_idx), ($right_level, $right_idx)) => {
                            let result_label = format!("{}.a{}", $result_level, $result_idx);
                            SystemArg::from_lmfdb(&result_label)
                        }
                    )*
                    _ => Err(format!("No composition rule for {:?} + {:?}", left_key, right_key)),
                }
            }
        }
    };
}

/// Generate orbit-based enum
#[macro_export]
macro_rules! mkorbit_enum {
    ($name:ident {
        $($variant:ident => $orbit_label:literal),*
    }) => {
        #[derive(Debug, Clone)]
        pub enum $name {
            $($variant(SystemArg)),*
        }

        impl $name {
            pub fn from_label(label: &str) -> Result<Self, String> {
                match label {
                    $($orbit_label => Ok(Self::$variant(SystemArg::from_lmfdb($orbit_label)?)),)*
                    _ => Err(format!("Unknown orbit label: {}", label)),
                }
            }
            
            pub fn execute(&self, input: &[u8]) -> Result<Vec<u8>, String> {
                match self {
                    $(Self::$variant(orbit) => orbit.execute(input),)*
                }
            }
            
            pub fn orbit(&self) -> &LmfdbOrbit {
                match self {
                    $(Self::$variant(orbit) => orbit.orbit(),)*
                }
            }
        }
    };
}

/// Generate orbit-based function
#[macro_export]
macro_rules! mkorbit_fn {
    ($name:ident($($arg:ident: $orbit_label:literal),*) -> $ret:ty {
        $($body:tt)*
    }) => {
        pub fn $name($($arg: &SystemArg),*) -> Result<$ret, String> {
            // Validate orbit labels
            $(
                if $arg.orbit().label != $orbit_label {
                    return Err(format!("Expected orbit {}, got {}", 
                                     $orbit_label, $arg.orbit().label));
                }
            )*
            
            $($body)*
        }
    };
}

// Pre-defined orbit systems using the new class macros
// These are now generated by mkorbit_class! in zos_system.rs

// Orbit-based enums for system arguments
mkorbit_enum!(CoreOrbit {
    Posix => "11.a1",
    Bash => "11.a2", 
    Cargo => "11.a3",
    Rust => "11.a4",
    Ssh => "11.a5",
    Curl => "11.a6",
    Ssl => "11.a7",
    Regex => "11.a8",
    Git => "11.a9"
});

mkorbit_enum!(ExtendedOrbit {
    Blockchain => "23.a1",
    ZkProof => "23.a2",
    Enterprise => "23.a3",
    Security => "23.a4",
    DataFlow => "23.a5",
    Knowledge => "23.a6",
    Modeling => "23.a7"
});

// Orbit-based functions
mkorbit_fn!(compose_orbits(a: "11.a1", b: "11.a2") -> Vec<u8> {
    // Compose two orbits mathematically
    let coeffs_a = &a.orbit().coefficients;
    let coeffs_b = &b.orbit().coefficients;
    
    let mut result = Vec::new();
    for i in 0..coeffs_a.len().max(coeffs_b.len()) {
        let ca = coeffs_a.get(i).unwrap_or(&0);
        let cb = coeffs_b.get(i).unwrap_or(&0);
        result.push((ca + cb) as u8);
    }
    
    Ok(result)
});

mkorbit_fn!(orbit_transform(core: "11.a1", extended: "23.a1") -> String {
    // Transform between orbit levels
    let level_diff = extended.orbit().level - core.orbit().level;
    let signature = format!("{}→{} (Δ{})", 
                          core.orbit().label, 
                          extended.orbit().label, 
                          level_diff);
    Ok(signature)
});
