//! Shared test harness for Armored integration tests (Groups C–F).
//!
//! Mirrors the `PendingCellDamage` / `enqueue_cell_damage` pattern from
//! `breaker-game/src/cells/behaviors/sequence/tests/helpers.rs`. The
//! duplication is intentional: the sequence helpers are `pub(super)` and
//! are not accessible across behavior folders.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::GlobalPosition2D;

use crate::{
    bolt::components::PiercingRemaining,
    cells::{
        behaviors::armored::{
            components::ArmorDirection, systems::check_armor_direction::check_armor_direction,
        },
        definition::{CellBehavior, CellTypeDefinition, Toughness},
        test_utils::spawn_cell_in_world,
    },
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

/// Default cell dimensions for test spawns.
pub(super) const TEST_CELL_DIM: f32 = 10.0;

/// Seeded `BoltImpactCell` messages drained into the queue before
/// `check_armor_direction` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
pub(super) struct PendingBoltImpactCell(pub(super) Vec<BoltImpactCell>);

/// Seeded `DamageDealt<Cell>` messages drained into the queue before
/// `ApplyDamage` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
pub(super) struct PendingCellDamage(pub(super) Vec<DamageDealt<Cell>>);

/// Drains `PendingBoltImpactCell` into the `BoltImpactCell` message queue.
pub(super) fn enqueue_bolt_impact(
    mut pending: ResMut<PendingBoltImpactCell>,
    mut writer: MessageWriter<BoltImpactCell>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

/// Drains `PendingCellDamage` into the `DamageDealt<Cell>` message queue.
pub(super) fn enqueue_cell_damage(
    mut pending: ResMut<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

/// Pushes a `BoltImpactCell` into the per-tick pending queue.
pub(super) fn push_bolt_impact(app: &mut App, msg: BoltImpactCell) {
    app.world_mut()
        .resource_mut::<PendingBoltImpactCell>()
        .0
        .push(msg);
}

/// Pushes a `DamageDealt<Cell>` into the per-tick pending queue.
pub(super) fn push_damage(app: &mut App, msg: DamageDealt<Cell>) {
    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push(msg);
}

/// Constructs a `BoltImpactCell` message.
pub(super) fn bolt_impact(
    bolt: Entity,
    cell: Entity,
    impact_normal: Vec2,
    piercing_remaining: u32,
) -> BoltImpactCell {
    BoltImpactCell {
        cell,
        bolt,
        impact_normal,
        piercing_remaining,
    }
}

/// Constructs a `DamageDealt<Cell>` with an explicit dealer entity.
pub(super) fn damage_msg_from(target: Entity, amount: f32, dealer: Entity) -> DamageDealt<Cell> {
    DamageDealt {
        dealer: Some(dealer),
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

/// Patches `GlobalPosition2D` on `entity` to match `pos`.
pub(super) fn patch_global_position(world: &mut World, entity: Entity, pos: Vec2) {
    world.entity_mut(entity).insert(GlobalPosition2D(pos));
}

/// Spawns an armored cell via `Cell::builder().armored_facing(value, facing)`.
pub(super) fn spawn_armored_cell(
    app: &mut App,
    pos: Vec2,
    value: u8,
    facing: ArmorDirection,
    hp: f32,
) -> Entity {
    let entity = spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .armored_facing(value, facing)
            .position(pos)
            .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
            .hp(hp)
            .headless()
            .spawn(commands)
    });
    patch_global_position(app.world_mut(), entity, pos);
    entity
}

/// Spawns a plain (non-armored) cell.
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

/// Spawns a headless entity with only `PiercingRemaining(piercing)`.
pub(super) fn spawn_test_bolt(app: &mut App, piercing: u32) -> Entity {
    app.world_mut().spawn(PiercingRemaining(piercing)).id()
}

/// Returns a valid `CellTypeDefinition` whose only behavior is
/// `CellBehavior::Armored { value, facing }`.
pub(super) fn armor_definition(value: u8, facing: ArmorDirection) -> CellTypeDefinition {
    CellTypeDefinition {
        id:                "test".to_owned(),
        alias:             "T".to_owned(),
        toughness:         Toughness::default(),
        color_rgb:         [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base:   4.0,
        damage_green_min:  0.2,
        damage_blue_range: 0.4,
        damage_blue_base:  0.2,
        behaviors:         Some(vec![CellBehavior::Armored { value, facing }]),
        effects:           None,
    }
}

/// Builds the integration test `App` with armor-specific wiring.
pub(super) fn build_armored_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_physics()
        .with_effects_pipeline()
        .build();
    attach_message_capture::<DamageDealt<Cell>>(&mut app);
    attach_message_capture::<Destroyed<Cell>>(&mut app);
    app.configure_sets(FixedUpdate, crate::bolt::sets::BoltSystems::CellCollision);
    app.init_resource::<PendingBoltImpactCell>();
    app.init_resource::<PendingCellDamage>();
    app.add_systems(
        FixedUpdate,
        enqueue_bolt_impact.before(check_armor_direction),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        check_armor_direction
            .after(crate::bolt::sets::BoltSystems::CellCollision)
            .before(DeathPipelineSystems::ApplyDamage)
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Walks `AppState::Game -> GameState::Run -> RunState::Node -> NodeState::Playing`.
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
