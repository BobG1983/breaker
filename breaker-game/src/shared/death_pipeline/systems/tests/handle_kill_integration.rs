//! Plugin-integration tests for `DeathPipelinePlugin` + `EffectV3Plugin`
//! verifying that `handle_kill<Cell>` and `handle_kill<Bolt>` are wired and
//! produce `Destroyed<T>` + `DespawnEntity` + actual despawn end-to-end
//! (Behaviors 13-14). The `Dead` insertion is covered by the unit tests
//! (Behavior 1) — it cannot be asserted here because a single `tick()` runs
//! both `FixedUpdate` (which inserts `Dead`) and `FixedPostUpdate` (which
//! despawns the entity via `process_despawn_requests`), so by the time the
//! assertion runs, the entity is gone.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{
    PendingBoltKills, PendingCellKills, PendingWallKills, attach_bolt_destroyed_collector,
    attach_cell_destroyed_collector, attach_despawn_collector, attach_wall_destroyed_collector,
    build_plugin_integration_app, enqueue_bolt_kills, enqueue_cell_kills, enqueue_wall_kills,
};
use crate::{
    bolt::components::Bolt,
    cells::components::Cell,
    shared::{
        death_pipeline::{
            despawn_entity::DespawnEntity, destroyed::Destroyed, hp::Hp,
            kill_yourself::KillYourself, killed_by::KilledBy, sets::DeathPipelineSystems,
        },
        test_utils::{MessageCollector, tick},
    },
    walls::components::Wall,
};

// ── Behavior 13: Cell plugin integration ────────────────────────────────

#[test]
fn plugin_integration_kill_yourself_cell_produces_dead_destroyed_despawn() {
    let mut app = build_plugin_integration_app();
    attach_cell_destroyed_collector(&mut app);
    attach_despawn_collector(&mut app);

    app.init_resource::<PendingCellKills>();
    app.add_systems(
        FixedUpdate,
        enqueue_cell_kills.before(DeathPipelineSystems::HandleKill),
    );

    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(1.0),
            KilledBy::default(),
            Position2D(Vec2::new(100.0, 200.0)),
        ))
        .id();

    app.insert_resource(PendingCellKills(vec![KillYourself::<Cell> {
        victim:  cell,
        killer:  None,
        _marker: PhantomData,
    }]));

    tick(&mut app);

    // `Dead` insertion is verified by the unit tests (Behavior 1). Here the
    // full pipeline runs in one tick: handle_kill inserts `Dead` and writes
    // `DespawnEntity` in `FixedUpdate`, then `process_despawn_requests` runs
    // in `FixedPostUpdate` and despawns the entity before this assertion.

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "exactly one Destroyed<Cell> should be emitted"
    );
    let msg = &destroyed.0[0];
    assert_eq!(msg.victim, cell);
    assert_eq!(msg.victim_pos, Vec2::new(100.0, 200.0));
    assert_eq!(msg.killer, None);
    assert_eq!(msg.killer_pos, None);

    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawns.0.len(),
        1,
        "exactly one DespawnEntity should be emitted"
    );
    assert_eq!(despawns.0[0].entity, cell);

    assert!(
        app.world().get_entity(cell).is_err(),
        "Cell entity should be despawned by process_despawn_requests within the same tick"
    );
}

// ── Behavior 14: Bolt plugin integration ────────────────────────────────

#[test]
fn plugin_integration_kill_yourself_bolt_produces_dead_destroyed_despawn() {
    let mut app = build_plugin_integration_app();
    attach_bolt_destroyed_collector(&mut app);
    attach_despawn_collector(&mut app);

    app.init_resource::<PendingBoltKills>();
    app.add_systems(
        FixedUpdate,
        enqueue_bolt_kills.before(DeathPipelineSystems::HandleKill),
    );

    // Bolt does NOT have #[require(Spatial2D)], so Position2D MUST be in the bundle.
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Hp::new(1.0),
            KilledBy::default(),
            Position2D(Vec2::new(0.0, -50.0)),
        ))
        .id();

    app.insert_resource(PendingBoltKills(vec![KillYourself::<Bolt> {
        victim:  bolt,
        killer:  None,
        _marker: PhantomData,
    }]));

    tick(&mut app);

    // `Dead` insertion is verified by the unit tests (Behavior 1). Here the
    // full pipeline runs in one tick: handle_kill inserts `Dead` and writes
    // `DespawnEntity` in `FixedUpdate`, then `process_despawn_requests` runs
    // in `FixedPostUpdate` and despawns the entity before this assertion.

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Bolt>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "exactly one Destroyed<Bolt> should be emitted"
    );
    let msg = &destroyed.0[0];
    assert_eq!(msg.victim, bolt);
    assert_eq!(msg.victim_pos, Vec2::new(0.0, -50.0));
    assert_eq!(msg.killer, None);

    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawns.0.len(),
        1,
        "exactly one DespawnEntity should be emitted"
    );
    assert_eq!(despawns.0[0].entity, bolt);

    assert!(
        app.world().get_entity(bolt).is_err(),
        "Bolt entity should be despawned by process_despawn_requests within the same tick"
    );
}

// ── Behavior 17: Wall plugin integration (F1 extension) ────────────────

#[test]
fn plugin_integration_kill_yourself_wall_produces_destroyed_despawn() {
    let mut app = build_plugin_integration_app();
    attach_wall_destroyed_collector(&mut app);
    attach_despawn_collector(&mut app);

    app.init_resource::<PendingWallKills>();
    app.add_systems(
        FixedUpdate,
        enqueue_wall_kills.before(DeathPipelineSystems::HandleKill),
    );

    // Wall has #[require(Spatial2D)], but Position2D must still be inserted
    // explicitly because `handle_kill<T>` reads `Position2D` from the victim.
    let wall = app
        .world_mut()
        .spawn((
            Wall,
            Hp::new(1.0),
            KilledBy::default(),
            Position2D(Vec2::new(0.0, 0.0)),
        ))
        .id();

    app.insert_resource(PendingWallKills(vec![KillYourself::<Wall> {
        victim:  wall,
        killer:  None,
        _marker: PhantomData,
    }]));

    tick(&mut app);

    // `Dead` insertion is verified by the unit tests (Behavior 1). Here the
    // full pipeline runs in one tick: handle_kill inserts `Dead` and writes
    // `DespawnEntity` in `FixedUpdate`, then `process_despawn_requests` runs
    // in `FixedPostUpdate` and despawns the entity before this assertion.

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Wall>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "exactly one Destroyed<Wall> should be emitted"
    );
    let msg = &destroyed.0[0];
    assert_eq!(msg.victim, wall);
    assert_eq!(msg.victim_pos, Vec2::new(0.0, 0.0));
    assert_eq!(msg.killer, None);
    assert_eq!(msg.killer_pos, None);

    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawns.0.len(),
        1,
        "exactly one DespawnEntity should be emitted"
    );
    assert_eq!(despawns.0[0].entity, wall);

    assert!(
        app.world().get_entity(wall).is_err(),
        "Wall entity should be despawned by process_despawn_requests within the same tick"
    );
}
