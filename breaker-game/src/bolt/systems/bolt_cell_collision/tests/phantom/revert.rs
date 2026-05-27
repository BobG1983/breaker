//! Group T13 — reverted phantom resumes normal cell rebound on next contact.
//!
//! Post-Wave-3B contract:
//! - Phantom reverts via `PhantomBolt::become_normal` → next cell contact uses the
//!   normal Reflect path.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::BaseSpeed;

use super::helpers::*;
use crate::{
    bolt::{
        components::{
            LastImpact, LifetimeEndBehavior, PhantomBolt, PhantomDamagedCells, PhantomDedupKey,
        },
        systems::bolt_cell_collision::tests::helpers::*,
    },
    prelude::*,
};

// ── T13 — Reverted phantom resumes normal cell rebound on next contact ─────

#[test]
fn reverted_phantom_bolt_resumes_normal_cell_rebound_on_next_contact() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_entity = spawn_cell_with_health(&mut app, 0.0, 100.0, 1000.0);

    // Spawn phantom, then revert BEFORE any collision
    let bolt = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut().entity_mut(bolt).insert(BaseSpeed(6000.0));

    // Sentinel: become_phantom stub must have inserted PhantomBolt — if absent,
    // the no-op stub false-passes all subsequent T13 assertions.
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_some(),
        "pre-condition: PhantomBolt must be present — become_phantom stub must have inserted it"
    );

    // Revert directly via world_mut — avoids depending on Wave 3A's lifespan tick
    app.world_mut().entity_mut(bolt).remove::<(
        PhantomBolt,
        PhantomDedupKey,
        PhantomDamagedCells,
        LifetimeEndBehavior,
    )>();

    // Pre-condition: no PhantomBolt marker
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "pre-condition: bolt must not have PhantomBolt marker before tick"
    );

    tick(&mut app);

    // Normal bolt must reflect (vy < 0 after bounce off cell top face)
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.y < 0.0,
        "reverted bolt must reflect off cell (vy < 0), got vy={}",
        vel.0.y
    );

    // LastImpact stamped (normal reflect path)
    assert!(
        app.world().get::<LastImpact>(bolt).is_some(),
        "reverted bolt must get LastImpact stamped"
    );

    // Damage emitted once
    let damages = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_entity)
        .count();
    assert_eq!(
        damages, 1,
        "reverted bolt must emit exactly one damage message"
    );

    // PhantomDamagedCells absent (removed by revert)
    assert!(
        app.world().get::<PhantomDamagedCells>(bolt).is_none(),
        "PhantomDamagedCells must be absent after revert"
    );

    // PhantomBolt absent
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "PhantomBolt must remain absent after revert"
    );
}

// ── T13 edge case 3a — phantom hits cell_x, reverts, then hits different cell_y normally ──

#[test]
fn reverted_phantom_hits_different_cell_with_normal_reflect() {
    let mut app = test_app_with_damage_and_wall_messages();

    // cell_x at x=200 — off the bolt's vertical (x=0) path during phase 2 rebound,
    // so no double-bounce back into it after reflecting off cell_y.
    let cell_x = spawn_cell_with_health(&mut app, 200.0, 100.0, 1000.0);
    let cell_y_entity = spawn_cell_with_health(&mut app, 0.0, 200.0, 1000.0);

    // Phase 1: phantom hits cell_x — bolt travels at angle to reach cell_x at x=200
    let bolt = spawn_phantom_bolt_for_cell_collision(&mut app, 200.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut().entity_mut(bolt).insert(BaseSpeed(6000.0));

    // Sentinel: become_phantom stub must have inserted PhantomBolt — bolt must be
    // phantom DURING phase 1's pierce tick; damage count alone doesn't catch a no-op stub.
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_some(),
        "pre-condition: PhantomBolt must be present — become_phantom stub must have inserted it"
    );

    tick(&mut app);

    let phantom_damage_count = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_x)
        .count();
    assert_eq!(phantom_damage_count, 1, "phantom phase: cell_x hit once");

    // Revert the bolt
    app.world_mut().entity_mut(bolt).remove::<(
        PhantomBolt,
        PhantomDedupKey,
        PhantomDamagedCells,
        LifetimeEndBehavior,
    )>();
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "bolt must be reverted before phase 2"
    );

    // Reposition to approach cell_y
    app.world_mut()
        .entity_mut(bolt)
        .insert(Position2D(Vec2::new(0.0, 178.0)));

    tick(&mut app);

    // Normal Reflect at cell_y
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.y < 0.0,
        "reverted bolt must reflect off cell_y, got vy={}",
        vel.0.y
    );

    // Two total damage messages (one phantom on cell_x, one normal on cell_y)
    let total_damage = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt))
        .count();
    assert_eq!(
        total_damage, 2,
        "total damage messages must be 2, got {total_damage}"
    );

    let cell_y_damage = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_y_entity)
        .count();
    assert_eq!(
        cell_y_damage, 1,
        "cell_y must receive exactly one normal-reflect damage"
    );
}

// ── T13 edge case 3b — phantom hits cell_x, reverts, re-enters cell_x normally ──

#[test]
fn reverted_phantom_re_enters_previously_damaged_cell_with_normal_reflect() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_x = spawn_cell_with_health(&mut app, 0.0, 100.0, 1000.0);

    // Phase 1: phantom hits cell_x (adds to PhantomDamagedCells)
    let bolt = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut().entity_mut(bolt).insert(BaseSpeed(6000.0));

    // Sentinel: become_phantom stub must have inserted PhantomBolt before phase 1 tick.
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_some(),
        "pre-condition: PhantomBolt must be present — become_phantom stub must have inserted it"
    );

    tick(&mut app);

    let phantom_count = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_x)
        .count();
    assert_eq!(phantom_count, 1, "phantom phase: one damage on cell_x");

    // Revert
    app.world_mut().entity_mut(bolt).remove::<(
        PhantomBolt,
        PhantomDedupKey,
        PhantomDamagedCells,
        LifetimeEndBehavior,
    )>();

    // Reposition back before cell_x
    app.world_mut()
        .entity_mut(bolt)
        .insert(Position2D(Vec2::new(0.0, 78.0)));

    tick(&mut app);

    // Normal Reflect at cell_x (dedup set was removed — fresh hit is allowed)
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.y < 0.0,
        "reverted bolt must reflect off cell_x, got vy={}",
        vel.0.y
    );
    assert!(
        app.world().get::<LastImpact>(bolt).is_some(),
        "LastImpact must be stamped on normal reflect"
    );

    // Two total damage messages on cell_x (one phantom + one normal)
    let total = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_x)
        .count();
    assert_eq!(
        total, 2,
        "cell_x must receive 2 damage messages (phantom + normal), got {total}"
    );
}

// ── T13 edge case 3c — become_normal called twice is idempotent ───────────

#[test]
fn become_normal_called_twice_is_idempotent_and_subsequent_collision_reflects() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_entity = spawn_cell_with_health(&mut app, 0.0, 100.0, 1000.0);

    let bolt = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut().entity_mut(bolt).insert(BaseSpeed(6000.0));

    // Sentinel: become_phantom stub must have inserted PhantomBolt — if absent,
    // both reverts are no-ops and the bolt was never phantom.
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_some(),
        "pre-condition: PhantomBolt must be present — become_phantom stub must have inserted it"
    );

    // First revert
    app.world_mut().entity_mut(bolt).remove::<(
        PhantomBolt,
        PhantomDedupKey,
        PhantomDamagedCells,
        LifetimeEndBehavior,
    )>();

    // Second revert (idempotent — removing absent components must not panic)
    app.world_mut().entity_mut(bolt).remove::<(
        PhantomBolt,
        PhantomDedupKey,
        PhantomDamagedCells,
        LifetimeEndBehavior,
    )>();

    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "PhantomBolt must remain absent after double revert"
    );

    tick(&mut app);

    // Bolt reflects normally
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.y < 0.0,
        "bolt after double-revert must reflect normally, got vy={}",
        vel.0.y
    );

    let damages = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_entity)
        .count();
    assert_eq!(
        damages, 1,
        "exactly one damage message on normal reflect after double-revert"
    );
}
