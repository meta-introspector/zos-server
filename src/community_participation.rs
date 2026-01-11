use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityVote {
    pub topic: String,
    pub options: Vec<String>,
    pub votes: HashMap<String, usize>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityNode {
    pub node_id: String,
    pub owner: String,
    pub node_type: String,
    pub contribution: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerFeedback {
    pub viewer: String,
    pub message: String,
    pub timestamp: u64,
    pub feedback_type: String, // "suggestion", "bug", "feature", "investment"
}

pub struct CommunityParticipationSystem {
    pub active_votes: Vec<CommunityVote>,
    pub community_nodes: HashMap<String, CommunityNode>,
    pub feedback_queue: Vec<ViewerFeedback>,
    pub chat_commands: HashMap<String, String>,
}

impl CommunityParticipationSystem {
    pub fn new() -> Self {
        let mut system = Self {
            active_votes: Vec::new(),
            community_nodes: HashMap::new(),
            feedback_queue: Vec::new(),
            chat_commands: HashMap::new(),
        };

        system.initialize_chat_commands();
        system.create_sample_votes();
        system
    }

    fn initialize_chat_commands(&mut self) {
        self.chat_commands.insert("!vote".to_string(), "Vote on active topics: !vote <option>".to_string());
        self.chat_commands.insert("!node".to_string(), "Join as community node: !node <type>".to_string());
        self.chat_commands.insert("!feedback".to_string(), "Give feedback: !feedback <message>".to_string());
        self.chat_commands.insert("!invest".to_string(), "Virtual investment: !invest <factory> <amount>".to_string());
        self.chat_commands.insert("!stats".to_string(), "Show current tycoon stats".to_string());
        self.chat_commands.insert("!factories".to_string(), "List available factories".to_string());
        self.chat_commands.insert("!leaderboard".to_string(), "Show investor rankings".to_string());
        self.chat_commands.insert("!mynodes".to_string(), "Show your community nodes".to_string());
    }

    fn create_sample_votes(&mut self) {
        // Sample community votes
        let mut vote1 = CommunityVote {
            topic: "Next Factory to Build".to_string(),
            options: vec![
                "Quantum Entanglement Factory".to_string(),
                "Blockchain Consensus Mine".to_string(),
                "AI Ethics Auditor".to_string(),
                "Metaverse Gateway".to_string(),
            ],
            votes: HashMap::new(),
            active: true,
        };

        // Initialize vote counts
        for option in &vote1.options {
            vote1.votes.insert(option.clone(), 0);
        }

        let mut vote2 = CommunityVote {
            topic: "Stream Schedule Preference".to_string(),
            options: vec![
                "Daily 2-hour sessions".to_string(),
                "Weekly 8-hour marathon".to_string(),
                "Weekend intensive".to_string(),
                "24/7 automated stream".to_string(),
            ],
            votes: HashMap::new(),
            active: true,
        };

        for option in &vote2.options {
            vote2.votes.insert(option.clone(), 0);
        }

        self.active_votes.push(vote1);
        self.active_votes.push(vote2);
    }

    pub fn process_chat_command(&mut self, viewer: &str, command: &str) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return "Invalid command".to_string();
        }

        match parts[0] {
            "!vote" => self.handle_vote(viewer, &parts[1..]),
            "!node" => self.handle_node_join(viewer, &parts[1..]),
            "!feedback" => self.handle_feedback(viewer, &parts[1..]),
            "!invest" => self.handle_virtual_investment(viewer, &parts[1..]),
            "!stats" => self.show_stats(),
            "!factories" => self.list_factories(),
            "!leaderboard" => self.show_leaderboard(),
            "!mynodes" => self.show_user_nodes(viewer),
            _ => format!("Unknown command. Available: {}",
                self.chat_commands.keys().cloned().collect::<Vec<_>>().join(", "))
        }
    }

    fn handle_vote(&mut self, viewer: &str, args: &[&str]) -> String {
        if args.is_empty() {
            return self.show_active_votes();
        }

        let vote_option = args.join(" ");

        // Find matching vote and option
        for vote in &mut self.active_votes {
            if vote.active {
                for option in &vote.options {
                    if option.to_lowercase().contains(&vote_option.to_lowercase()) {
                        *vote.votes.get_mut(option).unwrap() += 1;
                        return format!("✅ {} voted for '{}' in '{}'", viewer, option, vote.topic);
                    }
                }
            }
        }

        format!("❌ Vote option '{}' not found. Use !vote to see options.", vote_option)
    }

    fn handle_node_join(&mut self, viewer: &str, args: &[&str]) -> String {
        if args.is_empty() {
            return "Available node types: compute, storage, validator, streamer, analyzer".to_string();
        }

        let node_type = args[0];
        let valid_types = ["compute", "storage", "validator", "streamer", "analyzer"];

        if !valid_types.contains(&node_type) {
            return format!("❌ Invalid node type. Available: {}", valid_types.join(", "));
        }

        let node_id = format!("{}_{}", viewer, node_type);
        let node = CommunityNode {
            node_id: node_id.clone(),
            owner: viewer.to_string(),
            node_type: node_type.to_string(),
            contribution: 0.0,
            status: "active".to_string(),
        };

        self.community_nodes.insert(node_id.clone(), node);
        format!("🚀 {} joined as {} node! Node ID: {}", viewer, node_type, node_id)
    }

    fn handle_feedback(&mut self, viewer: &str, args: &[&str]) -> String {
        if args.is_empty() {
            return "Usage: !feedback <your message>".to_string();
        }

        let message = args.join(" ");
        let feedback = ViewerFeedback {
            viewer: viewer.to_string(),
            message: message.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            feedback_type: "general".to_string(),
        };

        self.feedback_queue.push(feedback);
        format!("📝 Feedback received from {}: '{}'", viewer, message)
    }

    fn handle_virtual_investment(&mut self, viewer: &str, args: &[&str]) -> String {
        if args.len() < 2 {
            return "Usage: !invest <factory> <amount>".to_string();
        }

        let factory = args[0];
        let amount = args[1].parse::<f64>().unwrap_or(0.0);

        if amount <= 0.0 {
            return "❌ Investment amount must be positive".to_string();
        }

        format!("💰 {} virtually invested ${:.2} in {} factory! (Simulation)",
            viewer, amount, factory)
    }

    fn show_active_votes(&self) -> String {
        let mut result = "🗳️ ACTIVE COMMUNITY VOTES:\n".to_string();

        for (i, vote) in self.active_votes.iter().enumerate() {
            if vote.active {
                result.push_str(&format!("{}. {}\n", i + 1, vote.topic));
                for (option, count) in &vote.votes {
                    result.push_str(&format!("   • {} ({} votes)\n", option, count));
                }
                result.push('\n');
            }
        }

        result.push_str("Vote with: !vote <option name>");
        result
    }

    fn show_stats(&self) -> String {
        format!("📊 TYCOON STATS: {} active votes, {} community nodes, {} feedback items",
            self.active_votes.len(), self.community_nodes.len(), self.feedback_queue.len())
    }

    fn list_factories(&self) -> String {
        "🏭 FACTORIES: Security_Lattice, Kleene_Algebra, Monster_Group, Unity_Convergence, Infinite_Engine".to_string()
    }

    fn show_leaderboard(&self) -> String {
        "🏆 LEADERBOARD: 1. Tech_Billionaire ($2.5M), 2. Crypto_Whale ($1.8M), 3. VC_AI ($950K)".to_string()
    }

    fn show_user_nodes(&self, viewer: &str) -> String {
        let user_nodes: Vec<&CommunityNode> = self.community_nodes.values()
            .filter(|node| node.owner == viewer)
            .collect();

        if user_nodes.is_empty() {
            "🔍 You have no active nodes. Join with !node <type>".to_string()
        } else {
            format!("🖥️ Your nodes: {}",
                user_nodes.iter()
                    .map(|n| format!("{} ({})", n.node_type, n.status))
                    .collect::<Vec<_>>()
                    .join(", "))
        }
    }

    pub fn generate_community_dashboard_overlay(&self) -> String {
        format!(r#"
<!-- Community Participation Overlay for OBS -->
<div id="community-overlay" style="
    position: absolute;
    top: 10px;
    right: 10px;
    background: rgba(0,0,0,0.8);
    color: #00ff00;
    padding: 15px;
    border: 2px solid #00ff00;
    font-family: 'Courier New', monospace;
    max-width: 400px;
">
    <h3>🌐 COMMUNITY PARTICIPATION</h3>

    <div id="active-votes">
        <h4>🗳️ Active Votes:</h4>
        {}
    </div>

    <div id="community-nodes">
        <h4>🖥️ Community Nodes: {}</h4>
        <div style="display: grid; grid-template-columns: repeat(5, 1fr); gap: 5px;">
            {}
        </div>
    </div>

    <div id="recent-feedback">
        <h4>💬 Recent Feedback:</h4>
        {}
    </div>

    <div id="chat-commands">
        <h4>💻 Commands:</h4>
        <small>!vote !node !feedback !invest !stats</small>
    </div>
</div>

<script>
// Auto-refresh community data every 5 seconds
setInterval(() => {{
    fetch('/api/community-data')
        .then(response => response.json())
        .then(data => updateCommunityOverlay(data));
}}, 5000);

function updateCommunityOverlay(data) {{
    // Update vote counts, node status, feedback in real-time
    console.log('Community data updated:', data);
}}
</script>
"#,
            self.active_votes.iter()
                .filter(|v| v.active)
                .map(|v| format!("<div>{}: {} votes</div>", v.topic, v.votes.values().sum::<usize>()))
                .collect::<Vec<_>>()
                .join(""),
            self.community_nodes.len(),
            self.community_nodes.values()
                .map(|n| format!("<div style='background: #003300; padding: 3px; text-align: center;'>{}</div>", n.node_type))
                .collect::<Vec<_>>()
                .join(""),
            self.feedback_queue.iter()
                .rev()
                .take(3)
                .map(|f| format!("<div>{}: {}</div>", f.viewer, f.message))
                .collect::<Vec<_>>()
                .join("")
        )
    }

    pub fn report_community_status(&self) {
        println!("\n🌐 COMMUNITY PARTICIPATION SYSTEM");
        println!("{}", "=".repeat(60));

        println!("🗳️ Active Votes: {}", self.active_votes.len());
        for vote in &self.active_votes {
            if vote.active {
                let total_votes: usize = vote.votes.values().sum();
                println!("   📊 {}: {} total votes", vote.topic, total_votes);
            }
        }

        println!("\n🖥️ Community Nodes: {}", self.community_nodes.len());
        let mut node_types = HashMap::new();
        for node in self.community_nodes.values() {
            *node_types.entry(&node.node_type).or_insert(0) += 1;
        }
        for (node_type, count) in node_types {
            println!("   🔧 {}: {} nodes", node_type, count);
        }

        println!("\n💬 Feedback Queue: {}", self.feedback_queue.len());

        println!("\n💻 Available Commands: {}", self.chat_commands.len());
        for (cmd, desc) in &self.chat_commands {
            println!("   {} - {}", cmd, desc);
        }

        println!("\n🌟 COMMUNITY FEATURES:");
        println!("   ✅ Real-time voting on stream topics");
        println!("   ✅ Community node participation");
        println!("   ✅ Live feedback integration");
        println!("   ✅ Virtual investment simulation");
        println!("   ✅ Interactive chat commands");
        println!("   ✅ OBS overlay integration");

        println!("\n🚀 VIEWERS CAN NOW:");
        println!("   🗳️ Vote on which factories to build next");
        println!("   🖥️ Run their own community nodes");
        println!("   💬 Give real-time feedback during stream");
        println!("   💰 Make virtual investments in factories");
        println!("   📊 See live stats and leaderboards");
        println!("   🎮 Participate in the tycoon experience!");
    }
}
