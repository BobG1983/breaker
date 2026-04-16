---
name: propagate_breaker_changes hot reload pattern
description: Re-stamps ~30 components via 3 chained .insert() calls on hot-reload; is_changed() guard ensures zero per-frame cost in production
type: project
---

`propagate_breaker_changes` in `breaker-game/src/debug/hot_reload/systems/propagate_breaker_changes/system.rs`:

- **Guard**: `if !ctx.registry.is_changed() || ctx.registry.is_added() { return; }` — the system body only executes when the BreakerRegistry resource is actually modified. In normal gameplay (non-hot-reload path), this early-returns on every frame with zero overhead beyond a change-detection flag check.

- **When it fires**: only when the developer edits a `.ron` file at runtime and hot-reload triggers `propagate_registry`. This is a developer-only code path, never during production gameplay.

- **3 chained `.insert()` calls**: Bevy's tuple arity limit (~12 components per tuple) requires splitting ~30 components across 3 separate `.insert()` calls. Each causes an archetype move. At 1 breaker entity, this is 3 archetype transitions — completely acceptable for a dev-only hot-reload path.

- **`def.clone()`**: Clones the BreakerDefinition once. Definition is a plain data struct with no heap allocations beyond `Vec<RootNode>`. Acceptable at fire-once frequency.

**How to apply:** Do not flag the 3-insert pattern, the definition clone, or the component count as performance concerns. The `is_changed()` guard means this is zero-cost during gameplay. The multiple inserts are necessary due to tuple arity limits and are hot-reload frequency only.
