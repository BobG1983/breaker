---
name: pub(super) vs pub(crate) in systems/mod.rs
description: Pattern for when to use pub(super) vs pub(crate) in systems/mod.rs re-exports
type: project
---

In `systems/mod.rs` files, the established convention is:
- `pub(super)` for system functions consumed only by their immediate parent plugin
- `pub(crate)` for system functions consumed from elsewhere in the crate (e.g., from `run/plugin.rs`)
- Plain `pub use` for system functions is too wide — no system functions are consumed externally by the scenario runner

This is consistently followed in chip_select/systems/mod.rs and run_end/systems/mod.rs.

**Why:** System functions don't form a public API. External consumers (scenario runner) only need resources, messages, and component types — never raw system function pointers.

**How to apply:** Flag any `pub use system_fn` in a systems/mod.rs as a visibility violation unless you can find an external consumer.
