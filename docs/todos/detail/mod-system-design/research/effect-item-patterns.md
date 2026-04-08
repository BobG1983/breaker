# Effect Item Patterns — Exact Code

Source files read: Bevy 0.18 project (`breaker-game/src/effect/`).

---

## 1. Simple Effect: `damage_boost.rs`

**File:** `breaker-game/src/effect/effects/damage_boost.rs` (139 lines, single file)

This is the canonical simple effect. No `register()`, no runtime systems — just `fire()` + `reverse()` + a component.

```rust
// breaker-game/src/effect/effects/damage_boost.rs

use bevy::prelude::*;

/// Tracks active damage boost multipliers on an entity.
///
/// The effective multiplier is the product of all entries (default 1.0).
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveDamageBoosts(pub Vec<f32>);

impl ActiveDamageBoosts {
    /// Returns the combined multiplier (product of all entries, default 1.0).
    #[must_use]
    pub fn multiplier(&self) -> f32 {
        if self.0.is_empty() {
            1.0
        } else {
            self.0.iter().product()
        }
    }
}

pub(crate) fn fire(entity: Entity, multiplier: f32, _source_chip: &str, world: &mut World) {
    if world.get_entity(entity).is_err() {
        return;
    }

    if world.get::<ActiveDamageBoosts>(entity).is_none() {
        world
            .entity_mut(entity)
            .insert(ActiveDamageBoosts::default());
    }

    if let Some(mut active) = world.get_mut::<ActiveDamageBoosts>(entity) {
        active.0.push(multiplier);
    }
}

pub(crate) fn reverse(entity: Entity, multiplier: f32, _source_chip: &str, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveDamageBoosts>(entity)
        && let Some(pos) = active
            .0
            .iter()
            .position(|&v| (v - multiplier).abs() < f32::EPSILON)
    {
        active.0.swap_remove(pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_pushes_multiplier_onto_active_damage_boosts() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![])).id();
        fire(entity, 2.0, "", &mut world);
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![2.0]);
    }
    // ... 8 more tests covering bare-entity insertion, non-matching reverse,
    //     double-call safety, stacking, and multiplier() computation
}
```

**Key patterns:**
- No `register()` function — simple effects have nothing to add to the App.
- `fire()` and `reverse()` take `&mut World` directly (called from `commands.rs` dispatch).
- Lazy component insertion: `fire()` inserts the component if absent, then pushes the value.
- `reverse()` uses `swap_remove` to find and remove one matching entry (first match wins).
- `_source_chip` parameter is received but unused for pure-passive effects.
- Tests run directly against `World::new()` — no App, no plugin, no schedule.

Other single-file passive effects following the same pattern: `bump_force.rs`, `piercing.rs`,
`quick_stop.rs`, `size_boost.rs`, `vulnerable.rs`, `flash_step.rs`.

### Variant: `speed_boost.rs` — fire/reverse with side-effect

`speed_boost.rs` follows the same structure but `fire()` and `reverse()` both call
`recalculate_velocity()` after mutating the component, because the velocity needs to be
recomputed immediately when the boost changes (not deferred to the next tick).

```rust
// breaker-game/src/effect/effects/speed_boost.rs (excerpt)

pub(crate) fn fire(entity: Entity, multiplier: f32, _source_chip: &str, world: &mut World) {
    // ... (same lazy-insert pattern) ...
    if let Some(mut active) = world.get_mut::<ActiveSpeedBoosts>(entity) {
        active.0.push(multiplier);
    }
    recalculate_velocity(entity, world);   // <-- immediate side-effect
}

fn recalculate_velocity(entity: Entity, world: &mut World) {
    let boosts = world.get::<ActiveSpeedBoosts>(entity).cloned();
    let mut query = world.query::<SpatialData>();
    let Ok(mut spatial) = query.get_mut(world, entity) else { return; };
    apply_velocity_formula(&mut spatial, boosts.as_ref());
}
```

This is still a single file with no `register()`.

---

## 2. Complex Effect: `shockwave/`

**Directory structure:**
```
breaker-game/src/effect/effects/shockwave/
  mod.rs       -- re-export surface only
  effect.rs    -- components, fire(), reverse(), runtime systems, register()
  tests/
    mod.rs           -- declares test sub-modules
    helpers.rs       -- shared test setup (app builders, spawn helpers)
    fire_tests.rs    -- tests for fire() behavior
    expansion_tests.rs -- tests for tick_shockwave, despawn_finished_shockwave
    damage_tests.rs  -- tests for apply_shockwave_damage
    visual_tests.rs  -- tests for sync_shockwave_visual
```

### `mod.rs`

```rust
// breaker-game/src/effect/effects/shockwave/mod.rs

pub(crate) use effect::*;

mod effect;

#[cfg(test)]
mod tests;
```

Pure wiring. No production code lives here.

### `effect.rs` — components, fire(), reverse(), runtime systems, register()

```rust
// breaker-game/src/effect/effects/shockwave/effect.rs (192 lines, full)

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{plugin::PhysicsSystems, resources::CollisionQuadtree};
use rantzsoft_spatial2d::components::Spatial;

use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    effect::core::EffectSourceChip,
    prelude::*,
    shared::{CELL_LAYER, GameDrawLayer},
};

const SHOCKWAVE_COLOR: Color = Color::linear_rgb(4.0, 1.5, 0.2);

#[derive(Component)] pub(crate) struct ShockwaveSource;
#[derive(Component)] pub(crate) struct ShockwaveRadius(pub(crate) f32);
#[derive(Component)] pub(crate) struct ShockwaveMaxRadius(pub(crate) f32);
#[derive(Component)] pub(crate) struct ShockwaveSpeed(pub(crate) f32);
#[derive(Component, Default)] pub(crate) struct ShockwaveDamaged(pub(crate) HashSet<Entity>);
#[derive(Component)] pub(crate) struct ShockwaveDamageMultiplier(pub(crate) f32);
#[derive(Component)] pub(crate) struct ShockwaveBaseDamage(pub(crate) f32);

/// fire() signature: entity=source, base_range, range_per_level, stacks, speed, source_chip
pub(crate) fn fire(
    entity: Entity,
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
    source_chip: &str,
    world: &mut World,
) {
    let effective_range =
        crate::effect::effects::effective_range(base_range, range_per_level, stacks);

    let position = crate::effect::effects::entity_position(world, entity);

    let edm = world
        .get::<ActiveDamageBoosts>(entity)
        .map_or(1.0, ActiveDamageBoosts::multiplier);

    let base_damage = world
        .get::<BoltBaseDamage>(entity)
        .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);

    let visual = { /* spawns Mesh2d + MeshMaterial2d from Assets, or None if headless */ };

    let mut entity = world.spawn((
        ShockwaveSource,
        ShockwaveRadius(0.0),
        ShockwaveMaxRadius(effective_range),
        ShockwaveSpeed(speed),
        ShockwaveDamaged::default(),
        ShockwaveDamageMultiplier(edm),
        ShockwaveBaseDamage(base_damage),
        EffectSourceChip::new(source_chip),
        Spatial::builder().at_position(position).build(),
        Scale2D { x: 0.0, y: 0.0 },
        GameDrawLayer::Fx,
        CleanupOnExit::<NodeState>::default(),
    ));
    if let Some((mesh, mat)) = visual {
        entity.insert((Mesh2d(mesh), MeshMaterial2d(mat)));
    }
}

// reverse() is a no-op — shockwaves run to completion
pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Runtime system 1: expand radius each FixedUpdate tick
pub(crate) fn tick_shockwave(
    time: Res<Time>,
    mut query: Query<(&mut ShockwaveRadius, &ShockwaveSpeed)>,
) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut query {
        radius.0 = speed.0.mul_add(dt, radius.0);
    }
}

/// Runtime system 2: despawn when max radius reached
pub(crate) fn despawn_finished_shockwave(
    mut commands: Commands,
    query: Query<(Entity, &ShockwaveRadius, &ShockwaveMaxRadius)>,
) {
    for (entity, radius, max_radius) in &query {
        if radius.0 >= max_radius.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Runtime system 3: damage cells inside the expanding ring via quadtree query
pub(crate) fn apply_shockwave_damage(
    quadtree: Res<CollisionQuadtree>,
    mut shockwaves: Query<ShockwaveDamageQuery, With<ShockwaveSource>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    for (position, radius, mut damaged, damage_mult, shockwave_base_damage, esc) in &mut shockwaves {
        if radius.0 <= 0.0 { continue; }
        let candidates = quadtree.quadtree.query_circle_filtered(center, radius.0, query_layers);
        for cell in candidates {
            if damaged.0.insert(cell) {  // at-most-once per cell per shockwave
                damage_writer.write(DamageCell { cell, damage: base_damage * multiplier, source_chip });
            }
        }
    }
}

/// Runtime system 4: sync visual scale to match radius
pub(crate) fn sync_shockwave_visual(
    mut query: Query<(&ShockwaveRadius, &mut Scale2D), With<ShockwaveSource>>,
) {
    for (radius, mut scale) in &mut query {
        scale.x = radius.0;
        scale.y = radius.0;
    }
}

/// register() — wires all runtime systems into FixedUpdate
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            tick_shockwave,
            sync_shockwave_visual,
            apply_shockwave_damage,
            despawn_finished_shockwave,
        )
            .chain()
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(NodeState::Playing)),
    );
}
```

**Key patterns:**
- `fire()` spawns a new entity — does NOT mutate the source entity.
- Snapshotting at fire-time: `ActiveDamageBoosts` and `BoltBaseDamage` are read from the source
  entity at spawn time and baked into `ShockwaveDamageMultiplier` / `ShockwaveBaseDamage`.
- `CleanupOnExit::<NodeState>` ensures despawn on node exit.
- Visual assets (`Mesh2d`, `MeshMaterial2d`) are optional — `Assets` is absent in headless tests.
- Systems are chained: tick → sync visual → damage → despawn.
- `after(PhysicsSystems::MaintainQuadtree)` ensures the quadtree is up to date before querying.

### `tests/mod.rs`

```rust
// breaker-game/src/effect/effects/shockwave/tests/mod.rs

mod helpers;

mod damage_tests;
mod expansion_tests;
mod fire_tests;
mod visual_tests;
```

No `pub` here — all sub-modules are private under `#[cfg(test)] mod tests`.

### `tests/helpers.rs` (excerpt — the shared test setup)

```rust
// shared imports from effect.rs via `use crate::effect::effects::shockwave::effect::*;`

#[derive(Resource, Default)]
pub(super) struct DamageCellCollector(pub(super) Vec<DamageCell>);

pub(super) fn collect_damage_cells(
    mut reader: MessageReader<DamageCell>,
    mut collector: ResMut<DamageCellCollector>,
) {
    for msg in reader.read() { collector.0.push(msg.clone()); }
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // ... state initialization for AppState → GameState → RunState → NodeState ...
    app.add_systems(Update, tick_shockwave);
    app.add_systems(Update, despawn_finished_shockwave);
    app
}

pub(super) fn damage_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.add_systems(Update, apply_shockwave_damage);
    app.add_systems(Update, collect_damage_cells.after(apply_shockwave_damage));
    app
}

pub(super) fn enter_playing(app: &mut App) { /* steps state machine through to NodeState::Playing */ }
pub(super) fn tick(app: &mut App) { /* accumulates one fixed timestep then calls app.update() */ }
pub(super) fn spawn_test_cell(app: &mut App, x: f32, y: f32) -> Entity { /* ... */ }
pub(super) fn spawn_shockwave(app: &mut App, x: f32, y: f32, radius: f32, damaged: HashSet<Entity>) -> Entity { /* ... */ }
```

**Key patterns for tests:**
- `fire_tests.rs` calls `fire()` directly on `World::new()` — no App, no schedule.
- `expansion_tests.rs` and `damage_tests.rs` use `App` with real systems, calling `app.update()`.
- Two distinct app builders: one for expansion/visual (just MinimalPlugins + systems),
  one for damage (also `RantzPhysics2dPlugin` needed for `CollisionQuadtree`).
- `tick()` helper accumulates a full fixed timestep before `update()` so FixedUpdate systems run.
- Helpers use `pub(super)` — visible within the `tests/` tree but not outside.

---

## 3. The Top-Level `effects/mod.rs` — Collection Point

```rust
// breaker-game/src/effect/effects/mod.rs (93 lines, full)

//! Effect modules — one per effect, each with `fire()`, `reverse()`, `register()`.

pub(crate) mod fire_helpers;
pub(crate) use fire_helpers::{bolt_visual_handles, effective_range, entity_position, insert_bolt_visuals};

pub mod anchor;
pub mod attraction;
pub mod bump_force;
pub mod chain_bolt;
pub mod chain_lightning;
pub mod circuit_breaker;
pub mod damage_boost;
pub mod entropy_engine;
pub mod explode;
pub mod flash_step;
pub mod gravity_well;
pub mod life_lost;
pub mod mirror_protocol;
pub mod piercing;
pub mod piercing_beam;
pub mod pulse;
pub mod quick_stop;
pub mod ramping_damage;
pub mod random_effect;
pub mod second_wind;
pub mod shield;
pub mod shockwave;
pub mod size_boost;
pub mod spawn_bolts;
pub mod spawn_phantom;
pub mod speed_boost;
pub mod tether_beam;
pub mod time_penalty;
pub mod vulnerable;

/// Register all effect runtime systems.
pub(crate) fn register(app: &mut bevy::prelude::App) {
    shockwave::register(app);
    chain_lightning::register(app);
    piercing_beam::register(app);
    pulse::register(app);
    shield::register(app);
    gravity_well::register(app);
    spawn_phantom::register(app);
    entropy_engine::register(app);
    ramping_damage::register(app);
    explode::register(app);
    flash_step::register(app);
    mirror_protocol::register(app);
    spawn_bolts::register(app);
    chain_bolt::register(app);
    attraction::register(app);
    tether_beam::register(app);
    life_lost::register(app);
    time_penalty::register(app);
    second_wind::register(app);
    random_effect::register(app);
    anchor::register(app);
    circuit_breaker::register(app);
}
```

**Key observations:**
- Only effects with runtime systems appear in `register()` — `damage_boost`, `bump_force`,
  `piercing`, `speed_boost`, `size_boost`, `quick_stop`, `vulnerable` are NOT called here.
  Those effects have no runtime systems; their `fire()`/`reverse()` act immediately on `World`.
- `fire_helpers` is a shared utility module (not an effect itself): provides `effective_range()`,
  `entity_position()`, `bolt_visual_handles()`, `insert_bolt_visuals()`.

---

## 4. Trigger Bridge Systems

The trigger layer has two kinds of bridges: **global** (fires on all entities with `BoundEffects`)
and **targeted** (fires on the specific collision participants only).

### 4a. Simple Global Bridge: `bump/system.rs`

```rust
// breaker-game/src/effect/triggers/bump/system.rs (49 lines, full)

//! Bridge system for the `bump` trigger.
use bevy::prelude::*;

use crate::{
    breaker::sets::BreakerSystems,
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    prelude::*,
};

pub(super) fn bridge_bump(
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext {
            bolt: msg.bolt,
            breaker: Some(msg.breaker),
            ..default()
        };
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Bump,
                entity,
                bound,
                &mut staged,
                &mut commands,
                context,
            );
            evaluate_staged_effects(&Trigger::Bump, entity, &mut staged, &mut commands, context);
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        bridge_bump
            .in_set(EffectSystems::Bridge)
            .after(BreakerSystems::GradeBump)
            .run_if(in_state(NodeState::Playing)),
    );
}
```

**Bridge file layout:** `bump/mod.rs` just re-exports `register` and declares `system` + `tests`:
```rust
pub(crate) mod system;
#[cfg(test)] mod tests;
pub(crate) use system::register;
```

### 4b. Simpler Inline Bridge: `cell_destroyed.rs` (single file with tests)

```rust
// breaker-game/src/effect/triggers/cell_destroyed.rs (211 lines)

fn bridge_cell_destroyed(
    mut reader: MessageReader<CellDestroyedAt>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for _msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::CellDestroyed,
                entity,
                bound,
                &mut staged,
                &mut commands,
                TriggerContext::default(),  // <-- no context fields populated
            );
            evaluate_staged_effects(
                &Trigger::CellDestroyed,
                entity,
                &mut staged,
                &mut commands,
                TriggerContext::default(),
            );
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        bridge_cell_destroyed
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(NodeState::Playing)),
    );
}
```

Note: `cell_destroyed` uses `TriggerContext::default()` — no context entity fields. The message
`CellDestroyedAt` carries `was_required_to_clear: bool` but the bridge ignores it for trigger
evaluation purposes. The context fields (`bolt`, `cell`, etc.) are only populated when the
trigger needs `On { target }` nodes to resolve to a specific entity.

### 4c. Targeted Bridge: `impacted/system.rs` (contrast with global `impact/system.rs`)

The **global** `Impact` bridge (`impact/system.rs`) iterates ALL entities with `BoundEffects`:
```rust
for (entity, bound, mut staged) in &mut query {  // sweeps every entity
    evaluate_bound_effects(&Trigger::Impact(ImpactTarget::Cell), entity, ...);
}
```

The **targeted** `Impacted` bridge (`impacted/system.rs`) looks up only the two collision
participants:
```rust
// BoltImpactCell -> Impacted(Cell) on bolt + Impacted(Bolt) on cell
pub(super) fn bridge_impacted_bolt_cell(
    mut reader: MessageReader<BoltImpactCell>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.bolt) {
            let context = TriggerContext { cell: Some(msg.cell), ..default() };
            evaluate_bound_effects(&Trigger::Impacted(ImpactTarget::Cell), entity, bound, &mut staged, &mut commands, context);
            evaluate_staged_effects(&Trigger::Impacted(ImpactTarget::Cell), entity, &mut staged, &mut commands, context);
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.cell) {
            let context = TriggerContext { bolt: Some(msg.bolt), ..default() };
            evaluate_bound_effects(&Trigger::Impacted(ImpactTarget::Bolt), entity, bound, &mut staged, &mut commands, context);
            evaluate_staged_effects(&Trigger::Impacted(ImpactTarget::Bolt), entity, &mut staged, &mut commands, context);
        }
    }
}
```

The targeted bridge uses `query.get_mut(specific_entity)` rather than iterating all entities.

Both global and targeted bridges register in `EffectSystems::Bridge` and are ordered:
```rust
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            bridge_impacted_bolt_cell.after(BoltSystems::CellCollision),
            bridge_impacted_bolt_wall.after(BoltSystems::CellCollision),
            bridge_impacted_bolt_breaker.after(BoltSystems::BreakerCollision),
            bridge_impacted_breaker_cell,
            bridge_impacted_breaker_wall,
            bridge_impacted_cell_wall,
        )
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(NodeState::Playing)),
    );
}
```

---

## 5. `EffectPlugin::build()`

```rust
// breaker-game/src/effect/plugin.rs (11 lines, full)

use bevy::prelude::*;

pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        super::effects::register(app);   // all effect runtime systems
        super::triggers::register(app);  // all trigger bridge systems
    }
}
```

### `triggers::register()` in full:

```rust
// breaker-game/src/effect/triggers/mod.rs — register() (88 lines)

pub(crate) fn register(app: &mut bevy::prelude::App) {
    use bevy::prelude::*;
    use crate::state::types::NodeState;

    // Global bump family
    bump::register(app);
    perfect_bump::register(app);
    early_bump::register(app);
    late_bump::register(app);
    bump_whiff::register(app);
    no_bump::register(app);

    // Targeted bump family
    bumped::register(app);
    perfect_bumped::register(app);
    early_bumped::register(app);
    late_bumped::register(app);

    // Collision family (global + targeted)
    impact::register(app);
    impacted::register(app);

    // Entity lifecycle
    cell_destroyed::register(app);
    death::register(app);
    died::register(app);
    bolt_lost::register(app);

    // Node lifecycle
    node_start::register(app);
    node_end::register(app);

    // Timer / Until
    timer::register(app);

    // Until desugaring (not inside a submodule's register — added inline)
    app.add_systems(
        FixedUpdate,
        until::desugar_until.run_if(in_state(NodeState::Playing)),
    );
}
```

---

## 6. Effect with Inter-Effect Calls: `circuit_breaker/effect.rs`

This shows that `fire()` can call other effects' `fire()` directly (not through the trigger
pipeline) for complex composed behaviors.

```rust
// breaker-game/src/effect/effects/circuit_breaker/effect.rs (118 lines)

pub(crate) struct CircuitBreakerConfig {
    pub bumps_required: u32,
    pub spawn_count: u32,
    pub inherit: bool,
    pub shockwave_range: f32,
    pub shockwave_speed: f32,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct CircuitBreakerCounter {
    pub remaining: u32,
    pub bumps_required: u32,
    pub spawn_count: u32,
    pub inherit: bool,
    pub shockwave_range: f32,
    pub shockwave_speed: f32,
}

pub(crate) fn fire(entity: Entity, config: &CircuitBreakerConfig, source_chip: &str, world: &mut World) {
    if world.get_entity(entity).is_err() { return; }

    if let Some(mut counter) = world.get_mut::<CircuitBreakerCounter>(entity) {
        counter.remaining -= 1;
        if counter.remaining == 0 {
            let sc = counter.spawn_count; /* ... extract before drop ... */
            counter.remaining = br;
            // Direct call into other effects' fire() functions:
            crate::effect::effects::spawn_bolts::fire(entity, sc, None, inh, source_chip, world);
            crate::effect::effects::shockwave::fire(entity, sr, 0.0, 1, ss, source_chip, world);
        }
    } else {
        // First call: insert counter
        world.entity_mut(entity).insert(CircuitBreakerCounter { remaining: config.bumps_required - 1, ... });
        if remaining == 0 {
            crate::effect::effects::spawn_bolts::fire(...);
            crate::effect::effects::shockwave::fire(...);
        }
    }
}

pub(crate) fn reverse(entity: Entity, _source_chip: &str, world: &mut World) {
    world.entity_mut(entity).remove::<CircuitBreakerCounter>();
}

// No runtime systems needed
pub(crate) const fn register(_app: &mut App) {}
```

`circuit_breaker/mod.rs`:
```rust
pub(crate) use effect::{CircuitBreakerConfig, fire, register, reverse};
mod effect;
#[cfg(test)] mod tests;
```

Note that `CircuitBreakerConfig` is re-exported alongside `fire`/`reverse`/`register` because
the `commands.rs` dispatcher needs it to construct the call.

---

## Pattern Summary

### Single-file effect (no runtime systems)
- One `.rs` file.
- Exports: `pub(crate) fn fire(entity, ...params..., source_chip, world)`, `pub(crate) fn reverse(entity, ...params..., source_chip, world)`.
- No `register()` — not called from `effects::register()`.
- Component(s) defined in the same file.
- Tests in `#[cfg(test)] mod tests` at the bottom of the file.

### Directory effect (with runtime systems)
- Directory `effect_name/mod.rs` + `effect_name/effect.rs` + `effect_name/tests/`.
- `mod.rs`: `pub(crate) use effect::*; mod effect; #[cfg(test)] mod tests;`
- `effect.rs`: all components, `fire()`, `reverse()`, runtime system functions, `register()`.
- `tests/mod.rs`: `mod helpers; mod group_a; mod group_b; ...`
- `tests/helpers.rs`: shared `test_app()`, `damage_test_app()`, `tick()`, spawn helpers. Uses `pub(super)`.
- Tests calling `fire()` directly use bare `World::new()`.
- Tests for runtime systems use `App` + `MinimalPlugins` + manual system registration.

### Trigger bridge module (global)
- `trigger_name/mod.rs` re-exports `register` from `system.rs`.
- `trigger_name/system.rs`: one `bridge_*` function per message type, sweeps all entities.
  Pattern: `for msg in reader.read() { let context = TriggerContext { ... }; for (entity, bound, mut staged) in &mut query { evaluate_bound_effects(...); evaluate_staged_effects(...); } }`
- `register()`: adds all bridge functions to `EffectSystems::Bridge` in `FixedUpdate`, `run_if(in_state(NodeState::Playing))`.

### Trigger bridge module (targeted)
- Same file structure as global.
- `bridge_*` function uses `query.get_mut(specific_entity)` instead of iterating all entities.
- Fires a different trigger variant: `Trigger::Impacted(...)` vs `Trigger::Impact(...)`.

### `EffectPlugin::build()`
- Two delegate calls only: `effects::register(app)` and `triggers::register(app)`.
- No direct system registrations in the plugin itself.
