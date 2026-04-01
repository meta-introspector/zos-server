# ZOS Server Documentation Site

This directory contains the configuration and templates for the automated documentation site.

## Features

- **📖 API Documentation**: Auto-generated rustdoc for all modules
- **📊 Performance Reports**: Binary size, build time, dependency metrics
- **🔍 Code Analysis**: Clippy reports and code quality metrics
- **📈 Version Tracking**: Historical performance data across releases
- **🎨 Responsive Design**: Clean, accessible documentation interface

## Automated Generation

The documentation site is automatically generated on:
- Every push to `main` branch
- Pull requests
- New releases

## Site Structure

```
docs-site/
├── index.html              # Main landing page
├── zos_server/            # Rustdoc API documentation
├── reports/               # Code metrics and analysis
├── perf/                  # Performance benchmarks
└── style.css             # Shared styling
```

## Performance Metrics

The site tracks:
- **Binary Size**: Executable size trends
- **Build Time**: Compilation performance
- **Dependencies**: Dependency count and tree analysis
- **Code Metrics**: Lines of code, file counts
- **Feature Flags**: Available feature configurations

## Access

The documentation site is available at:
`https://meta-introspector.github.io/zos-server/`

## Local Development

To generate docs locally:
```bash
cargo doc --all-features --no-deps --document-private-items --open
```

## Distributed Convergence

This repository's current implementation scope is bounded peer sync. Repo-facing semantic authority stays outside the convergence slice:

- `SL` is the truth and promotion boundary.
- `ZOS` is governed semantic state over promoted facts.
- `zos-server` sync moves inventory, reconciliation state, and bounded replay metadata; it must not claim semantic promotion authority.

The current sync implementation seam lives in:

- `src/node_coordinator.rs`
- `crates/zos-experimental/src/node_coordinator.rs`
- `src/extra_plugins/libp2p_c_interface.rs`
- `zos-libp2p/src/server.rs`

The intended implementation order is:

1. compact inventory exchange and missing-object reconciliation in `sync_with_peers()`
2. bind the serialized sync envelope to direct peer transport
3. validate live announcement-aware convergence and object replay across multiple peers
4. add transport refinements such as delta sync after convergence is proven
5. later trust scoring and MDL-aware replication

The current design authority for this work is:

- `docs/sync_convergence_architecture.md`

That note now carries:

- ZKP framing
- C4 and PlantUML views
- ITIL service reading
- ISO 9001 quality gates
- Six Sigma defect and control language

That note governs peer sync and replay behavior only; semantic promotion policy remains outside its scope.
