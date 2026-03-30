// LibP2P Plugin C Interface
// C-compatible functions for the libp2p plugin
//
// This module now owns a minimal gossipsub loop for sync frames. It exposes
// C-friendly entry points and Rust-friendly helpers to wire the sync transport.

use libp2p::futures::StreamExt;
use libp2p::{gossipsub, identity, Multiaddr, PeerId};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;
use tokio::sync::mpsc;

const TOPIC: &str = "zos-sync";

#[derive(Debug, Clone)]
pub struct InboundMessage {
    pub peer_id: PeerId,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncIdentityKind {
    Artifact,
    Receipt,
}

impl SyncIdentityKind {
    fn canonical_prefix(self) -> &'static str {
        match self {
            SyncIdentityKind::Artifact => "artifact:",
            SyncIdentityKind::Receipt => "receipt:",
        }
    }

    fn counterpart_prefix(self) -> &'static str {
        match self {
            SyncIdentityKind::Artifact => "receipt:",
            SyncIdentityKind::Receipt => "artifact:",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncInventoryIdentityMessage {
    #[serde(default = "default_inventory_identity_verb")]
    pub verb: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    pub kind: SyncIdentityKind,
    #[serde(alias = "object_id", alias = "artifact_id", alias = "receipt_id")]
    pub identity: String,
    #[serde(alias = "content_digest", alias = "sha256", alias = "hash")]
    pub digest: String,
    #[serde(
        default,
        alias = "related_artifact_ref",
        alias = "artifact_ref",
        alias = "source_ref",
        skip_serializing_if = "Option::is_none"
    )]
    pub backing_ref: Option<String>,
}

impl SyncInventoryIdentityMessage {
    pub fn new(
        plugin: Option<impl Into<String>>,
        kind: SyncIdentityKind,
        identity: impl Into<String>,
        digest: impl Into<String>,
        backing_ref: Option<impl Into<String>>,
    ) -> Self {
        Self {
            verb: "attach_inventory_identity".to_string(),
            plugin: plugin.map(|value| value.into()),
            kind,
            identity: identity.into(),
            digest: digest.into(),
            backing_ref: backing_ref.map(|value| value.into()),
        }
    }

    fn normalized(mut self) -> Result<Self, String> {
        self.verb = if self.verb.trim().is_empty() {
            default_inventory_identity_verb()
        } else {
            self.verb.trim().to_string()
        };
        self.plugin = self
            .plugin
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        self.identity = normalize_identity(self.kind.clone(), &self.identity)?;
        self.digest = normalize_digest(&self.digest)?;
        self.backing_ref = normalize_backing_ref(self.kind, self.backing_ref.as_deref());
        Ok(self)
    }
}

static PUBLISH_TX: OnceCell<mpsc::UnboundedSender<Vec<u8>>> = OnceCell::new();
static INBOUND_TX: OnceCell<mpsc::UnboundedSender<InboundMessage>> = OnceCell::new();
static INBOUND_RX: OnceCell<Mutex<Option<mpsc::UnboundedReceiver<InboundMessage>>>> =
    OnceCell::new();
static INIT: OnceCell<()> = OnceCell::new();

fn ensure_initialized() -> Result<(), String> {
    INIT.get_or_try_init(|| {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(5))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| format!("gossipsub config: {e}"))?;

        let topic = gossipsub::IdentTopic::new(TOPIC);

        let gossipsub: gossipsub::Behaviour<
            gossipsub::IdentityTransform,
            gossipsub::AllowAllSubscriptionFilter,
        > = gossipsub::Behaviour::new_with_subscription_filter(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
            None,
            gossipsub::AllowAllSubscriptionFilter {},
        )
        .map_err(|e| format!("gossipsub behavior: {e}"))?;

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )
            .map_err(|e| format!("tcp builder: {e}"))?
            .with_behaviour(|_| gossipsub)
            .map_err(|e| format!("behaviour: {e}"))?
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(std::time::Duration::from_secs(60))
            })
            .build();

        let listen_addr = sync_listen_addr()?;
        swarm
            .listen_on(listen_addr.clone())
            .map_err(|e| format!("listen_on {listen_addr}: {e}"))?;

        swarm
            .behaviour_mut()
            .subscribe(&topic)
            .map_err(|e| format!("subscribe: {e}"))?;

        let (publish_tx, mut publish_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (in_tx, in_rx) = mpsc::unbounded_channel::<InboundMessage>();

        PUBLISH_TX
            .set(publish_tx)
            .map_err(|_| "publish channel already set")?;
        INBOUND_TX
            .set(in_tx)
            .map_err(|_| "inbound tx already set")?;
        INBOUND_RX
            .set(Mutex::new(Some(in_rx)))
            .map_err(|_| "inbound rx already set")?;

        tokio::spawn(async move {
            let mut swarm = swarm;
            let topic = gossipsub::IdentTopic::new(TOPIC);
            eprintln!(
                "sync transport: local libp2p peer_id={} topic={} listen_addr={}",
                local_peer_id, TOPIC, listen_addr
            );
            for bootstrap in sync_bootstrap_addrs() {
                match swarm.dial(bootstrap.clone()) {
                    Ok(()) => eprintln!("sync transport: dialing bootstrap {bootstrap}"),
                    Err(error) => {
                        eprintln!("sync transport: failed to dial bootstrap {bootstrap}: {error}")
                    }
                }
            }
            loop {
                tokio::select! {
                    Some(bytes) = publish_rx.recv() => {
                        let _ = swarm.behaviour_mut().publish(topic.clone(), bytes);
                    }
                    event = swarm.select_next_some() => {
                        match event {
                            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                                eprintln!(
                                    "sync transport: peer_id={} listening_on={}",
                                    local_peer_id, address
                                );
                            }
                            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                                eprintln!(
                                    "sync transport: peer_id={} connected_to={} via={endpoint:?}",
                                    local_peer_id, peer_id
                                );
                            }
                            libp2p::swarm::SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                                eprintln!(
                                    "sync transport: peer_id={} outgoing_connection_error target={peer_id:?} err={error}",
                                    local_peer_id
                                );
                            }
                            libp2p::swarm::SwarmEvent::IncomingConnectionError { error, .. } => {
                                eprintln!(
                                    "sync transport: peer_id={} incoming_connection_error err={error}",
                                    local_peer_id
                                );
                            }
                            libp2p::swarm::SwarmEvent::Behaviour(gossipsub::Event::Message {
                                propagation_source,
                                message,
                                ..
                            }) => {
                                if let Some(tx) = INBOUND_TX.get() {
                                    let _ = tx.send(InboundMessage {
                                        peer_id: propagation_source,
                                        payload: message.data,
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Ok(())
    })
    .map(|_| ())
}

fn sync_listen_addr() -> Result<Multiaddr, String> {
    let configured = std::env::var("ZOS_SYNC_LISTEN_ADDR")
        .unwrap_or_else(|_| "/ip4/127.0.0.1/tcp/0".to_string());
    configured
        .parse::<Multiaddr>()
        .map_err(|error| format!("invalid ZOS_SYNC_LISTEN_ADDR={configured}: {error}"))
}

fn sync_bootstrap_addrs() -> Vec<Multiaddr> {
    std::env::var("ZOS_SYNC_BOOTSTRAP_ADDRS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .filter_map(|entry| entry.parse::<Multiaddr>().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_sync_identity_kind(raw: &str) -> Result<SyncIdentityKind, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "artifact" => Ok(SyncIdentityKind::Artifact),
        "erdfa_manifest" => Ok(SyncIdentityKind::Artifact),
        "manifest" => Ok(SyncIdentityKind::Artifact),
        "shard" => Ok(SyncIdentityKind::Artifact),
        "receipt" => Ok(SyncIdentityKind::Receipt),
        "observation" => Ok(SyncIdentityKind::Receipt),
        "zkperf_observation" => Ok(SyncIdentityKind::Receipt),
        "proof" => Ok(SyncIdentityKind::Receipt),
        other => Err(format!("unsupported sync identity kind: {other}")),
    }
}

fn default_inventory_identity_verb() -> String {
    "attach_inventory_identity".to_string()
}

fn normalize_identity(kind: SyncIdentityKind, identity: &str) -> Result<String, String> {
    let identity = identity.trim();
    if identity.is_empty() {
        return Err("identity cannot be empty".to_string());
    }

    let canonical = kind.canonical_prefix();
    if identity.starts_with(canonical) {
        return Ok(identity.to_string());
    }
    if identity.starts_with("artifact:") || identity.starts_with("receipt:") {
        return Ok(identity.to_string());
    }
    Ok(format!("{canonical}{identity}"))
}

fn normalize_digest(digest: &str) -> Result<String, String> {
    let digest = digest.trim();
    if digest.is_empty() {
        return Err("digest cannot be empty".to_string());
    }
    if digest.starts_with("sha256:") {
        Ok(digest.to_string())
    } else {
        Ok(format!("sha256:{digest}"))
    }
}

fn normalize_backing_ref(kind: SyncIdentityKind, backing_ref: Option<&str>) -> Option<String> {
    let value = backing_ref?.trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with("artifact:") || value.starts_with("receipt:") {
        return Some(value.to_string());
    }
    Some(format!("{}{}", kind.counterpart_prefix(), value))
}

pub fn encode_inventory_identity_message(
    plugin: Option<&str>,
    kind: SyncIdentityKind,
    identity: &str,
    digest: &str,
    backing_ref: Option<&str>,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&SyncInventoryIdentityMessage::new(
        plugin,
        kind,
        identity,
        digest,
        backing_ref,
    ))
    .map_err(|e| format!("encode sync identity message: {e}"))
}

pub fn decode_inventory_identity_message(
    payload: &[u8],
) -> Result<SyncInventoryIdentityMessage, String> {
    let parsed: SyncInventoryIdentityMessage = serde_json::from_slice(payload)
        .map_err(|e| format!("decode sync identity message: {e}"))?;
    parsed.normalized()
}

pub fn take_inbound_receiver() -> Option<mpsc::UnboundedReceiver<InboundMessage>> {
    let lock = INBOUND_RX.get()?;
    lock.lock().ok()?.take()
}

#[no_mangle]
pub unsafe extern "C" fn p2p_init() -> c_int {
    match ensure_initialized() {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn p2p_connect_peer(peer_id: *const c_char) -> c_int {
    if peer_id.is_null() {
        return -1;
    }
    let c_str = CStr::from_ptr(peer_id);
    let peer_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    println!("Connecting to peer (stub): {}", peer_str);
    0
}

#[no_mangle]
pub unsafe extern "C" fn p2p_disconnect_peer(peer_id: *const c_char) -> c_int {
    if peer_id.is_null() {
        return -1;
    }
    let c_str = CStr::from_ptr(peer_id);
    let peer_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    println!("Disconnecting from peer (stub): {}", peer_str);
    0
}

#[no_mangle]
pub unsafe extern "C" fn p2p_send_message(
    _peer_id: *const c_char,
    message: *const u8,
    message_len: usize,
) -> c_int {
    if message.is_null() {
        return -1;
    }
    if let Err(e) = ensure_initialized() {
        eprintln!("p2p_init failed: {e}");
        return -2;
    }
    let msg_slice = std::slice::from_raw_parts(message, message_len);
    let bytes = msg_slice.to_vec();
    if let Some(tx) = PUBLISH_TX.get() {
        if tx.send(bytes).is_ok() {
            0
        } else {
            -3
        }
    } else {
        -4
    }
}

#[no_mangle]
pub unsafe extern "C" fn p2p_send_inventory_identity(
    _peer_id: *const c_char,
    plugin: *const c_char,
    kind: *const c_char,
    identity: *const c_char,
    digest: *const c_char,
    backing_ref: *const c_char,
) -> c_int {
    if kind.is_null() || identity.is_null() || digest.is_null() {
        return -1;
    }
    if let Err(e) = ensure_initialized() {
        eprintln!("p2p_init failed: {e}");
        return -2;
    }

    let plugin = if plugin.is_null() {
        None
    } else {
        match CStr::from_ptr(plugin).to_str() {
            Ok("") => None,
            Ok(value) => Some(value),
            Err(_) => return -3,
        }
    };
    let kind = match CStr::from_ptr(kind).to_str() {
        Ok(value) => value,
        Err(_) => return -4,
    };
    let identity = match CStr::from_ptr(identity).to_str() {
        Ok(value) => value,
        Err(_) => return -5,
    };
    let digest = match CStr::from_ptr(digest).to_str() {
        Ok(value) => value,
        Err(_) => return -6,
    };
    let backing_ref = if backing_ref.is_null() {
        None
    } else {
        match CStr::from_ptr(backing_ref).to_str() {
            Ok("") => None,
            Ok(value) => Some(value),
            Err(_) => return -7,
        }
    };

    let kind = match parse_sync_identity_kind(kind) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("invalid sync identity kind: {err}");
            return -8;
        }
    };
    let bytes = match encode_inventory_identity_message(plugin, kind, identity, digest, backing_ref)
    {
        Ok(value) => value,
        Err(err) => {
            eprintln!("failed to encode sync identity payload: {err}");
            return -9;
        }
    };

    if let Some(tx) = PUBLISH_TX.get() {
        if tx.send(bytes).is_ok() {
            0
        } else {
            -10
        }
    } else {
        -11
    }
}

#[no_mangle]
pub unsafe extern "C" fn p2p_cleanup() -> c_int {
    // No-op cleanup; background task ends with process.
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_identity_message_round_trips() {
        let encoded = encode_inventory_identity_message(
            Some("sync-runtime"),
            SyncIdentityKind::Receipt,
            "receipt:zkperf-run-20260329-a",
            "sha256:obs-0001",
            Some("artifact:wikidata-2026-demo"),
        )
        .expect("encode inventory identity");

        let decoded =
            decode_inventory_identity_message(&encoded).expect("decode inventory identity");

        assert_eq!(
            decoded,
            SyncInventoryIdentityMessage::new(
                Some("sync-runtime"),
                SyncIdentityKind::Receipt,
                "receipt:zkperf-run-20260329-a",
                "sha256:obs-0001",
                Some("artifact:wikidata-2026-demo"),
            )
        );
    }

    #[test]
    fn inventory_identity_message_round_trips_without_plugin_context() {
        let encoded = encode_inventory_identity_message(
            None,
            SyncIdentityKind::Artifact,
            "artifact:wikidata-2026-demo",
            "sha256:artifact-0001",
            None,
        )
        .expect("encode inventory identity");

        let decoded =
            decode_inventory_identity_message(&encoded).expect("decode inventory identity");

        assert_eq!(
            decoded,
            SyncInventoryIdentityMessage::new(
                None::<String>,
                SyncIdentityKind::Artifact,
                "artifact:wikidata-2026-demo",
                "sha256:artifact-0001",
                None::<String>,
            )
        );
        assert!(decoded.plugin.is_none());
    }

    #[test]
    fn decode_inventory_identity_message_normalizes_zkperf_shape() {
        let payload = br#"{
            "kind":"receipt",
            "object_id":"zkperf-run-20260329-a",
            "hash":"obs-0001",
            "related_artifact_ref":"wikidata-2026-demo"
        }"#;

        let decoded =
            decode_inventory_identity_message(payload).expect("decode inventory identity");

        assert_eq!(decoded.verb, "attach_inventory_identity");
        assert!(decoded.plugin.is_none());
        assert_eq!(decoded.kind, SyncIdentityKind::Receipt);
        assert_eq!(decoded.identity, "receipt:zkperf-run-20260329-a");
        assert_eq!(decoded.digest, "sha256:obs-0001");
        assert_eq!(
            decoded.backing_ref.as_deref(),
            Some("artifact:wikidata-2026-demo")
        );
    }

    #[test]
    fn parse_sync_identity_kind_accepts_producer_aliases() {
        assert_eq!(
            parse_sync_identity_kind("manifest").expect("manifest"),
            SyncIdentityKind::Artifact
        );
        assert_eq!(
            parse_sync_identity_kind("zkperf_observation").expect("observation"),
            SyncIdentityKind::Receipt
        );
    }
}
