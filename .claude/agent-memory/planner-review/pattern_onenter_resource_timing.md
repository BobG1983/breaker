---
name: OnEnter resource timing with deferred commands
description: Systems reading resources inserted via deferred commands in the same OnEnter schedule need explicit ordering after the inserting chain
type: feedback
---

When a resource is inserted via `commands.insert_resource()` in an OnEnter system, the resource is not immediately available to other OnEnter systems unless explicit ordering places them AFTER an `ApplyDeferred` flush.

Example: `set_active_layout` inserts `ActiveNodeLayout` via commands. Any system that reads `Res<ActiveNodeLayout>` in the same `OnEnter(Playing)` must be ordered after `NodeSystems::Spawn` (or the chain that includes set_active_layout + ApplyDeferred).

**Why:** Bevy deferred commands are batched. Within an OnEnter schedule, systems run in parallel unless ordered. A system reading a resource that doesn't exist yet will panic (if using `Res<T>`) or see None (if using `Option<Res<T>>`).

**How to apply:** When reviewing specs that add systems on OnEnter that read a Resource, check: (1) who inserts that resource, (2) whether it's inserted via commands (deferred) or directly, (3) whether explicit ordering constraints ensure the reader runs after the inserter's commands are flushed.
