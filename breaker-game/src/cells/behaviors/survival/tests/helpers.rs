//! Shared test harness for Survival integration tests (Parts F and G).
//!
//! Mirrors the pattern from `cells/behaviors/armored/tests/helpers.rs`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::{
        behaviors::survival::systems::{
            kill_bump_vulnerable_cells::kill_bump_vulnerable_cells,
            suppress_bolt_immune_damage::suppress_bolt_immune_damage,
        },
        test_utils::spawn_cell_in_world,
    },
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

/// Default cell dimensions for test spawns.
pub(super) const TEST_CELL_DIM: f32 = 10.0;

/// Seeded `BoltImpactCell` messages drained into the queue before
/// `suppress_bolt_immune_damage` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
pub(super) struct PendingBoltImpactCell(pub(super) Vec<BoltImpactCell>);

/// Seeded `DamageDealt<Cell>` messages drained into the queue before
/// `ApplyDamage` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
pub(super) struct PendingCellDamage(pub(super) Vec<DamageDealt<Cell>>);

/// Seeded `BreakerImpactCell` messages drained into the queue before
/// `kill_bump_vulnerable_cells` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
pub(super) struct PendingBreakerImpactCell(pub(super) Vec<BreakerImpactCell>);

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

/// Drains `PendingBreakerImpactCell` into the `BreakerImpactCell` message queue.
pub(super) fn enqueue_breaker_impact(
    mut pending: ResMut<PendingBreakerImpactCell>,
    mut writer: MessageWriter<BreakerImpactCell>,
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

/// Pushes a `BreakerImpactCell` into the per-tick pending queue.
pub(super) fn push_breaker_impact(app: &mut App, msg: BreakerImpactCell) {
    app.world_mut()
        .resource_mut::<PendingBreakerImpactCell>()
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

/// Constructs a `DamageDealt<Cell>` with no dealer.
pub(super) fn damage_msg_dealerless(target: Entity, amount: f32) -> DamageDealt<Cell> {
    DamageDealt {
        dealer: None,
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

/// Constructs a `BreakerImpactCell` message.
pub(super) fn breaker_impact(breaker: Entity, cell: Entity) -> BreakerImpactCell {
    BreakerImpactCell { breaker, cell }
}

/// Spawns a bolt-immune cell with given HP.
pub(super) fn spawn_bolt_immune_cell(app: &mut App, hp: f32) -> Entity {
    spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .survival(crate::cells::definition::AttackPattern::StraightDown, 10.0)
            .position(Vec2::ZERO)
            .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
            .hp(hp)
            .headless()
            .spawn(commands)
    })
}

/// Spawns a plain (non-immune) cell with given HP.
pub(super) fn spawn_plain_cell(app: &mut App, hp: f32) -> Entity {
    spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
            .hp(hp)
            .headless()
            .spawn(commands)
    })
}

/// Spawns a headless bolt entity for impact messages.
pub(super) fn spawn_test_bolt(app: &mut App) -> Entity {
    app.world_mut().spawn(Bolt).id()
}

/// Spawns a headless breaker entity for impact messages.
pub(super) fn spawn_test_breaker(app: &mut App) -> Entity {
    app.world_mut().spawn(Breaker).id()
}

/// Builds the integration test `App` with bolt-immune wiring.
pub(super) fn build_bolt_immune_test_app() -> App {
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
        enqueue_bolt_impact.before(suppress_bolt_immune_damage),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage
            .before(suppress_bolt_immune_damage)
            .before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        suppress_bolt_immune_damage
            .after(crate::bolt::sets::BoltSystems::CellCollision)
            .before(DeathPipelineSystems::ApplyDamage)
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Builds the integration test `App` with bump-vulnerable wiring.
pub(super) fn build_bump_vulnerable_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_physics()
        .with_effects_pipeline()
        .build();
    attach_message_capture::<DamageDealt<Cell>>(&mut app);
    attach_message_capture::<Destroyed<Cell>>(&mut app);
    app.init_resource::<PendingBreakerImpactCell>();
    app.add_systems(
        FixedUpdate,
        enqueue_breaker_impact.before(kill_bump_vulnerable_cells),
    );
    app.add_systems(
        FixedUpdate,
        kill_bump_vulnerable_cells
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
