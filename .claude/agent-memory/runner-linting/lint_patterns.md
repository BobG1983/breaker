---
name: Lint Patterns
description: Recurring cargo fmt and clippy patterns specific to this codebase
type: reference
---

## Cargo Formatting Rules
- Line wrap conditional expressions in assignments (bolt_breaker_collision.rs lines 43-44)
- Avoid multi-line Color::srgb calls - use single line after assignment operator
- Multi-line `assert!` with format args must be wrapped: condition, message, args on separate lines
- Multi-line method chains: wrap at logical points when necessary
- Long function call arguments: one argument per line when wrapping
- Import order: nested imports should be ordered (ClearRemainingCount, NodeSystems before systems)

## Clippy Patterns
- Type aliases required for complex Query filters (CellQueryFilter, BreakerQueryFilter)
- Use `.mul_add()` for floating point operations to satisfy suboptimal_flops lint
- Collapse nested `if let` with inner `if` condition into single `if let ... && ...` (collapsible_if lint)
- Keep test helper structs and functions at module level, not inside test functions (items_after_statements)
