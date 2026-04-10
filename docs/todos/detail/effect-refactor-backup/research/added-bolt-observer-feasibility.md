# Research: Added<T> Query vs Observer Feasibility for New Bolt/Cell/Wall Detection

Bevy version: **0.18.1** (from `breaker-game/Cargo.toml`).

---

## Q1 — How are bolts spawned?

There are two distinct spawning contexts with different mechanics.

### A. Commands-based spawn (normal gameplay path)

`Bolt::builder()...spawn(&mut commands)` is the canonical API. The implementation in
`breaker-game/src/bolt/builder/core/terminal.rs` calls `commands.spawn(core)` and then
one or more `entity.insert(...)` calls on the returned `EntityCommands`. The core bundle
(`Bolt`, `Velocity2D`, `CollisionLayers`, all `Spatial` components, all radius components)
is submitted as a single bundle in the first `commands.spawn()` call. Optional components
(`PrimaryBolt`/`ExtraBolt`, `CleanupOnRunEnd`/`CleanupOnNodeExit`, `BoltDefinitionRef`,
`BoltBaseDamage`, `BoltAngleSpread`, `BoltSpawnOffsetY`, `BoundEffects`, etc.) are inserted
via subsequent `entity.insert()` calls on the same `EntityCommands` handle, meaning they are
all part of the same deferred command queue flush.

For **Rendered** bolt variants (`...rendered(...)...spawn(&mut commands)`), there is a second
`commands.entity(entity).insert(...)` call after `spawn_inner` returns, adding `Mesh2d`,
`MeshMaterial2d`, and `GameDrawLayer::Bolt`. This is still deferred — it goes into the same
`Commands` queue and is flushed atomically with everything else.

**Spawn sites that use this Commands path:**
- `setup_run` (`OnEnter(NodeState::Loading)`) — spawns the primary bolt
- `reset_bolt` (`OnEnter(NodeState::Loading)`) — respawns the bolt between nodes
- All test helpers in `bolt/builder/tests/`, `bolt/systems/*/tests/helpers.rs`, etc.

### B. World-direct spawn (effect `fire()` paths)

Effects that spawn bolts take `world: &mut World` as their parameter. They use a local
`CommandQueue::default()` → `Commands::new(&mut queue, world)` → `queue.apply(world)` pattern.
This applies the spawn immediately within the same `fire()` call. After `queue.apply(world)` the
entity exists in the world and subsequent `world.entity_mut(entity).insert(...)` calls on the
same line operate on a live entity.

**Effect fire() paths that spawn bolts using this pattern:**
- `spawn_bolts/effect.rs` — spawns N extra bolts
- `chain_bolt/effect.rs` — spawns one tethered extra bolt
- `spawn_phantom/effect.rs` — spawns phantom extra bolts
- `mirror_protocol/effect.rs` — spawns a mirrored extra bolt
- `tether_beam/effect.rs` — spawns two tether bolts via `spawn_tether_bolt()` helper
- `second_wind/system.rs` — spawns a Wall (not a bolt), but uses `world.spawn(...)` directly

**Critical nuance:** `commands.fire_effect(entity, effect, chip)` is itself a deferred
command (`FireEffectCommand` in `effect/commands/ext.rs`). When a trigger bridge system like
`bridge_node_start` calls `commands.fire_effect(...)`, the actual `effect.fire(&mut world)`
call happens at command flush time (end of system or next sync point). At that point the
effect `fire()` function receives `&mut World` and the `queue.apply(world)` inside it makes
the entity immediately live in the world — but this is still deferred relative to the
triggering system's frame of execution.

---

## Q2 — When does Added<Bolt> fire?

`Added<T>` in a Bevy 0.18 query filter tracks entities that had component `T` added since
the **last time the system ran**. The change-tick comparison is against the system's own
last-run tick, not the frame boundary.

### For Commands-based spawns (setup_run, reset_bolt)

`commands.spawn(...)` is deferred. Commands are applied at the next `apply_deferred` sync
point after the spawning system completes. A system querying `Added<Bolt>` that runs in
the **same schedule flush** as the command application will see the new entity. A system
that ran before the sync point will not.

In practice: setup_run runs in `OnEnter(NodeState::Loading)`. `dispatch_bolt_effects` runs
in `FixedUpdate` (no state guard for `NodeState::Loading`). There is a frame boundary
between them, so the new bolt's `Added<BoltDefinitionRef>` is visible to `dispatch_bolt_effects`
on the NEXT frame it runs after the bolt entity is materialized.

### For effect World-direct spawns (spawn_bolts, chain_bolt, etc.)

`queue.apply(world)` inside `fire()` materializes the entity immediately into the world.
However, the `Added<T>` change-tick for that component is set to the **current world tick**
at the time of insertion. A system that already ran this frame (same world tick range) will
NOT see `Added<T>` because its "last run tick" is already >= the component's added tick.
Systems that run **after** the command application point in the same frame, or on the next
frame, will see `Added<T>`.

The existing test in `dispatch_bolt_effects/tests/basic_dispatch.rs` (lines 447–494) confirms
this. It uses `world_mut().spawn(...)` (immediate) and requires `app.update()` (a full
schedule flush) before `Added<BoltDefinitionRef>` is detected by `dispatch_bolt_effects`.
This is consistent with Bevy 0.18's change-detection model: change ticks are compared
against the system's last-run tick, not an instantaneous check.

**Summary:** Whether spawned via Commands or immediate world insertion, `Added<Bolt>` (or
`Added<BoltDefinitionRef>`) fires in the **first system run after the tick when the
component was inserted**. There is always at least one scheduling hop.

---

## Q3 — What components does a bolt have when Added<Bolt> fires?

All components inserted via a single `spawn_inner()` call are delivered atomically when
commands are flushed. Because Bevy 0.18's command queue applies all insertions in order
before the next system observing the entity runs, by the time `Added<Bolt>` is seen by
any query:

**Always present (inserted in `build_core` / `spawn_inner` core bundle):**
- `Bolt` (the marker)
- `Velocity2D`
- `CollisionLayers`
- All `Spatial` components: `Position2D`, `GlobalPosition2D`, `Rotation2D`, `BaseSpeed`,
  `MinSpeed`, `MaxSpeed`, `MinAngleH`, `MinAngleV`, `PreviousPosition2D`
- `Scale2D`, `PreviousScale`, `Aabb2D`, `BaseRadius`

**Present when `.definition()` was called (optional block in `spawn_inner`):**
- `BoltBaseDamage`, `BoltDefinitionRef`, `BoltAngleSpread`, `BoltSpawnOffsetY`
- Optionally `MinRadius`, `MaxRadius`

**Present based on role:**
- `PrimaryBolt` + `CleanupOnRunEnd` (primary), or `ExtraBolt` + `CleanupOnNodeExit` (extra)
- `BoltServing` (if `.serving()` was called)

**Present if `.with_effects()` or inherited effects exist:**
- `BoundEffects`

**Exception — Rendered bolt:** For rendered `.spawn()` calls, `Mesh2d`,
`MeshMaterial2d`, and `GameDrawLayer::Bolt` are inserted via a SECOND
`commands.entity(entity).insert(...)` call queued after `spawn_inner`. These two
insertions are both in the same `Commands` queue and are applied atomically at the same
flush. So by the time any query sees `Added<Bolt>`, the visual components are also present.

**Exception — Effect-spawned bolts:** Effect `fire()` paths use `queue.apply(world)`
immediately, then call `super::super::insert_bolt_visuals(world, entity, visual)` which
inserts visuals directly via `world.entity_mut(entity).insert(...)`. Then further
`world.entity_mut(bolt_entity).insert(BoltLifespan(...))` calls follow. All of these happen
synchronously within the same `fire()` invocation, before control returns to the caller.
So by the time any system observing `Added<Bolt>` runs (next scheduling hop), all
components that the effect added are already present.

**Bottom line:** When `Added<Bolt>` fires in a system, the entity has its complete
component set — there are no multi-step deferred insertions that would leave a bolt in a
partially-constructed state.

The one exception worth noting: `NodeScalingFactor` is applied by `apply_node_scale_to_bolt`
(`OnEnter(NodeState::Loading)`) for the initial bolt, and by `apply_node_scale_to_late_bolts`
(`FixedUpdate`) for effect-spawned bolts. This component arrives one frame after bolt spawn
for effect-spawned bolts. A system using `Added<Bolt>` that reads `NodeScalingFactor` would
need to handle this one-frame delay or use a `Without<NodeScalingFactor>` guard to defer
until it's available.

---

## Q4 — Existing Added<T> patterns in the codebase

Searching all `.rs` files under `breaker-game/src/` for `Added<`:

**Only one occurrence:**
```
breaker-game/src/bolt/systems/dispatch_bolt_effects/system.rs:28
    new_bolts: Query<(Entity, &BoltDefinitionRef), Added<BoltDefinitionRef>>,
```

The `dispatch_bolt_effects` system uses `Added<BoltDefinitionRef>` (not `Added<Bolt>`)
because the bolt definition lookup depends on `BoltDefinitionRef`. Effect-spawned bolts
always have `BoltDefinitionRef` (via `.definition()`) so this works for all bolt types.

There are no `Added<Cell>`, `Added<Wall>`, or `Added<Breaker>` patterns in the codebase.

The `dispatch_cell_effects` system in
`state/run/node/systems/dispatch_cell_effects/system.rs` uses a **different pattern**:
a `Without<CellEffectsDispatched>` marker instead of `Added<Cell>`. The query is:
```rust
Query<(Entity, &CellTypeAlias), (With<Cell>, Without<CellEffectsDispatched>)>
```
After processing, it inserts `CellEffectsDispatched` to prevent re-dispatch. This is
equivalent to `Added<Cell>` behavior but is idempotent across frames.

---

## Q5 — Bevy 0.18 observer alternative (.observe())

**No observers are used anywhere in the codebase.** A search for `.observe(`, `On<Add`,
and `Trigger<` across all `.rs` files returns zero matches.

In Bevy 0.18, entity observers use `.observe(on_add::<Bolt>)` on an `EntityCommands`
or `World`, with `On<Add, Bolt>` as the trigger type. This would allow truly reactive
detection of component additions without the one-frame delay.

**Feasibility for this codebase:** The observer API is available in Bevy 0.18.1, but its
use would be a new pattern not yet established in this project. Key constraints:

1. Effect `fire()` functions take `&mut World` and spawn entities via local CommandQueues.
   An observer registered via `app.observe(...)` (global observer) would fire during
   `queue.apply(world)` at command flush time. This means the observer fires within the
   same `fire()` call, with the world in an exclusive-access state. The observer would see
   the entity but would need to accept a `Trigger<Add, Bolt>` parameter — it cannot call
   `commands.fire_effect(...)` (no `Commands` available in a world-exclusive context).
   It would need to operate directly on `&mut World`.

2. For the `AllBoltsEffects` resource approach described in the todo doc, an `Added<Bolt>`
   query (next-frame detection) is actually sufficient and simpler than observers. The
   resource is written during `dispatch_chip_effects` (which runs in `FixedUpdate`), and
   the stamping system can use `Added<BoltDefinitionRef>` to catch both the initial bolt
   and all effect-spawned bolts on the next frame. The existing `dispatch_bolt_effects`
   system already proves this pattern works correctly.

3. The one-frame delay is unlikely to be observable to the player because effect-spawned
   bolts are live and bouncing for many frames before any trigger that would read their
   `BoundEffects` fires.

**Recommendation:** Use `Added<BoltDefinitionRef>` (same as `dispatch_bolt_effects`) rather
than `Added<Bolt>`, since all spawning paths that set up a complete physics bolt also set
`BoltDefinitionRef`. Using the same discriminating component maintains consistency with the
existing pattern and avoids detecting headless test bolts that lack `BoltDefinitionRef`.

---

## Summary Table

| Spawn path | Mechanism | Entity live by | Added<Bolt> visible |
|---|---|---|---|
| `setup_run` (Commands) | `commands.spawn()` | Next `apply_deferred` | Next system run after flush |
| `reset_bolt` (Commands) | `commands.spawn()` | Next `apply_deferred` | Next system run after flush |
| Effect fire() paths (world-direct) | `queue.apply(world)` immediately | Same `fire()` call | Next system run in same or later frame |
| Observer `On<Add, Bolt>` | Not used | — | Immediate (within command flush) |

| Component | Present when Added<Bolt> fires? |
|---|---|
| `Bolt`, `Velocity2D`, `CollisionLayers`, Spatial components, radius components | Always |
| `BoltDefinitionRef`, `BoltBaseDamage`, `BoltAngleSpread` | Yes (if `.definition()` was called) |
| `PrimaryBolt`/`ExtraBolt`, `CleanupOnRunEnd`/`CleanupOnNodeExit` | Always |
| `BoundEffects` | Only if effect nodes were attached at build time |
| `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Bolt` | Yes (same flush, both Commands and world-direct) |
| `NodeScalingFactor` | No — added 1 frame later by `apply_node_scale_to_late_bolts` |

---

## Key Files

- `breaker-game/src/bolt/builder/core/terminal.rs` — all `spawn()` implementations; shows the
  Commands-based multi-insert pattern and the Rendered vs Headless split
- `breaker-game/src/bolt/systems/dispatch_bolt_effects/system.rs` — the one existing
  `Added<BoltDefinitionRef>` consumer; the established pattern to follow
- `breaker-game/src/bolt/systems/dispatch_bolt_effects/tests/basic_dispatch.rs` lines 447–494 —
  empirical test confirming one-frame delay before `Added<BoltDefinitionRef>` is seen
- `breaker-game/src/effect/commands/ext.rs` — `FireEffectCommand::apply()` shows that
  `commands.fire_effect()` is a deferred `Command`; `fire()` is called with `&mut World` at flush
- `breaker-game/src/effect/effects/spawn_bolts/effect.rs` — canonical effect spawn path:
  `CommandQueue::default()` → `queue.apply(world)` → immediate `world.entity_mut().insert()`
- `breaker-game/src/state/run/node/systems/dispatch_cell_effects/system.rs` — alternate
  new-entity detection pattern using `Without<CellEffectsDispatched>` marker
- `breaker-game/src/state/run/node/systems/apply_node_scale_to_bolt.rs` — `apply_node_scale_to_late_bolts`
  shows the `Without<NodeScalingFactor>` catch-up pattern for mid-gameplay spawns
