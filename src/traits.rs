use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait ZOSPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn commands(&self) -> Vec<&'static str>;
    async fn execute(&self, command: &str, args: Vec<String>) -> Result<Value, String>;
}

pub trait ZOSPluginRegistry {
    fn register_plugin(&mut self, plugin: Box<dyn ZOSPlugin>);
    fn get_plugin(&self, name: &str) -> Option<&Box<dyn ZOSPlugin>>;
    fn find_command(&self, command: &str) -> Option<&Box<dyn ZOSPlugin>>;
    fn list_commands(&self) -> Vec<(String, String)>; // (command, plugin_name)
}
