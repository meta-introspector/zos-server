// ZOS Node Coordinator - Memory-based Plugin Loading and Cooperation
// Loads plugins into memory and coordinates between nodes.

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use tokio::sync::mpsc;
use serde_json::Value;

pub struct ZosNode {
    node_id: PeerId,
    loaded_plugins: HashMap<String, libloading::Library>,
    peer_nodes: HashMap<PeerId, NodeInfo>,
    message_tx: mpsc::UnboundedSender<NodeMessage>,
    message_rx: mpsc::UnboundedReceiver<NodeMessage>,
    transport_tx: Option<mpsc::UnboundedSender<OutboundSyncFrame>>,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    peer_id: PeerId,
    capabilities: Vec<String>,
    plugin_layers: Vec<i32>,
    load_average: f64,
    advertised_inventory: SyncInventory,
    last_reconciliation: Option<ReconciliationPlan>,
    pending_replay: Option<ReplayIntentPlan>,
}

#[derive(Debug, Clone)]
pub enum NodeMessage {
    PluginRequest { plugin_name: String, args: Vec<u8> },
    PluginResponse { result: Result<Vec<u8>, String> },
    LoadBalanceRequest { computation_type: String },
    SyncRequest { layer: i32 },
    SyncInventory {
        peer_id: PeerId,
        inventory: SyncInventory,
    },
    ReconciliationResult {
        peer_id: PeerId,
        plan: ReconciliationPlan,
    },
    ReplayIntentPlanned {
        peer_id: PeerId,
        plan: ReplayIntentPlan,
    },
    SyncAnnouncement {
        peer_id: PeerId,
        inventory: SyncInventory,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundSyncFrame {
    pub peer_id: PeerId,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum SyncWireMessage {
    Inventory(SyncInventory),
    Reconciliation(ReconciliationPlan),
    Announcement(SyncInventory),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum InventoryKind {
    Plugin,
    Artifact,
    Receipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventoryItem {
    kind: InventoryKind,
    id: String,
    digest: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncInventory {
    items: Vec<InventoryItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    replay_metadata_catalog: Vec<ReplayMetadataRecord>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconciliationPlan {
    missing_from_local: Vec<InventoryItem>,
    missing_from_peer: Vec<InventoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReplaySource {
    PeerInventoryGap,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    container_object_ref: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    object_refs: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    related_artifact_refs: Vec<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    proof_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    artifact_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    acknowledged_revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayMetadataRecord {
    kind: InventoryKind,
    id: String,
    metadata: ReplayMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayRecoveryRequest {
    item: InventoryItem,
    metadata: ReplayMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayIntentPlan {
    source: ReplaySource,
    requested_from_peer: Vec<InventoryItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    recovery_requests: Vec<ReplayRecoveryRequest>,
}

impl ZosNode {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::new_internal(None).await
    }

    pub async fn new_with_transport(
        transport_tx: mpsc::UnboundedSender<OutboundSyncFrame>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new_internal(Some(transport_tx)).await
    }

    async fn new_internal(
        transport_tx: Option<mpsc::UnboundedSender<OutboundSyncFrame>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let node_id = PeerId::from(local_key.public());
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        println!("🚀 Initializing ZOS Node: {}", node_id);

        Ok(ZosNode {
            node_id,
            loaded_plugins: HashMap::new(),
            peer_nodes: HashMap::new(),
            message_tx,
            message_rx,
            transport_tx,
        })
    }

    pub fn with_transport_tx(&mut self, transport_tx: mpsc::UnboundedSender<OutboundSyncFrame>) {
        self.transport_tx = Some(transport_tx);
    }

    pub async fn load_all_plugins(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📦 Loading all ZOS plugins into memory...");

        // Load plugins by layer (deepest first)
        self.load_layer_plugins(-4, "Advanced ZK").await?;
        self.load_layer_plugins(-3, "Zero Knowledge").await?;
        self.load_layer_plugins(-2, "Regulatory").await?;
        self.load_layer_plugins(-1, "Governance").await?;
        self.load_layer_plugins(0, "Foundation").await?;
        self.load_layer_plugins(1, "System").await?;
        self.load_layer_plugins(2, "Data Formats").await?;

        println!("✅ All plugins loaded successfully");
        Ok(())
    }

    async fn load_layer_plugins(&mut self, layer: i32, layer_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Loading Layer {}: {}", layer, layer_name);

        match layer {
            -4 => {
                // Advanced ZK plugins
                self.load_plugin("rollup", "/nix/store/.../lib/zos-plugins/rollup_plugin.so").await?;
                self.load_plugin("lattice_folding", "/nix/store/.../lib/zos-plugins/lattice_folding_plugin.so").await?;
                self.load_plugin("hme", "/nix/store/.../lib/zos-plugins/hme_plugin.so").await?;
                self.load_plugin("metacoq", "/nix/store/.../lib/zos-plugins/metacoq_plugin.so").await?;
                self.load_plugin("lean4", "/nix/store/.../lib/zos-plugins/lean4_plugin.so").await?;
            },
            -3 => {
                // Zero Knowledge plugins
                self.load_plugin("zksnark", "/nix/store/.../lib/zos-plugins/zksnark_plugin.so").await?;
                self.load_plugin("zkstark", "/nix/store/.../lib/zos-plugins/zkstark_plugin.so").await?;
                self.load_plugin("correctness", "/nix/store/.../lib/zos-plugins/correctness_plugin.so").await?;
            },
            -2 => {
                // Regulatory plugins
                self.load_plugin("sec", "/nix/store/.../lib/zos-plugins/sec_plugin.so").await?;
                self.load_plugin("quality", "/nix/store/.../lib/zos-plugins/quality_plugin.so").await?;
                self.load_plugin("regulatory", "/nix/store/.../lib/zos-plugins/regulatory_plugin.so").await?;
            },
            -1 => {
                // Governance plugins
                self.load_plugin("voting", "/nix/store/.../lib/zos-plugins/voting_plugin.so").await?;
                self.load_plugin("resource", "/nix/store/.../lib/zos-plugins/resource_plugin.so").await?;
                self.load_plugin("odoo", "/nix/store/.../lib/zos-plugins/odoo_plugin.so").await?;
            },
            0 => {
                // Foundation plugins
                self.load_plugin("lmfdb", "/nix/store/.../lib/zos-plugins/lmfdb_plugin.so").await?;
                self.load_plugin("wikidata", "/nix/store/.../lib/zos-plugins/wikidata_plugin.so").await?;
                self.load_plugin("osm", "/nix/store/.../lib/zos-plugins/osm_plugin.so").await?;
                self.load_plugin("archive", "/nix/store/.../lib/zos-plugins/archive_plugin.so").await?;
                self.load_plugin("sdf", "/nix/store/.../lib/zos-plugins/sdf_plugin.so").await?;
            },
            1 => {
                // System plugins (all 19)
                self.load_plugin("systemd", "/nix/store/.../lib/zos-plugins/systemd_plugin.so").await?;
                self.load_plugin("docker", "/nix/store/.../lib/zos-plugins/docker_plugin.so").await?;
                // ... load all 19 system plugins
            },
            2 => {
                // Data format plugins
                self.load_plugin("parquet", "/nix/store/.../lib/zos-plugins/parquet_plugin.so").await?;
                self.load_plugin("huggingface", "/nix/store/.../lib/zos-plugins/huggingface_plugin.so").await?;
                // ... load all data format plugins
            },
            _ => {}
        }

        Ok(())
    }

    async fn load_plugin(&mut self, name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("  📋 Loading plugin: {}", name);

        // Load plugin using libloading
        let lib = unsafe { libloading::Library::new(path)? };

        // Store plugin in memory
        self.loaded_plugins.insert(name.to_string(), lib);

        println!("  ✅ Plugin {} loaded successfully", name);
        Ok(())
    }

    pub async fn start_cooperation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤝 Starting node cooperation...");

        loop {
            tokio::select! {
                Some(message) = self.message_rx.recv() => {
                    self.handle_node_message(message).await?;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                    self.sync_with_peers().await?;
                }
            }
        }
    }

    async fn handle_node_message(&mut self, message: NodeMessage) -> Result<(), Box<dyn std::error::Error>> {
        match message {
            NodeMessage::PluginRequest { plugin_name, args: _args } => {
                println!("🔧 Handling plugin request: {}", plugin_name);
                // Execute plugin and send response
            },
            NodeMessage::LoadBalanceRequest { computation_type } => {
                println!("⚖️  Load balancing request for: {}", computation_type);
                // Find best node for computation
            },
            NodeMessage::SyncRequest { layer } => {
                println!("🔄 Syncing layer: {}", layer);
                // Synchronize plugin state across nodes
            },
            NodeMessage::SyncInventory { peer_id, inventory } => {
                self.register_peer_inventory(peer_id, inventory);
            },
            NodeMessage::ReconciliationResult { peer_id, plan } => {
                if let Some(node_info) = self.peer_nodes.get_mut(&peer_id) {
                    node_info.last_reconciliation = Some(plan);
                }
            },
            NodeMessage::ReplayIntentPlanned { peer_id, plan } => {
                if let Some(node_info) = self.peer_nodes.get_mut(&peer_id) {
                    node_info.pending_replay = Some(plan);
                }
            }
            NodeMessage::SyncAnnouncement {
                peer_id,
                inventory,
            } => {
                self.register_peer_inventory(peer_id, inventory);
            }
            _ => {}
        }
        Ok(())
    }

    async fn sync_with_peers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Syncing with {} peer nodes", self.peer_nodes.len());

        let local_inventory = self.build_local_inventory();
        self.emit_inventory_announcement(local_inventory.clone())?;
        let peer_ids: Vec<PeerId> = self.peer_nodes.keys().copied().collect();

        for peer_id in peer_ids {
            println!("  📡 Syncing with peer: {}", peer_id);
            let maybe_plan = if let Some(node_info) = self.peer_nodes.get_mut(&peer_id) {
                let plan = Self::reconcile_inventory(&local_inventory, &node_info.advertised_inventory);
                node_info.last_reconciliation = Some(plan.clone());
                let replay_plan = Self::plan_replay_intent(&plan, &node_info.advertised_inventory);
                node_info.pending_replay = replay_plan.clone();
                Some((plan, replay_plan))
            } else {
                None
            };

            if let Some((plan, replay_plan)) = maybe_plan {
                println!(
                    "    ↕ reconciliation: local_missing={}, peer_missing={}",
                    plan.missing_from_local.len(),
                    plan.missing_from_peer.len()
                );

                self.message_tx.send(NodeMessage::ReconciliationResult {
                    peer_id,
                    plan: plan.clone(),
                })?;
                self.emit_transport_frame(peer_id, SyncWireMessage::Reconciliation(plan.clone()))?;

                if let Some(replay_plan) = replay_plan {
                    self.message_tx.send(NodeMessage::ReplayIntentPlanned {
                        peer_id,
                        plan: replay_plan,
                    })?;
                }

                if !plan.missing_from_peer.is_empty() {
                    self.message_tx.send(NodeMessage::SyncInventory {
                        peer_id,
                        inventory: local_inventory.clone(),
                    })?;
                    self.emit_transport_frame(
                        peer_id,
                        SyncWireMessage::Inventory(local_inventory.clone()),
                    )?;
                }
            }
        }

        Ok(())
    }

    fn build_local_inventory(&self) -> SyncInventory {
        self.build_local_inventory_with_path(None)
    }

    fn build_local_inventory_with_path(&self, source_path: Option<&Path>) -> SyncInventory {
        let mut items = self
            .loaded_plugins
            .keys()
            .map(|name| InventoryItem::from_plugin(name))
            .collect::<Vec<_>>();
        let mut replay_metadata_catalog = Vec::new();
        match source_path {
            Some(path) => {
                let overlay = Self::load_inventory_overrides_with_metadata_from_path(path);
                items.extend(overlay.items);
                replay_metadata_catalog.extend(overlay.replay_metadata_catalog);
            }
            None => {
                let overlay = Self::load_local_inventory_overrides_with_metadata();
                items.extend(overlay.items);
                replay_metadata_catalog.extend(overlay.replay_metadata_catalog);
            }
        }
        Self::normalize_sync_inventory(SyncInventory {
            items: Self::normalize_inventory(items),
            replay_metadata_catalog,
        })
    }

    fn load_local_inventory_overrides() -> Vec<InventoryItem> {
        Self::load_local_inventory_overrides_with_metadata().items
    }

    fn load_local_inventory_overrides_with_metadata() -> SyncInventory {
        let Some(source_path) = std::env::var_os("ZOS_SYNC_INVENTORY_FILE") else {
            return SyncInventory::default();
        };
        Self::load_inventory_overrides_with_metadata_from_path(Path::new(&source_path))
    }

    fn load_inventory_overrides_from_path(path: &Path) -> Vec<InventoryItem> {
        Self::load_inventory_overrides_with_metadata_from_path(path).items
    }

    fn load_inventory_overrides_with_metadata_from_path(path: &Path) -> SyncInventory {
        let content = match fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(_) => return SyncInventory::default(),
        };
        Self::load_inventory_overrides_with_metadata_from_json(&content)
    }

    fn load_inventory_overrides_from_json(raw_json: &str) -> Vec<InventoryItem> {
        Self::load_inventory_overrides_with_metadata_from_json(raw_json).items
    }

    fn load_inventory_overrides_with_metadata_from_json(raw_json: &str) -> SyncInventory {
        let Ok(document) = serde_json::from_str::<serde_json::Value>(raw_json) else {
            return SyncInventory::default();
        };
        let parsed = Self::parse_inventory_overlay_with_metadata(document);
        Self::normalize_sync_inventory(SyncInventory {
            items: parsed.items,
            replay_metadata_catalog: parsed.replay_metadata_catalog,
        })
    }

    fn parse_inventory_overlay(document: serde_json::Value) -> Vec<InventoryItem> {
        Self::parse_inventory_overlay_with_metadata(document).items
    }

    fn parse_inventory_overlay_with_metadata(document: serde_json::Value) -> SyncInventory {
        let mut items = Vec::new();
        let mut replay_metadata_catalog = Vec::new();

        match document {
            serde_json::Value::Object(mut map) => {
                if let Some(artifacts) = map.remove("artifacts") {
                    items.extend(Self::parse_inventory_records(
                        &artifacts,
                        Some(InventoryKind::Artifact),
                    ));
                }
                if let Some(receipts) = map.remove("receipts") {
                    items.extend(Self::parse_inventory_records(
                        &receipts,
                        Some(InventoryKind::Receipt),
                    ));
                }
                if let Some(plugins) = map.remove("plugins") {
                    items.extend(Self::parse_inventory_records(
                        &plugins,
                        Some(InventoryKind::Plugin),
                    ));
                }
                if let Some(items_json) = map.remove("items") {
                    items.extend(Self::parse_inventory_records(&items_json, None));
                }
                Self::parse_contract_overlay_inventory(
                    &map,
                    &mut items,
                    &mut replay_metadata_catalog,
                );
                Self::parse_stream_overlay_inventory(
                    &map,
                    &mut items,
                    &mut replay_metadata_catalog,
                );
                Self::parse_generic_contract_inventory(&map, &mut items);
            }
            serde_json::Value::Array(values) => {
                items.extend(Self::parse_inventory_records(
                    &serde_json::Value::Array(values),
                    None,
                ));
            }
            _ => {}
        }

        SyncInventory {
            items: Self::normalize_inventory(items),
            replay_metadata_catalog: Self::normalize_replay_metadata(replay_metadata_catalog),
        }
    }

    fn parse_inventory_records(
        records: &serde_json::Value,
        default_kind: Option<InventoryKind>,
    ) -> Vec<InventoryItem> {
        let Some(items) = records.as_array() else {
            return Vec::new();
        };
        items
            .iter()
            .filter_map(|item| Self::parse_inventory_record(item, default_kind.clone()))
            .collect()
    }

    fn parse_inventory_record(
        record: &serde_json::Value,
        default_kind: Option<InventoryKind>,
    ) -> Option<InventoryItem> {
        let obj = record.as_object()?;

        let id = Self::pick_string(
            obj,
            &["id", "artifact_id", "artifactId", "receipt_id", "receiptId"],
        )?;

        let digest = Self::pick_string(obj, &["digest", "sha256", "contentDigest"]).unwrap_or("");

        let kind = default_kind.or_else(|| {
            obj.get("kind")
                .and_then(|value| value.as_str())
                .and_then(Self::parse_inventory_kind)
        })?;

        Some(match kind {
            InventoryKind::Plugin => InventoryItem::from_plugin(id),
            InventoryKind::Artifact => InventoryItem::from_artifact(id, digest),
            InventoryKind::Receipt => InventoryItem::from_receipt(id, digest),
        })
    }

    fn parse_contract_overlay_inventory(
        map: &serde_json::Map<String, serde_json::Value>,
        items: &mut Vec<InventoryItem>,
        replay_metadata_catalog: &mut Vec<ReplayMetadataRecord>,
    ) {
        if map
            .get("contractVersion")
            .and_then(|value| value.as_str())
            .is_none()
        {
            return;
        }

        let artifact_id = Self::pick_string(
            map,
            &["artifactId", "artifact_id", "artifactIdRef", "id"],
        );
        let artifact_digest = Self::pick_string(
            map,
            &[
                "contentDigest",
                "sha256",
                "digest",
                "artifactDigest",
                "containerDigest",
            ],
        )
        .or_else(|| {
            map.get("containerObjectRef")
                .and_then(serde_json::Value::as_object)
                .and_then(|container| {
                    Self::pick_string(container, &["contentDigest", "sha256", "digest"])
                })
        });

        if let Some(artifact_id) = artifact_id {
            let metadata = ReplayMetadata {
                container_object_ref: map.get("containerObjectRef").cloned(),
                object_refs: Vec::new(),
                source_ref: Self::pick_string(map, &["source_ref"]).map(str::to_string),
                related_artifact_refs: Vec::new(),
                proof_refs: Vec::new(),
                artifact_revision: Self::pick_string(map, &["artifactRevision", "artifact_revision"])
                    .map(str::to_string),
                acknowledged_revision: Self::pick_string(
                    map,
                    &[
                        "acknowledgedRevision",
                        "acknowledged_revision",
                        "ackRevision",
                        "ack_revision",
                        "receiptRevision",
                        "receipt_revision",
                    ],
                )
                .map(str::to_string),
            };
            items.push(InventoryItem::from_artifact(
                artifact_id,
                artifact_digest.unwrap_or(""),
            ));
            replay_metadata_catalog.push(ReplayMetadataRecord {
                kind: InventoryKind::Artifact,
                id: artifact_id.to_string(),
                metadata,
            });
        }

        let receipt_id = Self::pick_string(
            map,
            &["receiptId", "receipt_id", "runtimeReceipt", "sourceReceipt"],
        )
        .or_else(|| {
            map.get("source_ref")
                .and_then(|value| value.as_str())
                .and_then(|value| Self::parse_prefixed_receipt_id(Some(value), "receipt"))
        });
        if let Some(receipt_id) = receipt_id {
            let metadata = ReplayMetadata {
                container_object_ref: map.get("containerObjectRef").cloned(),
                object_refs: Vec::new(),
                source_ref: Self::pick_string(map, &["source_ref"]).map(str::to_string),
                related_artifact_refs: Vec::new(),
                proof_refs: map
                    .get("proof_refs")
                    .and_then(Value::as_array)
                    .map(|proofs| {
                        proofs
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                artifact_revision: Self::pick_string(map, &["artifactRevision", "artifact_revision"])
                    .map(str::to_string),
                acknowledged_revision: Self::pick_string(
                    map,
                    &[
                        "acknowledgedRevision",
                        "acknowledged_revision",
                        "ackRevision",
                        "ack_revision",
                        "receiptRevision",
                        "receipt_revision",
                    ],
                )
                .map(str::to_string),
            };
            items.push(InventoryItem::from_receipt(
                receipt_id,
                Self::pick_string(map, &["receiptDigest", "digest", "contentDigest"])
                    .unwrap_or(""),
            ));
            replay_metadata_catalog.push(ReplayMetadataRecord {
                kind: InventoryKind::Receipt,
                id: receipt_id.to_string(),
                metadata,
            });
        }

        if let Some(shards_value) = map.get("shards") {
            if let Some(shards) = shards_value.as_array() {
                for shard in shards {
                    if let Some(shard_object) = shard.as_object() {
                        let artifact_member_id = Self::pick_string(
                            shard_object,
                            &["shardId", "artifactId", "artifact_id", "id"],
                        );
                        if let Some(id) = artifact_member_id {
                            let object_refs = shard_object
                                .get("objectRefs")
                                .and_then(Value::as_array)
                                .cloned()
                                .unwrap_or_default();
                            let metadata = ReplayMetadata {
                                container_object_ref: map.get("containerObjectRef").cloned(),
                                object_refs,
                                source_ref: Self::pick_string(map, &["source_ref"]).map(str::to_string),
                                related_artifact_refs: Vec::new(),
                                proof_refs: Vec::new(),
                                artifact_revision: Self::pick_string(
                                    map,
                                    &["artifactRevision", "artifact_revision"],
                                )
                                .map(str::to_string),
                                acknowledged_revision: Self::pick_string(
                                    map,
                                    &[
                                        "acknowledgedRevision",
                                        "acknowledged_revision",
                                        "ackRevision",
                                        "ack_revision",
                                        "receiptRevision",
                                        "receipt_revision",
                                    ],
                                )
                                .map(str::to_string),
                            };
                            let digest = Self::pick_string(
                                shard_object,
                                &["contentDigest", "digest", "sha256"],
                            )
                            .or_else(|| {
                                shard_object
                                    .get("objectRefs")
                                    .and_then(Value::as_array)
                                    .and_then(|object_refs: &Vec<Value>| {
                                        object_refs.iter().find_map(|object_ref: &Value| {
                                            Self::pick_string(
                                                object_ref.as_object()?,
                                                &["contentDigest", "sha256", "digest"],
                                            )
                                        })
                                    })
                            });
                            items.push(InventoryItem::from_artifact(id, digest.unwrap_or("")));
                            replay_metadata_catalog.push(ReplayMetadataRecord {
                                kind: InventoryKind::Artifact,
                                id: id.to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }
    }

    fn parse_stream_overlay_inventory(
        map: &serde_json::Map<String, serde_json::Value>,
        items: &mut Vec<InventoryItem>,
        replay_metadata_catalog: &mut Vec<ReplayMetadataRecord>,
    ) {
        let Some(windows) = map.get("windows").and_then(Value::as_array) else {
            return;
        };

        let stream_artifact_id = Self::pick_string(map, &["artifactId", "artifact_id"]);
        let stream_digest = map
            .get("containerObjectRef")
            .and_then(Value::as_object)
            .and_then(|object| Self::pick_string(object, &["contentDigest", "sha256", "digest"]))
            .or_else(|| Self::pick_string(map, &["contentDigest", "sha256", "digest"]));
        if let Some(id) = stream_artifact_id {
            let metadata = ReplayMetadata {
                container_object_ref: map.get("containerObjectRef").cloned(),
                object_refs: Vec::new(),
                source_ref: None,
                related_artifact_refs: Vec::new(),
                proof_refs: Vec::new(),
                artifact_revision: Self::pick_string(map, &["artifactRevision", "artifact_revision"])
                    .map(str::to_string),
                acknowledged_revision: Self::pick_string(
                    map,
                    &[
                        "acknowledgedRevision",
                        "acknowledged_revision",
                        "ackRevision",
                        "ack_revision",
                        "receiptRevision",
                        "receipt_revision",
                    ],
                )
                .map(str::to_string),
            };
            items.push(InventoryItem::from_artifact(
                id,
                stream_digest.unwrap_or(""),
            ));
            replay_metadata_catalog.push(ReplayMetadataRecord {
                kind: InventoryKind::Artifact,
                id: id.to_string(),
                metadata,
            });
        }

        for window in windows {
            let Some(window_object) = window.as_object() else {
                continue;
            };
            let Some(window_payload) = window_object.get("payload") else {
                continue;
            };
            let Some(observations) = window_payload.get("observations").and_then(Value::as_array)
            else {
                continue;
            };
            for observation in observations {
                Self::parse_zkperf_observation_inventory(
                    observation,
                    items,
                    replay_metadata_catalog,
                );
            }
        }
    }

    fn parse_zkperf_observation_inventory(
        observation: &serde_json::Value,
        items: &mut Vec<InventoryItem>,
        replay_metadata_catalog: &mut Vec<ReplayMetadataRecord>,
    ) {
        let Some(observation) = observation.as_object() else {
            return;
        };

        let source_ref = observation
            .get("source_ref")
            .and_then(Value::as_str)
            .map(str::to_string);
        let related_artifact_refs = observation
            .get("related_artifact_refs")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let proof_refs = observation
            .get("proof_refs")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let artifact_revision =
            Self::pick_string(observation, &["artifactRevision", "artifact_revision"])
                .map(str::to_string);
        let acknowledged_revision = Self::pick_string(
            observation,
            &[
                "acknowledgedRevision",
                "acknowledged_revision",
                "ackRevision",
                "ack_revision",
                "receiptRevision",
                "receipt_revision",
            ],
        )
        .map(str::to_string);

        if let Some(receipt_id) = Self::parse_prefixed_receipt_id(
            observation.get("source_ref").and_then(Value::as_str),
            "receipt",
        ) {
            let digest = Self::pick_string(observation, &["hash", "digest", "contentDigest", "sha256"])
                .unwrap_or("");
            items.push(InventoryItem::from_receipt(receipt_id, digest));
            replay_metadata_catalog.push(ReplayMetadataRecord {
                kind: InventoryKind::Receipt,
                id: receipt_id.to_string(),
                metadata: ReplayMetadata {
                    container_object_ref: None,
                    object_refs: Vec::new(),
                    source_ref: source_ref.clone(),
                    related_artifact_refs: related_artifact_refs.clone(),
                    proof_refs: proof_refs.clone(),
                    artifact_revision: artifact_revision.clone(),
                    acknowledged_revision: acknowledged_revision.clone(),
                },
            });
        }

        if let Some(related_artifacts) = observation.get("related_artifact_refs").and_then(Value::as_array) {
            for artifact in related_artifacts {
                let Some(artifact) = artifact.as_object() else {
                    continue;
                };
                if let Some(artifact_id) = Self::pick_string(artifact, &["artifactId", "artifact_id"]) {
                    let digest = Self::pick_string(
                        artifact,
                        &["contentDigest", "digest", "sha256"],
                    )
                    .or_else(|| {
                        artifact
                            .get("members")
                            .and_then(Value::as_array)
                            .and_then(|members: &Vec<Value>| {
                                members.iter().find_map(|member: &Value| {
                                    Self::pick_string(
                                        member.as_object()?,
                                        &["contentDigest", "digest", "sha256"],
                                    )
                                })
                            })
                    })
                    .unwrap_or("");
                    items.push(InventoryItem::from_artifact(artifact_id, digest));
                    replay_metadata_catalog.push(ReplayMetadataRecord {
                        kind: InventoryKind::Artifact,
                        id: artifact_id.to_string(),
                        metadata: ReplayMetadata {
                            container_object_ref: None,
                            object_refs: artifact
                                .get("objectRefs")
                                .and_then(Value::as_array)
                                .cloned()
                                .unwrap_or_default(),
                            source_ref: source_ref.clone(),
                            related_artifact_refs: related_artifact_refs.clone(),
                            proof_refs: proof_refs.clone(),
                            artifact_revision: artifact_revision.clone(),
                            acknowledged_revision: acknowledged_revision.clone(),
                        },
                    });
                }
            }
        }

        if let Some(proof_refs) = observation.get("proof_refs").and_then(Value::as_array) {
            for proof in proof_refs {
                let Some(proof_ref) = proof.as_str() else {
                    continue;
                };
                if let Some(receipt_id) =
                    Self::parse_prefixed_receipt_id(Some(proof_ref), "proof")
                {
                    items.push(InventoryItem::from_receipt(receipt_id, ""));
                    replay_metadata_catalog.push(ReplayMetadataRecord {
                        kind: InventoryKind::Receipt,
                        id: receipt_id.to_string(),
                        metadata: ReplayMetadata {
                            container_object_ref: None,
                            object_refs: Vec::new(),
                            source_ref: source_ref.clone(),
                            related_artifact_refs: related_artifact_refs.clone(),
                            proof_refs: proof_refs.clone(),
                            artifact_revision: artifact_revision.clone(),
                            acknowledged_revision: acknowledged_revision.clone(),
                        },
                    });
                }
            }
        }
    }

    fn parse_prefixed_receipt_id<'a>(value: Option<&'a str>, prefix: &str) -> Option<&'a str> {
        value.and_then(|value| {
            if let Some(stripped) = value.strip_prefix(&format!("{prefix}:")) {
                Some(stripped)
            } else {
                None
            }
        })
    }

    fn parse_generic_contract_inventory(
        map: &serde_json::Map<String, serde_json::Value>,
        items: &mut Vec<InventoryItem>,
    ) {
        if map.contains_key("artifactId")
            && !map.contains_key("contractVersion")
            && !map.contains_key("windows")
            && !map.contains_key("contract")
        {
            let id = Self::pick_string(map, &["artifactId", "artifact_id"]).unwrap_or("");
            if !id.is_empty() {
                let digest =
                    Self::pick_string(map, &["contentDigest", "sha256", "digest"]).unwrap_or("");
                items.push(InventoryItem::from_artifact(id, digest));
            }
        }
    }

    fn pick_string<'a>(
        object: &'a serde_json::Map<String, serde_json::Value>,
        fields: &[&str],
    ) -> Option<&'a str> {
        fields
            .iter()
            .find_map(|field| object.get(*field).and_then(|value| value.as_str()))
    }

    fn parse_inventory_kind(value: &str) -> Option<InventoryKind> {
        match value {
            "plugin" => Some(InventoryKind::Plugin),
            "artifact" => Some(InventoryKind::Artifact),
            "receipt" => Some(InventoryKind::Receipt),
            _ => None,
        }
    }

    fn normalize_inventory(mut items: Vec<InventoryItem>) -> Vec<InventoryItem> {
        items.sort_unstable_by(|left, right| Self::sort_key(left).cmp(&Self::sort_key(right)));
        items.dedup();
        let preferred_ids = items
            .iter()
            .filter(|item| item.kind != InventoryKind::Plugin)
            .map(|item| item.id.clone())
            .collect::<HashSet<_>>();
        items.retain(|item| {
            item.kind != InventoryKind::Plugin || !preferred_ids.contains(&item.id)
        });
        items
    }

    fn sort_key(item: &InventoryItem) -> (u8, &str, &str) {
        (
            match item.kind {
                InventoryKind::Artifact => 0,
                InventoryKind::Receipt => 1,
                InventoryKind::Plugin => 2,
            },
            item.id.as_str(),
            item.digest.as_str(),
        )
    }

    fn inventory_item_key(item: &InventoryItem) -> (InventoryKind, &str) {
        (item.kind.clone(), item.id.as_str())
    }

    fn replay_metadata_key(record: &ReplayMetadataRecord) -> (InventoryKind, &str) {
        (record.kind.clone(), record.id.as_str())
    }

    fn is_canonical_replay_identity(item: &InventoryItem) -> bool {
        matches!(item.kind, InventoryKind::Artifact | InventoryKind::Receipt)
    }

    fn normalize_replay_metadata(
        mut replay_metadata_catalog: Vec<ReplayMetadataRecord>,
    ) -> Vec<ReplayMetadataRecord> {
        replay_metadata_catalog.retain(|record| {
            matches!(record.kind, InventoryKind::Artifact | InventoryKind::Receipt)
        });
        replay_metadata_catalog.sort_unstable_by(|left, right| {
            let left_metadata = serde_json::to_string(&left.metadata).unwrap_or_default();
            let right_metadata = serde_json::to_string(&right.metadata).unwrap_or_default();
            (
                match left.kind {
                    InventoryKind::Artifact => 0,
                    InventoryKind::Receipt => 1,
                    InventoryKind::Plugin => 2,
                },
                left.id.as_str(),
                left_metadata.as_str(),
            )
                .cmp(&(
                    match right.kind {
                        InventoryKind::Artifact => 0,
                        InventoryKind::Receipt => 1,
                        InventoryKind::Plugin => 2,
                    },
                    right.id.as_str(),
                    right_metadata.as_str(),
                ))
        });
        replay_metadata_catalog.dedup();
        replay_metadata_catalog
    }

    fn normalize_sync_inventory(inventory: SyncInventory) -> SyncInventory {
        let items = Self::normalize_inventory(inventory.items);
        let keys = items
            .iter()
            .filter(|item| Self::is_canonical_replay_identity(item))
            .map(|item| (item.kind.clone(), item.id.clone()))
            .collect::<HashSet<_>>();
        let replay_metadata_catalog = Self::normalize_replay_metadata(
            inventory
                .replay_metadata_catalog
                .into_iter()
                .filter(|record| keys.contains(&(record.kind.clone(), record.id.clone())))
                .collect::<Vec<_>>(),
        );

        SyncInventory {
            items,
            replay_metadata_catalog,
        }
    }

    fn find_replay_metadata(
        inventory: &SyncInventory,
        item: &InventoryItem,
    ) -> Option<ReplayMetadata> {
        inventory
            .replay_metadata_catalog
            .iter()
            .find(|record| Self::replay_metadata_key(record) == Self::inventory_item_key(item))
            .map(|record| record.metadata.clone())
    }

    fn reconcile_inventory(
        local_inventory: &SyncInventory,
        peer_inventory: &SyncInventory,
    ) -> ReconciliationPlan {
        let local_inventory = Self::normalize_sync_inventory(local_inventory.clone());
        let peer_inventory = Self::normalize_sync_inventory(peer_inventory.clone());
        let peer_by_name = peer_inventory
            .items
            .iter()
            .map(|item| (Self::inventory_item_key(item), item))
            .collect::<HashMap<_, _>>();
        let local_by_name = local_inventory
            .items
            .iter()
            .map(|item| (Self::inventory_item_key(item), item))
            .collect::<HashMap<_, _>>();

        let missing_from_peer = local_inventory
            .items
            .iter()
            .filter(|item| peer_by_name.get(&Self::inventory_item_key(item)) != Some(&item))
            .cloned()
            .collect::<Vec<_>>();
        let missing_from_local = peer_inventory
            .items
            .iter()
            .filter(|item| local_by_name.get(&Self::inventory_item_key(item)) != Some(&item))
            .cloned()
            .collect::<Vec<_>>();

        ReconciliationPlan {
            missing_from_local,
            missing_from_peer,
        }
    }

    fn plan_replay_intent(
        plan: &ReconciliationPlan,
        peer_inventory: &SyncInventory,
    ) -> Option<ReplayIntentPlan> {
        let requested_from_peer = plan
            .missing_from_local
            .iter()
            .filter(|item| Self::is_canonical_replay_identity(item))
            .cloned()
            .collect::<Vec<_>>();

        if requested_from_peer.is_empty() {
            return None;
        }

        let recovery_requests = requested_from_peer
            .iter()
            .filter_map(|item| {
                Self::find_replay_metadata(peer_inventory, item).map(|metadata| {
                    ReplayRecoveryRequest {
                        item: item.clone(),
                        metadata,
                    }
                })
            })
            .collect::<Vec<_>>();

        Some(ReplayIntentPlan {
            source: ReplaySource::PeerInventoryGap,
            requested_from_peer,
            recovery_requests,
        })
    }

    fn register_peer_inventory(&mut self, peer_id: PeerId, inventory: SyncInventory) {
        let inventory = Self::normalize_sync_inventory(inventory);
        if let Some(node_info) = self.peer_nodes.get_mut(&peer_id) {
            node_info.advertised_inventory = inventory;
            return;
        }

        self.peer_nodes.insert(
            peer_id,
            NodeInfo {
                peer_id,
                capabilities: Vec::new(),
                plugin_layers: Vec::new(),
                load_average: 0.0,
                advertised_inventory: inventory,
                last_reconciliation: None,
                pending_replay: None,
            },
        );
    }

    fn emit_transport_frame(
        &self,
        peer_id: PeerId,
        message: SyncWireMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(transport_tx) = &self.transport_tx else {
            return Ok(());
        };

        let payload = serde_json::to_vec(&message)?;
        transport_tx.send(OutboundSyncFrame { peer_id, payload })?;
        Ok(())
    }

    fn emit_inventory_announcement(
        &self,
        inventory: SyncInventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.emit_transport_frame(self.node_id, SyncWireMessage::Announcement(inventory))
    }

    pub(crate) async fn handle_transport_frame(
        &mut self,
        peer_id: PeerId,
        payload: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        match serde_json::from_slice::<SyncWireMessage>(payload)? {
            SyncWireMessage::Inventory(inventory) => {
                self.register_peer_inventory(peer_id, inventory)
            }
            SyncWireMessage::Reconciliation(plan) => {
                if let Some(node_info) = self.peer_nodes.get_mut(&peer_id) {
                    node_info.last_reconciliation = Some(plan);
                }
            }
            SyncWireMessage::Announcement(inventory) => {
                self.register_peer_inventory(peer_id, inventory);
            }
        }
        Ok(())
    }

    /// Expose a handle for wiring inbound transport into the coordinator's existing
    /// message path. This keeps the external transport adapter from reaching into
    /// internal state directly.
    pub fn message_sender(&self) -> mpsc::UnboundedSender<NodeMessage> {
        self.message_tx.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[cfg(unix)]
    fn placeholder_library() -> libloading::Library {
        unsafe { libloading::Library::new("libc.so.6").expect("libc") }
    }

    fn inventory(names: &[&str]) -> SyncInventory {
        SyncInventory {
            items: names
                .iter()
                .map(|name| InventoryItem::from_plugin(name))
                .collect(),
            replay_metadata_catalog: Vec::new(),
        }
    }

    #[test]
    fn reconcile_inventory_detects_missing_items_on_both_sides() {
        let local_inventory = inventory(&["docker", "systemd"]);
        let peer_inventory = inventory(&["docker", "wikidata"]);

        let plan = ZosNode::reconcile_inventory(&local_inventory, &peer_inventory);

        assert_eq!(plan.missing_from_local, inventory(&["wikidata"]).items);
        assert_eq!(plan.missing_from_peer, inventory(&["systemd"]).items);
    }

    #[test]
    fn plan_replay_intent_includes_only_canonical_missing_local_items() {
        let plan = ReconciliationPlan {
            missing_from_local: vec![
                InventoryItem::from_plugin("plugin://debug-only"),
                InventoryItem::from_artifact("artifact://a1", "sha256:a1"),
                InventoryItem::from_receipt("receipt://r1", "sha256:r1"),
            ],
            missing_from_peer: vec![InventoryItem::from_plugin("docker")],
        };

        let replay_plan =
            ZosNode::plan_replay_intent(&plan, &SyncInventory::default()).expect("replay plan");
        assert_eq!(
            replay_plan,
            ReplayIntentPlan {
                source: ReplaySource::PeerInventoryGap,
                requested_from_peer: vec![
                    InventoryItem::from_artifact("artifact://a1", "sha256:a1"),
                    InventoryItem::from_receipt("receipt://r1", "sha256:r1"),
                ],
                recovery_requests: Vec::new(),
            }
        );
    }

    #[test]
    fn plan_replay_intent_is_none_for_plugin_only_gaps() {
        let plan = ReconciliationPlan {
            missing_from_local: vec![
                InventoryItem::from_plugin("docker"),
                InventoryItem::from_plugin("systemd"),
            ],
            missing_from_peer: Vec::new(),
        };

        assert!(ZosNode::plan_replay_intent(&plan, &SyncInventory::default()).is_none());
    }

    #[tokio::test]
    async fn sync_with_peers_records_reconciliation_for_known_peer() {
        let mut node = ZosNode::new().await.expect("node");
        node.loaded_plugins
            .insert("docker".to_string(), placeholder_library());
        node.loaded_plugins
            .insert("systemd".to_string(), placeholder_library());

        let peer_id = PeerId::random();
        node.register_peer_inventory(peer_id, inventory(&["docker"]));

        node.sync_with_peers().await.expect("sync");

        let node_info = node.peer_nodes.get(&peer_id).expect("peer");
        let plan = node_info
            .last_reconciliation
            .as_ref()
            .expect("reconciliation plan");
        assert_eq!(plan.missing_from_peer, inventory(&["systemd"]).items);
        assert!(plan.missing_from_local.is_empty());
    }

    #[tokio::test]
    async fn sync_with_peers_records_replay_plan_for_missing_local_artifacts() {
        let mut node = ZosNode::new().await.expect("node");
        node.loaded_plugins
            .insert("docker".to_string(), placeholder_library());

        let peer_id = PeerId::random();
        node.register_peer_inventory(
            peer_id,
            SyncInventory {
                items: vec![
                    InventoryItem::from_plugin("docker"),
                    InventoryItem::from_artifact("artifact://missing-local", "sha256:artifact"),
                    InventoryItem::from_receipt("receipt://missing-local", "sha256:receipt"),
                ],
                replay_metadata_catalog: Vec::new(),
            },
        );

        node.sync_with_peers().await.expect("sync");

        let node_info = node.peer_nodes.get(&peer_id).expect("peer");
        assert_eq!(
            node_info.pending_replay,
            Some(ReplayIntentPlan {
                source: ReplaySource::PeerInventoryGap,
                requested_from_peer: vec![
                    InventoryItem::from_artifact("artifact://missing-local", "sha256:artifact"),
                    InventoryItem::from_receipt("receipt://missing-local", "sha256:receipt"),
                ],
                recovery_requests: Vec::new(),
            })
        );
    }

    #[tokio::test]
    async fn sync_with_peers_emits_wire_frames_for_announcement_and_reconciliation() {
        let (transport_tx, mut transport_rx) = mpsc::unbounded_channel();
        let mut node = ZosNode::new_with_transport(transport_tx)
            .await
            .expect("node");
        node.loaded_plugins
            .insert("docker".to_string(), placeholder_library());
        node.loaded_plugins
            .insert("systemd".to_string(), placeholder_library());

        let peer_id = PeerId::random();
        node.register_peer_inventory(peer_id, inventory(&["docker"]));

        node.sync_with_peers().await.expect("sync");

        let announcement = transport_rx.recv().await.expect("announcement frame");
        assert_eq!(announcement.peer_id, node.node_id);
        assert_eq!(
            serde_json::from_slice::<SyncWireMessage>(&announcement.payload)
                .expect("wire announcement"),
            SyncWireMessage::Announcement(inventory(&["docker", "systemd"]))
        );

        let reconciliation = transport_rx.recv().await.expect("reconciliation frame");
        assert_eq!(reconciliation.peer_id, peer_id);
        assert_eq!(
            serde_json::from_slice::<SyncWireMessage>(&reconciliation.payload)
                .expect("wire reconciliation"),
            SyncWireMessage::Reconciliation(ReconciliationPlan {
                missing_from_local: Vec::new(),
                missing_from_peer: inventory(&["systemd"]).items,
            })
        );

        let inventory_frame = transport_rx.recv().await.expect("inventory frame");
        assert_eq!(inventory_frame.peer_id, peer_id);
        assert_eq!(
            serde_json::from_slice::<SyncWireMessage>(&inventory_frame.payload)
                .expect("wire inventory"),
            SyncWireMessage::Inventory(inventory(&["docker", "systemd"]))
        );
    }

    #[test]
    fn parse_overlay_inventory_from_json_normalizes_records() {
        let overlay = r#"{
            "artifacts": [
                {"id":"artifact://a1","digest":"sha256:a1"},
                {"id":"artifact://a2","digest":"sha256:a2"}
            ],
            "receipts": [
                {"id":"receipt://r1","digest":"sha256:r1"},
                {"id":"receipt://r2","digest":"sha256:r2","kind":"receipt"}
            ],
            "items": [
                {"kind":"artifact","id":"artifact://a2","digest":"sha256:a2"}
            ]
        }"#;

        let parsed = ZosNode::load_inventory_overrides_from_json(overlay);

        assert_eq!(
            parsed,
            vec![
                InventoryItem::from_artifact("artifact://a1", "sha256:a1"),
                InventoryItem::from_artifact("artifact://a2", "sha256:a2"),
                InventoryItem::from_receipt("receipt://r1", "sha256:r1"),
                InventoryItem::from_receipt("receipt://r2", "sha256:r2"),
            ]
        );
    }

    #[test]
    fn parse_overlay_inventory_prefers_artifact_and_receipt_identities_over_plugin_fallbacks() {
        let overlay = r#"{
            "plugins": [
                {"id":"shared://same-id","digest":"plugin:shared"},
                {"id":"plugin://compat","digest":"plugin:compat"}
            ],
            "artifacts": [
                {"id":"shared://same-id","digest":"sha256:shared-artifact"}
            ],
            "receipts": [
                {"id":"shared://same-id-receipt","digest":"sha256:shared-receipt"}
            ]
        }"#;

        let parsed = ZosNode::load_inventory_overrides_from_json(overlay);

        assert_eq!(
            parsed,
            vec![
                InventoryItem::from_artifact("shared://same-id", "sha256:shared-artifact"),
                InventoryItem::from_receipt("shared://same-id-receipt", "sha256:shared-receipt"),
                InventoryItem::from_plugin("plugin://compat"),
            ]
        );
    }

    #[test]
    fn parse_overlay_inventory_from_zkperf_stream_shaped_json() {
        let overlay = r#"{
            "contractVersion": "zkperf-stream/v1",
            "streamId": "zkperf-stream-demo",
            "artifactId": "wikidata-2026-demo",
            "containerObjectRef": {
                "sink": "hf",
                "uri": "hf://datasets/acrion/zelph-demo/streams/stream.tar",
                "contentDigest": "sha256:stream-container-example"
            },
            "windows": [
                {
                    "windowId": "window-0001",
                    "sequence": 1,
                    "payload": {
                        "observations": [
                            {
                                "source_ref": "receipt:zkperf-run-20260329-a",
                                "hash": "sha256:obs-0001",
                                "related_artifact_refs": [
                                    {
                                        "artifactId": "shard-left-001",
                                        "contentDigest": "sha256:left-bucket-001-example"
                                    }
                                ],
                                "proof_refs": [
                                    "proof:zkperf:run-20260329-a:summary"
                                ]
                            }
                        ]
                    }
                }
            ]
        }"#;

        let parsed = ZosNode::load_inventory_overrides_from_json(overlay);

        assert_eq!(
            parsed,
            vec![
                InventoryItem::from_artifact("shard-left-001", "sha256:left-bucket-001-example"),
                InventoryItem::from_artifact("wikidata-2026-demo", "sha256:stream-container-example"),
                InventoryItem::from_receipt("zkperf-run-20260329-a", "sha256:obs-0001"),
                InventoryItem::from_receipt("zkperf:run-20260329-a:summary", ""),
            ]
        );
    }

    #[test]
    fn parse_overlay_inventory_from_erdfa_manifest_shaped_json() {
        let overlay = r#"{
            "contractVersion": "erdfa-manifest-promotion/v1",
            "artifactId": "wikidata-2026-demo",
            "artifactRevision": "rev-20260329-a",
            "containerObjectRef": {
                "sink": "hf",
                "uri": "hf://datasets/acrion/zelph-demo/containers/manifest.tar.zst",
                "contentDigest": "sha256:manifest-container"
            },
            "shards": [
                {
                    "id": "left-bucket-001",
                    "shardId": "left-bucket-001",
                    "objectRefs": [
                        {
                            "sink": "hf",
                            "uri": "hf://datasets/acrion/zelph-demo/shards/left-bucket-001.cbor",
                            "contentDigest": "sha256:left-bucket-001-example"
                        }
                    ]
                },
                {
                    "id": "nodeOfName-en-017",
                    "objectRefs": [
                        {
                            "sink": "hf",
                            "uri": "hf://datasets/acrion/zelph-demo/shards/nodeOfName-en-017.cbor",
                            "contentDigest": "sha256:nodeofname-en-017-example"
                        }
                    ]
                }
            ]
        }"#;

        let parsed = ZosNode::load_inventory_overrides_from_json(overlay);

        assert_eq!(
            parsed,
            vec![
                InventoryItem::from_artifact("left-bucket-001", "sha256:left-bucket-001-example"),
                InventoryItem::from_artifact("nodeOfName-en-017", "sha256:nodeofname-en-017-example"),
                InventoryItem::from_artifact("wikidata-2026-demo", "sha256:manifest-container"),
            ]
        );
    }

    #[test]
    fn parse_overlay_inventory_extracts_replay_locator_metadata() {
        let overlay = r#"{
            "contractVersion": "erdfa-manifest-promotion/v1",
            "artifactId": "wikidata-2026-demo",
            "artifactRevision": "rev-20260329-a",
            "acknowledgedRevision": "ack-20260330-b",
            "containerObjectRef": {
                "sink": "hf",
                "uri": "hf://datasets/acrion/zelph-demo/containers/manifest.tar.zst",
                "contentDigest": "sha256:manifest-container"
            },
            "shards": [
                {
                    "shardId": "left-bucket-001",
                    "objectRefs": [
                        {
                            "sink": "hf",
                            "uri": "hf://datasets/acrion/zelph-demo/shards/left-bucket-001.cbor",
                            "contentDigest": "sha256:left-bucket-001-example"
                        }
                    ]
                }
            ]
        }"#;

        let parsed = ZosNode::load_inventory_overrides_with_metadata_from_json(overlay);
        let top_level = parsed
            .replay_metadata_catalog
            .iter()
            .find(|record| {
                record.kind == InventoryKind::Artifact && record.id == "wikidata-2026-demo"
            })
            .expect("top-level artifact metadata");
        assert_eq!(
            top_level.metadata.artifact_revision.as_deref(),
            Some("rev-20260329-a")
        );
        assert_eq!(
            top_level.metadata.acknowledged_revision.as_deref(),
            Some("ack-20260330-b")
        );
        assert!(top_level.metadata.container_object_ref.is_some());

        let shard = parsed
            .replay_metadata_catalog
            .iter()
            .find(|record| {
                record.kind == InventoryKind::Artifact && record.id == "left-bucket-001"
            })
            .expect("shard artifact metadata");
        assert_eq!(shard.metadata.object_refs.len(), 1);
    }

    #[test]
    fn plan_replay_intent_carries_bounded_recovery_metadata_for_canonical_missing_items() {
        let plan = ReconciliationPlan {
            missing_from_local: vec![
                InventoryItem::from_plugin("plugin://debug-only"),
                InventoryItem::from_artifact("artifact://a1", "sha256:a1"),
                InventoryItem::from_receipt("receipt://r1", "sha256:r1"),
            ],
            missing_from_peer: Vec::new(),
        };
        let peer_inventory = SyncInventory {
            items: vec![
                InventoryItem::from_plugin("plugin://debug-only"),
                InventoryItem::from_artifact("artifact://a1", "sha256:a1"),
                InventoryItem::from_receipt("receipt://r1", "sha256:r1"),
            ],
            replay_metadata_catalog: vec![
                ReplayMetadataRecord {
                    kind: InventoryKind::Artifact,
                    id: "artifact://a1".to_string(),
                    metadata: ReplayMetadata {
                        source_ref: Some("receipt:run-01".to_string()),
                        artifact_revision: Some("rev-a1".to_string()),
                        ..ReplayMetadata::default()
                    },
                },
                ReplayMetadataRecord {
                    kind: InventoryKind::Receipt,
                    id: "receipt://r1".to_string(),
                    metadata: ReplayMetadata {
                        proof_refs: vec!["proof:run-01:summary".to_string()],
                        acknowledged_revision: Some("ack-r1".to_string()),
                        ..ReplayMetadata::default()
                    },
                },
            ],
        };

        let replay_plan =
            ZosNode::plan_replay_intent(&plan, &peer_inventory).expect("replay plan");
        assert_eq!(
            replay_plan.requested_from_peer,
            vec![
                InventoryItem::from_artifact("artifact://a1", "sha256:a1"),
                InventoryItem::from_receipt("receipt://r1", "sha256:r1"),
            ]
        );
        assert_eq!(replay_plan.recovery_requests.len(), 2);
        assert_eq!(
            replay_plan.recovery_requests[0].metadata.artifact_revision.as_deref(),
            Some("rev-a1")
        );
        assert_eq!(
            replay_plan.recovery_requests[1]
                .metadata
                .acknowledged_revision
                .as_deref(),
            Some("ack-r1")
        );
    }

    #[tokio::test]
    async fn build_local_inventory_merges_plugin_and_overlay_records() {
        let mut node = ZosNode::new().await.expect("node");
        node.loaded_plugins.insert("docker".into(), placeholder_library());
        node.loaded_plugins.insert("systemd".into(), placeholder_library());

        let overlay = r#"{
            "artifacts": [
                {"id":"artifact://a1","digest":"sha256:a1"},
                {"id":"artifact://a2","digest":"sha256:a2"}
            ],
            "receipts": [
                {"id":"receipt://r1","digest":"sha256:r1"}
            ],
            "items": [
                {"kind":"artifact","id":"artifact://a2","digest":"sha256:a2"}
            ]
        }"#;
        let overlay_path = PathBuf::from(std::env::temp_dir())
            .join("zos-server-test-overlay-exp.json");
        fs::write(&overlay_path, overlay).expect("write overlay");

        let mut expected = node
            .loaded_plugins
            .keys()
            .map(|name| InventoryItem::from_plugin(name))
            .collect::<Vec<_>>();
        expected.extend(ZosNode::load_inventory_overrides_from_path(&overlay_path));
        let expected = ZosNode::normalize_inventory(expected);

        assert_eq!(node.build_local_inventory_with_path(Some(&overlay_path)).items, expected);
        let _ = fs::remove_file(&overlay_path);
    }

    #[tokio::test]
    async fn handle_transport_frame_registers_peer_inventory() {
        let mut node = ZosNode::new().await.expect("node");
        let peer_id = PeerId::random();
        let payload = serde_json::to_vec(&SyncWireMessage::Inventory(inventory(&["docker"])))
            .expect("payload");

        node.handle_transport_frame(peer_id, &payload)
            .await
            .expect("handle transport frame");

        let node_info = node.peer_nodes.get(&peer_id).expect("peer");
        assert_eq!(node_info.advertised_inventory, inventory(&["docker"]));
    }

    #[tokio::test]
    async fn handle_transport_frame_prefers_artifact_identities_over_plugin_fallbacks() {
        let mut node = ZosNode::new().await.expect("node");
        let peer_id = PeerId::random();
        let payload = serde_json::to_vec(&SyncWireMessage::Inventory(SyncInventory {
            items: vec![
                InventoryItem::from_plugin("shared://same-id"),
                InventoryItem::from_artifact("shared://same-id", "sha256:shared-artifact"),
                InventoryItem::from_plugin("plugin://compat"),
            ],
            replay_metadata_catalog: Vec::new(),
        }))
        .expect("payload");

        node.handle_transport_frame(peer_id, &payload)
            .await
            .expect("handle transport frame");

        let node_info = node.peer_nodes.get(&peer_id).expect("peer");
        assert_eq!(
            node_info.advertised_inventory.items,
            vec![
                InventoryItem::from_artifact("shared://same-id", "sha256:shared-artifact"),
                InventoryItem::from_plugin("plugin://compat"),
            ]
        );
    }
}

impl InventoryItem {
    fn from_plugin(name: &str) -> Self {
        Self {
            kind: InventoryKind::Plugin,
            id: name.to_string(),
            digest: format!("plugin:{name}"),
        }
    }

    #[allow(dead_code)]
    fn from_artifact(id: &str, digest: &str) -> Self {
        Self {
            kind: InventoryKind::Artifact,
            id: id.to_string(),
            digest: digest.to_string(),
        }
    }

    #[allow(dead_code)]
    fn from_receipt(id: &str, digest: &str) -> Self {
        Self {
            kind: InventoryKind::Receipt,
            id: id.to_string(),
            digest: digest.to_string(),
        }
    }
}
