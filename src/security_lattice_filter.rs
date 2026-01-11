use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_file, visit_mut::VisitMut, Item, ItemFn, Attribute, Meta, Lit};
use std::collections::HashSet;

/// Security lattice filter that removes code based on security context
pub struct SecurityLatticeFilter {
    pub target_security_level: SecurityLevel,
    pub allowed_price_tier: f64,
    pub matrix_access: MatrixAccess,
    pub filtered_functions: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SecurityLevel {
    Public = 0,
    Guest = 1,
    User = 2,
    Admin = 3,
    SuperAdmin = 4,
}

#[derive(Debug, Clone)]
pub enum MatrixAccess {
    DiagonalOnly,
    LowerTriangular,
    UpperTriangular,
    FullMatrix,
}

impl SecurityLatticeFilter {
    pub fn new(target_level: SecurityLevel, price_tier: f64) -> Self {
        Self {
            target_security_level: target_level,
            allowed_price_tier: price_tier,
            matrix_access: Self::matrix_access_for_level(&target_level),
            filtered_functions: HashSet::new(),
        }
    }

    fn matrix_access_for_level(level: &SecurityLevel) -> MatrixAccess {
        match level {
            SecurityLevel::Public => MatrixAccess::DiagonalOnly,
            SecurityLevel::Guest => MatrixAccess::LowerTriangular,
            SecurityLevel::User => MatrixAccess::LowerTriangular,
            SecurityLevel::Admin => MatrixAccess::UpperTriangular,
            SecurityLevel::SuperAdmin => MatrixAccess::FullMatrix,
        }
    }

    /// Harmonic band-pass filter: remove functions outside security frequency band
    pub fn harmonic_filter(&mut self, source_code: &str) -> Result<String, syn::Error> {
        let mut syntax_tree = parse_file(source_code)?;

        // Apply harmonic filtering to AST
        self.visit_file_mut(&mut syntax_tree);

        // Regenerate filtered code with quote
        let filtered_tokens = quote! { #syntax_tree };
        Ok(filtered_tokens.to_string())
    }

    /// Check if function passes through the security filter
    fn passes_security_filter(&self, item_fn: &ItemFn) -> bool {
        let security_context = self.extract_security_context(&item_fn.attrs);

        match security_context {
            Some((level, price_tier, _)) => {
                // Harmonic filtering: only allow functions within our security band
                level <= self.target_security_level && price_tier <= self.allowed_price_tier
            }
            None => {
                // Functions without security context are filtered out (insecure)
                false
            }
        }
    }

    fn extract_security_context(&self, attrs: &[Attribute]) -> Option<(SecurityLevel, f64, String)> {
        for attr in attrs {
            if attr.path().is_ident("security_context") {
                if let Ok(Meta::List(meta_list)) = attr.meta.clone() {
                    let mut level = SecurityLevel::Public;
                    let mut price_tier = 0.0;
                    let mut matrix_access = "DiagonalOnly".to_string();

                    // Parse security context parameters
                    for nested in meta_list.tokens.into_iter() {
                        // Simplified parsing - in practice would use proper syn parsing
                        let token_str = nested.to_string();
                        if token_str.contains("Admin") {
                            level = SecurityLevel::Admin;
                            price_tier = 1000.0;
                        } else if token_str.contains("User") {
                            level = SecurityLevel::User;
                            price_tier = 100.0;
                        }
                    }

                    return Some((level, price_tier, matrix_access));
                }
            }
        }
        None
    }
}

impl VisitMut for SecurityLatticeFilter {
    fn visit_item_mut(&mut self, item: &mut Item) {
        match item {
            Item::Fn(item_fn) => {
                if !self.passes_security_filter(item_fn) {
                    // Remove insecure function - replace with security stub
                    let fn_name = &item_fn.sig.ident;
                    self.filtered_functions.insert(fn_name.to_string());

                    // Replace with security violation stub
                    let security_stub = self.generate_security_stub(fn_name);
                    *item = security_stub;
                }
            }
            _ => {
                // Continue visiting other items
                syn::visit_mut::visit_item_mut(self, item);
            }
        }
    }

    fn visit_file_mut(&mut self, file: &mut syn::File) {
        // Filter items based on security lattice
        file.items.retain_mut(|item| {
            match item {
                Item::Fn(item_fn) => {
                    if self.passes_security_filter(item_fn) {
                        true // Keep secure functions
                    } else {
                        // Replace with security stub
                        let fn_name = &item_fn.sig.ident;
                        self.filtered_functions.insert(fn_name.to_string());
                        *item = self.generate_security_stub(fn_name);
                        true
                    }
                }
                _ => true, // Keep non-function items
            }
        });
    }

    fn generate_security_stub(&self, fn_name: &syn::Ident) -> Item {
        // Generate a security violation stub using quote
        let stub_tokens = quote! {
            pub fn #fn_name() -> Result<(), SecurityViolation> {
                Err(SecurityViolation {
                    function_name: stringify!(#fn_name),
                    required_security_level: SecurityLevel::Admin,
                    current_security_level: SecurityLevel::Public,
                    required_price_tier: 1000.0,
                    current_price_tier: 0.0,
                    message: format!("Function '{}' requires higher security context", stringify!(#fn_name)),
                })
            }
        };

        syn::parse2(stub_tokens).unwrap()
    }
}

/// Build script integration for security lattice filtering
pub fn build_security_filtered_modules() -> Result<(), Box<dyn std::error::Error>> {
    let security_contexts = vec![
        ("public", SecurityLevel::Public, 0.0),
        ("guest", SecurityLevel::Guest, 10.0),
        ("user", SecurityLevel::User, 100.0),
        ("admin", SecurityLevel::Admin, 1000.0),
        ("superadmin", SecurityLevel::SuperAdmin, 1_000_000.0),
    ];

    // Read source files
    let source_files = vec![
        "src/core.rs",
        "src/web.rs",
        "src/minimal_server_plugin.rs",
    ];

    for (context_name, security_level, price_tier) in security_contexts {
        for source_file in &source_files {
            let source_code = std::fs::read_to_string(source_file)?;

            // Apply harmonic security filter
            let mut filter = SecurityLatticeFilter::new(security_level, price_tier);
            let filtered_code = filter.harmonic_filter(&source_code)?;

            // Generate security-context-specific module
            let output_path = format!("target/security_contexts/{}/{}", context_name, source_file);
            std::fs::create_dir_all(std::path::Path::new(&output_path).parent().unwrap())?;
            std::fs::write(&output_path, filtered_code)?;

            println!("Generated security-filtered module: {}", output_path);
            println!("Filtered functions: {:?}", filter.filtered_functions);
        }
    }

    Ok(())
}

/// Harmonic frequency analysis of security contexts
pub struct SecurityHarmonics {
    pub fundamental_frequency: f64,    // Base security frequency
    pub harmonics: Vec<f64>,          // Security harmonic frequencies
    pub filter_bands: Vec<(f64, f64)>, // Band-pass filter ranges
}

impl SecurityHarmonics {
    pub fn new() -> Self {
        Self {
            fundamental_frequency: 1.0,   // Public = fundamental
            harmonics: vec![
                1.0,    // Public
                2.0,    // Guest (2nd harmonic)
                4.0,    // User (4th harmonic)
                8.0,    // Admin (8th harmonic)
                16.0,   // SuperAdmin (16th harmonic)
            ],
            filter_bands: vec![
                (0.5, 1.5),   // Public band
                (1.5, 3.0),   // Guest band
                (3.0, 6.0),   // User band
                (6.0, 12.0),  // Admin band
                (12.0, 24.0), // SuperAdmin band
            ],
        }
    }

    pub fn security_frequency(&self, level: &SecurityLevel) -> f64 {
        self.harmonics[*level as usize]
    }

    pub fn passes_band_filter(&self, function_frequency: f64, target_level: &SecurityLevel) -> bool {
        let (low, high) = self.filter_bands[*target_level as usize];
        function_frequency >= low && function_frequency <= high
    }
}

#[derive(Debug)]
pub struct SecurityViolation {
    pub function_name: &'static str,
    pub required_security_level: SecurityLevel,
    pub current_security_level: SecurityLevel,
    pub required_price_tier: f64,
    pub current_price_tier: f64,
    pub message: String,
}
