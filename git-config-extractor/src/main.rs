use std::collections::HashSet;
use std::fs;

fn main() {
    println!("🔍 Extracting remote URLs from .git/config files");

    let config_list = fs::read_to_string("git_config_list.txt").unwrap();
    let mut all_urls = HashSet::new();
    let mut processed = 0;

    for config_path in config_list.lines() {
        if let Ok(content) = fs::read_to_string(config_path) {
            extract_remote_urls(&content, &mut all_urls);
            processed += 1;

            if processed % 100 == 0 {
                println!("Processed {} config files, found {} URLs", processed, all_urls.len());
            }
        }
    }

    println!("\n📊 Results:");
    println!("Processed {} .git/config files", processed);
    println!("Found {} unique remote URLs", all_urls.len());

    // Save URLs to file
    let urls_list: Vec<String> = all_urls.into_iter().collect();
    let urls_content = urls_list.join("\n");
    fs::write("git_remote_urls.txt", urls_content).unwrap();

    println!("✅ Saved to git_remote_urls.txt");
}

fn extract_remote_urls(content: &str, urls: &mut HashSet<String>) {
    let mut in_remote_section = false;

    for line in content.lines() {
        let line = line.trim();

        // Check if we're entering a remote section
        if line.starts_with("[remote ") {
            in_remote_section = true;
            continue;
        }

        // Check if we're leaving the remote section
        if line.starts_with("[") && !line.starts_with("[remote ") {
            in_remote_section = false;
            continue;
        }

        // Extract URL if we're in a remote section
        if in_remote_section && line.starts_with("url = ") {
            let url = line[6..].trim();
            if !url.is_empty() {
                urls.insert(url.to_string());
            }
        }
    }
}
