---
name: deferred_commands_observer
description: Observers using commands.insert() cannot read back the inserted value in the same observer call
type: feedback
---

In Bevy observers, when `stack_u32` or `stack_f32` takes the `None` path, it calls `commands.entity(entity).insert(...)`. This insert is DEFERRED — the component is not present on the entity until after the next `flush()`.

**Why this matters for specs:** If a spec says "after stack_u32, read back the new value to sync a second component," that only works for the `Some` (already-existing) path. For the `None` (first insertion) path, the spec must provide a separate strategy — typically: if `field` was `None`, the new value is `per_stack` (known statically from the function arguments).

**How to apply:** Any spec that says "insert component B to mirror component A after A is set via commands" must explicitly handle:
- `None` path: infer the value from `per_stack`, insert B also via commands
- `Some` path: read mutably borrowed `existing` after `stack_u32`, then insert/update B
