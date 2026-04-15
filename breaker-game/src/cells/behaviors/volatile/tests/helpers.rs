//! Shared test harness for Groups D/E/F Volatile integration tests.
//!
//! These helpers duplicate the `PendingCellDamage` / `enqueue_cell_damage`
//! pattern from `breaker-game/src/shared/death_pipeline/systems/tests/helpers.rs`.
//! The duplication is intentional — `death_pipeline`'s helpers are
//! `pub(super)` and are not accessible from the cells domain. Per the Wave 1
//! test spec, we duplicate rather than promote visibility.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::GlobalPosition2D;

use crate::{
    cells::test_utils::spawn_cell_in_world, prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

/// Default cell dimensions for test spawns. Width/height are immaterial to the
/// behaviors under test — only the center position and radius/distance math
/// matter. 10×10 gives a small, uniform AABB that the quadtree indexes cleanly.
const TEST_CELL_DIM: f32 = 10.0;

/// Seeded `DamageDealt<Cell>` messages drained into the queue before
/// `ApplyDamage` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
pub(super) struct PendingCellDamage(pub(super) Vec<DamageDealt<Cell>>);

/// Drains `PendingCellDamage` into the `DamageDealt<Cell>` message queue — one
/// shot per seeded damage, so subsequent ticks do not re-inject the same damage.
pub(super) fn enqueue_cell_damage(
    mut pending: ResMut<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

/// Injected `Destroyed<Cell>` messages (used by Group F behavior 28 to drive
/// the bridge directly without going through `handle_kill`).
#[derive(Resource, Default)]
pub(super) struct TestCellDestroyedMessages(pub(super) Vec<Destroyed<Cell>>);

pub(super) fn inject_cell_destroyed(
    mut messages: ResMut<TestCellDestroyedMessages>,
    mut writer: MessageWriter<Destroyed<Cell>>,
) {
    for msg in messages.0.drain(..) {
        writer.write(msg);
    }
}

/// Patches `GlobalPosition2D` on `entity` to match `pos`. Required because
/// `Aabb2D`'s `#[require(Spatial2D)]` chain inserts `GlobalPosition2D` with
/// its `Default` (`Vec2::ZERO`), and the propagation system that normally
/// copies `Position2D` into `GlobalPosition2D` lives in `RunFixedMainLoop`,
/// which these FixedUpdate-only tests never run. Without this patch, the
/// quadtree would index every test cell at the origin.
fn patch_global_position(world: &mut World, entity: Entity, pos: Vec2) {
    world.entity_mut(entity).insert(GlobalPosition2D(pos));
}

/// Spawns a volatile cell via `Cell::builder().volatile(damage, radius)` at
/// `pos` with `hp` hit points. The builder inserts all the production
/// components (`Aabb2D`, `CollisionLayers`, `Spatial2D`, `GlobalPosition2D`
/// via `#[require]`, etc.) and stamps the volatile `BoundEffects` tree
/// through `commands.stamp_effect`, matching the production spawn path.
pub(super) fn spawn_volatile_cell(
    app: &mut App,
    pos: Vec2,
    damage: f32,
    radius: f32,
    hp: f32,
) -> Entity {
    let entity = spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .position(pos)
            .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
            .hp(hp)
            .volatile(damage, radius)
            .headless()
            .spawn(commands)
    });
    patch_global_position(app.world_mut(), entity, pos);
    entity
}

/// Spawns a plain (non-volatile) cell via `Cell::builder()` at `pos` with
/// `hp` hit points. Same reasoning as `spawn_volatile_cell`: the builder
/// owns the full component set, we only patch `GlobalPosition2D` post-spawn.
pub(super) fn spawn_plain_cell(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    let entity = spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .position(pos)
            .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
            .hp(hp)
            .headless()
            .spawn(commands)
    });
    patch_global_position(app.world_mut(), entity, pos);
    entity
}

/// Spawns a cell via the builder and then inserts `Dead`. Used by Group F
/// behaviors that need a dead target inside a volatile's radius to verify
/// that `ExplodeConfig::fire()`'s narrow phase skips entities with `Dead`.
pub(super) fn spawn_dead_cell(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    let entity = spawn_plain_cell(app, pos, hp);
    app.world_mut().entity_mut(entity).insert(Dead);
    entity
}

/// Spawns a cell via the builder and then inserts `Invulnerable`. Used by
/// Group D behavior 23 edge to verify that `apply_damage` filters invulnerable
/// cells even when they are inside the explosion radius.
pub(super) fn spawn_invulnerable_cell(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    let entity = spawn_plain_cell(app, pos, hp);
    app.world_mut().entity_mut(entity).insert(Invulnerable);
    entity
}

/// Spawns a pre-dead volatile cell via the builder (volatile sugar + `Dead` +
/// `Hp::new(0.0)`). Used by Group F behaviors 28 and 28-edge which inject
/// `Destroyed<Cell>` messages directly to drive the death bridge without
/// going through `handle_kill`.
pub(super) fn spawn_dead_volatile_cell(
    app: &mut App,
    pos: Vec2,
    damage: f32,
    radius: f32,
) -> Entity {
    let entity = spawn_volatile_cell(app, pos, damage, radius, 0.0);
    app.world_mut().entity_mut(entity).insert(Dead);
    entity
}

pub(super) fn damage_msg(target: Entity, amount: f32) -> DamageDealt<Cell> {
    DamageDealt {
        dealer: None,
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

/// Builds a plugin-integration `App` with volatile wiring and the given
/// pending-damage seeds. Collectors for `Destroyed<Cell>` and `DamageDealt<Cell>`
/// are attached. Includes `RantzPhysics2dPlugin` so `ExplodeConfig::fire()`
/// can use the quadtree radius query.
pub(super) fn build_volatile_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_physics()
        .with_effects_pipeline()
        .build();
    attach_message_capture::<Destroyed<Cell>>(&mut app);
    attach_message_capture::<DamageDealt<Cell>>(&mut app);
    app.init_resource::<PendingCellDamage>();
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app
}

/// Runs N ticks and returns the set of `Destroyed<Cell>.victim` values seen
/// across all ticks. This works around `MessageCollector`'s per-tick clear by
/// reading the collector between ticks.
pub(super) fn run_ticks_and_collect_destroyed(app: &mut App, ticks: usize) -> Vec<Entity> {
    let mut out: Vec<Entity> = Vec::new();
    for _ in 0..ticks {
        tick(app);
        let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
        for msg in &destroyed.0 {
            out.push(msg.victim);
        }
    }
    out
}

/// Runs N ticks and returns every `Destroyed<Cell>` message seen across all
/// ticks (full message — includes `victim`, `victim_pos`, etc). Works around
/// `MessageCollector`'s per-tick clear by cloning the collector between ticks.
pub(super) fn run_ticks_capture_destroyed(app: &mut App, ticks: usize) -> Vec<Destroyed<Cell>> {
    let mut out: Vec<Destroyed<Cell>> = Vec::new();
    for _ in 0..ticks {
        tick(app);
        let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
        out.extend(destroyed.0.iter().cloned());
    }
    out
}

/// Runs N ticks and returns every `Destroyed<Cell>` and `DamageDealt<Cell>`
/// message seen across all ticks. Works around `MessageCollector`'s per-tick
/// clear by cloning both collectors between ticks.
pub(super) fn run_ticks_capture_destroyed_and_damage(
    app: &mut App,
    ticks: usize,
) -> (Vec<Destroyed<Cell>>, Vec<DamageDealt<Cell>>) {
    let mut destroyed_out: Vec<Destroyed<Cell>> = Vec::new();
    let mut damage_out: Vec<DamageDealt<Cell>> = Vec::new();
    for _ in 0..ticks {
        tick(app);
        let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
        destroyed_out.extend(destroyed.0.iter().cloned());
        let damage = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        damage_out.extend(damage.0.iter().cloned());
    }
    (destroyed_out, damage_out)
}
