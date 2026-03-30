use crate::node_coordinator::{NodeMessage, OutboundSyncFrame, SyncWireMessage};
use libp2p::PeerId;
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::{error::Error, ffi::CString, str::FromStr};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

// Pull in the C-facing libp2p bridge without promoting the entire extra_plugins tree.
#[path = "extra_plugins/libp2p_c_interface.rs"]
mod libp2p_c_interface;

const RECENT_INBOUND_FRAME_WINDOW: usize = 1024;

/// Start outbound + inbound transport tasks and return the coordinator-facing
/// outbound sender plus the join handles.
pub fn start_libp2p_transport(
    message_tx: mpsc::UnboundedSender<NodeMessage>,
) -> (
    mpsc::UnboundedSender<OutboundSyncFrame>,
    JoinHandle<()>,
    Option<JoinHandle<()>>,
) {
    let (out_tx, out_rx) = mpsc::unbounded_channel();
    eprintln!("sync transport: starting outbound task");
    let outbound = start_libp2p_sync_transport(out_rx);
    eprintln!("sync transport: starting inbound task");
    let inbound = start_libp2p_inbound_task(message_tx);
    if inbound.is_none() {
        eprintln!(
            "sync transport: inbound receiver unavailable; inbound sync frames will be dropped"
        );
    }
    (out_tx, outbound, inbound)
}

/// Spawn a background task that forwards coordinator sync frames over the libp2p C bridge.
/// Returns the sender to hand to `ZosNode::new_with_transport` and the join handle for the task.
pub fn start_libp2p_sync_transport(
    mut rx: mpsc::UnboundedReceiver<OutboundSyncFrame>,
) -> JoinHandle<()> {
    // Ensure the bridge is initialized once.
    unsafe {
        let _ = libp2p_c_interface::p2p_init();
    }

    tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            let peer = frame.peer_id.to_string();
            if let Ok(c_peer) = CString::new(peer) {
                unsafe {
                    libp2p_c_interface::p2p_send_message(
                        c_peer.as_ptr(),
                        frame.payload.as_ptr(),
                        frame.payload.len(),
                    );
                }
            } else {
                eprintln!(
                    "sync transport: dropped outbound frame; peer id contained interior NUL byte"
                );
            }
        }
    })
}

/// Spawn a background task to pull inbound sync frames from the libp2p bridge and
/// re-inject them as coordinator messages. Returns None if the bridge receiver is
/// not available (not initialized).
pub fn start_libp2p_inbound_task(
    message_tx: mpsc::UnboundedSender<NodeMessage>,
) -> Option<JoinHandle<()>> {
    let mut rx = libp2p_c_interface::take_inbound_receiver()?;

    Some(tokio::spawn(async move {
        let mut recent_frames = RecentFrameFilter::new(RECENT_INBOUND_FRAME_WINDOW);

        while let Some(frame) = rx.recv().await {
            if recent_frames.is_duplicate(frame_fingerprint(&frame.peer_id, &frame.payload)) {
                eprintln!(
                    "sync transport: dropped duplicate inbound frame peer={} bytes={}",
                    frame.peer_id,
                    frame.payload.len()
                );
                continue;
            }

            match decode_wire_message(&frame.payload) {
                Ok(SyncWireMessage::Inventory(inventory)) => {
                    if message_tx
                        .send(NodeMessage::SyncInventory {
                            peer_id: frame.peer_id,
                            inventory,
                        })
                        .is_err()
                    {
                        eprintln!(
                            "sync transport: coordinator channel closed while forwarding inventory"
                        );
                        break;
                    }
                }
                Ok(SyncWireMessage::Announcement(inventory)) => {
                    if message_tx
                        .send(NodeMessage::SyncAnnouncement {
                            peer_id: frame.peer_id,
                            inventory,
                        })
                        .is_err()
                    {
                        eprintln!(
                            "sync transport: coordinator channel closed while forwarding announcement"
                        );
                        break;
                    }
                }
                Ok(SyncWireMessage::Reconciliation(plan)) => {
                    if message_tx
                        .send(NodeMessage::ReconciliationResult {
                            peer_id: frame.peer_id,
                            plan,
                        })
                        .is_err()
                    {
                        eprintln!(
                            "sync transport: coordinator channel closed while forwarding reconciliation"
                        );
                        break;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "sync transport: failed to decode inbound frame peer={} bytes={} err={e}",
                        frame.peer_id,
                        frame.payload.len(),
                    );
                }
            }
        }
    }))
}

fn decode_wire_message(bytes: &[u8]) -> Result<SyncWireMessage, Box<dyn Error>> {
    Ok(serde_json::from_slice::<SyncWireMessage>(bytes)?)
}

fn frame_fingerprint(peer_id: &PeerId, payload: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    peer_id.hash(&mut hasher);
    payload.hash(&mut hasher);
    hasher.finish()
}

struct RecentFrameFilter {
    capacity: usize,
    seen: HashSet<u64>,
    order: VecDeque<u64>,
}

impl RecentFrameFilter {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            seen: HashSet::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
        }
    }

    fn is_duplicate(&mut self, fingerprint: u64) -> bool {
        if self.seen.contains(&fingerprint) {
            return true;
        }

        if self.order.len() >= self.capacity {
            if let Some(evicted) = self.order.pop_front() {
                self.seen.remove(&evicted);
            }
        }

        self.order.push_back(fingerprint);
        self.seen.insert(fingerprint);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_frame_filter_drops_repeated_fingerprints() {
        let mut filter = RecentFrameFilter::new(4);
        assert!(!filter.is_duplicate(10));
        assert!(!filter.is_duplicate(20));
        assert!(filter.is_duplicate(10));
    }

    #[test]
    fn recent_frame_filter_evicts_old_entries() {
        let mut filter = RecentFrameFilter::new(2);
        assert!(!filter.is_duplicate(1));
        assert!(!filter.is_duplicate(2));
        assert!(!filter.is_duplicate(3));
        assert!(!filter.is_duplicate(1));
    }
}

/// Handle inbound sync bytes from the transport and apply them to coordinator state.
pub async fn handle_inbound_sync_bytes(
    node: &mut crate::node_coordinator::ZosNode,
    peer_id: &str,
    payload: &[u8],
) -> Result<(), Box<dyn Error>> {
    let peer = PeerId::from_str(peer_id)?;
    node.handle_transport_frame(peer, payload).await
}
