# Plugin Organization Patterns

Research for the Protocol and Hazard plugin architecture decision.

Bevy version: **0.18.1**

---

## 1. How Domain Plugins Are Structured

Every domain follows the same top-level layout. The module tree for `bolt/` is the clearest template:

```
bolt/
  builder/           # typestate builder for spawning
  components/        # all ECS components owned by this domain
  definition.rs      # RON-loaded data definition type
  filters.rs         # query filter aliases
  messages.rs        # all messages sent/received by this domain
  mod.rs             # pub re-exports only; no production logic
  plugin.rs          # Plugin impl; the only place `add_systems` is called
  queries.rs         # named query type aliases
  registry/          # typed asset registry if needed
  resources.rs       # resources owned by this domain
  sets.rs            # SystemSet enum exported for cross-domain ordering
  systems/           # one subdirectory or file per system function
```

`cells/` and `breaker/` follow this same pattern, differing only in which subdirectories they need. `chips/` is a smaller domain and omits `sets.rs` and `registry/` since it has no cross-domain ordering requirements of its own.

`effect/` uses a two-level variant because it is architecturally a meta-domain:

```
effect/
  commands/          # EffectCommandsExt (Commands extension trait)
  core/              # all effect enums + component types (EffectKind, Trigger, etc.)
  effects/           # per-effect modules (22 effects)
    shockwave/
      mod.rs         # re-exports + #[cfg(test)] mod tests
      effect.rs      # fire(), reverse(), components, runtime systems, register()
      tests/
    speed_boost.rs   # simpler effects fit in one file
    ...
  triggers/          # per-trigger bridge systems (20 triggers)
    bump/
    cell_destroyed.rs
    ...
  mod.rs
  plugin.rs          # delegates to effects::register() and triggers::register()
  sets.rs            # EffectSystems::Bridge
```

The `effect/` domain is exceptional in its size — 279 Rust files total. Every other domain is far smaller (bolt: 106, breaker: 88, cells: 42, chips: 63).

### The `mod.rs` contract

`mod.rs` is wiring-only in every domain. It declares submodules and re-exports public items. It never contains production logic or tests. Example from `bolt/mod.rs`:

```rust
pub(crate) mod components;
pub mod definition;
// ...
pub(crate) use definition::BoltDefinition;
pub use plugin::BoltPlugin;
pub use sets::BoltSystems;
```

---

## 2. What a `build()` Looks Like

All domain plugins follow this exact structure inside `build()`:

1. Register messages (`add_message::<T>()`) — each domain registers the messages it _owns_ (sends)
2. Register resources (`init_resource::<T>()` or `insert_resource(...)`)
3. Add systems per schedule with ordering constraints

Concrete example from `BoltPlugin::build()`:

```rust
fn build(&self, app: &mut App) {
    app.init_resource::<GameRng>()
        .add_message::<BoltSpawned>()
        .add_message::<BoltImpactCell>()
        // ...
        .add_systems(
            OnEnter(NodeState::Loading),
            (apply_node_scale_to_bolt.after(NodeSystems::Spawn), reset_bolt.in_set(BoltSystems::Reset)),
        )
        .add_systems(OnEnter(NodeState::AnimateIn), begin_node_birthing)
        .add_systems(
            FixedUpdate,
            (
                launch_bolt,
                bolt_cell_collision
                    .after(normalize_bolt_speed_after_constraints)
                    .in_set(BoltSystems::CellCollision),
                // ... 10+ more systems
            )
                .run_if(in_state(NodeState::Playing)),
        )
        .add_systems(Update, sync_bolt_scale.run_if(in_state(NodeState::Playing)));
}
```

`EffectPlugin::build()` is the outlier — it delegates entirely:

```rust
fn build(&self, app: &mut App) {
    super::effects::register(app);
    super::triggers::register(app);
}
```

Each effect module owns its own `register(app)` function. Each trigger module owns its own `register(app)` function. The plugin's `build()` only chains those calls together. This is the delegation pattern.

Every plugin has a `plugin_builds` unit test using `App::new().add_plugins(MinimalPlugins)` that verifies the plugin doesn't panic during `build()`.

---

## 3. Cross-Domain Communication: Messages

All cross-domain communication goes through Bevy 0.18 `Message` + `MessageWriter<T>` / `MessageReader<T>`. There are no Bevy 0.15-style `Event<T>` uses. There are no `add_observer` / `Observer` uses anywhere in the game crate.

### Message ownership rule

A domain _registers_ the messages it _produces_. Readers do not register the message type — they rely on the producer having registered it first. The plugin registration order in `game.rs` enforces this.

### Cross-domain message catalog

| Message | Registered by | Primary consumers |
|---|---|---|
| `BoltSpawned` | bolt | node (spawn coordinator) |
| `BoltImpactBreaker` | bolt | breaker (grade_bump) |
| `BoltImpactCell` | bolt | chips, cells, audio |
| `BoltLost` | bolt | breaker |
| `BoltImpactWall` | bolt | effect (bridge_wall_impact) |
| `RequestBoltDestroyed` | bolt | effect (bridge_bolt_death), bolt (cleanup) |
| `BumpPerformed` | breaker | audio, chips, effect triggers, UI |
| `BumpWhiffed` | breaker | UI |
| `BreakerSpawned` | breaker | node (spawn coordinator) |
| `BreakerImpactCell` | breaker | effect (bridge_cell_impact) |
| `BreakerImpactWall` | breaker | effect (bridge_wall_impact) |
| `RequestCellDestroyed` | cells | effect (bridge_death), cells (cleanup) |
| `CellDestroyedAt` | cells | node (run tracking, lock release) |
| `DamageCell` | cells | cells (handle_cell_hit) |
| `CellImpactWall` | cells | effect (bridge_wall_impact) |
| `ChipSelected` | node (UI) | chips (dispatch_chip_effects) |
| `NodeCleared` | node | run lifecycle |
| `TimerExpired` | node | run lifecycle |
| `ApplyTimePenalty` | node | node (apply_time_penalty) |
| `ReverseTimePenalty` | node | node (reverse_time_penalty) |

**Cross-domain message flow pattern**: bolt domain detects collisions → sends `BoltImpactCell` → cells domain handles damage via `DamageCell` → sends `RequestCellDestroyed` → effect domain evaluates `OnDeath` chains → sends `CellDestroyedAt` → node domain tracks completion.

### The `prelude/` module

Widely-consumed cross-domain types are re-exported from `src/prelude/`. Using `use crate::prelude::*` imports the most common messages, components, and state types. This is how systems in one domain reference types from another without reaching into that domain's internal paths.

---

## 4. Scheduling Patterns

### Schedules in use

| Schedule | Used for |
|---|---|
| `OnEnter(NodeState::Loading)` | Spawn entities, initialize resources for a node |
| `OnEnter(NodeState::AnimateIn)` | Trigger animation-in logic |
| `FixedUpdate` | All physics, collision, gameplay logic |
| `Update` | Visual sync (scale, animation, HUD display) |
| `Startup` | Camera spawn only |

**The dominant pattern**: game logic goes in `FixedUpdate` with `.run_if(in_state(NodeState::Playing))`. Visual synchronization goes in `Update` with the same guard. State initialization goes in `OnEnter(...)`.

### System sets

System sets are defined in `sets.rs` and are used exclusively for cross-domain ordering. They are not used for intra-domain ordering (intra-domain ordering uses `.before()`/`.after()` with direct system references).

| Domain | Set enum | Variants |
|---|---|---|
| bolt | `BoltSystems` | Reset, CellCollision, WallCollision, BreakerCollision, BoltLost |
| breaker | `BreakerSystems` | Move, Reset, GradeBump, UpdateState |
| effect | `EffectSystems` | Bridge |
| node (state) | `NodeSystems` | Spawn, TrackCompletion, TickTimer, ApplyTimePenalty, InitTimer |
| node HUD | `UiSystems` | SpawnTimerHud |

Cross-domain ordering examples:
- `bolt_cell_collision.after(BreakerSystems::Move)` — bolt must wait for breaker to move
- `dispatch_bolt_effects.before(EffectSystems::Bridge)` — effects dispatched before bridges evaluate
- `cleanup_cell.after(EffectSystems::Bridge)` — don't despawn until effect chains resolve
- `bridge_bump.after(BreakerSystems::GradeBump)` — bridge waits for grade before evaluating

---

## 5. Observer Patterns

**No Bevy observers (`add_observer`) exist anywhere in the game crate.** The codebase does not use the Bevy 0.14+ observer/trigger system at all.

The "observer" concept in this codebase refers to the _effect system's own trigger-bridge pattern_: a system reads a message (e.g., `BumpPerformed`), then evaluates `BoundEffects` trees on all entities with that component, firing sub-effects via direct `World` mutation. This is implemented entirely with standard systems and messages — not Bevy's `Observer` API.

The effect domain's bridge systems are the closest analog:

```rust
fn bridge_bump(
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(&Trigger::Bump, entity, bound, &mut staged, ...);
        }
    }
}
```

Each trigger type has its own bridge system in `effect/triggers/`. Each bridge is its own module (or file) with a `register(app)` function.

---

## 6. File and System Counts for Medium-Complexity Plugins

### `chips/` — smallest domain plugin

- 63 Rust files total (includes tests)
- Systems: `dispatch_chip_effects`, `build_chip_catalog` (from separate `offering/` system)
- Plugin `build()`: registers 1 resource, 1 system in `Update` with state guard
- No `sets.rs` — no cross-domain ordering exposure
- Submodules: `components`, `definition`, `inventory`, `offering`, `resources`, `systems`

### `cells/` — medium domain plugin

- 42 Rust files total
- Systems: `handle_cell_hit`, `check_lock_release`, `tick_cell_regen`, `rotate_shield_cells`, `sync_orbit_cell_positions`, `cleanup_cell`, `cell_wall_collision`
- Plugin `build()`: registers 4 messages, 1 resource, 7 systems all in `FixedUpdate`
- No `sets.rs` — receives ordering constraint via `EffectSystems::Bridge` from effect domain
- Submodules: `builder`, `components`, `definition`, `filters`, `messages`, `queries`, `resources`, `systems`

### `breaker/` — medium domain plugin

- 88 Rust files total
- Systems: `move_breaker`, `update_bump`, `grade_bump`, `update_breaker_state`, `perfect_bump_dash_cancel`, `spawn_bump_grade_text`, `spawn_whiff_text`, `trigger_bump_visual`, `animate_bump_visual`, `animate_tilt_visual`, `breaker_cell_collision`, `breaker_wall_collision`, `sync_breaker_scale`, `reset_breaker`, `apply_node_scale_to_breaker` (15 systems)
- Plugin `build()`: registers 5 messages, 2 resources, systems across `OnEnter(Loading)`, `FixedUpdate`, and `Update`
- Has `sets.rs` — exports `BreakerSystems` with 4 variants
- Submodules: `builder`, `components`, `definition`, `filters`, `messages`, `queries`, `registry`, `resources`, `sets`, `systems`

### `bolt/` — large domain plugin

- 106 Rust files total
- Systems: 15+ systems across spawning, collision, lifecycle, effects dispatch, visual sync
- Plugin `build()`: registers 6 messages, 1 resource, systems across `OnEnter(Loading)`, `OnEnter(AnimateIn)`, `FixedUpdate` (run_if Playing), `FixedUpdate` (run_if AnimateIn or Playing), `Update`
- Has `sets.rs` — exports `BoltSystems` with 5 variants
- Cross-domain ordering in plugin: references `BreakerSystems`, `EffectSystems`, `NodeSystems`, `PhysicsSystems` from rantzsoft

### `effect/` — exceptional (meta-domain)

- 279 Rust files total (22 effects + 20 triggers, each with tests)
- Systems: 70+ (each effect and trigger contributes 1-5 systems)
- Plugin `build()` delegates entirely to `effects::register(app)` and `triggers::register(app)`
- Each effect module owns: `fire()`, `reverse()`, `register()`, any runtime systems, and components
- Has `sets.rs` — exports `EffectSystems::Bridge` (single variant)

---

## 7. The `register(app)` Delegation Pattern (Key for Protocol/Hazard)

The `effect/` plugin's delegation pattern is the most relevant precedent for a multi-item plugin. The pattern is:

1. The plugin struct's `build()` calls module-level `register(app)` functions
2. Each item module owns its own `register(app)` which calls `app.add_systems(...)` directly
3. The top-level `mod.rs` for `effects/` and `triggers/` each have a `register(app)` that calls all sub-`register(app)` functions in sequence

This is how 22 effects and 20 triggers register themselves without making the plugin `build()` unreadable. Each item is a fully self-contained module: its components, its `fire()`, its runtime systems, its `register()`, and its tests.

The flat `.rs` file variant (e.g., `speed_boost.rs`) is used when the effect is simple enough not to need a subdirectory. The `shockwave/` subdirectory pattern is used when the effect has multiple runtime systems and many tests.

---

## Architectural Decision: Protocol and Hazard Plugin Organization

### The question

With 15 protocols and 16 hazards, each being a code-implemented system with its own logic, which organization fits the established patterns?

### What the codebase tells us

**Option A: One `protocol/` plugin + one `hazard/` plugin** — this is the direct analog to the existing domain pattern. `bolt/` has 15 systems; `breaker/` has 15 systems. A domain with 15-16 items is normal. Each plugin owns its registration, its sets (if needed), its resources, and its messages. The `effect/` delegation pattern (each item has a `register()`) scales cleanly to 15-16 items without bloating the plugin `build()`.

**Option B: Combined `mods/` plugin** — analogous to the `effect/` plugin, which unifies the effect and trigger concerns under one roof. This would work but blurs the boundary between a positive and negative system. The existing precedent groups things by _domain responsibility_, not by category. Protocols and hazards are conceptually distinct (player-chosen upgrades vs. player-imposed debuffs with stacking mechanics), suggesting separate plugins.

**Option C: Per-protocol/hazard modules within a single plugin** — this is what `effect/` already does internally. The question is where to draw the plugin boundary. If all protocols are owned by one plugin and all hazards by another, that is Option A (with Option C as the internal implementation).

### Recommendation based on patterns

**Option A is the best fit.** Here is why:

1. The existing pattern is one plugin per domain. Protocols are a player-upgrade domain; hazards are a run-difficulty domain. They have separate lifecycles (protocol selected once per tier, hazard selected every tier 9+), separate UI screens, and separate state management.

2. The `effect/` delegation pattern (each item has `register(app: &mut App)`) solves the "15 items registered in build()" problem without any new patterns. The plugin `build()` stays clean — it calls `effects::register(app)` or analogously `protocols::register(app)`.

3. The internal structure of each plugin would mirror `effect/effects/`: one module per protocol/hazard, each with its own components, systems, `register()`, and tests. Simple protocols/hazards use a single `.rs` file; complex ones use a subdirectory.

4. System sets: only needed if other domains need to order against protocol/hazard systems. Given that hazards will likely write to components that the physics and collision domains already own (e.g., bolt velocity for Haste, node timer for Decay), they may need to expose a set or order explicitly against existing sets like `BoltSystems::CellCollision` or `NodeSystems::TickTimer`.

### Recommended directory layout

```
breaker-game/src/
  protocol/
    mod.rs              # pub use plugin::ProtocolPlugin; etc.
    plugin.rs           # build() calls protocols::register(app)
    sets.rs             # ProtocolSystems (if needed for ordering)
    messages.rs         # ProtocolSelected or similar
    resources.rs        # active protocol tracking, registry
    protocols/
      mod.rs            # pub mod echo; pub mod ...; fn register(app) { ... }
      echo.rs           # simple protocol — fire(), register(), components
      mirror_protocol/  # complex protocol — mod.rs, effect.rs, tests/
      ...               # 15 total
  hazard/
    mod.rs              # pub use plugin::HazardPlugin; etc.
    plugin.rs           # build() calls hazards::register(app)
    sets.rs             # HazardSystems (likely needed — stacking hazards interact)
    messages.rs         # HazardSelected or similar
    resources.rs        # active hazard stack tracking, stacking counts
    hazards/
      mod.rs            # pub mod decay; pub mod drift; ...; fn register(app) { ... }
      decay.rs          # timer tick-rate modifier + register()
      drift/            # drift has ongoing systems — subdirectory
        mod.rs
        effect.rs       # components, fire(), runtime systems, register()
        tests/
      ...               # 16 total
```

### Where new messages will likely live

- `ProtocolSelected` — owned by protocol domain (or UI → protocol consumer)
- `HazardSelected` — owned by hazard domain (or UI → hazard consumer)
- Hazards that react to existing game events (`DamageCell`, `CellDestroyedAt`, `BumpPerformed`) will read those messages as consumers, not re-register them
- Hazards that need to influence existing systems will likely do so by inserting components onto existing entities (e.g., adding a `WindForce` resource the bolt velocity system reads) or via new messages that existing systems consume

### Scale calibration

15-16 items at the complexity of `breaker/` systems is roughly `effect/effects/` scale (22 items). The `effect/` codebase handles this with 279 files. A hazard plugin with 16 hazards, each at average `breaker/` system complexity, would produce roughly 100-150 Rust files — comfortable for a single plugin.

---

## Summary Reference

| Pattern | Location | Key takeaway |
|---|---|---|
| Plugin `build()` | Every `plugin.rs` | Registers messages, resources, systems per schedule |
| Delegation pattern | `effect/plugin.rs` | Delegates to `module::register(app)` for many-item plugins |
| Per-item `register()` | `effect/effects/shockwave/effect.rs` | Each item registers its own systems |
| System sets | `*/sets.rs` | Only for cross-domain ordering exposure |
| Message ownership | `*/messages.rs` + plugin registration | Producer domain registers; consumers just read |
| No observers | (absent) | All events via MessageReader/MessageWriter |
| `mod.rs` = wiring only | Every `mod.rs` | No production logic in mod.rs |
| State guards | `plugin.rs` add_systems calls | `.run_if(in_state(NodeState::Playing))` on FixedUpdate systems |
