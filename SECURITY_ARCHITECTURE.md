# ZOS Server Security Architecture

## Overview

The ZOS Server implements a comprehensive security model based on:
1. **Plugin-based Authentication** - Modular auth system with SSH key support
2. **Complexity-based Access Control** - Code complexity determines access levels
3. **Intent Verification** - Users must prove their intent to access code paths
4. **Syscall Protection** - Root-only access to system calls with syn rewrite detection

## Security Layers

### Layer 1: Authentication Plugins (`auth_manager.rs`)

- **SSH Auth Plugin**: Automatically registers `~/.ssh/id_rsa.pub` as root admin
- **Pluggable Architecture**: Easy to add new auth methods (OAuth, LDAP, etc.)
- **Role-based Access**: admin, developer, user roles with different permissions

### Layer 2: Complexity Analysis (`instruction_filter.rs`)

- **Safe (0)**: Pure functions, immutable data
- **Limited (1)**: File I/O, network (rate limited)
- **Privileged (2)**: System configuration
- **Syscall (3)**: Direct syscall access (root only)

**Code Filtering**:
- Detects dangerous patterns: `asm!`, `syscall()`, `unsafe`, etc.
- Strips complex code sections that exceed user's complexity limit
- Rate limiting per user based on role

### Layer 3: Syscall Security (`syscall_security_plugin.rs`)

**Forbidden Operations for Non-Root**:
- Direct syscalls via `syscall()` function
- Syn AST manipulation (`syn::`, `parse_quote!`, `TokenStream`)
- Dangerous syscalls: `execve`, `ptrace`, `mount`, `setuid`, etc.

**Detection Methods**:
- Pattern matching for syscall usage
- AST manipulation detection to prevent code rewriting attacks
- Complexity analysis of code operations

### Layer 4: Intent Verification (`intent_verifier.rs`)

Users must declare their intent to access code path P in orbit O:

**Intent Components**:
- **Code Path**: `orbit/path/function` (e.g., `safe/math/add`)
- **Declared Purpose**: Human-readable explanation
- **Usage Pattern**: ReadOnly, Computation, DataTransform, FileAccess, NetworkAccess, SystemConfig
- **Expected Complexity**: Must match actual code complexity
- **Proof Signature**: Cryptographic proof of intent

**Orbit Policies**:
- **safe**: Basic operations, no proof required
- **system**: Admin operations, proof required
- **network**: Network access, proof required
- **kernel**: Syscall access, root only, proof required

### Layer 5: Integrated Security (`security_layer.rs`)

Complete verification flow:
1. **Authenticate** user with public key
2. **Analyze** code complexity and safety
3. **Verify** user intent matches declared purpose
4. **Apply** rate limiting
5. **Generate** usage constraints

## Usage Examples

### Safe Computation (Allowed)
```rust
// User intent: "Adding integers for calculation"
// Orbit: safe, Pattern: Computation, Complexity: Safe
fn add(a: i32, b: i32) -> i32 { a + b }
```

### Syscall Attempt (Denied)
```rust
// User intent: "Direct system call"
// Orbit: kernel, Pattern: SystemConfig, Complexity: Syscall
// Result: DENIED - Non-root cannot access syscalls
unsafe { syscall(1, 2, 3) }
```

### Admin System Config (Allowed with Proof)
```rust
// Admin intent: "Updating system configuration"
// Orbit: system, Pattern: SystemConfig, Complexity: Privileged
// Result: ALLOWED - Admin with required proof
std::fs::write("/etc/config", "new_value")
```

## Security Guarantees

1. **Zero Trust**: Every access requires full verification
2. **Complexity Enforcement**: Users cannot access code above their complexity limit
3. **Intent Verification**: All access must be declared and proven
4. **Syscall Protection**: Only root can access system calls
5. **Syn Rewrite Prevention**: AST manipulation attacks are blocked
6. **Rate Limiting**: Prevents abuse through usage quotas

## Rate Limits by Role

- **Root**: Unlimited access
- **Admin**: 1000 requests/hour, 100MB data/hour
- **Developer**: 500 requests/hour, 50MB data/hour
- **User**: 100 requests/hour, 10MB data/hour

## Audit Trail

All security decisions are logged with:
- User ID and role
- Code path and orbit
- Intent declaration
- Verification result
- Applied constraints
- Timestamp and proof signature

This creates a complete audit trail for compliance and security analysis.
