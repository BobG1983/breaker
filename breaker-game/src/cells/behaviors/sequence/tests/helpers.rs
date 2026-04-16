//! Shared test harness for Sequence integration tests (Groups B–G).
//!
//! Mirrors the `PendingCellDamage` / `enqueue_cell_damage` pattern from
//! `breaker-game/src/cells/behaviors/volatile/tests/helpers.rs`. The
//! duplication is intentional: the volatile helpers are `pub(super)` and
//! are not accessible across behavior folders.
//!
//! The `build_sequence_test_app()` helper registers the three sequence
//! systems in their intended production schedules so that tests observe
//! the same ordering `CellsPlugin` will eventually provide:
//!
//! - `init_sequence_groups` on `OnEnter(NodeState::Playing)`
//! - `reset_inactive_sequence_hp` in `FixedUpdate`, ordered
//!   `.after(DeathPipelineSystems::ApplyDamage).before(DeathPipelineSystems::DetectDeaths)`
//! - `advance_sequence` in `FixedUpdate`, ordered `.after(EffectV3Systems::Death)`
//!
//! During the RED phase the three system bodies are empty stubs, so every
//! behavioral assertion fails. Writer-code replaces the stub bodies during
//! the GREEN phase without touching this file.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::GlobalPosition2D;

use crate::{
    cells::{
        behaviors::sequence::systems::{
            advance_sequence::advance_sequence, init_sequence_groups::init_sequence_groups,
            reset_inactive_sequence_hp::reset_inactive_sequence_hp,
        },
        test_utils::spawn_cell_in_world,
    },
    effect_v3::sets::EffectV3Systems,
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

/// Default cell dimensions for test spawns. 10×10 is small and uniform so
/// the quadtree indexes cleanly; only the center position and radius math
/// matter for the behaviors under test.
pub(super) const TEST_CELL_DIM: f32 = 10.0;

/// Seeded `DamageDealt<Cell>` messages drained into the queue before
/// `ApplyDamage` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
pub(super) struct PendingCellDamage(pub(super) Vec<DamageDealt<Cell>>);

/// Drains `PendingCellDamage` into the `DamageDealt<Cell>` message queue —
/// one shot per seeded damage, so subsequent ticks do not re-inject the
/// same damage.
pub(super) fn enqueue_cell_damage(
    mut pending: ResMut<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

/// Patches `GlobalPosition2D` on `entity` to match `pos`. Required because
/// the propagation system that normally copies `Position2D` into
/// `GlobalPosition2D` lives in `RunFixedMainLoop`, which these tests never
/// run. Without this patch, the quadtree would index every test cell at
/// the origin.
pub(super) fn patch_global_position(world: &mut World, entity: Entity, pos: Vec2) {
    world.entity_mut(entity).insert(GlobalPosition2D(pos));
}

/// Spawns a sequence cell via `Cell::builder().sequence(group, position)` at
/// `pos` with `hp` hit points.
pub(super) fn spawn_sequence_cell(
    app: &mut App,
    pos: Vec2,
    group: u32,
    position: u32,
    hp: f32,
) -> Entity {
    let entity = spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .sequence(group, position)
            .position(pos)
            .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
            .hp(hp)
            .headless()
            .spawn(commands)
    });
    patch_global_position(app.world_mut(), entity, pos);
    entity
}

/// Spawns a sequence cell with a custom `Hp.max` ceiling. Used by Group C
/// behavior 11 to verify that the reset target is `hp.max.unwrap_or(hp.starting)`.
pub(super) fn spawn_sequence_cell_with_max(
    app: &mut App,
    pos: Vec2,
    group: u32,
    position: u32,
    starting: f32,
    max: f32,
) -> Entity {
    let entity = spawn_sequence_cell(app, pos, group, position, starting);
    if let Some(mut hp) = app.world_mut().get_mut::<Hp>(entity) {
        hp.max = Some(max);
    }
    entity
}

/// Spawns a plain (non-sequence) cell via `Cell::builder()` at `pos` with
/// `hp` hit points.
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

/// Spawns a volatile cell via `Cell::builder().volatile(damage, radius)` at
/// `pos` with `hp` hit points. Used by Groups D and F.
pub(super) fn spawn_volatile_cell(
    app: &mut App,
    pos: Vec2,
    damage: f32,
    radius: f32,
    hp: f32,
) -> Entity {
    let entity = spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .volatile(damage, radius)
            .position(pos)
            .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
            .hp(hp)
            .headless()
            .spawn(commands)
    });
    patch_global_position(app.world_mut(), entity, pos);
    entity
}

/// Standard `DamageDealt<Cell>` with no dealer and no source chip.
pub(super) fn damage_msg(target: Entity, amount: f32) -> DamageDealt<Cell> {
    DamageDealt {
        dealer: None,
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

/// `DamageDealt<Cell>` with an explicit dealer entity. Used by Group C
/// behavior 12's edge cases to verify `KilledBy.dealer` clearing.
pub(super) fn damage_msg_from(target: Entity, amount: f32, dealer: Entity) -> DamageDealt<Cell> {
    DamageDealt {
        dealer: Some(dealer),
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

/// Pushes a `DamageDealt<Cell>` into the per-tick `PendingCellDamage` queue.
/// Drained by `enqueue_cell_damage.before(ApplyDamage)` on the next `tick()`.
pub(super) fn push_damage(app: &mut App, msg: DamageDealt<Cell>) {
    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push(msg);
}

/// Builds a plugin-integration `App` with sequence wiring and damage
/// scaffolding. Collectors for `Destroyed<Cell>` and `DamageDealt<Cell>`
/// are attached. Includes `RantzPhysics2dPlugin` because Groups D and F
/// exercise volatile explosions whose `ExplodeConfig::fire()` queries the
/// quadtree — see `breaker-game/src/effect_v3/effects/explode/config.rs`.
///
/// The three sequence systems are registered in their production schedules
/// directly (not via `CellsPlugin`) so tests exercise this wiring in
/// isolation from the rest of the cells domain. Cross-plugin registration
/// is covered by behaviors 30–32 in `breaker-game/src/cells/plugin.rs`.
pub(super) fn build_sequence_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
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
    app.add_systems(OnEnter(NodeState::Playing), init_sequence_groups);
    app.add_systems(
        FixedUpdate,
        (
            reset_inactive_sequence_hp
                .after(DeathPipelineSystems::ApplyDamage)
                .before(DeathPipelineSystems::DetectDeaths),
            advance_sequence.after(EffectV3Systems::Death),
        )
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Walks the state machine `AppState::Game → GameState::Run → RunState::Node → NodeState::Playing`
/// by setting `NextState<...>` and calling `app.update()` at each step.
/// Mirrors `TestAppBuilder::in_state_node_playing` but is a free function so
/// the caller can spawn cells between initial build and state navigation.
pub(super) fn advance_to_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
}

/// Runs N ticks and returns every `Destroyed<Cell>` message seen across all
/// ticks. Works around `MessageCollector`'s per-tick clear by cloning the
/// collector between ticks.
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
/// message seen across all ticks.
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
