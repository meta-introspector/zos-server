# SErust - SELinux-style Rust Security

SErust brings SELinux-style mandatory access control to Rust through friendly declarative macros and Clippy lints.

## Features

- **Domain-based Access Control**: Functions and modules declare their security domain (L0-L4)
- **Orbit Classification**: Mathematical orbit theory for complexity-based security
- **Capability Requirements**: Fine-grained permission system
- **Syscall Allowlists**: Explicit syscall permissions with virtual alternatives
- **Provenance Tracking**: Complete audit trail of execution and data flow
- **Friendly Clippy Lints**: Helpful suggestions for security annotations

## Quick Start

Add security annotations to your functions:

```rust
use serust_macros::*;

#[serust_domain(level = 0, orbits = "trivial", capabilities = "read,compute")]
#[orbit(trivial)]
#[track_provenance]
pub fn safe_calculator(a: i32, b: i32) -> i32 {
    a + b  // O(1) operation in trivial orbit
}

#[serust_domain(level = 2, orbits = "symmetric", capabilities = "read,compute,data")]
#[orbit(symmetric)]
#[requires(data)]
#[track_provenance]
pub fn sort_data<T: Ord>(mut data: Vec<T>) -> Vec<T> {
    data.sort();  // O(n log n) operation in symmetric orbit
    data
}
```

## Security Domains

- **L0 Public**: Basic computation, no syscalls
- **L1 System**: File operations with restricted syscalls
- **L2 Data**: Data processing with transformation capabilities
- **L3 Admin**: Administrative operations with elevated privileges
- **L4 Kernel**: Unrestricted access for kernel-level operations

## Orbit Classifications

Based on LMFDB mathematical group theory:

- **Trivial**: O(1) operations, basic arithmetic
- **Cyclic**: O(n) operations, linear algorithms
- **Symmetric**: O(n!) operations, sorting, permutations
- **Alternating**: O(2^n) operations, complex algorithms
- **Sporadic**: Irregular complexity, special cases
- **Monster**: Unrestricted complexity, kernel operations

## Clippy Lints

SErust provides helpful Clippy lints that suggest security annotations:

```rust
// This function will trigger lint suggestions:
pub fn unsafe_function() {
    std::process::Command::new("rm").arg("-rf").arg("/");  // Suggests #[allow_syscalls]
    std::fs::read("file.txt");  // Suggests #[track_provenance]
}

// Properly annotated version:
#[serust_domain(level = 1, orbits = "trivial", capabilities = "read,file")]
#[orbit(trivial)]
#[requires(file)]
#[allow_syscalls("read", "open", "close")]
#[track_provenance]
pub fn safe_function() -> Result<String, String> {
    // Implementation with virtual file system
    Ok("File content".to_string())
}
```

## Testing

SErust generates compliance tests automatically:

```rust
// Generate orbit compliance test
test_orbit_compliance!(function = "safe_calculator", orbit = "trivial");

// Generate security domain test
security_test!(name = "public_calc_allowed", domain = "l0_public", should_allow = true);
security_test!(name = "admin_op_denied", domain = "l0_public", should_allow = false);
```

## Demo

Run the SErust demonstration:

```bash
cargo run serust-demo
```

This shows all security domains in action with proper enforcement and tracking.

## Integration with ZOS

SErust integrates with the ZOS security lattice:

- Compile-time verification of security properties
- Runtime enforcement of domain restrictions
- Mathematical proofs of orbit compliance
- Complete provenance tracking for audit trails
- SAT solver verification of execution paths

The system ensures that code cannot escape its declared security domain and provides mathematical guarantees about complexity and resource usage.
