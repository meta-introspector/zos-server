# CRQ-ERDFA-001: Split erdfa-publish, erdfa-clean, zkperf into Plugins

**Document ID**: CRQ-ERDFA-001  
**Date**: 2026-03-30  
**Status**: Open  
**Priority**: P2

## Current State

### erdfa-publish (15 bins, 14 lib modules)

Bins:
| Binary | Spinoff Name | Description |
|--------|-------------|-------------|
| erdfa-cli.rs | erdfa-cli | Main CLI |
| solfunmeme_cli.rs | solfunmeme-cli | DAO CLI |
| solfunmeme_service.rs | solfunmeme-service | HTTP service |
| zkperf_services.rs | zkperf-services | 15-service zkperf |
| zkperf_verify.rs | zkperf-verify | Witness verifier |
| campaign_planner.rs | erdfa-campaign-planner | Campaign planner |
| cbor2agda.rs | erdfa-cbor2agda | CBOR → Agda |
| cbor2lean4.rs | erdfa-cbor2lean4 | CBOR → Lean4 |
| erdfa_mixer.rs | erdfa-mixer | Mixer |
| shard_dedup.rs | erdfa-shard-dedup | Shard dedup |
| solana_ingest.rs | erdfa-solana-ingest | Solana TX ingest |
| ledger_to_nft.rs | erdfa-ledger-nft | Ledger → NFT |
| stego_gossip.rs | erdfa-stego-gossip | Stego P2P gossip |
| benchmark.rs | erdfa-benchmark | Benchmarks |
| git_index.rs | git-index | (already standalone) |

Lib modules:
| Module | Spinoff Name | Description |
|--------|-------------|-------------|
| cft.rs | erdfa-cft | CFT model |
| dasl.rs | erdfa-dasl | 0xDA51 classification |
| sheaf.rs | erdfa-sheaf | Sheaf sections |
| hecke.rs | erdfa-hecke | Hecke eigenvalue sharding |
| maass.rs | erdfa-maass | Maass forms |
| privacy.rs | erdfa-privacy | Merkle commitments |
| stego.rs | erdfa-stego | Steganography |
| federation.rs | erdfa-federation | ActivityPub |
| ipfs.rs | erdfa-ipfs | IPFS integration |
| mixer.rs | erdfa-mixer-lib | Mixer core |
| distribute.rs | erdfa-distribute | Distribution |
| ingest.rs | erdfa-ingest | Ingest pipeline |
| render.rs | erdfa-render | HTML rendering |
| lib.rs | erdfa-core | Core types (keep) |

### erdfa-clean (10 lib modules, 1 bin)

| Module | Spinoff Name | Description |
|--------|-------------|-------------|
| acl.rs | erdfa-acl | Access control |
| blockchain.rs | erdfa-blockchain | Blockchain types |
| coverage.rs | erdfa-coverage | Coverage analysis |
| crypto.rs | erdfa-crypto | Crypto primitives |
| homomorphic_mixer.rs | erdfa-homomorphic | Homomorphic mixing |
| lean4.rs | erdfa-lean4 | Lean4 integration |
| modular.rs | erdfa-modular | Modular forms |
| shards.rs | erdfa-shards | Shard management |
| stego.rs | erdfa-stego-clean | Stego (clean ver) |
| symmetry.rs | erdfa-symmetry | Symmetry groups |
| zk_migration.rs | erdfa-zk-migration | ZK migration |
| zkreach.rs | erdfa-zkreach | ZK reachability |
| erdfa.rs (bin) | erdfa-bin | Main binary |

Also: `lean/`, `minizinc/`, `relay/`, `publish-rs/`, `book_src/`

### zkperf (8 bins, scripts, data)

Bins:
| Binary | Spinoff Name | Description |
|--------|-------------|-------------|
| record.rs | zkperf-record | Perf recording |
| witness.rs | zkperf-witness | Witness generation |
| parse.rs | zkperf-parse | Perf data parser |
| audit.rs | zkperf-audit | Security audit |
| regs.rs | zkperf-regs | Register analysis |
| sample.rs | zkperf-sample | Sampling |
| schema.rs | zkperf-schema | Schema tools |
| service.rs | zkperf-service | DA51 service |

Scripts:
| Script | Spinoff Name | Description |
|--------|-------------|-------------|
| selinux_static_analyze.py | zkperf-selinux-static | SELinux static analysis |
| selinux_harden.py | zkperf-selinux-harden | Harden .service files |
| selinux_merge.py | zkperf-selinux-merge | Merge → Monster torus ZKP |
| kagenti_service_gen.py | zkperf-kagenti-gen | Generate kagenti services |
| record-service.sh | zkperf-record-service | Dynamic bench |

Data dirs: `kagenti-generated/`, `selinux-merged/`, `selinux-static/`, `selinux-hardened/`, `witnesses/`, `proofs/`

## Procedure

Same as CRQ-ZOS-001: copy dir → git init → symlink .git → Forgejo adopt.

## Script

`~/03-march/30/zos-clean/scripts/split-spinoffs.sh` (reuse with new source dirs)
