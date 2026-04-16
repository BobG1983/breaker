//! Tests for `salvo_breaker_collision` — behaviors 34-38.

use bevy::prelude::*;

use super::helpers::*;
use crate::{cells::messages::SalvoImpactBreaker, prelude::*};

// ── Behavior 34: Salvo overlapping breaker writes SalvoImpactBreaker and despawns ──

#[test]
fn salvo_overlapping_breaker_writes_impact_and_despawns() {
    let mut app = build_salvo_breaker_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(0.0, -200.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let breaker = spawn_collision_breaker(&mut app, Vec2::new(0.0, -200.0), Vec2::new(60.0, 10.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    // Check SalvoImpactBreaker message
    let collector = app
        .world()
        .resource::<MessageCollector<SalvoImpactBreaker>>();
    assert_eq!(
        collector.0.len(),
        1,
        "one SalvoImpactBreaker should be written"
    );
    let msg = &collector.0[0];
    assert_eq!(
        msg.salvo, salvo,
        "SalvoImpactBreaker.salvo should be the salvo entity"
    );
    assert_eq!(
        msg.breaker, breaker,
        "SalvoImpactBreaker.breaker should be the breaker entity"
    );

    // Salvo should be despawned
    assert!(
        app.world().get_entity(salvo).is_err(),
        "salvo should be despawned after hitting breaker"
    );
}

// ── Behavior 35: Breaker collision does NOT write DamageDealt<Breaker> ──

#[test]
fn breaker_collision_does_not_write_damage_dealt() {
    let mut app = build_salvo_breaker_collision_app();
    // Also capture DamageDealt<Cell> to verify no cross-contamination
    attach_message_capture::<DamageDealt<Cell>>(&mut app);

    let turret = app.world_mut().spawn_empty().id();
    let _salvo = spawn_salvo(
        &mut app,
        Vec2::new(0.0, -200.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let _breaker = spawn_collision_breaker(&mut app, Vec2::new(0.0, -200.0), Vec2::new(60.0, 10.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    // Only SalvoImpactBreaker should be written, NOT DamageDealt<Cell>
    let cell_damage = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert!(
        cell_damage.0.is_empty(),
        "breaker collision should not write DamageDealt<Cell>"
    );
}

// ── Behavior 36: Multiple salvos hitting breaker: all write messages and despawn ──

#[test]
fn multiple_salvos_hitting_breaker_all_despawn_and_write_messages() {
    let mut app = build_salvo_breaker_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo1 = spawn_salvo(
        &mut app,
        Vec2::new(0.0, -200.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let salvo2 = spawn_salvo(
        &mut app,
        Vec2::new(5.0, -200.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let _breaker = spawn_collision_breaker(&mut app, Vec2::new(0.0, -200.0), Vec2::new(60.0, 10.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<SalvoImpactBreaker>>();
    assert_eq!(
        collector.0.len(),
        2,
        "2 SalvoImpactBreaker messages should be written"
    );

    assert!(
        app.world().get_entity(salvo1).is_err(),
        "salvo1 should be despawned"
    );
    assert!(
        app.world().get_entity(salvo2).is_err(),
        "salvo2 should be despawned"
    );
}

// ── Behavior 37: Salvo not overlapping breaker: no message, salvo survives ──

#[test]
fn salvo_not_overlapping_breaker_survives() {
    let mut app = build_salvo_breaker_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(0.0, 200.0), // far from breaker
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let _breaker = spawn_collision_breaker(&mut app, Vec2::new(0.0, -200.0), Vec2::new(60.0, 10.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<SalvoImpactBreaker>>();
    assert!(
        collector.0.is_empty(),
        "no SalvoImpactBreaker when salvo doesn't overlap breaker"
    );

    assert!(
        app.world().get_entity(salvo).is_ok(),
        "salvo should survive when not overlapping breaker"
    );
}

// ── Behavior 38: No salvos or breaker: system is a no-op ──

#[test]
fn no_salvos_no_breaker_no_crash() {
    let mut app = build_salvo_breaker_collision_app();

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<SalvoImpactBreaker>>();
    assert!(collector.0.is_empty(), "no messages in empty world");
}

#[test]
fn salvos_exist_but_no_breaker_no_crash() {
    let mut app = build_salvo_breaker_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(0.0, -200.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<SalvoImpactBreaker>>();
    assert!(collector.0.is_empty(), "no messages when no breaker exists");
    assert!(
        app.world().get_entity(salvo).is_ok(),
        "salvo should survive when no breaker"
    );
}
