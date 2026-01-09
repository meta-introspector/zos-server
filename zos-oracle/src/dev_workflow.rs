use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServer {
    pub server_id: String,
    pub owner_address: String,
    pub status: ServerStatus,
    pub eigenmatrix_version: String,
    pub stakeholders: HashMap<String, Stakeholder>,
    pub audit_items: HashMap<String, AuditItem>,
    pub proposals: HashMap<String, Proposal>,
    pub test_results: HashMap<String, TestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Development,
    Testing,
    StakeholderReview,
    AuditPhase,
    ProposalPhase,
    ReadyForDAO,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stakeholder {
    pub address: String,
    pub role: StakeholderRole,
    pub onboarded_at: u64,
    pub trust_level: u32,
    pub audit_flags: Vec<String>,
    pub proposals_submitted: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StakeholderRole {
    Developer,
    Auditor,
    Investor,
    CommunityMember,
    TechnicalReviewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditItem {
    pub item_id: String,
    pub component: String,
    pub description: String,
    pub flagged_by: String,
    pub severity: AuditSeverity,
    pub status: AuditStatus,
    pub revision_requested: Option<String>,
    pub dev_response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditStatus {
    Open,
    UnderReview,
    RevisionRequested,
    Resolved,
    Accepted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub proposal_id: String,
    pub title: String,
    pub description: String,
    pub proposed_by: String,
    pub changes: Vec<ProposedChange>,
    pub votes: HashMap<String, Vote>,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedChange {
    pub component: String,
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Vote {
    Approve,
    Reject,
    RequestChanges(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Draft,
    UnderReview,
    Voting,
    Approved,
    Rejected,
    Implemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub component: String,
    pub test_type: String,
    pub passed: bool,
    pub output: String,
    pub timestamp: u64,
}

impl DevServer {
    pub fn new(owner_address: &str) -> Self {
        Self {
            server_id: format!("dev_{}", &owner_address[..8]),
            owner_address: owner_address.to_string(),
            status: ServerStatus::Development,
            eigenmatrix_version: "1.0.0-dev".to_string(),
            stakeholders: HashMap::new(),
            audit_items: HashMap::new(),
            proposals: HashMap::new(),
            test_results: HashMap::new(),
        }
    }

    pub fn launch_dev_server(&mut self) -> Result<String, String> {
        println!("🚀 Launching development server: {}", self.server_id);

        // Start with eigenmatrix bootstrap
        let bootstrap_result = self.run_bootstrap_test()?;

        // Generate initial audit report
        self.generate_initial_audit();

        self.status = ServerStatus::Testing;

        Ok(format!("Dev server launched at http://localhost:3000/dev/{}", self.server_id))
    }

    pub fn onboard_stakeholder(&mut self, address: &str, role: StakeholderRole) -> Result<(), String> {
        let stakeholder = Stakeholder {
            address: address.to_string(),
            role: role.clone(),
            onboarded_at: chrono::Utc::now().timestamp() as u64,
            trust_level: match role {
                StakeholderRole::Developer => 80,
                StakeholderRole::Auditor => 90,
                StakeholderRole::Investor => 60,
                StakeholderRole::CommunityMember => 40,
                StakeholderRole::TechnicalReviewer => 85,
            },
            audit_flags: Vec::new(),
            proposals_submitted: 0,
        };

        println!("👥 Onboarded stakeholder: {} as {:?}", &address[..8], role);
        self.stakeholders.insert(address.to_string(), stakeholder);

        // Send onboarding package
        self.send_onboarding_package(address)?;

        Ok(())
    }

    pub fn flag_component(&mut self, flagger: &str, component: &str, description: &str, severity: AuditSeverity) -> Result<String, String> {
        // Check if stakeholder can flag
        let stakeholder = self.stakeholders.get(flagger)
            .ok_or("Stakeholder not found")?;

        if stakeholder.trust_level < 50 {
            return Err("Insufficient trust level to flag components".to_string());
        }

        let audit_id = format!("audit_{}_{}", component, chrono::Utc::now().timestamp());

        let audit_item = AuditItem {
            item_id: audit_id.clone(),
            component: component.to_string(),
            description: description.to_string(),
            flagged_by: flagger.to_string(),
            severity,
            status: AuditStatus::Open,
            revision_requested: None,
            dev_response: None,
        };

        println!("🚩 Component flagged: {} by {}", component, &flagger[..8]);
        self.audit_items.insert(audit_id.clone(), audit_item);

        Ok(audit_id)
    }

    pub fn respond_to_audit(&mut self, audit_id: &str, response: &str) -> Result<(), String> {
        let audit_item = self.audit_items.get_mut(audit_id)
            .ok_or("Audit item not found")?;

        audit_item.dev_response = Some(response.to_string());
        audit_item.status = AuditStatus::UnderReview;

        println!("💬 Dev response to audit {}: {}", &audit_id[..12], &response[..50]);

        Ok(())
    }

    pub fn request_revision(&mut self, audit_id: &str, revision_details: &str) -> Result<(), String> {
        let audit_item = self.audit_items.get_mut(audit_id)
            .ok_or("Audit item not found")?;

        audit_item.revision_requested = Some(revision_details.to_string());
        audit_item.status = AuditStatus::RevisionRequested;

        println!("🔄 Revision requested for audit {}", &audit_id[..12]);

        Ok(())
    }

    pub fn submit_proposal(&mut self, proposer: &str, title: &str, description: &str, changes: Vec<ProposedChange>) -> Result<String, String> {
        let stakeholder = self.stakeholders.get_mut(proposer)
            .ok_or("Stakeholder not found")?;

        let proposal_id = format!("prop_{}_{}", proposer[..8].to_string(), stakeholder.proposals_submitted);

        let proposal = Proposal {
            proposal_id: proposal_id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            proposed_by: proposer.to_string(),
            changes,
            votes: HashMap::new(),
            status: ProposalStatus::Draft,
        };

        stakeholder.proposals_submitted += 1;

        println!("📋 Proposal submitted: {} by {}", title, &proposer[..8]);
        self.proposals.insert(proposal_id.clone(), proposal);

        Ok(proposal_id)
    }

    pub fn vote_on_proposal(&mut self, voter: &str, proposal_id: &str, vote: Vote) -> Result<(), String> {
        let stakeholder = self.stakeholders.get(voter)
            .ok_or("Stakeholder not found")?;

        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        proposal.votes.insert(voter.to_string(), vote);

        println!("🗳️  Vote cast on proposal {} by {}", &proposal_id[..12], &voter[..8]);

        // Check if proposal has enough votes to proceed
        self.check_proposal_consensus(proposal_id)?;

        Ok(())
    }

    pub fn get_stakeholder_dashboard(&self, address: &str) -> Result<String, String> {
        let stakeholder = self.stakeholders.get(address)
            .ok_or("Stakeholder not found")?;

        let dashboard = serde_json::json!({
            "server_id": self.server_id,
            "server_status": self.status,
            "your_role": stakeholder.role,
            "trust_level": stakeholder.trust_level,
            "open_audits": self.audit_items.values().filter(|a| a.status == AuditStatus::Open).count(),
            "your_flags": stakeholder.audit_flags.len(),
            "active_proposals": self.proposals.values().filter(|p| matches!(p.status, ProposalStatus::Voting)).count(),
            "your_proposals": stakeholder.proposals_submitted,
            "eigenmatrix_version": self.eigenmatrix_version,
            "components": [
                "foundation", "build_system", "core_services",
                "plugin_loader", "governance", "libp2p_verbs"
            ]
        });

        Ok(dashboard.to_string())
    }

    fn run_bootstrap_test(&mut self) -> Result<String, String> {
        println!("🧪 Running bootstrap test...");

        // Simulate bootstrap test
        let test_result = TestResult {
            test_id: "bootstrap_001".to_string(),
            component: "eigenmatrix".to_string(),
            test_type: "bootstrap".to_string(),
            passed: true,
            output: "All stages completed successfully".to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.test_results.insert("bootstrap_001".to_string(), test_result);

        Ok("Bootstrap test passed".to_string())
    }

    fn generate_initial_audit(&mut self) {
        println!("📊 Generating initial audit items...");

        let components = vec![
            "foundation", "build_system", "core_services",
            "plugin_loader", "governance", "libp2p_verbs"
        ];

        for component in components {
            let audit_id = format!("initial_{}_{}", component, chrono::Utc::now().timestamp());

            let audit_item = AuditItem {
                item_id: audit_id.clone(),
                component: component.to_string(),
                description: format!("Initial review of {} component", component),
                flagged_by: "system".to_string(),
                severity: AuditSeverity::Info,
                status: AuditStatus::Open,
                revision_requested: None,
                dev_response: None,
            };

            self.audit_items.insert(audit_id, audit_item);
        }
    }

    fn send_onboarding_package(&self, address: &str) -> Result<(), String> {
        println!("📦 Sending onboarding package to {}", &address[..8]);

        // Would send:
        // - Server access credentials
        // - Audit guidelines
        // - Proposal templates
        // - Component documentation
        // - Test environment access

        Ok(())
    }

    fn check_proposal_consensus(&mut self, proposal_id: &str) -> Result<(), String> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        let total_stakeholders = self.stakeholders.len();
        let votes_cast = proposal.votes.len();

        // Need majority of stakeholders to vote
        if votes_cast >= (total_stakeholders * 2 / 3) {
            let approvals = proposal.votes.values()
                .filter(|v| matches!(v, Vote::Approve))
                .count();

            if approvals > votes_cast / 2 {
                proposal.status = ProposalStatus::Approved;
                println!("✅ Proposal approved: {}", proposal.title);
            } else {
                proposal.status = ProposalStatus::Rejected;
                println!("❌ Proposal rejected: {}", proposal.title);
            }
        }

        Ok(())
    }
}
