//! Group H — `bolt_cell_collision` pierce integration for entities
//! carrying `PhantomBolt` (Behaviors H1–H6).
//!
//! A bolt entity with the `PhantomBolt` marker deals damage but does NOT
//! reflect off a cell, does NOT consume `PiercingRemaining` charges, and
//! does NOT stamp `LastImpact`. Real bolts (no `PhantomBolt`) continue to
//! behave exactly as before.
//!
//! These tests pin the cross-domain integration: writer-code adds a
//! `Query<(), With<PhantomBolt>>` check to `bolt_cell_collision` before
//! the `can_pierce && would_destroy` branch, promoting the entity to
//! "pierce always" when the marker is present.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{BaseSpeed, GlobalPosition2D, Spatial2D};

use super::helpers::*;
use crate::{
    bolt::{
        components::{BoltBaseDamage, LastImpact, PiercingRemaining},
        test_utils::piercing_stack,
    },
    cells::resources::CellConfig,
    effect_v3::effects::phantom_bolt::components::{PhantomBolt, PhantomLifetime, PhantomOwner},
    prelude::*,
    shared::size::BaseRadius,
};

// ── Phantom spawner ─────────────────────────────────────────────────────────

/// Spawns an entity that is itself a full bolt, carrying the `PhantomBolt`
/// marker + lifetime + owner. Uses the canonical afterimage-spawn bundle
/// (including `CELL_LAYER` in the mask — which is the diff vs.
/// `SpawnPhantomConfig::fire`).
fn spawn_phantom_bolt_for_cell_collision(
    app: &mut App,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    owner: Entity,
    radius: f32,
) -> Entity {
    let pos = Vec2::new(x, y);
    let velocity = Vec2::new(vx, vy);
    app.world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomLifetime(3.0),
            PhantomOwner(owner),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            Velocity2D(velocity),
            // `BaseSpeed` is required by the `SpatialData` subquery of
            // `BoltCollisionData` — without it the phantom is filtered out of
            // the bolt collision system entirely. Mirror the real-bolt
            // afterimage spawn: use the spawn-time velocity magnitude so the
            // phantom's `apply_velocity_formula` pass preserves the intended
            // travel speed for the CCD loop.
            BaseSpeed(velocity.length()),
            BoltBaseDamage(10.0),
            BaseRadius(radius),
            CollisionLayers::new(
                BOLT_LAYER,
                BOLT_LAYER | WALL_LAYER | BREAKER_LAYER | CELL_LAYER,
            ),
        ))
        .id()
}

// ── H1 — phantom deals damage but does NOT reflect ────────────────────────

#[test]
fn phantom_bolt_entity_deals_damage_but_does_not_reflect_off_cell() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();
    let owner = app.world_mut().spawn_empty().id();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, owner, bc.radius);

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "phantom MUST NOT be reflected — velocity.y should remain upward, got {}",
        velocity.0.y
    );

    let damages: Vec<_> = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom))
        .collect();
    assert_eq!(
        damages.len(),
        1,
        "exactly one DamageDealt<Cell> from the phantom, got {}",
        damages.len()
    );
    assert!(
        (damages[0].amount - 10.0).abs() < 1e-4,
        "damage amount expected 10.0 (base damage), got {}",
        damages[0].amount
    );
}

// ── H1 (edge case) — phantom does NOT get LastImpact stamped ──────────────

#[test]
fn phantom_bolt_entity_does_not_get_last_impact_stamped() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();
    let owner = app.world_mut().spawn_empty().id();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);
    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, owner, bc.radius);

    tick(&mut app);

    assert!(
        app.world().get::<LastImpact>(phantom).is_none(),
        "phantom MUST NOT have LastImpact stamped (pierce-through semantics)"
    );
}

// ── H2 — phantom pierces a cell it would NOT destroy ──────────────────────

#[test]
fn phantom_bolt_pierces_cell_it_would_not_destroy() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();
    let owner = app.world_mut().spawn_empty().id();

    let cell_y = 100.0;
    // 100 Hp — damage 10 would NOT kill → normal pierce gate reflects.
    // PhantomBolt marker must override.
    spawn_cell_with_health(&mut app, 0.0, cell_y, 100.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, owner, bc.radius);

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "PhantomBolt marker must override would-destroy requirement — \
         phantom must pierce even high-Hp cells. Got velocity.y={}",
        velocity.0.y
    );

    let damages: Vec<_> = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom))
        .collect();
    assert_eq!(damages.len(), 1, "exactly one damage message emitted");
    assert!(
        (damages[0].amount - 10.0).abs() < 1e-4,
        "damage amount 10.0 (unchanged), got {}",
        damages[0].amount
    );
}

// ── H3 — phantom pierces multiple cells in one frame ─────────────────────-

#[test]
fn phantom_bolt_pierces_multiple_cells_in_one_frame() {
    let mut app = test_app_with_damage_and_wall_messages();
    let owner = app.world_mut().spawn_empty().id();

    let cell_a = spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);
    let cell_b = spawn_cell_with_health(&mut app, 0.0, 130.0, 30.0);

    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 50.0, 0.0, 6000.0, owner, 6.0);

    tick(&mut app);

    let damages: Vec<_> = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom))
        .collect();
    let targets: Vec<Entity> = damages.iter().map(|d| d.target).collect();
    assert_eq!(
        damages.len(),
        2,
        "expected two DamageDealt<Cell> messages, got {}",
        damages.len()
    );
    assert!(
        targets.contains(&cell_a),
        "must target cell_a, got {targets:?}"
    );
    assert!(
        targets.contains(&cell_b),
        "must target cell_b, got {targets:?}"
    );

    let velocity = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "phantom velocity direction preserved through pierces, got {}",
        velocity.0.y
    );
    let position = app.world().get::<Position2D>(phantom).unwrap();
    assert!(
        position.0.y > 130.0 - 6.0,
        "phantom must travel past cell_b (y > 130.0 - radius), got y={}",
        position.0.y
    );
}

// ── H4 — phantom does NOT consume PiercingRemaining charges ───────────────

#[test]
fn phantom_bolt_does_not_consume_piercing_remaining_charges() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();
    let owner = app.world_mut().spawn_empty().id();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, owner, bc.radius);
    // Attach PiercingRemaining + piercing_stack.
    app.world_mut()
        .entity_mut(phantom)
        .insert((piercing_stack(&[3]), PiercingRemaining(3)));

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(phantom)
        .expect("PiercingRemaining must still be present");
    assert_eq!(
        pr.0, 3,
        "PhantomBolt pierce must NOT consume PiercingRemaining charges, got {}",
        pr.0
    );
    let velocity = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "phantom must not reflect even with PiercingRemaining, got velocity.y={}",
        velocity.0.y
    );
}

// ── H4 (edge case) — phantom with PiercingRemaining(0) still pierces ──────

#[test]
fn phantom_bolt_with_zero_piercing_remaining_still_pierces() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();
    let owner = app.world_mut().spawn_empty().id();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, owner, bc.radius);
    app.world_mut()
        .entity_mut(phantom)
        .insert((piercing_stack(&[]), PiercingRemaining(0)));

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "phantom pierce is unconditional on marker — charges irrelevant. \
         Got velocity.y={}",
        velocity.0.y
    );
}

// ── H5 — real bolt (no PhantomBolt marker) reflects normally ──────────────

#[test]
fn real_bolt_without_phantom_marker_reflects_off_cell() {
    let mut app = test_app();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y < 0.0,
        "real bolt (no PhantomBolt marker) must reflect — regression guard. \
         Got velocity.y={}",
        vel.0.y
    );
    let hits = app.world().resource::<HitCells>();
    assert_eq!(hits.0.len(), 1, "BoltImpactCell must still be emitted");
}

// ── H6 — real + phantom co-existing: real reflects, phantom pierces ───────

#[test]
fn real_and_phantom_bolt_co_exist_real_reflects_phantom_pierces() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell = spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);

    let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;
    let real = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, real, bc.radius);

    tick(&mut app);

    let real_vel = app.world().get::<Velocity2D>(real).unwrap();
    assert!(
        real_vel.0.y < 0.0,
        "real bolt must reflect, got velocity.y={}",
        real_vel.0.y
    );
    let phantom_vel = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        phantom_vel.0.y > 0.0,
        "phantom bolt must pierce, got velocity.y={}",
        phantom_vel.0.y
    );

    let damages: Vec<_> = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|d| d.target == cell)
        .collect();
    assert_eq!(
        damages.len(),
        2,
        "both bolts must hit the cell — two DamageDealt<Cell>, got {}",
        damages.len()
    );

    assert!(
        app.world().get_entity(real).is_ok(),
        "real bolt still alive"
    );
    assert!(
        app.world().get_entity(phantom).is_ok(),
        "phantom bolt still alive"
    );
}

// ── H6 (edge case) — real gets LastImpact stamped, phantom does NOT ───────

#[test]
fn real_bolt_gets_last_impact_but_phantom_does_not() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);

    let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;
    let real = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, real, bc.radius);

    tick(&mut app);

    assert!(
        app.world().get::<LastImpact>(real).is_some(),
        "real bolt must get LastImpact stamped on reflect"
    );
    assert!(
        app.world().get::<LastImpact>(phantom).is_none(),
        "phantom bolt must NOT get LastImpact stamped"
    );
}
