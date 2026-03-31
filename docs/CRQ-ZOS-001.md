# CRQ-ZOS-001: Spin Off Apps and Door Games into Standalone Repos

**Document ID**: CRQ-ZOS-001  
**Date**: 2026-03-30  
**Status**: Open  
**Priority**: P2

## Problem

zos-server has 61 .rs files in src/, 28 extra plugins, 13 crates, 19 top-level zos-* dirs, plus scripts, docs, and configs all in one repo. Hard to navigate, slow to build, unclear boundaries.

## Current Structure

### Core (keep in zos-server)
- `src/main.rs` — server binary
- `src/lib.rs` — pub mods: minimal_server_plugin, node_coordinator, sync_transport, traits
- `src/node_coordinator.rs` — peer sync (just merged PR #1)
- `src/sync_transport.rs` — libp2p transport
- `src/traits.rs` — plugin traits
- `zos-libp2p/` — libp2p crate
- `zos-traits/` — shared traits
- `zos-types/` — shared types

### Spinoff candidates → standalone example repos

| Dir/Files | Proposed Repo | Description |
|-----------|--------------|-------------|
| `zos-retro-games/` | `zos-retro-games` | Door games (retro BBS style) |
| `zos-telegram-bot/` | `zos-telegram-bot` | Telegram bot integration |
| `zos-community-economy/` | `zos-community-economy` | Economy/token system |
| `zos-oracle/` | `zos-oracle` | Oracle cloud deployment |
| `zos-oci/` | `zos-oci` | OCI container tooling |
| `zos-public-gateway/` | `zos-public-gateway` | Public HTTP gateway |
| `zos-minimal-server/` | `zos-minimal-server` | Minimal standalone server |
| `zos-stage1-server/` | `zos-stage1-server` | Stage 1 bootstrap server |
| `zos-deploy/` | `zos-deploy` | Deployment scripts |
| `zos-bootstrap/` | `zos-bootstrap` | Bootstrap tooling |
| `zos-plan/` | `zos-plan` | Planning docs |
| `zos-analysis/` | `zos-analysis` | Code analysis tools |
| `zos-unix-accounts/` | `zos-unix-accounts` | Unix account management |
| `browser-extension/` | `zos-browser-extension` | Browser extension |

### Crates to evaluate (keep or spinoff)

| Crate | Keep/Spinoff | Reason |
|-------|-------------|--------|
| `zos-plugins` | keep | core plugin system |
| `zos-experimental` | keep | experimental coordinator |
| `zos-metameme` | spinoff | meta-meme integration |
| `zos-monsters` | spinoff | monster symmetry breaking |
| `zos-eigenmatrix` | spinoff | eigenmatrix math |
| `zos-biosemiotic` | spinoff | biosemiotic modeling |
| `zos-godel` | spinoff | Gödel numbering |
| `zos-orbitals` | spinoff | orbital mechanics |
| `zos-prime-flags` | spinoff | prime flag lattice |
| `zos-cryptography` | spinoff | crypto primitives |
| `zos-auto-label` | spinoff | auto-labeling |
| `zos-legacy` | spinoff | legacy compat |
| `zos-protocols` | keep | protocol definitions |

### Extra plugins (28 files) → `zos-plugins-contrib` repo

All `src/extra_plugins/*.rs` — move to a separate plugins contrib repo.

### Loose src/ files (61 total) → categorize

| Category | Files | Action |
|----------|-------|--------|
| Core server | main, lib, traits, node_coordinator, sync_transport, minimal_server_plugin | keep |
| Auth | auth_manager, auth_system | keep or `zos-auth` |
| Compiler | compiler_integration, llm_compiler_service, bytecode_manipulator | `zos-compiler` |
| Security | exploit_detector, entropy_scanner, binary_classifier, binary_inspector | `zos-security` |
| LMFDB | lmfdb_orbit_filter, lmfdb_orbits, lmfdb_risk_matrix | `zos-lmfdb` |
| Lattice | lattice_builder, harmonic_code_filter | `zos-lattice` |
| Other | fools_path, notebooklm_cli, metacoq_nat, etc. | evaluate per file |

## Procedure

1. For each spinoff: `git subtree split --prefix=<dir> -b split-<name>`
2. Create new repo on Forgejo + GitHub
3. Push split branch as main
4. Remove dir from zos-server, add as git submodule or optional dep
5. Update zos-server Cargo.toml workspace members

## Quality Gate

- zos-server builds and 28 tests pass after each spinoff
- Each spinoff repo builds standalone
- No broken imports
