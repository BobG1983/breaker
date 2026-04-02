---
name: Builder pattern pedantic lint patterns
description: Recurring clippy pedantic/nursery lint patterns that appear when adding builder methods returning Self in this codebase
type: project
---

Builder methods returning `Self` require `#[must_use]` (`clippy::return_self_not_must_use`).

`pub fn with_X(mut self, ...) -> Self` — each builder method needs `#[must_use]` on the method.

`Option<Option<T>>` fields in builder intermediate state (`clippy::option_option`) — used in BreakerBuilder to distinguish "not set" vs "explicitly None". Genuine use case that clippy pedantic flags; needs a custom enum or type alias if the lint must pass.

`collapsible_if` in build methods: nested `if let Some(x) = opt { if !x.is_empty() { ... } }` must be collapsed to `if let Some(x) = opt && !x.is_empty() { ... }` (requires let-chains, Rust 2024+).

`doc_markdown`: identifiers like `y_position`, `color_rgb`, `max_speed` in doc comments must be wrapped in backticks.

`items_after_statements` in tests: helper structs defined after `let` statements in a test body must be moved to the top of the test or outside the test function.

`similar_names` in tests: variable names like `bw_a`/`by_a` are flagged as too similar — rename to be clearly distinct (e.g., `base_width_a`/`base_y_a`).

**Why:** These all surfaced together in Wave 6 BreakerBuilder implementation (feature/breaker-builder-pattern).
**How to apply:** When reviewing builder pattern code, expect all of these to appear together.

## Additional nursery patterns (post-fix residuals, same branch)

`option_if_let_else` (`clippy::nursery`): `match opt { Some(x) => Foo(x), None => Bar }` should be `opt.map_or(Bar, |x| Foo(x))`. Appears in builder `build()` methods that map `Option<u32>` to a custom enum.

`redundant_clone` (`clippy::nursery`): `let x = val.clone();` where `val` is dropped immediately after — clippy infers the clone is unnecessary. Appears in test helpers that clone a definition for a single call.

`or_fun_call` (`clippy::nursery`): `.map_or("None".to_string(), ...)` must be `.map_or_else(|| "None".to_string(), ...)` because the eager default allocates unconditionally. Appears in test helpers that format `Option<u32>` for assertion messages.
