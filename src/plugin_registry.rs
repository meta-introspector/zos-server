// Plugin Registry - Collects and exports all plugin verbs
use crate::verb_export::{PluginVerb, VerbExport, VerbExporter};
use std::collections::HashMap;

pub struct PluginRegistry {
    plugins: HashMap<String, Vec<PluginVerb>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        PluginRegistry {
            plugins: HashMap::new(),
        }
    }

    pub fn register_plugin_verbs(&mut self, plugin_name: &str, verbs: Vec<PluginVerb>) {
        self.plugins.insert(plugin_name.to_string(), verbs);
    }

    pub fn get_all_verbs(&self) -> VerbExport {
        let mut all_verbs = Vec::new();
        let mut metadata = HashMap::new();
        
        for (plugin_name, verbs) in &self.plugins {
            all_verbs.extend(verbs.clone());
            metadata.insert(format!("{}_count", plugin_name), verbs.len().to_string());
        }

        metadata.insert("total_plugins".to_string(), self.plugins.len().to_string());
        metadata.insert("total_verbs".to_string(), all_verbs.len().to_string());

        VerbExport {
            verbs: all_verbs,
            metadata,
        }
    }

    // Export to all data formats
    pub fn export_all_formats(&self) -> HashMap<String, Result<Vec<u8>, String>> {
        let verb_export = self.get_all_verbs();
        let mut exports = HashMap::new();

        exports.insert("libp2p".to_string(), verb_export.export_to_libp2p());
        exports.insert("parquet".to_string(), verb_export.export_to_parquet());
        exports.insert("protobuf".to_string(), verb_export.export_to_protobuf());
        
        // Text formats converted to bytes
        if let Ok(hf) = verb_export.export_to_huggingface() {
            exports.insert("huggingface".to_string(), Ok(hf.into_bytes()));
        }
        if let Ok(xml) = verb_export.export_to_xml() {
            exports.insert("xml".to_string(), Ok(xml.into_bytes()));
        }
        if let Ok(rdf) = verb_export.export_to_rdf() {
            exports.insert("rdf".to_string(), Ok(rdf.into_bytes()));
        }
        if let Ok(jsonld) = verb_export.export_to_jsonld() {
            exports.insert("jsonld".to_string(), Ok(jsonld.into_bytes()));
        }
        if let Ok(sql) = verb_export.export_to_sql() {
            exports.insert("sql".to_string(), Ok(sql.into_bytes()));
        }
        if let Ok(gobject) = verb_export.export_to_gobject() {
            exports.insert("gobject".to_string(), Ok(gobject.into_bytes()));
        }
        
        // Protocol formats
        if let Ok(mcp) = verb_export.export_to_mcp() {
            exports.insert("mcp".to_string(), Ok(mcp.into_bytes()));
        }
        if let Ok(soap) = verb_export.export_to_soap() {
            exports.insert("soap".to_string(), Ok(soap.into_bytes()));
        }
        if let Ok(openapi) = verb_export.export_to_openapi() {
            exports.insert("openapi".to_string(), Ok(openapi.into_bytes()));
        }
        if let Ok(rest) = verb_export.export_to_rest_routes() {
            exports.insert("rest".to_string(), Ok(rest.into_bytes()));
        }

        exports
    }
}

// Macro to easily register plugin verbs
#[macro_export]
macro_rules! register_plugin_verbs {
    ($registry:expr, $plugin_name:expr, [$(($verb:expr, $func:expr, $desc:expr)),*]) => {
        {
            let mut verbs = Vec::new();
            $(
                verbs.push(PluginVerb {
                    plugin_name: $plugin_name.to_string(),
                    verb_name: $verb.to_string(),
                    function_name: $func.to_string(),
                    parameters: vec![],
                    return_type: "Result<Vec<u8>, String>".to_string(),
                    description: $desc.to_string(),
                });
            )*
            $registry.register_plugin_verbs($plugin_name, verbs);
        }
    };
}
