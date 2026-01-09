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
