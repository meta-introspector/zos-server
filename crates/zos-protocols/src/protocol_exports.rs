// Protocol Export Extensions - MCP, SOAP, OpenAPI/REST
use crate::verb_export::{VerbExport, VerbExporter};
use serde_json::json;

impl VerbExporter for VerbExport {
    // ... existing methods ...

    fn export_to_mcp(&self) -> Result<String, String> {
        // Export as Model Context Protocol format
        let mut mcp = json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "result": {
                "tools": []
            }
        });

        let tools = mcp["result"]["tools"].as_array_mut().unwrap();
        for verb in &self.verbs {
            tools.push(json!({
                "name": format!("{}_{}", verb.plugin_name, verb.verb_name),
                "description": verb.description,
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "args": {
                            "type": "array",
                            "description": "Function arguments"
                        }
                    }
                }
            }));
        }

        serde_json::to_string_pretty(&mcp)
            .map_err(|e| format!("MCP serialization failed: {}", e))
    }

    fn export_to_soap(&self) -> Result<String, String> {
        // Export as SOAP WSDL
        let mut wsdl = String::new();
        wsdl.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        wsdl.push_str("<definitions xmlns=\"http://schemas.xmlsoap.org/wsdl/\" ");
        wsdl.push_str("targetNamespace=\"http://zos-server.org/wsdl\">\n");

        wsdl.push_str("  <types>\n");
        wsdl.push_str("    <schema targetNamespace=\"http://zos-server.org/types\">\n");
        wsdl.push_str("      <element name=\"VerbRequest\" type=\"string\"/>\n");
        wsdl.push_str("      <element name=\"VerbResponse\" type=\"string\"/>\n");
        wsdl.push_str("    </schema>\n");
        wsdl.push_str("  </types>\n");

        for verb in &self.verbs {
            wsdl.push_str(&format!("  <message name=\"{}Request\">\n", verb.verb_name));
            wsdl.push_str("    <part name=\"parameters\" element=\"VerbRequest\"/>\n");
            wsdl.push_str("  </message>\n");
            wsdl.push_str(&format!("  <message name=\"{}Response\">\n", verb.verb_name));
            wsdl.push_str("    <part name=\"parameters\" element=\"VerbResponse\"/>\n");
            wsdl.push_str("  </message>\n");
        }

        wsdl.push_str("</definitions>\n");
        Ok(wsdl)
    }

    fn export_to_openapi(&self) -> Result<String, String> {
        // Export as OpenAPI 3.0 specification
        let mut openapi = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "ZOS Server API",
                "version": "1.0.0",
                "description": "Zero Ontology System Plugin API"
            },
            "servers": [
                {
                    "url": "http://localhost:8080/api/v1",
                    "description": "Local ZOS Server"
                }
            ],
            "paths": {}
        });

        let paths = openapi["paths"].as_object_mut().unwrap();
        for verb in &self.verbs {
            let path = format!("/{}/{}", verb.plugin_name, verb.verb_name);
            paths.insert(path, json!({
                "post": {
                    "summary": verb.description,
                    "operationId": format!("{}_{}", verb.plugin_name, verb.verb_name),
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "args": {
                                            "type": "array",
                                            "items": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Success",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "result": { "type": "string" },
                                            "error": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }));
        }

        serde_json::to_string_pretty(&openapi)
            .map_err(|e| format!("OpenAPI serialization failed: {}", e))
    }

    fn export_to_rest_routes(&self) -> Result<String, String> {
        // Export as REST route definitions
        let mut routes = String::new();
        routes.push_str("# ZOS Server REST Routes\n\n");

        for verb in &self.verbs {
            routes.push_str(&format!("POST /{}/{}\n", verb.plugin_name, verb.verb_name));
            routes.push_str(&format!("  Description: {}\n", verb.description));
            routes.push_str(&format!("  Function: {}\n", verb.function_name));
            routes.push_str("  Content-Type: application/json\n");
            routes.push_str("  Body: { \"args\": [...] }\n\n");
        }

        Ok(routes)
    }
}
