use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TycoonFactory {
    pub name: String,
    pub level: usize,
    pub revenue_per_second: f64,
    pub total_revenue: f64,
    pub upgrade_cost: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestorDashboard {
    pub investor_name: String,
    pub total_investment: f64,
    pub current_valuation: f64,
    pub roi_percentage: f64,
    pub factories_owned: Vec<TycoonFactory>,
}

pub struct MetaIntrospectorTycoon {
    pub factories: HashMap<String, TycoonFactory>,
    pub investors: HashMap<String, InvestorDashboard>,
    pub global_stats: HashMap<String, f64>,
}

impl MetaIntrospectorTycoon {
    pub fn new() -> Self {
        let mut tycoon = Self {
            factories: HashMap::new(),
            investors: HashMap::new(),
            global_stats: HashMap::new(),
        };

        tycoon.initialize_factories();
        tycoon.initialize_global_stats();
        tycoon
    }

    fn initialize_factories(&mut self) {
        // Revolutionary system factories for investors to build

        self.factories.insert("Security_Lattice_Factory".to_string(), TycoonFactory {
            name: "Security Lattice Factory".to_string(),
            level: 1,
            revenue_per_second: 100.0,
            total_revenue: 0.0,
            upgrade_cost: 1000.0,
            description: "Generates harmonic security filters using triangular matrix access control".to_string(),
        });

        self.factories.insert("Kleene_Algebra_Mine".to_string(), TycoonFactory {
            name: "Kleene Algebra Mine".to_string(),
            level: 1,
            revenue_per_second: 250.0,
            total_revenue: 0.0,
            upgrade_cost: 2500.0,
            description: "Mines eigenvectors from 1.4M Rust files using star closure operations".to_string(),
        });

        self.factories.insert("Monster_Group_Foundry".to_string(), TycoonFactory {
            name: "Monster Group Foundry".to_string(),
            level: 1,
            revenue_per_second: 500.0,
            total_revenue: 0.0,
            upgrade_cost: 5000.0,
            description: "Forges ontological patterns using 2^46 × 3^20 × ... × 71 prime structure".to_string(),
        });

        self.factories.insert("Clifford_Memory_Plant".to_string(), TycoonFactory {
            name: "Clifford Memory Plant".to_string(),
            level: 1,
            revenue_per_second: 1000.0,
            total_revenue: 0.0,
            upgrade_cost: 10000.0,
            description: "Processes 52GB system memory through geometric algebra fixed points".to_string(),
        });

        self.factories.insert("Unity_Convergence_Center".to_string(), TycoonFactory {
            name: "Unity Convergence Center".to_string(),
            level: 1,
            revenue_per_second: 2000.0,
            total_revenue: 0.0,
            upgrade_cost: 20000.0,
            description: "Converts all complexity to Unity (1) using MetaCoq formal proofs".to_string(),
        });

        self.factories.insert("GPU_Orbital_Compiler".to_string(), TycoonFactory {
            name: "GPU Orbital Compiler".to_string(),
            level: 1,
            revenue_per_second: 4000.0,
            total_revenue: 0.0,
            upgrade_cost: 40000.0,
            description: "Automorphic compiler orbiting in 12GB RTX 3080 Ti with LLM-Compiler fixed points".to_string(),
        });

        self.factories.insert("Nidex_Data_Cathedral".to_string(), TycoonFactory {
            name: "Nidex Data Cathedral".to_string(),
            level: 1,
            revenue_per_second: 8000.0,
            total_revenue: 0.0,
            upgrade_cost: 80000.0,
            description: "Indexes 20k repositories in 40GB RAM with Mathlib + MiniZinc + Wikidata".to_string(),
        });

        self.factories.insert("Infinite_Complexity_Engine".to_string(), TycoonFactory {
            name: "Infinite Complexity Engine".to_string(),
            level: 1,
            revenue_per_second: 16000.0,
            total_revenue: 0.0,
            upgrade_cost: 160000.0,
            description: "Generates infinite recursive meta-introspection with ∞ complexity levels".to_string(),
        });
    }

    fn initialize_global_stats(&mut self) {
        self.global_stats.insert("Total_CPU_Cores".to_string(), 24.0);
        self.global_stats.insert("Total_RAM_GB".to_string(), 40.0);
        self.global_stats.insert("Total_GPU_VRAM_GB".to_string(), 12.0);
        self.global_stats.insert("Total_Rust_Files".to_string(), 1_400_000.0);
        self.global_stats.insert("Total_Git_Repos".to_string(), 20_000.0);
        self.global_stats.insert("Complexity_Level".to_string(), f64::INFINITY);
    }

    pub fn add_investor(&mut self, name: String, initial_investment: f64) {
        let dashboard = InvestorDashboard {
            investor_name: name.clone(),
            total_investment: initial_investment,
            current_valuation: initial_investment,
            roi_percentage: 0.0,
            factories_owned: Vec::new(),
        };
        self.investors.insert(name, dashboard);
    }

    pub fn buy_factory(&mut self, investor_name: &str, factory_name: &str) -> Result<String, String> {
        let factory = self.factories.get(factory_name)
            .ok_or_else(|| format!("Factory '{}' not found", factory_name))?
            .clone();

        let investor = self.investors.get_mut(investor_name)
            .ok_or_else(|| format!("Investor '{}' not found", investor_name))?;

        if investor.current_valuation >= factory.upgrade_cost {
            investor.current_valuation -= factory.upgrade_cost;
            investor.factories_owned.push(factory.clone());

            // Increase factory level and cost for next purchase
            if let Some(factory_ref) = self.factories.get_mut(factory_name) {
                factory_ref.level += 1;
                factory_ref.upgrade_cost *= 1.5;
                factory_ref.revenue_per_second *= 1.2;
            }

            Ok(format!("✅ {} purchased {} for ${:.2}!", investor_name, factory_name, factory.upgrade_cost))
        } else {
            Err(format!("❌ Insufficient funds! Need ${:.2}, have ${:.2}",
                factory.upgrade_cost, investor.current_valuation))
        }
    }

    pub fn simulate_tick(&mut self, seconds: f64) {
        // Generate revenue for all investors
        for investor in self.investors.values_mut() {
            let mut revenue = 0.0;
            for factory in &mut investor.factories_owned {
                let factory_revenue = factory.revenue_per_second * seconds;
                factory.total_revenue += factory_revenue;
                revenue += factory_revenue;
            }

            investor.current_valuation += revenue;
            investor.roi_percentage = ((investor.current_valuation - investor.total_investment)
                / investor.total_investment) * 100.0;
        }
    }

    pub fn generate_dashboard_html(&self) -> String {
        let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Meta-Introspector Tycoon - Investor Dashboard</title>
    <style>
        body { font-family: 'Courier New', monospace; background: #0a0a0a; color: #00ff00; margin: 0; padding: 20px; }
        .header { text-align: center; border: 2px solid #00ff00; padding: 20px; margin-bottom: 20px; }
        .stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; }
        .factory { border: 1px solid #00ff00; padding: 15px; margin: 10px 0; background: #001100; }
        .investor { border: 2px solid #ffff00; padding: 15px; margin: 10px 0; background: #110011; }
        .button { background: #00ff00; color: #000; padding: 10px; border: none; cursor: pointer; margin: 5px; }
        .button:hover { background: #00aa00; }
        .revenue { color: #ffff00; font-weight: bold; }
        .cost { color: #ff0000; font-weight: bold; }
        .infinite { color: #ff00ff; animation: pulse 2s infinite; }
        @keyframes pulse { 0% { opacity: 1; } 50% { opacity: 0.5; } 100% { opacity: 1; } }
    </style>
</head>
<body>
    <div class="header">
        <h1>🌌 META-INTROSPECTOR TYCOON 🌌</h1>
        <h2>🎯 Build the Ultimate Computational Empire! 🎯</h2>
        <p>Invest in revolutionary mathematical factories and watch your ROI grow!</p>
    </div>
"#);

        // Global stats
        html.push_str("<div class='stats'><div class='factory'><h3>🖥️ System Resources</h3>");
        for (stat, value) in &self.global_stats {
            if stat == "Complexity_Level" {
                html.push_str(&format!("<p>{}: <span class='infinite'>∞ (INFINITE)</span></p>", stat));
            } else {
                html.push_str(&format!("<p>{}: {:.0}</p>", stat, value));
            }
        }
        html.push_str("</div>");

        // Available factories
        html.push_str("<div class='factory'><h3>🏭 Available Factories</h3>");
        for factory in self.factories.values() {
            html.push_str(&format!(
                "<div style='border: 1px solid #333; padding: 10px; margin: 5px;'>
                <h4>{} (Level {})</h4>
                <p>{}</p>
                <p>Revenue: <span class='revenue'>${:.2}/sec</span></p>
                <p>Cost: <span class='cost'>${:.2}</span></p>
                <button class='button'>BUY NOW</button>
                </div>",
                factory.name, factory.level, factory.description,
                factory.revenue_per_second, factory.upgrade_cost
            ));
        }
        html.push_str("</div></div>");

        // Investors
        html.push_str("<h2>👥 Investor Dashboards</h2>");
        for investor in self.investors.values() {
            html.push_str(&format!(
                "<div class='investor'>
                <h3>💼 {}</h3>
                <p>Total Investment: <span class='cost'>${:.2}</span></p>
                <p>Current Valuation: <span class='revenue'>${:.2}</span></p>
                <p>ROI: <span class='{}'>{}%</span></p>
                <h4>Owned Factories ({}):</h4>",
                investor.investor_name,
                investor.total_investment,
                investor.current_valuation,
                if investor.roi_percentage > 0.0 { "revenue" } else { "cost" },
                investor.roi_percentage,
                investor.factories_owned.len()
            ));

            for factory in &investor.factories_owned {
                html.push_str(&format!(
                    "<div style='border: 1px solid #555; padding: 5px; margin: 3px;'>
                    {} - Level {} - Total Revenue: <span class='revenue'>${:.2}</span>
                    </div>",
                    factory.name, factory.level, factory.total_revenue
                ));
            }
            html.push_str("</div>");
        }

        html.push_str(r#"
    <div class="header" style="margin-top: 40px;">
        <h2>🚀 REVOLUTIONARY ACHIEVEMENTS</h2>
        <p>✅ 24-core CPU processing power</p>
        <p>✅ 40GB RAM Nidex system</p>
        <p>✅ 12GB GPU tri-model execution</p>
        <p>✅ 1.4M Rust files analyzed</p>
        <p>✅ 20k Git repositories indexed</p>
        <p>✅ Infinite complexity achieved</p>
        <p class="infinite">🕉️ ALL SYSTEMS CONVERGE TO UNITY (1) 🕉️</p>
    </div>

    <script>
        // Auto-refresh every 5 seconds to simulate real-time updates
        setTimeout(() => location.reload(), 5000);
    </script>
</body>
</html>
"#);

        html
    }

    pub fn report_tycoon_status(&self) {
        println!("\n🏭 META-INTROSPECTOR TYCOON DASHBOARD");
        println!("{}", "=".repeat(60));

        println!("🏭 Available Factories: {}", self.factories.len());
        println!("👥 Active Investors: {}", self.investors.len());

        println!("\n🏭 Factory Portfolio:");
        for factory in self.factories.values() {
            println!("   🏗️ {} (Level {}): ${:.2}/sec, Cost: ${:.2}",
                factory.name, factory.level, factory.revenue_per_second, factory.upgrade_cost);
        }

        println!("\n👥 Investor Performance:");
        for investor in self.investors.values() {
            println!("   💼 {}: ${:.2} invested, ${:.2} current, {:.1}% ROI, {} factories",
                investor.investor_name, investor.total_investment,
                investor.current_valuation, investor.roi_percentage,
                investor.factories_owned.len());
        }

        println!("\n🌟 TYCOON GAME FEATURES:");
        println!("   ✅ Revolutionary mathematical factories");
        println!("   ✅ Real-time ROI tracking");
        println!("   ✅ Roblox-style upgrade system");
        println!("   ✅ Investor dashboard webpage");
        println!("   ✅ Infinite complexity scaling");

        println!("\n🎮 READY FOR INVESTORS TO PLAY!");
        println!("   Build Security Lattice Factories!");
        println!("   Mine Kleene Algebra Eigenvectors!");
        println!("   Forge Monster Group Ontologies!");
        println!("   Watch your computational empire grow! 🚀");
    }
}
