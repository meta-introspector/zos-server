use axum::response::Response;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type HandlerFn = Box<dyn Fn() -> Response<String> + Send + Sync>;

#[derive(Clone)]
pub struct WebPlugin {
    pub name: String,
    pub path: String,
    pub description: String,
    pub icon: String,
    pub handler: Arc<HandlerFn>,
}

pub struct PluginRegistry {
    plugins: Arc<Mutex<HashMap<String, WebPlugin>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_plugin(&self, plugin: WebPlugin) {
        let mut plugins = self.plugins.lock().unwrap();
        plugins.insert(plugin.path.clone(), plugin);
    }

    pub fn get_plugin(&self, path: &str) -> Option<WebPlugin> {
        let plugins = self.plugins.lock().unwrap();
        plugins.get(path).cloned()
    }

    pub fn list_plugins(&self) -> Vec<WebPlugin> {
        let plugins = self.plugins.lock().unwrap();
        plugins.values().cloned().collect()
    }

    pub fn generate_navigation_html(&self) -> String {
        let plugins = self.list_plugins();
        let mut nav_html = String::new();

        for plugin in plugins {
            nav_html.push_str(&format!(
                r#"<a href="{}" class="nav-link">{} {}</a>"#,
                plugin.path, plugin.icon, plugin.name
            ));
        }

        nav_html
    }

    pub fn generate_dashboard_cards(&self) -> String {
        let plugins = self.list_plugins();
        let mut cards_html = String::new();

        for plugin in plugins {
            cards_html.push_str(&format!(
                r#"
                <div class="dashboard-card">
                    <div class="card-icon">{}</div>
                    <h3>{}</h3>
                    <p>{}</p>
                    <a href="{}" class="card-link">Open →</a>
                </div>
                "#,
                plugin.icon, plugin.name, plugin.description, plugin.path
            ));
        }

        cards_html
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
