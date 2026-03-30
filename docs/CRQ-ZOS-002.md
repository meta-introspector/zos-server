# CRQ-ZOS-002: Wire Social Plugins into zos-server

**Document ID**: CRQ-ZOS-002  
**Date**: 2026-03-30  
**Status**: Open  
**Priority**: P1  
**Depends on**: CRQ-SOCIAL-001

## Description

Add the 6 social plugins as optional dependencies in zos-server, loadable via the existing ZOSPlugin trait.

## Changes to zos-server

### Cargo.toml — add optional deps

```toml
[dependencies]
zos-plugin-pastebin-threads = { path = "../spinoffs/zos-plugin-pastebin-threads", optional = true }
zos-plugin-activitypub = { path = "../spinoffs/zos-plugin-activitypub", optional = true }
zos-plugin-iroh-gossip = { path = "../spinoffs/zos-plugin-iroh-gossip", optional = true }
zos-plugin-mesh = { path = "../spinoffs/zos-plugin-mesh", optional = true }
zos-plugin-stego = { path = "../spinoffs/zos-plugin-stego", optional = true }
zos-plugin-ipfs = { path = "../spinoffs/zos-plugin-ipfs", optional = true }

[features]
social = ["dep:zos-plugin-pastebin-threads", "dep:zos-plugin-activitypub", "dep:zos-plugin-iroh-gossip", "dep:zos-plugin-mesh", "dep:zos-plugin-stego", "dep:zos-plugin-ipfs"]
```

### src/main.rs — register plugins

```rust
#[cfg(feature = "social")]
{
    registry.register(Box::new(zos_plugin_pastebin_threads::PastePlugin::new()));
    registry.register(Box::new(zos_plugin_activitypub::ActivityPubPlugin::new()));
    registry.register(Box::new(zos_plugin_iroh_gossip::GossipPlugin::new()));
    registry.register(Box::new(zos_plugin_mesh::MeshPlugin::new()));
    registry.register(Box::new(zos_plugin_stego::StegoPlugin::new()));
    registry.register(Box::new(zos_plugin_ipfs::IpfsPlugin::new()));
}
```

### Build

```bash
cd ~/03-march/30/zos-clean
cargo build --features social
```

## Quality Gate

- `cargo build` without `social` feature still works (no regression)
- `cargo build --features social` compiles all 6 plugins
- `cargo test --features social` passes
