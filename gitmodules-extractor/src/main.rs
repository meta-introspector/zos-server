use std::collections::HashSet;
use std::fs;

fn main() {
    println!("🔍 Extracting URLs from .gitmodules files");

    let gitmodules_list = fs::read_to_string("gitmodules_list.txt").unwrap();
    let mut all_urls = HashSet::new();
    let mut processed = 0;

    for gitmodules_path in gitmodules_list.lines() {
        if let Ok(content) = fs::read_to_string(gitmodules_path) {
            extract_urls_from_gitmodules(&content, &mut all_urls);
            processed += 1;

            if processed % 100 == 0 {
                println!("Processed {} .gitmodules files, found {} URLs", processed, all_urls.len());
            }
        }
    }

    println!("\n📊 Results:");
    println!("Processed {} .gitmodules files", processed);
    println!("Found {} unique submodule URLs", all_urls.len());

    // Save URLs to file
    let urls_list: Vec<String> = all_urls.into_iter().collect();
    let urls_content = urls_list.join("\n");
    fs::write("gitmodules_urls.txt", urls_content).unwrap();

    println!("✅ Saved to gitmodules_urls.txt");
}

fn extract_urls_from_gitmodules(content: &str, urls: &mut HashSet<String>) {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("url = ") {
            let url = line[6..].trim();
            // Remove quotes if present
            let url = url.trim_matches('"').trim_matches('\'');
            if !url.is_empty() {
                urls.insert(url.to_string());
            }
        }
    }
}
