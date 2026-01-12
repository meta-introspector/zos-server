use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoOntology {
    pub name: String,
    pub path: String,
    pub domain: String, // tld like "org", "com", "io"
    pub file_count: usize,
    pub ontology_types: Vec<String>,
    pub canonical_forms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaIntrospectorStats {
    pub total_repos: usize,
    pub total_files: usize,
    pub domains: HashMap<String, usize>,
    pub ontology_coverage: f64,
    pub canonical_index_size: usize,
}

pub struct MetaIntrospectorManager {
    base_path: String,
    repos: Vec<RepoOntology>,
}

impl MetaIntrospectorManager {
    pub fn new(base_path: String) -> Self {
        Self {
            base_path,
            repos: Vec::new(),
        }
    }

    pub fn scan_repositories(&mut self) -> Result<(), String> {
        let base = Path::new(&self.base_path);

        // Scan TLD directories (org, com, io, etc.)
        for entry in fs::read_dir(base).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(domain) = path.file_name().and_then(|n| n.to_str()) {
                    if domain.len() <= 3 && !domain.starts_with('.') {
                        self.scan_domain_repos(&path, domain)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn scan_domain_repos(&mut self, domain_path: &Path, domain: &str) -> Result<(), String> {
        for entry in fs::read_dir(domain_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let repo = self.analyze_repo(&path, name, domain)?;
                    self.repos.push(repo);
                }
            }
        }
        Ok(())
    }

    fn analyze_repo(
        &self,
        repo_path: &Path,
        name: &str,
        domain: &str,
    ) -> Result<RepoOntology, String> {
        let mut file_count = 0;
        let mut ontology_types = Vec::new();
        let mut canonical_forms = Vec::new();

        // Count files and detect ontology patterns
        if let Ok(entries) = fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    file_count += 1;

                    if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                        match ext {
                            "owl" | "rdf" | "ttl" => ontology_types.push("RDF/OWL".to_string()),
                            "json" => ontology_types.push("JSON-LD".to_string()),
                            "rs" => ontology_types.push("Rust Types".to_string()),
                            "py" => ontology_types.push("Python Classes".to_string()),
                            _ => {}
                        }
                    }
                }
            }
        }

        // Check for canonical forms
        if repo_path.join("canonical").exists() {
            canonical_forms.push("Canonical Structure".to_string());
        }
        if repo_path.join("schema.json").exists() {
            canonical_forms.push("JSON Schema".to_string());
        }

        ontology_types.sort();
        ontology_types.dedup();

        Ok(RepoOntology {
            name: name.to_string(),
            path: repo_path.to_string_lossy().to_string(),
            domain: domain.to_string(),
            file_count,
            ontology_types,
            canonical_forms,
        })
    }

    pub fn get_repos(&self) -> &[RepoOntology] {
        &self.repos
    }

    pub fn get_stats(&self) -> MetaIntrospectorStats {
        let mut domains = HashMap::new();
        let mut total_files = 0;

        for repo in &self.repos {
            *domains.entry(repo.domain.clone()).or_insert(0) += 1;
            total_files += repo.file_count;
        }

        let ontology_repos = self
            .repos
            .iter()
            .filter(|r| !r.ontology_types.is_empty())
            .count();

        let ontology_coverage = if self.repos.is_empty() {
            0.0
        } else {
            ontology_repos as f64 / self.repos.len() as f64
        };

        // Check canonical index size
        let canonical_index_size = Path::new(&self.base_path)
            .join("master_canonical_index.json")
            .metadata()
            .map(|m| m.len() as usize)
            .unwrap_or(0);

        MetaIntrospectorStats {
            total_repos: self.repos.len(),
            total_files,
            domains,
            ontology_coverage,
            canonical_index_size,
        }
    }

    pub fn generate_html_dashboard(&self) -> String {
        let stats = self.get_stats();

        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Meta-Introspector Dashboard</title>
    <style>
        body {{ font-family: monospace; background: #0a0a0a; color: #00ff00; padding: 20px; }}
        .stat-box {{ border: 1px solid #00ff00; padding: 15px; margin: 10px 0; }}
        .domain-list {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 10px; }}
        .domain {{ background: #001100; padding: 10px; border: 1px solid #004400; }}
        .progress {{ background: #004400; height: 20px; }}
        .progress-bar {{ background: #00ff00; height: 100%; }}
    </style>
</head>
<body>
    <h1>🔍 Meta-Introspector Dashboard</h1>
    
    <div class="stat-box">
        <h3>Repository Statistics</h3>
        <p>Total Repositories: {}</p>
        <p>Total Files: {}</p>
        <p>Ontology Coverage: {:.1}%</p>
        <p>Canonical Index Size: {} bytes</p>
    </div>

    <div class="stat-box">
        <h3>Domain Distribution</h3>
        <div class="domain-list">
            {}
        </div>
    </div>

    <div class="stat-box">
        <h3>Ontology Integration Status</h3>
        <div class="progress">
            <div class="progress-bar" style="width: {:.1}%"></div>
        </div>
        <p>{:.1}% of repositories contain extractable ontologies</p>
    </div>
</body>
</html>
        "#,
            stats.total_repos,
            stats.total_files,
            stats.ontology_coverage * 100.0,
            stats.canonical_index_size,
            stats
                .domains
                .iter()
                .map(|(domain, count)| format!(
                    "<div class='domain'><strong>.{}</strong><br>{} repos</div>",
                    domain, count
                ))
                .collect::<Vec<_>>()
                .join(""),
            stats.ontology_coverage * 100.0,
            stats.ontology_coverage * 100.0
        )
    }
}
