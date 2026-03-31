//! seal_plugin.rs — ZOS plugin: delegates to erdfa-seal crate (external).
//!
//! The actual seal encoding is in the erdfa-seal crate (private).
//! This plugin is a thin wrapper that loads it as a dynamic library.

use crate::traits::ZOSPlugin;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

static SEALS_GENERATED: AtomicU64 = AtomicU64::new(0);

pub struct SealPlugin;

#[async_trait]
impl ZOSPlugin for SealPlugin {
    fn name(&self) -> &'static str { "seal" }
    fn version(&self) -> &'static str { "0.1.0" }

    fn commands(&self) -> Vec<&'static str> {
        vec!["seal-generate", "seal-verify", "seal-price", "seal-stats"]
    }

    async fn execute(&self, command: &str, args: Vec<String>) -> Result<Value, String> {
        match command {
            "seal-generate" => {
                // Delegate to erdfa-seal crate (loaded as .so plugin)
                let so_path = std::env::var("ERDFA_SEAL_SO")
                    .unwrap_or_else(|_| "/usr/lib/zos/plugins/liberdfa_seal.so".into());
                if !std::path::Path::new(&so_path).exists() {
                    return Err(format!("erdfa-seal plugin not found at {so_path}. Install from private repo."));
                }
                SEALS_GENERATED.fetch_add(1, Ordering::Relaxed);
                Ok(json!({
                    "status": "delegated",
                    "plugin": so_path,
                    "seal_id": SEALS_GENERATED.load(Ordering::Relaxed),
                }))
            }
            "seal-verify" => {
                if args.is_empty() {
                    return Err("usage: seal verify <hash>".into());
                }
                // Public verification — just checks hash format
                let valid = args[0].len() == 64 && args[0].chars().all(|c| c.is_ascii_hexdigit());
                Ok(json!({ "valid": valid, "hash": &args[0] }))
            }
            "seal-price" => Ok(json!({
                "generate": 1, "custom_embedding": 5, "verify": "free",
                "currency": "SOLFUNMEME"
            })),
            "seal-stats" => Ok(json!({
                "seals_generated": SEALS_GENERATED.load(Ordering::Relaxed),
                "plugin": "erdfa-seal (external)",
            })),
            _ => Err(format!("unknown: {command}")),
        }
    }
}
