use crate::cicd_dashboard::{CICDDashboard, ProjectStatus};
use crate::git_analyzer::GitAnalyzer;
use crate::github_importer::GitHubDataImporter;
use crate::meta_introspector::MetaIntrospectorManager;
use crate::value_lattice_manager::ValueLatticeManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactProjectStatus {
    pub name: String,
    pub branch: String,
    pub git: String,
    pub lattice: String,
    pub rustc: String,
    pub files: usize,
    pub owner: String,
    pub org: String,
    pub fork: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactDashboard {
    pub total: usize,
    pub indexed: usize,
    pub compiled: usize,
    pub projects: Vec<CompactProjectStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

pub struct ZosApi;

impl ZosApi {
    pub fn get_projects_json() -> Result<String, String> {
        let mut meta_manager =
            MetaIntrospectorManager::new("/mnt/data1/meta-introspector".to_string());

        meta_manager.scan_repositories()?;
        let repos = meta_manager.get_repos();

        let mut dashboard = CICDDashboard::new();
        dashboard.scan_projects(repos)?;

        let projects: Vec<CompactProjectStatus> = dashboard
            .get_projects()
            .iter()
            .map(|p| CompactProjectStatus {
                name: p.name.clone(),
                branch: p.git_branch.clone(),
                git: p.git_status.clone(),
                lattice: p.lattice_status.clone(),
                rustc: p.rustc_status.clone(),
                files: p.file_count,
                owner: p.ownership.root_owner.clone(),
                org: p.ownership.organization.clone(),
                fork: p.ownership.is_fork,
            })
            .collect();

        let compact = CompactDashboard {
            total: projects.len(),
            indexed: projects.iter().filter(|p| p.lattice == "indexed").count(),
            compiled: projects.iter().filter(|p| p.rustc == "compiled").count(),
            projects,
        };

        let response = ApiResponse {
            success: true,
            data: Some(compact),
            error: None,
        };

        serde_json::to_string_pretty(&response)
            .map_err(|e| format!("JSON serialization error: {}", e))
    }

    pub fn get_lattice_status_json() -> Result<String, String> {
        let manager = ValueLatticeManager::new(
            "/home/mdupont/zombie_driver2/target/release/value_lattice_server".to_string(),
        );

        let status = manager.status();
        let response = ApiResponse {
            success: true,
            data: Some(status),
            error: None,
        };

        serde_json::to_string_pretty(&response)
            .map_err(|e| format!("JSON serialization error: {}", e))
    }

    pub fn start_lattice() -> Result<String, String> {
        let manager = ValueLatticeManager::new(
            "/home/mdupont/zombie_driver2/target/release/value_lattice_server".to_string(),
        );

        match manager.start() {
            Ok(_) => {
                let response = ApiResponse {
                    success: true,
                    data: Some("Lattice process started"),
                    error: None,
                };
                serde_json::to_string_pretty(&response)
                    .map_err(|e| format!("JSON serialization error: {}", e))
            }
            Err(e) => {
                let response = ApiResponse::<String> {
                    success: false,
                    data: None,
                    error: Some(e),
                };
                serde_json::to_string_pretty(&response)
                    .map_err(|e| format!("JSON serialization error: {}", e))
            }
        }
    }

    pub fn get_top_owners_json() -> Result<String, String> {
        let mut importer = GitHubDataImporter::new();
        importer.load_from_index("/home/mdupont/nix/index/github_meta-introspector_repos.json")?;

        let top_owners = importer.get_top_owners(20);
        let response = ApiResponse {
            success: true,
            data: Some(top_owners),
            error: None,
        };

        serde_json::to_string_pretty(&response)
            .map_err(|e| format!("JSON serialization error: {}", e))
    }

    pub fn get_git_analysis_json() -> Result<String, String> {
        let report = GitAnalyzer::analyze_all_projects("/mnt/data1/meta-introspector")?;
        let response = ApiResponse {
            success: true,
            data: Some(report),
            error: None,
        };

        serde_json::to_string_pretty(&response)
            .map_err(|e| format!("JSON serialization error: {}", e))
    }
}
