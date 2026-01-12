# Kiro Rules - Conservation Principles

## Rule 1: Conservation of Code
- **No Deletion**: Code cannot be permanently deleted
- **No Force Push**: History must be preserved
- **No rm -rf**: Destructive operations are forbidden
- **Balance Requirement**: All changes must balance - removed code must be added elsewhere

## Implementation
- Moves: `git mv` instead of delete + create
- Refactoring: Extract to new files instead of deletion
- Deprecation: Mark as deprecated, move to archive/ directory
- History: All git history must be preserved
- Backups: Removed code goes to `.archive/` or `deprecated/` directories

## Enforcement
- Pre-commit hooks to prevent destructive operations
- Git aliases that enforce safe operations
- Code review requirements for any "removal"
- Automated archiving of deprecated code

## Rule 2: Production Reality
- **No Simplifications**: Code must handle real-world complexity
- **No Workarounds**: Fix root causes, not symptoms
- **No Fakes**: No mock data, fake responses, or placeholder implementations
- **No Shims**: Direct integration, no compatibility layers
- **No Broken Demos**: Every demo must be fully functional production code
- **Demo Flag Required**: Mark demo code with `demo!("this code needs to be made stronger")`
- **Source Data Only**: All data must flow from real inputs and sources

## Implementation
- `demo!()` macro for marking temporary demo code
- CI checks to prevent fake data in production paths
- Real data pipelines from day one
- Integration tests with actual services
- No mocking in production code paths

## Philosophy
Information and code have inherent value. Rather than destroying, we transform and preserve.
We build production systems, not prototypes. Every line of code must be production-ready.
