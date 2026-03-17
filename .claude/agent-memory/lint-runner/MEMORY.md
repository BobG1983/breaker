## CRITICAL RULES
**NEVER edit source files.** Only report and describe fixes. Only write memory files under `.claude/agent-memory/lint-runner/`.
**NEVER use bare cargo commands.** Use `cargo dclippy`. Only exception: `cargo fmt`.

# Lint Rules (Stable)

## Cargo Formatting Rules
- Line wrap conditional expressions in assignments (bolt_breaker_collision.rs lines 43-44)
- Avoid multi-line Color::srgb calls - use single line after assignment operator
- Multi-line `assert!` with format args must be wrapped: condition, message, args on separate lines
- Multi-line method chains: wrap at logical points when necessary
- Long function call arguments: one argument per line when wrapping
- Import order: nested imports should be ordered (ClearRemainingCount, NodeSystems before systems)

## Clippy Patterns (Stable)
- Type aliases required for complex Query filters (CellQueryFilter, BreakerQueryFilter)
- Use `.mul_add()` for floating point operations to satisfy suboptimal_flops lint
- Message struct fields marked with `#[allow(dead_code)]` if intentional API not yet consumed
- Collapse nested `if let` with inner `if` condition into single `if let ... && ...` (collapsible_if lint)
- Keep test helper structs and functions at module level, not inside test functions (items_after_statements)

## Session History
See [ephemeral/](ephemeral/) — not committed.
