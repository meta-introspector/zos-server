# Changelog

## 2026-03-30

- Documented the sync convergence implementation plan for `zos-server`.
- Aligned repo context and TODOs around inventory exchange, reconciliation, and deferred pubsub/trust follow-on work.
- Implemented inventory-building, reconciliation-plan logic, and a serialized sync wire envelope in `src/node_coordinator.rs` and `crates/zos-experimental/src/node_coordinator.rs`, including focused unit tests in those files.
- Promoted the root `node_coordinator` seam into the active library surface and verified it with `cargo test --lib node_coordinator -- --nocapture`.
- Added `src/sync_transport.rs` to forward coordinator outbound sync frames into the libp2p C bridge.
- Wired `main.rs` and `ZosNode` so outbound/inbound sync envelopes are attached to the live coordinator loop.
- Added typed sync inventory identities (`InventoryKind::Plugin/Artifact/Receipt`) to align future artifact and receipt sources with current plugin-backed compatibility.
- Added overlay inventory ingestion from `ZOS_SYNC_INVENTORY_FILE` in both coordinators, supporting plugin + artifact + receipt records and deterministic dedupe/sort semantics.
- Narrowed remaining gaps to transport identity semantics, then layering pubsub on top once direct transport and inventory identity are stable.
- Added `docs/sync_convergence_architecture.md` to formalize the sync slice with ZKP framing, C4 and PlantUML views, and ITIL, ISO 9001, and Six Sigma release language.
- Added announcement handling to the typed sync wire envelope so the active coordinator and the experimental mirror both emit and accept peer inventory announcements over the live transport path.
- Updated focused coordinator tests to cover the current sync frame order: announcement, reconciliation, then inventory when the peer is missing local objects.
- Re-aligned README, architecture notes, TODO priorities, and compactified context to reflect current whole-surface progress and the next control gates: live multi-node validation, replay semantics, and higher-level producer integration.
- Added bounded replay-intent planning in `src/node_coordinator.rs`, so reconciliation now records canonical artifact and receipt recovery intent without claiming the fetch pipeline exists yet.
- Extended the bounded replay slice so `src/node_coordinator.rs` and `crates/zos-experimental/src/node_coordinator.rs` now carry replay metadata catalogs and recovery requests for canonical artifact and receipt gaps.
- Added focused coordinator tests for replay metadata extraction, canonical recovery-request planning, and artifact-over-plugin identity preference during inventory normalization.
- Extended active-path replay metadata toward the ITIR acknowledgement contract with publish status, revision, member-count, etag/commit, replay-token, and stream member-path coverage where those fields are present in overlays.
- Implemented bounded replay execution in `src/node_coordinator.rs` for canonical artifact and receipt gaps, including direct object fetch, tar-member extraction, and digest-verified admission into the in-memory local inventory.
- Added focused coordinator tests for direct artifact recovery and zkperf-style receipt recovery from tar members using canonical observation digest verification.
- Upgraded `src/extra_plugins/libp2p_c_interface.rs` from process-local pubsub scaffolding to a real same-host libp2p listen/dial bridge with `ZOS_SYNC_LISTEN_ADDR` and `ZOS_SYNC_BOOTSTRAP_ADDRS`.
- Added `scripts/smoke_two_peer_sync.sh` and verified a same-host two-process smoke run showing connection establishment, reconciliation, and recovered artifact admission on the second peer.
- Tightened producer-facing identity parsing in `src/extra_plugins/libp2p_c_interface.rs` and `zos-libp2p/src/server.rs` with canonical normalization and zkperf/erdfa-shaped field aliases.
- Added bounded sync transport operational controls in `src/sync_transport.rs` and `src/main.rs`, including startup visibility logs, duplicate inbound frame suppression, environment-gated transport startup, and clearer drop-path logging.
