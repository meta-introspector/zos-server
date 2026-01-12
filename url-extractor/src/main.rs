use std::collections::HashSet;
use std::fs;

fn main() {
    println!("🔍 Extracting all URLs from GitHub JSON files");

    let mut all_urls = HashSet::new();
    let mut all_repos = HashSet::new();

    // Process all JSON files in /mnt/data1/nix/index/
    let json_files = [
        "github_meta-introspector_repos.json",
        "github_metaintrospector_repos.json",
        "starred.json",
        "stars.json"
    ];

    for file in &json_files {
        let path = format!("/mnt/data1/nix/index/{}", file);
        if let Ok(data) = fs::read_to_string(&path) {
            println!("Processing: {}", file);
            extract_urls_from_json(&data, &mut all_urls, &mut all_repos);
        }
    }

    println!("\n📊 Results:");
    println!("Total unique URLs: {}", all_urls.len());
    println!("Total unique repos: {}", all_repos.len());

    // Save URLs to file
    let urls_list: Vec<String> = all_urls.into_iter().collect();
    let urls_content = urls_list.join("\n");
    fs::write("all_github_urls.txt", urls_content).unwrap();

    // Save repo names to file
    let repos_list: Vec<String> = all_repos.into_iter().collect();
    let repos_content = repos_list.join("\n");
    fs::write("all_github_repos.txt", repos_content).unwrap();

    println!("✅ Saved to all_github_urls.txt and all_github_repos.txt");
}

fn extract_urls_from_json(json_data: &str, urls: &mut HashSet<String>, repos: &mut HashSet<String>) {
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_data) {
        if let Some(repo_array) = data.as_array() {
            for repo in repo_array {
                // Extract all URL fields
                if let Some(clone_url) = repo["clone_url"].as_str() {
                    urls.insert(clone_url.to_string());
                }
                if let Some(html_url) = repo["html_url"].as_str() {
                    urls.insert(html_url.to_string());
                }
                if let Some(git_url) = repo["git_url"].as_str() {
                    urls.insert(git_url.to_string());
                }
                if let Some(ssh_url) = repo["ssh_url"].as_str() {
                    urls.insert(ssh_url.to_string());
                }
                if let Some(svn_url) = repo["svn_url"].as_str() {
                    urls.insert(svn_url.to_string());
                }

                // Extract repo names
                if let Some(full_name) = repo["full_name"].as_str() {
                    repos.insert(full_name.to_string());
                }
                if let Some(name) = repo["name"].as_str() {
                    repos.insert(name.to_string());
                }
            }
        }
    }
}
