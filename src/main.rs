// ZOS Server - Zero Ontology System with Plugin Architecture
// AGPL-3.0 License

use std::collections::HashMap;
use std::env;
use tokio;

mod minimal_server_plugin;
mod traits;

use crate::minimal_server_plugin::MinimalServerPlugin;
use crate::traits::{ZOSPlugin, ZOSPluginRegistry};

struct ZOSCore {
    plugins: HashMap<String, Box<dyn ZOSPlugin>>,
}

impl ZOSPluginRegistry for ZOSCore {
    fn register_plugin(&mut self, plugin: Box<dyn ZOSPlugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    fn get_plugin(&self, name: &str) -> Option<&Box<dyn ZOSPlugin>> {
        self.plugins.get(name)
    }

    fn find_command(&self, command: &str) -> Option<&Box<dyn ZOSPlugin>> {
        for plugin in self.plugins.values() {
            if plugin.commands().contains(&command) {
                return Some(plugin);
            }
        }
        None
    }

    fn list_commands(&self) -> Vec<(String, String)> {
        let mut commands = Vec::new();
        for plugin in self.plugins.values() {
            for cmd in plugin.commands() {
                commands.push((cmd.to_string(), plugin.name().to_string()));
            }
        }
        commands.sort();
        commands
    }
}

impl ZOSCore {
    fn new() -> Self {
        let mut core = ZOSCore {
            plugins: HashMap::new(),
        };

        // Register minimal server plugin statically
        let minimal_server = Box::new(MinimalServerPlugin::new());
        core.register_plugin(minimal_server);

        core
    }

    async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<(), String> {
        if let Some(plugin) = self.find_command(command) {
            match plugin.execute(command, args).await {
                Ok(result) => {
                    if !result.is_null() {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&result).unwrap_or_default()
                        );
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Err(format!("Unknown command: {}", command))
        }
    }

    fn show_help(&self) {
        println!("🚀 ZOS Server - Zero Ontology System");
        println!("Foundation build with plugin architecture\n");
        println!("Available commands:");

        let mut by_plugin: HashMap<String, Vec<String>> = HashMap::new();
        for (cmd, plugin) in self.list_commands() {
            by_plugin.entry(plugin).or_default().push(cmd);
        }

        for (plugin, commands) in by_plugin {
            println!("\n  {} plugin:", plugin);
            for cmd in commands {
                println!("    {}", cmd);
            }
        }
        println!();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let core = ZOSCore::new();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        core.show_help();
        return Ok(());
    }

    let command = &args[1];
    let cmd_args = args[2..].to_vec();

    match core.execute_command(command, cmd_args).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
