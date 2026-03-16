## CRITICAL RULES
**NEVER edit, remove, rename, or create source files (.rs, .ron, .toml, etc.).** Only report what needs fixing — never apply fixes. The only files you may write are memory files under `.claude/agent-memory/test-runner/`.

**NEVER use bare cargo commands.** Always use dev aliases: `cargo dbuild`, `cargo dcheck`, `cargo dclippy`, `cargo dtest`. Only exception: `cargo fmt`.

# Build Status

**Last Validation: PASS** (2026-03-16 20:23 UTC, main branch)

## Latest Validation Summary (2026-03-16 20:23 UTC)

### Format: PASS
- No files need formatting

### Clippy: PASS
- Zero warnings, zero errors

### Tests: PASS
- 407 passed, 0 failed, 0 ignored
- All test suites pass cleanly

## Bevy 0.18.1 API Notes
- MessageWriter uses `.write()` method, not `.send()`
- Fixed across: bump.rs, bolt_breaker_collision.rs, bolt_cell_collision.rs, bolt_lost.rs
- Camera: `hdr` field removed — use `Camera::default()` without hdr setting
- App resource access: use `app.world_mut().resource_mut::<T>()`, not `app.world_resource_mut::<T>()`

## Formatting Rules
- Line wrap conditional expressions in assignments (bolt_breaker_collision.rs lines 43-44)
- Avoid multi-line Color::srgb calls - use single line after assignment operator
- Multi-line `assert!` with format args must be wrapped: condition, message, args on separate lines
- Multi-line method chains: wrap at logical points when necessary
- Long function call arguments: one argument per line when wrapping
- Import order: nested imports should be ordered (ClearRemainingCount, NodeSystems before systems)

## Key Patterns
- Type aliases required for complex Query filters (CellQueryFilter, BreakerQueryFilter)
- Use `.mul_add()` for floating point operations to satisfy suboptimal_flops lint
- Message struct fields marked with `#[allow(dead_code)]` if intentional API not yet consumed
- Collapse nested `if let` with inner `if` condition into single `if let ... && ...` (collapsible_if lint)
- Keep test helper structs and functions at module level, not inside test functions (items_after_statements)
