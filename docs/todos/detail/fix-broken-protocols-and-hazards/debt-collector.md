# Debt Collector

Assumes: #2 (`mutators/protocols/debt_collector/`). No other TODO dependencies — this is a pure encapsulation fix.

## What's broken

`DebtStack` and `DebtCashOut` are declared `pub struct` and re-exported with `pub use` from the module root. Design spec says `pub(crate)`. Unnecessary public API surface that leaks internal types to anything outside the crate.

## Fix

### Narrow visibility

`mutators/protocols/debt_collector/system.rs` (or wherever the types currently live):

```rust
// before
pub struct DebtStack { ... }
pub struct DebtCashOut { ... }

// after
pub(crate) struct DebtStack { ... }
pub(crate) struct DebtCashOut { ... }
```

`mutators/protocols/debt_collector/mod.rs`:

```rust
// before
pub use system::{DebtStack, DebtCashOut};

// after
pub(crate) use system::{DebtStack, DebtCashOut};
```

### Verify no external dependency

Run `cargo dcheck` after the change. If it surfaces a caller outside the crate:

- If the caller is inside `breaker-game` (a test helper, a debug tool, another domain): it keeps working — `pub(crate)` is crate-scoped, and `breaker-game` is a single crate.
- If the caller is outside `breaker-game` (another workspace member): surfacing is a design question — stop and re-escalate. `DebtStack` / `DebtCashOut` are internal state types that should never be read outside the protocol's own module.

## Tests

No new tests. No behavior change.

## Code changes summary

| File | Change |
|------|--------|
| `mutators/protocols/debt_collector/system.rs` | `pub struct DebtStack` → `pub(crate) struct DebtStack`; same for `DebtCashOut` |
| `mutators/protocols/debt_collector/mod.rs` | `pub use` → `pub(crate) use` |

## Out of scope

- RON tuning fix (→ `ron-tuning-values.md`)
- Any HUD/visibility work showing the player's debt (that's a Phase 5 UI concern)
- Cash-out mechanics themselves (already working via Pattern B on the damage crate)
