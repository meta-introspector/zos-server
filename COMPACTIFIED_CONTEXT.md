# Compactified Context

## 2026-04-01

- External context source: ChatGPT thread `JMD FORMAL EXPLAIN - Meme System Explanation`.
- Online UUID: `69c4a9b1-d014-83a0-8bb0-873e4eaa4098`.
- Canonical thread ID: `c6e383233d0d7c4efde671be1432c825054cb222`.
- Source used: `db` via `/home/c/chat_archive.sqlite` after resolving the referenced online thread in the canonical archive.
- Main decision pulled from thread: `SL` remains the truth and promotion boundary; `ZOS` should be described as governed semantic state over promoted facts, not as a truth override layer.
- Repo-facing consequence: `zos-server` should describe its live sync slice as transport, reconciliation, and bounded replay over artifact state, while keeping semantic promotion out of scope.

- External context source: ChatGPT thread `Agda Haskell P2P Integration`.
- Online UUID: `69cbe91f-e340-839a-8532-22153f7ac8dc`.
- Canonical thread ID: `d339a0868d473d7e6421bb6bba132fe1dea7d59c`.
- Source used: `db` via `/home/c/chat_archive.sqlite` after direct pull into the canonical archive.
- Main decision pulled from thread: for repo-facing architecture, P2P is the state movement and replication layer below SL and below the ZOS/MDL structure layer; ZOS should not be described as a layer above SL.
- Corrected stack wording for `zos-server` notes:
  - Layer 5: interface and apps
  - Layer 4: SL as truth and promotion boundary
  - Layer 3: ZOS plus DASHI plus MDL as the structure and selection layer
  - Layer 2: P2P as state movement, sync, and replication
  - Layer 1: storage and transport
- Workspace note: `mesh-sync-rs` exists at `/home/c/Documents/code/mesh-sync-rs` as a sibling repo and is not currently part of the `zos-server` git tree.

## 2026-03-30

- Task focus: make `zos-server` the working project for node-to-node artifact sync rather than general ZOS platform work.
- External context source: ChatGPT thread `Context Alignment and Stack Mapping`.
- Online UUID: `69c9f301-8a48-8398-a2a2-9ef2548f52e7`.
- Canonical thread ID: `70307e6a53d613c3ae51e7fa9786eddd04d0b205`.
- Source used: `db` via `/home/c/chat_archive.sqlite`.
- Main decision pulled from thread: the missing core is convergence, specifically inventory exchange, missing-object reconciliation, and state-sync semantics.
- Implementation order pulled from thread: `sync_with_peers()` inventory/reconciliation first, pubsub/delta sync second, trust scoring later, MDL-aware replication later.
- Repo-specific seam: `src/node_coordinator.rs`, `crates/zos-experimental/src/node_coordinator.rs`, `src/extra_plugins/libp2p_c_interface.rs`, and `zos-libp2p/src/server.rs`.
- Process note for the current pass: govern sync work through direct transport first, with explicit ITIL, ISO 9001, Six Sigma, and C4/PlantUML documentation before further behavior changes.
- Current implementation state:
  - direct inventory and reconciliation transport is wired into the active coordinator loop
  - typed inventory identity covers plugin, artifact, and receipt records
  - announcement frames are now emitted and accepted in both the active and experimental coordinator paths
- Current progress estimate for the whole `zkperf` + `zos` + `SL` + p2p integration surface:
  - overall: about 60-70 percent complete
  - data model and inventory identity: about 75-85 percent
  - transport wiring: about 65-75 percent
  - proven multi-node convergence behavior: about 40-50 percent
  - operational hardening and simplification: about 25-35 percent
- Current prioritized gap list:
  1. move from same-host two-process libp2p proof to remote multi-operator peer validation
  2. finish acknowledged locator coverage across all producer surfaces and sinks now that active-path replay/object-fetch and digest verification exist
  3. tighten higher-level integration with zkperf and sibling producer surfaces beyond parse-surface normalization
  4. extend observability, idempotency, and startup/failure controls beyond bounded transport guards
  5. reduce coordinator duplication between the active and experimental paths
- ITIR constraint now governing replay work:
  - `zos-server` may perform bounded consumer-style recovery over acknowledged artifact and receipt locators
  - `zos-server` must not become a StatiBaker-style receipt ledger, observer memory store, or timeline authority
- Newly landed bounded implementation slices in this turn:
  - replay intent planning plus bounded replay locator metadata for canonical missing local artifact and receipt identities in the active and experimental coordinator paths
  - active-path bounded replay execution for canonical artifact and receipt gaps, including file/http/hf/ipfs locator fetch, tar-member extraction, and digest-verified admission into ephemeral local inventory
  - stream receipt metadata now preserves container/member-path context so zkperf observation receipts can be recovered against the matched observation payload
  - same-host libp2p peer transport now really listens and dials via `ZOS_SYNC_LISTEN_ADDR` and `ZOS_SYNC_BOOTSTRAP_ADDRS`
  - `scripts/smoke_two_peer_sync.sh` proved two-process connection establishment, reconciliation, and artifact recovery on one machine
  - producer identity normalization and alias support on the bridge/server parse surfaces
  - bounded transport operational controls for startup visibility, duplicate inbound frame suppression, and decode/drop logging
