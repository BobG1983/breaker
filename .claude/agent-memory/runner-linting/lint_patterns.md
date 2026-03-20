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
- Doc comments: wrap Rust identifiers and types in backticks (e.g., `HashMap`, `BreakerState`) for doc_markdown lint
- `private_interfaces` warning: `pub(crate)` structs in `chips/components.rs` (BoltSpeedBoost, BreakerSpeedBoost, WidthBoost, BumpForceBoost) referenced in `pub` query type aliases (breaker/queries.rs) and `pub` system functions. Fix: either make the chip components `pub`, or make the query aliases `pub(crate)`. Not an error — just a warning — but appears in bulk (9 instances) whenever chip components are `pub(crate)`.
- `E0364` re-export visibility error: Stream 4 pub→pub(crate) visibility fixes introduced a mismatch between function visibility and mod.rs `pub use` re-exports. Pattern: function is `pub(crate)` in its own file but re-exported with `pub use` in `mod.rs`. The `pub use` in mod.rs must be downgraded to `pub(crate) use` to match. Affected in phase4b2: `bolt/systems/prepare_bolt_velocity.rs:18`, `breaker/systems/bump.rs:67+133` (grade_bump, update_bump), `breaker/systems/move_breaker.rs:19`.
- `similar_names` error (clippy::pedantic): test-local bindings `cell_a_y` and `cell_b_y` are too similar. Occurs in test code at `physics/systems/bolt_cell_collision.rs:1213-1214`. Fix: rename one binding (e.g., `cell_below_y` instead of `cell_b_y`).
- `type_complexity` error: Query with 6-tuple data + filter tuple triggers this in `bolt/systems/reset_bolt.rs` (the bolt_query with Entity, Transform, BoltVelocity, Option<PiercingRemaining>, Option<Piercing>, Option<PhysicsTranslation>) and `bolt/systems/spawn_bolt.rs` (bundled tuple `(Query<&Transform, With<Breaker>>, Query<Entity, With<Bolt>>)`). Fix: extract named type aliases to `bolt/queries.rs` following the `LostQuery` pattern already there.
- `if_not_else` error (clippy::pedantic): `if !condition { A } else { B }` pattern. Fix: invert the condition to `if condition { B } else { A }`. Occurs in `chips/effects/piercing.rs:35` (`if !had_piercing`).
- `uninlined_format_args` error (clippy::pedantic): `assert!(..., "msg {}", var)` where `var` can be inlined as `{var}`. Fix: replace `{}` + trailing arg with `{var}`. Occurs in `chips/effects/mod.rs:179`.
