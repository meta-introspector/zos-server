use crate::common::ffi_plugin::FfiPlugin;

pub struct ItilPlugin {
    plugin: FfiPlugin,
}

impl ItilPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        Ok(ItilPlugin {
            plugin: FfiPlugin::new(plugin_path)?,
        })
    }

    pub fn create_incident(&self, title: &str, description: &str) -> Result<i32, String> {
        self.plugin
            .call_two_string_int_fn(b"itil_create_incident", title, description)
    }

    pub fn update_ticket(&self, ticket_id: i32, update: &str) -> Result<(), String> {
        let result = self
            .plugin
            .call_string_int_fn(b"itil_update_ticket", &format!("{}:{}", ticket_id, update))?;
        if result >= 0 {
            Ok(())
        } else {
            Err(format!("Update failed: {}", result))
        }
    }

    pub fn get_service_status(&self, service: &str) -> Result<String, String> {
        self.plugin
            .call_string_fn(b"itil_get_service_status", service)
    }
}

pub struct C4Plugin {
    plugin: FfiPlugin,
}

impl C4Plugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        Ok(C4Plugin {
            plugin: FfiPlugin::new(plugin_path)?,
        })
    }

    pub fn generate_diagram(&self, diagram_type: &str, spec: &str) -> Result<String, String> {
        self.plugin.call_string_fn(
            b"c4_generate_diagram",
            &format!("{}:{}", diagram_type, spec),
        )
    }
}

pub struct PlantUmlPlugin {
    plugin: FfiPlugin,
}

impl PlantUmlPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        Ok(PlantUmlPlugin {
            plugin: FfiPlugin::new(plugin_path)?,
        })
    }

    pub fn render_diagram(&self, uml_source: &str, format: &str) -> Result<String, String> {
        self.plugin
            .call_string_fn(b"plantuml_render", &format!("{}:{}", format, uml_source))
    }
}
