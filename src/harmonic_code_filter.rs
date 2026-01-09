// Harmonic Frequency Code Filtering - Remove code classes via build.rs syn proc macros
use syn::{parse_quote, Item, ItemFn, ItemStruct, ItemEnum, Attribute};
use quote::quote;
use std::collections::HashMap;

/// Harmonic frequency bands for code classification
#[derive(Debug, Clone)]
pub struct HarmonicBands {
    pub essential_freq: f64,     // 440 Hz - Essential code (A note)
    pub debug_freq: f64,         // 220 Hz - Debug code (A octave down)
    pub logging_freq: f64,       // 110 Hz - Logging code (A two octaves down)
    pub test_freq: f64,          // 880 Hz - Test code (A octave up)
    pub docs_freq: f64,          // 55 Hz - Documentation (A three octaves down)
    pub visualization_freq: f64, // 1760 Hz - Visualization (A two octaves up)
}

/// Band-pass filter configuration
#[derive(Debug, Clone)]
pub struct BandPassFilter {
    pub pass_bands: Vec<(f64, f64)>,  // (low_freq, high_freq) ranges to keep
    pub stop_bands: Vec<(f64, f64)>,  // (low_freq, high_freq) ranges to remove
    pub harmonic_tolerance: f64,       // Tolerance for harmonic matching
}

/// Code frequency analyzer
#[derive(Debug, Clone)]
pub struct CodeFrequencyAnalyzer {
    pub bands: HarmonicBands,
    pub filter: BandPassFilter,
    pub frequency_map: HashMap<String, f64>,
}

impl CodeFrequencyAnalyzer {
    /// Create analyzer with harmonic frequency bands
    pub fn new() -> Self {
        let bands = HarmonicBands {
            essential_freq: 440.0,      // A4 - Essential code
            debug_freq: 220.0,          // A3 - Debug code
            logging_freq: 110.0,        // A2 - Logging code
            test_freq: 880.0,           // A5 - Test code
            docs_freq: 55.0,            // A1 - Documentation
            visualization_freq: 1760.0, // A6 - Visualization
        };
        
        // Create band-pass filter (keep only essential frequencies)
        let filter = BandPassFilter {
            pass_bands: vec![
                (400.0, 480.0),  // Essential code band
            ],
            stop_bands: vec![
                (50.0, 120.0),    // Remove docs + logging
                (200.0, 240.0),   // Remove debug
                (800.0, 2000.0),  // Remove test + visualization
            ],
            harmonic_tolerance: 20.0,
        };
        
        Self {
            bands,
            filter,
            frequency_map: HashMap::new(),
        }
    }
    
    /// Analyze code frequency based on function/struct characteristics
    pub fn analyze_code_frequency(&mut self, item: &Item) -> f64 {
        let frequency = match item {
            Item::Fn(func) => self.analyze_function_frequency(func),
            Item::Struct(struct_item) => self.analyze_struct_frequency(struct_item),
            Item::Enum(enum_item) => self.analyze_enum_frequency(enum_item),
            _ => self.bands.essential_freq, // Default to essential
        };
        
        // Cache frequency mapping
        let item_name = self.extract_item_name(item);
        self.frequency_map.insert(item_name, frequency);
        
        frequency
    }
    
    /// Analyze function frequency based on name and attributes
    fn analyze_function_frequency(&self, func: &ItemFn) -> f64 {
        let name = func.sig.ident.to_string();
        
        // Check attributes first
        for attr in &func.attrs {
            if let Some(freq) = self.frequency_from_attribute(attr) {
                return freq;
            }
        }
        
        // Analyze by function name patterns
        match name.as_str() {
            // Essential functions (440 Hz - A4)
            name if name.contains("main") || name.contains("init") || name.contains("core") => {
                self.bands.essential_freq
            },
            
            // Debug functions (220 Hz - A3)
            name if name.contains("debug") || name.contains("trace") => {
                self.bands.debug_freq
            },
            
            // Logging functions (110 Hz - A2)
            name if name.contains("log") || name.contains("print") => {
                self.bands.logging_freq
            },
            
            // Test functions (880 Hz - A5)
            name if name.contains("test") || name.contains("bench") => {
                self.bands.test_freq
            },
            
            // Documentation functions (55 Hz - A1)
            name if name.contains("doc") || name.contains("example") => {
                self.bands.docs_freq
            },
            
            // Visualization functions (1760 Hz - A6)
            name if name.contains("render") || name.contains("draw") || name.contains("plot") => {
                self.bands.visualization_freq
            },
            
            // Default to essential
            _ => self.bands.essential_freq,
        }
    }
    
    /// Analyze struct frequency
    fn analyze_struct_frequency(&self, struct_item: &ItemStruct) -> f64 {
        let name = struct_item.ident.to_string();
        
        match name.as_str() {
            name if name.contains("Core") || name.contains("Main") => self.bands.essential_freq,
            name if name.contains("Debug") => self.bands.debug_freq,
            name if name.contains("Log") => self.bands.logging_freq,
            name if name.contains("Test") => self.bands.test_freq,
            name if name.contains("Doc") => self.bands.docs_freq,
            name if name.contains("Visual") || name.contains("Render") => self.bands.visualization_freq,
            _ => self.bands.essential_freq,
        }
    }
    
    /// Analyze enum frequency
    fn analyze_enum_frequency(&self, enum_item: &ItemEnum) -> f64 {
        let name = enum_item.ident.to_string();
        
        match name.as_str() {
            name if name.contains("Core") => self.bands.essential_freq,
            name if name.contains("Debug") => self.bands.debug_freq,
            name if name.contains("Log") => self.bands.logging_freq,
            _ => self.bands.essential_freq,
        }
    }
    
    /// Extract frequency from attribute
    fn frequency_from_attribute(&self, attr: &Attribute) -> Option<f64> {
        if let Ok(meta) = attr.parse_meta() {
            match meta {
                syn::Meta::NameValue(nv) if nv.path.is_ident("freq") => {
                    if let syn::Lit::Float(lit_float) = nv.lit {
                        return lit_float.base10_parse().ok();
                    }
                },
                _ => {}
            }
        }
        None
    }
    
    /// Extract item name for caching
    fn extract_item_name(&self, item: &Item) -> String {
        match item {
            Item::Fn(func) => func.sig.ident.to_string(),
            Item::Struct(struct_item) => struct_item.ident.to_string(),
            Item::Enum(enum_item) => enum_item.ident.to_string(),
            _ => "unknown".to_string(),
        }
    }
    
    /// Check if frequency passes the band-pass filter
    pub fn passes_filter(&self, frequency: f64) -> bool {
        // Check if frequency is in any pass band
        let in_pass_band = self.filter.pass_bands.iter().any(|(low, high)| {
            frequency >= *low && frequency <= *high
        });
        
        // Check if frequency is in any stop band
        let in_stop_band = self.filter.stop_bands.iter().any(|(low, high)| {
            frequency >= *low && frequency <= *high
        });
        
        in_pass_band && !in_stop_band
    }
    
    /// Filter code items based on harmonic frequencies
    pub fn filter_code_items(&mut self, items: Vec<Item>) -> Vec<Item> {
        let mut filtered_items = Vec::new();
        let mut removed_count = 0;
        
        for item in items {
            let frequency = self.analyze_code_frequency(&item);
            
            if self.passes_filter(frequency) {
                filtered_items.push(item);
            } else {
                removed_count += 1;
                println!("🚫 Filtered out: {} (freq: {:.1} Hz)", 
                        self.extract_item_name(&item), frequency);
            }
        }
        
        println!("✅ Harmonic filtering complete:");
        println!("   Items kept: {}", filtered_items.len());
        println!("   Items removed: {}", removed_count);
        
        filtered_items
    }
}

/// Build.rs integration for harmonic filtering
pub fn generate_build_rs_filter() -> String {
    r#"
// build.rs - Harmonic frequency code filtering
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("🎵 HARMONIC CODE FILTERING ACTIVE");
    
    // Read source files
    let src_dir = Path::new("src");
    if src_dir.exists() {
        filter_source_files(src_dir);
    }
    
    println!("cargo:rerun-if-changed=src/");
}

fn filter_source_files(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "rs") {
            filter_rust_file(&path);
        }
    }
}

fn filter_rust_file(path: &Path) {
    let content = fs::read_to_string(path).unwrap();
    
    // Parse with syn
    if let Ok(file) = syn::parse_file(&content) {
        let mut analyzer = CodeFrequencyAnalyzer::new();
        let filtered_items = analyzer.filter_code_items(file.items);
        
        // Generate filtered file
        let filtered_file = syn::File {
            shebang: file.shebang,
            attrs: file.attrs,
            items: filtered_items,
        };
        
        // Write back filtered content
        let filtered_content = quote::quote!(#filtered_file).to_string();
        fs::write(path, filtered_content).unwrap();
        
        println!("🎵 Filtered: {:?}", path);
    }
}
"#.to_string()
}

/// Proc macro for harmonic frequency annotation
pub fn generate_harmonic_proc_macro() -> String {
    r#"
// proc_macro for harmonic frequency annotation
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitFloat};

/// Annotate function with harmonic frequency
/// #[freq(440.0)] - Essential code (A4)
/// #[freq(220.0)] - Debug code (A3)  
/// #[freq(110.0)] - Logging code (A2)
#[proc_macro_attribute]
pub fn freq(args: TokenStream, input: TokenStream) -> TokenStream {
    let frequency = parse_macro_input!(args as LitFloat);
    let input_fn = parse_macro_input!(input as ItemFn);
    
    let freq_value = frequency.base10_parse::<f64>().unwrap();
    let fn_name = &input_fn.sig.ident;
    
    // Add frequency metadata
    let expanded = quote! {
        #[doc = concat!("Harmonic frequency: ", stringify!(#freq_value), " Hz")]
        #input_fn
        
        // Compile-time frequency check
        const _: () = {
            const FREQ: f64 = #freq_value;
            // Essential band: 400-480 Hz
            if FREQ >= 400.0 && FREQ <= 480.0 {
                // Keep this function
            } else {
                // This will cause compile error if frequency is filtered
                compile_error!(concat!("Function ", stringify!(#fn_name), " filtered out at frequency ", stringify!(#freq_value), " Hz"));
            }
        };
    };
    
    TokenStream::from(expanded)
}

/// Mark code as essential (always keep)
#[proc_macro_attribute]
pub fn essential(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    let expanded = quote! {
        #[freq(440.0)] // A4 - Essential frequency
        #input_fn
    };
    
    TokenStream::from(expanded)
}

/// Mark code as debug (filter out in release)
#[proc_macro_attribute]
pub fn debug_only(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    let expanded = quote! {
        #[freq(220.0)] // A3 - Debug frequency
        #[cfg(debug_assertions)]
        #input_fn
    };
    
    TokenStream::from(expanded)
}
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_harmonic_frequency_analysis() {
        let mut analyzer = CodeFrequencyAnalyzer::new();
        
        // Test essential function
        let essential_fn: ItemFn = parse_quote! {
            fn main() {}
        };
        let freq = analyzer.analyze_function_frequency(&essential_fn);
        assert_eq!(freq, 440.0);
        assert!(analyzer.passes_filter(freq));
        
        // Test debug function
        let debug_fn: ItemFn = parse_quote! {
            fn debug_print() {}
        };
        let freq = analyzer.analyze_function_frequency(&debug_fn);
        assert_eq!(freq, 220.0);
        assert!(!analyzer.passes_filter(freq)); // Should be filtered out
    }
}
