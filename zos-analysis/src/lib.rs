use quote;
use serde_json;
use std::collections::HashMap;
use std::env;
use syn;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments for signature filter
    let mut target_signature: Option<u128> = None;
    let mut input_file: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--signature" => {
                if i + 1 < args.len() {
                    target_signature = args[i + 1].parse::<u128>().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--filter" => {
                if i + 1 < args.len() {
                    // Legacy frequency filter support
                    i += 2;
                } else {
                    i += 1;
                }
            }
            arg if !arg.starts_with('-') => {
                input_file = Some(arg.to_string());
                i += 1;
            }
            _ => i += 1,
        }
    }

    println!("🧟 Zombie Rustc - Spectral Analysis Driver");
    println!("==========================================");

    if let Some(freq) = filter_freq {
        println!("🎛️ Spectral filter: {:.2}", freq);

        // Load spectral filter mapping
        let filter_map = load_spectral_filters();
        let target_class = find_class_for_frequency(&filter_map, freq);

        if let Some(class) = target_class {
            println!("🎯 Target class: {}", class);
            if let Some(file) = input_file {
                println!("📄 Analyzing with {} filter: {}", class, file);
                run_spectral_analysis(&file, freq, &class);
            }
        } else {
            println!("❌ No class found for frequency {:.2}", freq);
        }
    } else {
        if let Some(file) = input_file {
            println!("📄 Analyzing (no filter): {}", file);
            run_normal_analysis(&file);
        }
    }
}

fn load_spectral_filters() -> HashMap<String, f64> {
    if let Ok(content) = std::fs::read_to_string("spectral_filters.json") {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

fn find_class_for_frequency(filter_map: &HashMap<String, f64>, target_freq: f64) -> Option<String> {
    filter_map
        .iter()
        .find(|(_, &freq)| (freq - target_freq).abs() < 0.025)
        .map(|(class, _)| class.clone())
}

fn run_spectral_analysis(input_file: &str, filter_freq: f64, target_class: &str) {
    println!(
        "🔬 Running spectral analysis with filter {:.2}",
        filter_freq
    );
    println!("✅ Filtering for class: {}", target_class);

    // Parse the input file and extract matching AST nodes
    let filtered_items = extract_spectral_items(input_file, target_class);

    // Generate output file with all filtered items
    let output_file = format!("{}.{}.rs", input_file.trim_end_matches(".rs"), target_class);
    generate_filtered_file(&output_file, &filtered_items, target_class);

    println!("🎯 Spectral compilation complete!");
    println!("   Filter frequency: {:.2}", filter_freq);
    println!("   Target class: {}", target_class);
    println!("   Items extracted: {}", filtered_items.len());
    println!("   Output: {}", output_file);
}

fn extract_spectral_items(input_file: &str, target_class: &str) -> Vec<String> {
    let mut items = Vec::new();

    if let Ok(content) = std::fs::read_to_string(input_file) {
        if let Ok(syntax) = syn::parse_file(&content) {
            // Extract items based on spectral class
            for item in &syntax.items {
                match target_class {
                    "fn" => {
                        if let syn::Item::Fn(func) = item {
                            items.push(quote::quote!(#func).to_string());
                        }
                    }
                    "struct" => {
                        if let syn::Item::Struct(s) = item {
                            items.push(quote::quote!(#s).to_string());
                        }
                    }
                    "enum" => {
                        if let syn::Item::Enum(e) = item {
                            items.push(quote::quote!(#e).to_string());
                        }
                    }
                    "impl" => {
                        if let syn::Item::Impl(i) = item {
                            items.push(quote::quote!(#i).to_string());
                        }
                    }
                    "macro" => {
                        if let syn::Item::Macro(m) = item {
                            items.push(quote::quote!(#m).to_string());
                        }
                    }
                    "group" => {
                        // For group, extract all token groups/parentheses
                        items.push(format!("// Group item: {}", quote::quote!(#item)));
                    }
                    _ => {
                        // Default: include all items for unknown classes
                        items.push(quote::quote!(#item).to_string());
                    }
                }
            }
        }
    }

    items
}

fn generate_filtered_file(output_file: &str, items: &[String], target_class: &str) {
    let mut content = String::new();

    // Add header
    content.push_str(&format!("// Spectral compilation output\n"));
    content.push_str(&format!("// Filter class: {}\n", target_class));
    content.push_str(&format!("// Generated items: {}\n\n", items.len()));

    // Add all filtered items
    for (i, item) in items.iter().enumerate() {
        content.push_str(&format!("// === Item {} ===\n", i + 1));
        content.push_str(item);
        content.push_str("\n\n");
    }

    // Write to file
    std::fs::write(output_file, content)
        .unwrap_or_else(|e| println!("❌ Failed to write {}: {}", output_file, e));

    println!("📂 Generated spectral file: {}", output_file);
}

fn run_normal_analysis(input_file: &str) {
    println!("🔬 Running normal analysis");
    println!("✅ Normal analysis complete for: {}", input_file);
}
