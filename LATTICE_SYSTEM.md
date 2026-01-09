# ZOS Server Lattice Logging System

The ZOS Server uses a canonical lattice coordinate system to organize and identify build logs from GitHub Actions. Each build configuration gets a unique lattice ID that encodes its parameters.

## Lattice ID Format

```
L<level>.<weight>.<character>.<orbit>
```

### Coordinate Mapping

- **Level** (1-100): Derived from feature hash - represents feature complexity
- **Weight** (1-10): Derived from target hash - represents architecture weight
- **Character** (0-4): Derived from profile hash - represents build character
- **Orbit** (1-20): Derived from trace hash - represents tracing orbit

### Examples

- `L42.3.1.7` - Feature hash→42, Target hash→3, Profile hash→1, Trace hash→7
- `L89.7.0.15` - Different configuration with different coordinate values

## Bucket Structure

Each lattice creates a structured bucket with organized logs:

```
logs/lattice-L42.3.1.7/
├── matrix-config.json          # Build matrix configuration
├── lattice-manifest.json       # Complete lattice metadata
├── build-detailed.log          # Detailed build output
├── build-status.txt            # Build result status
├── feature-flags.txt           # Extracted feature flags
├── enums.txt                   # Extracted enum definitions
├── cfg-options.txt             # Compile-time options
├── rustc-info.txt             # Compiler version info
├── cargo-info.txt             # Cargo version info
├── target-info.txt            # Target architecture info
├── strace.log                 # System call trace (if strace)
├── perf.data                  # Performance data (if perf)
├── perf-report.txt            # Performance report (if perf)
├── coverage.lcov              # Coverage report (if full-coverage)
├── analysis-summary.json      # Analysis metadata (if full-coverage)
├── mir/                       # MIR dumps (if full-coverage)
└── hir/                       # HIR dumps (if full-coverage)
```

## Matrix Parameters

### Features
- `default`, `all-plugins`, `core-only`, `extra-plugins`
- `notebooklm`, `reqwest`, `self-build`, `uuid`

### Targets
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`

### Profiles
- `dev` - Development build
- `release` - Optimized release build

### Trace Types
- `strace` - System call tracing
- `perf` - Performance profiling
- `self-trace` - Application-level tracing
- `full-coverage` - Complete coverage with MIR/HIR

## Using the Lattice Analyzer

### Installation
```bash
# Make sure GitHub CLI is installed and authenticated
gh auth login

# Make the analyzer executable
chmod +x lattice-analyzer.sh
```

### Basic Usage

```bash
# List available lattice buckets
./lattice-analyzer.sh list

# Pull specific lattice
./lattice-analyzer.sh pull L42.3.1.7

# Pull all lattices from latest run
./lattice-analyzer.sh pull-all

# Analyze specific lattice
./lattice-analyzer.sh analyze L42.3.1.7

# Show lattice index
./lattice-analyzer.sh index

# Search lattices by pattern
./lattice-analyzer.sh search "all-plugins.*release"
```

### Advanced Usage

```bash
# Use specific GitHub run
./lattice-analyzer.sh --run-id 12345 pull-all

# Use different repository
./lattice-analyzer.sh --repo owner/repo list

# Use GitHub token for private repos
./lattice-analyzer.sh --token ghp_xxx pull-all
```

## Lattice Index

The system generates a master index containing:

- **lattices.json** - Complete metadata for all lattices
- **statistics.json** - Aggregate statistics
- **README.md** - Human-readable lattice listing

### Index Structure

```json
{
  "lattice_id": "L42.3.1.7",
  "coordinates": {
    "level": 42,
    "weight": 3,
    "character": 1,
    "orbit": 7
  },
  "matrix": {
    "features": "all-plugins",
    "target": "x86_64-unknown-linux-gnu",
    "profile": "release",
    "trace": "full-coverage"
  },
  "files": {
    "logs": ["build-detailed.log"],
    "data": ["perf.data"],
    "reports": ["perf-report.txt"],
    "coverage": ["coverage.lcov"]
  },
  "metadata": {
    "github_run_id": "12345",
    "github_sha": "abc123",
    "timestamp": "2026-01-08T23:36:46Z",
    "size_bytes": 1048576
  }
}
```

## Analysis Capabilities

### Build Analysis
- Compilation success/failure rates
- Build time analysis across configurations
- Error pattern identification
- Warning trend analysis

### Performance Analysis
- System call patterns (strace)
- CPU usage profiles (perf)
- Memory allocation patterns
- I/O bottleneck identification

### Coverage Analysis
- Code coverage by feature combination
- MIR/HIR analysis for compiler insights
- Dead code identification
- Feature interaction analysis

### Cross-Matrix Analysis
- Feature impact on build times
- Target architecture differences
- Profile optimization effectiveness
- Trace method comparison

## Integration with CI/CD

The lattice system integrates with GitHub Actions to:

1. **Generate** canonical IDs for each build matrix combination
2. **Collect** detailed logs and traces in organized buckets
3. **Upload** artifacts with retention policies
4. **Index** all lattices for easy discovery
5. **Enable** post-build analysis and debugging

## Benefits

- **Reproducible** - Each lattice ID uniquely identifies a build configuration
- **Organized** - Structured buckets prevent log chaos
- **Searchable** - Pattern-based lattice discovery
- **Analyzable** - Rich metadata enables deep analysis
- **Scalable** - Handles large build matrices efficiently
- **Debuggable** - Detailed traces for issue investigation

## Mathematical Foundation

The lattice coordinate system is based on modular arithmetic applied to SHA256 hashes of build parameters, ensuring:

- **Deterministic** - Same parameters always generate same ID
- **Distributed** - Hash collisions are extremely rare
- **Bounded** - Coordinates fit within reasonable ranges
- **Meaningful** - Each coordinate represents a build dimension
