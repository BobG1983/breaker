---
name: ScreenLifecycle trait — associated methods, not associated types or derive
description: How to express "enum must expose Loading/AnimateIn/AnimateOut/Teardown phases" via a trait for rantzsoft_stateflow
type: project
---

## Decision

Use associated methods on a supertrait of `States`:

```rust
pub trait ScreenLifecycle: States {
    fn loading()    -> Self;
    fn animate_in() -> Self;
    fn animate_out() -> Self;
    fn teardown()   -> Self;
}
```

Manual impls per enum (4 lines each). No derive macro yet.

## Why Not Derive Macro Now

Three impls at migration time. Project YAGNI rule: no abstraction until 6+ impls or the boilerplate clearly costs more than the crate overhead.

**Why:** `rantzsoft_defaults_derive` precedent exists and would be the model, but the overhead (new crate, Cargo.toml, `syn`/`quote` deps) is not justified at 3 impls.

**How to apply:** If screen state types grow to 6+, revisit. At that point the derive macro compares variant names against `["Loading", "AnimateIn", "AnimateOut", "Teardown"]` using `syn::Error::new(name.span(), ...)` for missing variant errors.

## States Compatibility

`States` defines no methods — only `DEPENDENCY_DEPTH: usize`. No conflict with `ScreenLifecycle` methods.

## Active Phase

Do NOT add `fn active() -> Self` to the trait unless a generic system needs to advance to it. Each screen owns its own advance-to-active system.

## Full Research

See `docs/todos/detail/spawn-bolt-setup-run-migration/research/enum-trait-constraints.md`
