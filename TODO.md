# TODO

## Sync Convergence

- Keep `docs/sync_convergence_architecture.md` aligned with any transport or inventory behavior change.
- [x] Define `canonical sync identity v1` in `docs/sync_convergence_architecture.md` for artifact/receipt-backed reconciliation.
- [x] Make reconciliation precedence explicit: artifact/receipt identity and digest first, plugin name as legacy fallback only.
- [x] Mark plugin-name inventory as compatibility/debug metadata, not canonical convergence identity.
- [x] Expand the newly added compact peer inventory exchange in `sync_with_peers()` into real transport-backed reconciliation.
- [x] Reconcile missing objects by diffing local and peer inventories before any transport refinements.
- [x] Replace plugin-name-only inventory fallback with artifact and receipt identities from overlay manifests while preserving plugin compatibility.
- [x] Keep the coordinator-level reconciliation logic transport-agnostic enough to verify correctness without requiring full pubsub adoption.
- [x] Bind outbound `OutboundSyncFrame` traffic to a real direct peer transport in the libp2p layer (channel-to-C bridge adapter exists in `src/sync_transport.rs`; wired to the active libp2p path).
- [x] Add inbound transport handling that deserializes the sync wire envelope back into coordinator inventory/reconciliation state once transport delivers bytes.
- [x] Wire announcement handling into the sync envelope so peer inventory can be refreshed through the live transport path.
- [x] Extend focused coordinator tests to cover announcement, reconciliation, and inventory frame emission order.
- [x] Normalize producer-facing artifact and receipt identity fields on the bridge/server parse surfaces for zkperf and erdfa-shaped payloads.
- [x] Add bounded transport operational controls for startup visibility, duplicate inbound frame suppression, and decode/drop logging.
- [x] Make replay intent explicit for canonical missing local artifact and receipt identities without faking the fetch pipeline.
- Keep `zos-server` replay recovery ephemeral and consumer-like; do not let it become a StatiBaker-style receipt or timeline ledger.
- Defer trust scoring and MDL-aware replication until basic convergence is stable.

## Whole-Surface Priorities

- [x] Run a same-host two-process smoke validation over the live libp2p path and capture connection, reconciliation, and recovery traces.
- `P0`: confirm the same flow in a remote multi-operator run so peer setup is no longer same-host-only.
- [x] Carry bounded replay locator metadata in sync inventory so canonical artifact and receipt gaps can drive recovery planning against `objectRef`-style sources.
- [x] Extend replay metadata beyond bare locator stubs so acknowledged revision, publish status, member count, etag/commit hints, replay token, and stream member-path locators are carried where present.
- [x] Execute bounded object replay and recovery for canonical artifact and receipt gaps, including digest verification and tar-member extraction for receipt-shaped payloads.
- `P0`: finish acknowledged locator coverage across all producer surfaces and sinks, especially where current overlays still omit stable member-path or authoritative read-back details.
- `P1`: connect the current sync identity contract cleanly to zkperf and sibling producer outputs so overlay-driven identity becomes the normal path, not just a parse-surface seam.
- `P1`: extend operational controls from bounded transport guards into replay-attempt visibility, duplicate semantic handling, and startup sequencing evidence.
- `P2`: reduce duplication between `src/node_coordinator.rs` and `crates/zos-experimental/src/node_coordinator.rs` once the active path is proven.
- `P2`: evaluate whether broader pubsub fanout or delta sync still adds value after direct convergence and replay are stable.
