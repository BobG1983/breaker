---
name: let_chains_collapsible_if
description: Clippy collapsible_if requires let-chains when nesting if bool { if let Some(x) = y { ... } }
type: reference
---

This codebase uses Rust edition 2024, which stabilizes let-chains.

When clippy reports `collapsible_if` on:
```rust
if bool_condition {
    if let Some(x) = expr {
        body
    }
}
```

The fix is to collapse using `&&`:
```rust
if bool_condition
    && let Some(x) = expr
{
    body
}
```

This applies to all the `propagate_*_defaults` hot-reload systems where `event.is_modified(id)` guards an `assets.get(id)` pattern match.
