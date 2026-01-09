# ZOS Server Compilation Analysis

This directory contains comprehensive analysis of the ZOS Server compilation process.

## Structure

- `features/` - Feature flag definitions and usage
- `enums/` - Enum type definitions and derive macros
- `flags/` - Compile-time configuration flags
- `traces/` - Build traces with different profiling methods
- `mir/` - MIR (Mid-level IR) dumps
- `hir/` - HIR (High-level IR) dumps
- `coverage/` - Code coverage reports

## Permutation Matrix

The build system tests all combinations of:
- Features: default, all-plugins, core-only, extra-plugins, notebooklm, reqwest, self-build, uuid
- Targets: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu
- Profiles: dev, release
- Tracing: basic, strace, perf, coverage

Total permutations: 8 × 2 × 2 × 4 = 128 builds

## Usage

Run `./analyze-compilation.sh` to generate fresh analysis data.
