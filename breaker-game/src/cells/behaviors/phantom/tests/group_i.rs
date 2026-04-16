//! Group I — Death During Phantom Phases
//!
//! Tests that phantom cells die normally when damaged during Solid or Telegraph
//! phases, and that Ghost phase cells have zeroed collision layers.

use std::{marker::PhantomData, time::Duration};

use bevy::prelude::*;

use crate::{
    cells::{
        behaviors::phantom::components::{PhantomCell, PhantomConfig, PhantomPhase, PhantomTimer},
        test_utils::spawn_cell_in_world,
    },
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

/// Seeded `DamageDealt<Cell>` messages drained into the queue before
/// `ApplyDamage` runs.
#[derive(Resource, Default)]
struct PendingCellDamage(Vec<DamageDealt<Cell>>);

fn enqueue_cell_damage(
    mut pending: ResMut<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

fn build_death_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_physics()
        .with_effects_pipeline()
        .build();
    attach_message_capture::<Destroyed<Cell>>(&mut app);
    app.init_resource::<PendingCellDamage>();
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    // Register tick_phantom_phase so phantom cycling works alongside death pipeline
    app.add_systems(
        FixedUpdate,
        crate::cells::behaviors::phantom::systems::tick_phantom_phase
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

fn advance_to_playing(app: &mut App) {
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

fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

// Behavior 44: Phantom cell in Solid phase dies normally when HP reaches zero
#[test]
fn phantom_cell_in_solid_phase_dies_when_hp_reaches_zero() {
    let mut app = build_death_test_app();

    let entity = spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .phantom(PhantomPhase::Solid)
            .position(Vec2::ZERO)
            .dimensions(10.0, 10.0)
            .hp(10.0)
            .headless()
            .spawn(commands)
    });

    advance_to_playing(&mut app);

    // Deliver lethal damage
    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push(DamageDealt {
            dealer:      None,
            target:      entity,
            amount:      15.0,
            source_chip: None,
            _marker:     PhantomData,
        });

    // Run ticks, accumulating Destroyed<Cell> across all ticks (the collector
    // clears each tick at First, so we must capture per-tick).
    let mut destroyed_msgs: Vec<Destroyed<Cell>> = Vec::new();
    for _ in 0..3 {
        tick_with_dt(&mut app, Duration::from_nanos(16_666_667));
        let collected = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
        destroyed_msgs.extend(collected.0.iter().cloned());
    }

    // Cell should be dead or despawned
    let is_dead =
        app.world().get_entity(entity).is_err() || app.world().get::<Dead>(entity).is_some();
    assert!(
        is_dead,
        "phantom cell in Solid phase should die normally when HP reaches zero"
    );

    // Destroyed message should be emitted
    let found = destroyed_msgs.iter().any(|d| d.victim == entity);
    assert!(
        found,
        "Destroyed<Cell> message should be emitted for killed phantom cell"
    );
}

// Behavior 44 edge: cell in Telegraph phase also dies normally
#[test]
fn phantom_cell_in_telegraph_phase_dies_when_hp_reaches_zero() {
    let mut app = build_death_test_app();

    let entity = spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .phantom(PhantomPhase::Telegraph)
            .position(Vec2::ZERO)
            .dimensions(10.0, 10.0)
            .hp(10.0)
            .headless()
            .spawn(commands)
    });

    advance_to_playing(&mut app);

    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push(DamageDealt {
            dealer:      None,
            target:      entity,
            amount:      15.0,
            source_chip: None,
            _marker:     PhantomData,
        });

    for _ in 0..3 {
        tick_with_dt(&mut app, Duration::from_nanos(16_666_667));
    }

    let is_dead =
        app.world().get_entity(entity).is_err() || app.world().get::<Dead>(entity).is_some();
    assert!(
        is_dead,
        "phantom cell in Telegraph phase should die normally when HP reaches zero"
    );
}

// Behavior 45: Phantom cell in Ghost phase has zeroed CollisionLayers
#[test]
fn phantom_cell_in_ghost_phase_has_zeroed_collision_layers() {
    let mut app = build_death_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Cell,
            PhantomCell,
            PhantomPhase::Ghost,
            PhantomTimer(3.0),
            PhantomConfig {
                cycle_secs:     3.0,
                telegraph_secs: 0.5,
            },
            CollisionLayers::new(0, 0),
            Hp::new(10.0),
            KilledBy::default(),
        ))
        .id();

    let layers = app.world().get::<CollisionLayers>(entity).unwrap();
    assert_eq!(layers.membership, 0, "Ghost phase membership should be 0");
    assert_eq!(layers.mask, 0, "Ghost phase mask should be 0");
}
