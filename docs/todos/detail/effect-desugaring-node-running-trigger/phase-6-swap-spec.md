# Phase 6: Swap Spec

**What this is:** The complete list of changes required to swap the old `src/effect/` with `src/new_effect/` and unify damage/death messaging. An implementing agent with zero context should be able to read this document and know exactly what to change.

**Prerequisite:** `src/new_effect/` is fully built and tested (Phases 1-5 complete). The old `src/effect/` is untouched and still working.

---

## Step 1: Delete `src/effect/`

Delete the entire `breaker-game/src/effect/` directory. Everything in it is replaced by `src/new_effect/`. This includes:
- All effect implementations (`effects/shockwave/`, `effects/explode/`, etc.)
- All trigger bridges (`triggers/died.rs`, `triggers/death.rs`, `triggers/cell_destroyed.rs`, etc.)
- `BoundEffects`, `StagedEffects` components (old shape)
- `EffectCommandsExt` command extensions
- `EffectSystems` set definitions
- `EffectPlugin` registration

## Step 2: Rename `src/new_effect/` → `src/effect/`

Rename the directory and update the `mod` declaration in `lib.rs`.

## Step 3: Migrate RON files

Copy migrated RON files from `docs/todos/detail/effect-desugaring-node-running-trigger/ron-migration/` to their asset locations:

| Source | Destination |
|---|---|
| `ron-migration/breakers/*.breaker.ron` | `breaker-game/assets/breakers/` |
| `ron-migration/standard/*.chip.ron` | `breaker-game/assets/chips/standard/` |
| `ron-migration/evolutions/*.evolution.ron` | `breaker-game/assets/chips/evolutions/` |

55 files total. Overwrite the originals.

## Step 4: Update plugin registration

In `breaker-game/src/lib.rs` (or `game.rs`):
- Remove old `EffectPlugin` registration
- Add new `NewEffectPlugin` (renamed to `EffectPlugin` after Step 2)
- Register new message types: `DamageDealt<T>`, `KillYourself<Cell>`, `KillYourself<Bolt>`, `KillYourself<Cell>`, `KillYourself<Cell>`, `KillYourself<Wall>`, `KillYourself<Wall>`, `Destroyed<Cell>`, `Destroyed<Bolt>`, etc.
- Remove old message registrations: `DamageCell`, `RequestCellDestroyed`, `CellDestroyedAt`, `RequestBoltDestroyed`

---

## Step 5: Update domain systems (outside `src/effect/`)

### 5a. Cell domain (`src/cells/`)

**Messages to remove** (defined in `cells/messages.rs`):
- `DamageCell { cell: Entity, damage: f32, source_chip: Option<String> }` → replaced by `DamageDealt<T>`
- `RequestCellDestroyed { cell: Entity, was_required_to_clear: bool }` → replaced by `KillYourself<Cell>`
- `CellDestroyedAt { was_required_to_clear: bool }` → replaced by `Destroyed<Cell>`

**`handle_cell_hit`** (`cells/systems/handle_cell_hit/system.rs`):
- Currently reads `MessageReader<DamageCell>`, calls `health.take_damage()`, sends `RequestCellDestroyed`
- **Replace with:** reads `MessageReader<DamageDealt<T>>` (filtered for cell targets), calls `health.take_damage()`, sets `KilledBy { dealer }` on the killing blow only (when HP crosses zero). Does NOT send any death request — the new `detect_deaths` system in `new_effect/damage/` handles that.
- Remove the same-frame dedup guard (`Local<Vec<Entity>>`) — `detect_deaths` handles dedup via `Changed<Hp>` + `hp.current <= 0` check.
- Remove the `Locked` guard — move to the new `apply_damage` system or keep as a filter.

**`cleanup_cell`** (`cells/systems/cleanup_cell.rs`):
- Currently reads `RequestCellDestroyed`, despawns entity, sends `CellDestroyedAt`
- **Replace with:** reads `Destroyed<Cell>` from the unified death pipeline. The domain handler for cells (registered in `new_effect/damage/`) processes `KillYourself<Cell>`, checks shields/invuln/Locked, and sends `Destroyed<Cell>`. `cleanup_cell` reads `Destroyed<Cell>`, does domain-specific cleanup (if any), and marks entity with `DespawnEntity`.
- **Remove** `CellDestroyedAt` emission — replaced by `Destroyed<Cell>`.

**`tick_cell_regen`** (`cells/systems/tick_cell_regen.rs`):
- No change needed. Still mutates `CellHealth.current` directly. Regen is not damage.

**`check_lock_release`** (`cells/systems/check_lock_release/system.rs`):
- Currently reads `CellDestroyedAt`
- **Replace with:** reads `Destroyed<Cell>` (or a more specific post-death message if needed). The cell entity may still exist at this point (DespawnEntity is deferred).

**Plugin** (`cells/plugin.rs`):
- Remove `DamageCell`, `RequestCellDestroyed`, `CellDestroyedAt` message registrations
- Remove `handle_cell_hit` system (replaced by `apply_damage` in new_effect)
- Update `cleanup_cell` to read new message types
- Update system ordering: cell domain systems run after `new_effect` damage/death systems

### 5b. Bolt domain (`src/bolt/`)

**Messages to remove** (defined in `bolt/messages.rs`):
- `RequestBoltDestroyed { bolt: Entity }` → replaced by `KillYourself<Bolt>`

**`bolt_cell_collision`** (`bolt/systems/bolt_cell_collision/system.rs`):
- Currently computes `effective_damage` and sends `DamageCell`
- **Replace with:** sends `DamageDealt<T> { dealer: Some(bolt_entity), target: cell_entity, amount: effective_damage }`
- Damage calculation (BoltBaseDamage * ActiveDamageBoosts * ActiveVulnerability) stays the same — just the message type changes.

**`bolt_lost`** (`bolt/systems/bolt_lost/system.rs`):
- Currently sends `RequestBoltDestroyed` for extra bolts, respawns primary bolts
- **Replace with:** sends `KillYourself<Bolt> { victim: bolt_entity, killer: None }` for extra bolts. Primary bolt respawn logic unchanged.

**`tick_bolt_lifespan`** (`bolt/systems/tick_bolt_lifespan.rs`):
- Currently sends `RequestBoltDestroyed` when lifespan expires
- **Replace with:** sends `KillYourself<Bolt> { victim: bolt_entity, killer: None }`

**`cleanup_destroyed_bolts`** (`bolt/systems/cleanup_destroyed_bolts.rs`):
- Currently reads `RequestBoltDestroyed`, despawns entity
- **Replace with:** reads `Destroyed<Bolt>`, marks with `DespawnEntity`

**Plugin** (`bolt/plugin.rs`):
- Remove `RequestBoltDestroyed` message registration
- Update system ordering

### 5c. State/tracking domain (`src/state/`)

**`track_node_completion`** (`state/run/node/systems/track_node_completion.rs`):
- Currently reads `CellDestroyedAt`
- **Replace with:** reads `Destroyed<Cell>` — decrements `ClearRemainingCount`, sends `NodeCleared` when zero. Needs `was_required_to_clear` — this field must be on `Destroyed<Cell>` or queried from the entity (if still alive via DespawnEntity).

**`track_cells_destroyed`** (`state/run/node/tracking/systems/track_cells_destroyed.rs`):
- Currently reads `CellDestroyedAt`
- **Replace with:** reads `Destroyed<Cell>` — increments `RunStats.cells_destroyed`

**`track_evolution_damage`** (`state/run/node/tracking/systems/track_evolution_damage.rs`):
- Currently reads `DamageCell` for per-chip damage tracking
- **Replace with:** reads `DamageDealt<T>`. The `source_chip` field needs to be available — either on `DamageDealt<T>` directly or via a lookup from the TriggerContext/SourceId.

### 5d. Effect implementations that send DamageCell

All of these are inside `src/effect/effects/` and will be **rewritten in `src/new_effect/effects/`**. They are listed here for completeness — the new implementations send `DamageDealt<T>` instead of `DamageCell`.

| Effect | Old sender | New sender |
|---|---|---|
| Shockwave | `apply_shockwave_damage` sends `DamageCell` | sends `DamageDealt<T> { dealer: context.bolt(), ... }` |
| Explode | `process_explode_requests` sends `DamageCell` | sends `DamageDealt<T> { dealer: context.killer(), ... }` |
| PiercingBeam | `process_piercing_beam` sends `DamageCell` | sends `DamageDealt<T> { dealer: context.bolt(), ... }` |
| Pulse | `apply_pulse_damage` sends `DamageCell` | sends `DamageDealt<T> { dealer: context.bolt(), ... }` |
| TetherBeam | `tick_tether_beam` sends `DamageCell` | sends `DamageDealt<T> { dealer: context.bolt(), ... }` |
| ChainLightning | `tick_chain_lightning` sends `DamageCell` | sends `DamageDealt<T> { dealer: context.bolt(), ... }` |

### 5e. Trigger bridges to remove

These are inside `src/effect/triggers/` and will be **deleted with `src/effect/`**. The new equivalents live in `src/new_effect/dispatch/`. Listed for cross-reference:

| Old bridge | Old trigger | New bridge | New trigger(s) |
|---|---|---|---|
| `bridge_died` | `Trigger::Died` (targeted, on dying entity) | `bridge_destroyed` | `Died` (on victim) + `Killed(KillTarget)` (on killer) |
| `bridge_death` | `Trigger::Death` (global) | `bridge_destroyed` | `DeathOccurred(DeathTarget)` (global) |
| `bridge_cell_destroyed` | `Trigger::CellDestroyed` (global, post-despawn) | **Removed** | Absorbed into `bridge_destroyed` — `DeathOccurred(Cell)` fires while entity is still alive (DespawnEntity) |

**Key change:** The old `bridge_cell_destroyed` fired AFTER the entity was despawned (it read `CellDestroyedAt`). The new `bridge_destroyed` fires BEFORE despawn (it reads `Destroyed<T>` while the entity is in DespawnEntity state). Effects that need the dying entity's data can now access it.

---

## Step 6: `DespawnEntity` system

New unified despawn system. Entities marked with `DespawnEntity` are despawned at the end of the frame (or in a dedicated cleanup system after all triggers have been evaluated).

```rust
#[derive(Component)]
struct DespawnEntity;

fn despawn_pending(
    query: Query<Entity, With<DespawnEntity>>,
    mut commands: Commands,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

Runs AFTER all trigger evaluation, AFTER all bridge systems, AFTER domain-specific cleanup. This replaces:
- `cleanup_cell` despawn call
- `cleanup_destroyed_bolts` despawn call
- Any other gameplay entity despawn triggered by death

Effect lifecycle despawns (shockwave finished, pulse ring finished, etc.) are NOT affected — those are internal to effect systems and use their own cleanup.

---

## Step 7: System ordering

The new ordering within FixedUpdate:

```
1. Physics / collision systems (bolt_cell_collision, etc.)
     → sends DamageDealt<T>

2. apply_damage (new_effect/damage/)
     → reads DamageDealt<T>, decrements HP, sets KilledBy on killing blow

3. detect_deaths (new_effect/damage/)
     → queries Changed<Hp> where hp <= 0
     → sends KillYourself<T>

4. Domain kill handlers (cells, bolt, wall)
     → reads KillYourself<T>
     → checks shields/invuln/Locked
     → sends Destroyed<T>
     → inserts DespawnEntity

5. bridge_destroyed (new_effect/dispatch/)
     → reads Destroyed<T>
     → fires Died on victim, Killed on killer (if alive), DeathOccurred globally
     → entity still alive (DespawnEntity, not yet despawned)

6. Effect evaluation (walk_effects)
     → Died/Killed/DeathOccurred triggers fire here
     → effects can access dying entity's components

7. Domain post-death systems
     → track_node_completion reads Destroyed<Cell>
     → track_cells_destroyed reads Destroyed<Cell>
     → check_lock_release reads Destroyed<Cell>

8. despawn_pending
     → despawns all entities with DespawnEntity
```

---

## Step 8: DamageDealt<T> fields

```rust
struct DamageDealt<T> {
    pub dealer: Option<Entity>,     // who caused this (propagated through effect chains)
    pub target: Entity,             // who takes damage
    pub amount: f32,                // pre-calculated damage amount
    pub source_chip: Option<String>, // for evolution damage tracking (track_evolution_damage)
}
```

`source_chip` is carried forward from the old `DamageCell.source_chip` field. Needed by `track_evolution_damage` to attribute damage to specific chips for highlight/evolution tracking.

---

## Step 9: Verify and clean up

After the swap:
1. `cargo dcheck` — no compilation errors
2. `cargo dclippy` — no warnings
3. `cargo dtest` — all tests pass (old effect tests are deleted, new tests from Phases 1-5 cover the new system)
4. `cargo scenario -- --all` — all scenarios pass
5. Remove old message types from the codebase: grep for `DamageCell`, `RequestCellDestroyed`, `CellDestroyedAt`, `RequestBoltDestroyed` — should find zero references
6. Remove the migration RON files from `docs/todos/detail/` (they're now in the asset directories)

---

## Summary of message replacements

| Old message | Defined in | New message | Defined in |
|---|---|---|---|
| `DamageCell` | `cells/messages.rs` | `DamageDealt<T>` | `new_effect/damage/` |
| `RequestCellDestroyed` | `cells/messages.rs` | `KillYourself<Cell>` | `new_effect/damage/` |
| `CellDestroyedAt` | `cells/messages.rs` | `Destroyed<Cell>` | `new_effect/damage/` |
| `RequestBoltDestroyed` | `bolt/messages.rs` | `KillYourself<Bolt>` | `new_effect/damage/` |

## Summary of system replacements

| Old system | Domain | Replaced by | Domain |
|---|---|---|---|
| `handle_cell_hit` | cells | `apply_damage` | new_effect/damage |
| `cleanup_cell` (despawn part) | cells | `despawn_pending` | new_effect/damage |
| `cleanup_destroyed_bolts` (despawn part) | bolt | `despawn_pending` | new_effect/damage |
| `bridge_died` | effect/triggers | `bridge_destroyed` | new_effect/dispatch |
| `bridge_death` | effect/triggers | `bridge_destroyed` | new_effect/dispatch |
| `bridge_cell_destroyed` | effect/triggers | **removed** (absorbed into `bridge_destroyed`) | — |

## Systems that need message type updates only (no logic change)

| System | File | Old message | New message |
|---|---|---|---|
| `bolt_cell_collision` | `bolt/systems/bolt_cell_collision/system.rs` | `DamageCell` | `DamageDealt<T>` |
| `bolt_lost` | `bolt/systems/bolt_lost/system.rs` | `RequestBoltDestroyed` | `KillYourself<Bolt>` |
| `tick_bolt_lifespan` | `bolt/systems/tick_bolt_lifespan.rs` | `RequestBoltDestroyed` | `KillYourself<Bolt>` |
| `track_node_completion` | `state/run/node/systems/track_node_completion.rs` | `CellDestroyedAt` | `Destroyed<Cell>` |
| `track_cells_destroyed` | `state/run/node/tracking/systems/track_cells_destroyed.rs` | `CellDestroyedAt` | `Destroyed<Cell>` |
| `track_evolution_damage` | `state/run/node/tracking/systems/track_evolution_damage.rs` | `DamageCell` | `DamageDealt<T>` |
| `check_lock_release` | `cells/systems/check_lock_release/system.rs` | `CellDestroyedAt` | `Destroyed<Cell>` |

## Edge cases to handle during swap

1. **`was_required_to_clear`**: Currently on `RequestCellDestroyed` and `CellDestroyedAt`. Needs to be on `Destroyed<Cell>` (or queried from the still-alive DespawnEntity entity). Used by `track_node_completion` and `check_lock_release`.

2. **`Locked` cell immunity**: Currently checked in `handle_cell_hit`. Needs to move to the cell domain's `KillYourself` handler (or the new `apply_damage` filters it).

3. **`RampingDamageState` gap**: Accumulated but never read. Document as known gap — not a swap concern.

4. **`chain_lightning` direct world resource access**: Currently writes `DamageCell` via `world.resource_mut::<Messages<DamageCell>>()` in `fire()`. New implementation must use `DamageDealt<T>` via the same pattern or restructure to use `MessageWriter`.

5. **`track_evolution_damage` needs `source_chip`**: The `DamageDealt<T>` must carry `source_chip: Option<String>` to preserve this functionality.
