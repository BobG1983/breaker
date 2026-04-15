---
name: Locked/Invulnerable archetype and scheduling patterns
description: Wave F2/F3 locked-cell patterns — component hooks, empty sync system, check_lock_release scheduling — all confirmed acceptable at 50–200 cell scale
type: project
---

System files:
- `breaker-game/src/cells/behaviors/locked/components.rs`
- `breaker-game/src/cells/behaviors/locked/systems/sync_lock_invulnerable.rs`
- `breaker-game/src/cells/behaviors/locked/systems/check_lock_release/system.rs`
- `breaker-game/src/cells/plugin.rs`

## Confirmed patterns

**Invulnerable archetype churn via Locked hooks**
- `Locked::on_insert` adds `Invulnerable`; `Locked::on_remove` removes it
- Happens on lock-cell spawn and on lock-cell unlock (one-time per cell, not per-frame)
- Archetype moves are bounded by grid layout — at most N locked cells in the grid, each unlocking once
- Not per-frame; negligible at 50–200 cells

**`Without<Invulnerable>` on `DamageTargetQuery` in `apply_damage<T>`**
- Adds one archetype filter; narrows the matched set to non-immune entities
- This is a benefit (fewer entities iterated), not a cost
- No false-positive concern — filter is on presence of a ZST marker

**`sync_lock_invulnerable` — empty const fn with Commands**
- The `Commands` parameter is intentional: it creates an `ApplyDeferred` sync point so hook-queued `insert(Invulnerable)` / `remove::<Invulnerable>()` commands from the same tick land before downstream systems observe the change
- Zero work inside the function — no allocation, no query
- Forcing a sync point in FixedUpdate is the correct and documented design choice here
- Runs after `check_lock_release` — ordering is correct

**`check_lock_release` scheduling: `.after(DeathPipelineSystems::HandleKill)`**
- Dead is inserted via `commands.entity(v).insert(Dead)` inside `handle_kill`, which is deferred
- `dead_query: Query<(), With<Dead>>` in `check_lock_release` reads the already-flushed `Dead` component from prior ticks; same-tick kills are caught via `all_entities.contains(*adj)` (entity still alive) vs. the `Destroyed<Cell>` message count
- No false-positive from `Dead` not yet visible within the same tick: the system explicitly checks `!all_entities.contains(*adj) || dead_query.contains(*adj)` — covers both despawned-prior-tick and dead-this-tick via the message count gate
- The `ApplyDeferred` that `handle_kill`'s `Commands` emits runs before `check_lock_release` sees its results; ordering is correct

**`Locks(pub Vec<Entity>)` per-tick iteration**
- `locks.0.iter().all(...)` walks the adjacency list per locked cell per tick when `destroyed_count > 0`
- Gated by `destroyed_count > 0` — only runs when a cell actually died this tick
- Adjacent count per lock cell is small (grid neighbors, typically 2–6)
- Not a hot path at 50–200 cells

**`Changed<Hp>` in `update_cell_damage_visuals`**
- False-positive risk: any system that writes `Hp` mutably (even without changing value) would mark `Changed`
- `apply_damage` only writes `Hp` when a `DamageDealt<Cell>` message exists, so false positives require a rogue `&mut Hp` borrow elsewhere
- No current producers of spurious `Hp` mutations observed
- Acceptable; note if regen system or future buffs add unconditional `&mut Hp` writes

**`KillYourself<T>` PhantomData<T>**
- `PhantomData<T>` is a ZST — zero runtime cost for the field itself
- Manual `Clone` impl confirmed; no derive overhead from T: Clone constraint
- Confirmed zero-cost at current bolt scale (1–few)

**Why:** All patterns are correct and acceptable at current entity scale (1 Breaker, 1–few Bolts, 50–200 Cells).
**How to apply:** Do not re-flag in future reviews unless entity counts grow significantly or new systems add spurious `&mut Hp` borrows.
