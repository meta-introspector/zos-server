# CRQ-SOCIAL-001: Pastebin Threads + Social Functions as ZOS Plugins

**Document ID**: CRQ-SOCIAL-001  
**Date**: 2026-03-30  
**Status**: Open  
**Priority**: P1

## Description

Split social/federation features from solfunmeme-dioxus and erdfa-publish into ZOS plugins. Add threading to pastebin. Integrate iroh-gossip via podping.alpha.

## Current State

### solfunmeme-dioxus/src/node.rs — paste + stego + ipfs + forge
- `POST /paste`, `GET /paste/{id}` — basic paste CRUD
- `POST /stego/encode`, `POST /stego/decode` — steganography
- `GET /ipfs/{cid}`, `POST /ipfs/add`, `POST /ipfs/publish` — IPFS
- `GET /forge/repos` — Forgejo repo listing
- `GET /peers` — peer list
- No threading, no replies, no feeds

### erdfa-publish/src/bin/solfunmeme_service.rs — paste + mesh + activitypub
- `POST /paste`, `GET /paste`, `GET /paste/<id>` — paste CRUD
- `POST /mesh/logs`, `GET /mesh/logs` — Merkle-committed mesh logs
- `POST /mesh/peers`, `GET /mesh/peers` — WireGuard mesh peers
- `GET /actor` — ActivityPub actor
- `GET /outbox` — ActivityPub outbox
- No threading, no replies

### erdfa-publish/src/federation.rs — ActivityPub types
- `WitnessNote` — zkTLS witness as ActivityPub Note
- `Actor`, `Outbox` — federation primitives
- DASL/CBOR envelopes, IPFS content-addressing

### podping.alpha — iroh-gossip P2P broadcast
- `gossip-writer/` — ZMQ → Cap'n Proto → iroh-gossip publish
- `gossip-listener/` — iroh-gossip subscribe (iroh 0.97, iroh-gossip 0.97)
- `podping/` — HTTP frontend
- Source: `/mnt/data1/meta-introspector/submodules/podping.alpha/`
- GitHub: https://github.com/meta-introspector/podping.alpha

## Proposed ZOS Plugins

### 1. zos-plugin-pastebin-threads
**Source from**: node.rs paste routes + new threading
- `POST /paste` — create paste (add `parent_id` for replies)
- `GET /paste/{id}` — get paste with thread
- `GET /paste/{id}/thread` — full thread tree
- `GET /paste/recent` — recent pastes
- `GET /paste/search?q=` — search pastes
- Thread model: each paste has optional `parent_id`, `reply_count`, `thread_root`

### 2. zos-plugin-activitypub
**Source from**: erdfa-publish/src/federation.rs
- `GET /actor` — ActivityPub actor profile
- `GET /outbox` — outbox (recent WitnessNotes)
- `POST /inbox` — receive federated messages
- `GET /feed` — Atom/JSON feed
- WitnessNote wrapping for pastes and mesh logs

### 3. zos-plugin-iroh-gossip
**Source from**: podping.alpha gossip-listener + gossip-writer
- iroh-gossip topic subscription for paste broadcasts
- New paste → gossip publish to all peers
- Gossip receive → store as remote paste
- Config: `ZOS_GOSSIP_TOPIC`, `ZOS_GOSSIP_BOOTSTRAP`

### 4. zos-plugin-mesh
**Source from**: erdfa-publish mesh routes
- `POST /mesh/logs` — Merkle-committed log ingestion
- `GET /mesh/logs` — read logs
- `POST /mesh/peers`, `GET /mesh/peers` — peer management
- ML-DSA-44 signatures, CBOR storage

### 5. zos-plugin-stego
**Source from**: node.rs stego routes
- `POST /stego/encode` — encode message in image
- `POST /stego/decode` — decode message from image

### 6. zos-plugin-ipfs
**Source from**: node.rs ipfs routes
- `GET /ipfs/{cid}` — fetch content
- `POST /ipfs/add` — add content
- `POST /ipfs/publish` — publish to IPNS

## Procedure

1. Create each plugin as standalone repo in `~/03-march/30/spinoffs/`
2. Extract relevant code from source files
3. Add ZOS plugin trait impl (`ZOSPlugin` from zos-traits)
4. Symlink into Forgejo
5. Wire into zos-server as optional plugins

## Quality Gate

Each plugin builds standalone. Pastebin threading has tests for create/reply/thread-tree.
