//! Group C — Damage gating via `reset_inactive_sequence_hp`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::components::SequenceActive, prelude::*,
    shared::death_pipeline::kill_yourself::KillYourself,
};

// Behavior 10

#[test]
fn active_sequence_cell_takes_normal_damage() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 5.0));
    tick(&mut app);

    let hp = app.world().get::<Hp>(e0).expect("e0 should still have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "active e0 should have hp.current == 15.0, got {}",
        hp.current
    );
    assert!(app.world().get::<Dead>(e0).is_none());
    assert!(app.world().get_entity(e0).is_ok());
}

// Behavior 10 edge: two ticks of 5.0 damage each
#[test]
fn active_sequence_cell_accumulates_damage_over_two_ticks() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 5.0));
    tick(&mut app);
    push_damage(&mut app, damage_msg(e0, 5.0));
    tick(&mut app);

    let hp = app.world().get::<Hp>(e0).expect("e0 should still have Hp");
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "active e0 should accumulate to hp.current == 10.0 after two 5.0 hits, got {}",
        hp.current
    );
    assert!(app.world().get::<Dead>(e0).is_none());
}

// Behavior 11

#[test]
fn non_active_sequence_cell_has_hp_restored_after_damage() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e1, 5.0));
    tick(&mut app);

    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (e1_hp.current - 20.0).abs() < f32::EPSILON,
        "non-active e1 should be reset to 20.0 after damage, got {}",
        e1_hp.current
    );
    assert!(app.world().get::<Dead>(e1).is_none());
    assert!(app.world().get_entity(e1).is_ok());

    // e0 untouched.
    let e0_hp = app.world().get::<Hp>(e0).expect("e0 should have Hp");
    assert!((e0_hp.current - 20.0).abs() < f32::EPSILON);
    assert!(
        app.world().get::<SequenceActive>(e0).is_some(),
        "e0 should still be active — reset must not disturb active cells"
    );
}

// Behavior 11 edge: Hp.max is honored as the reset ceiling
#[test]
fn non_active_cell_reset_uses_hp_max_unwrap_or_starting() {
    let mut app = build_sequence_test_app();

    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell_with_max(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0, 30.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e1, 5.0));
    tick(&mut app);

    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (e1_hp.current - 30.0).abs() < f32::EPSILON,
        "reset ceiling must be Hp.max.unwrap_or(starting) == 30.0, got {}",
        e1_hp.current
    );
    assert!(app.world().get::<Dead>(e1).is_none());
}

// Behavior 12

#[test]
fn non_active_sequence_cell_survives_lethal_damage() {
    let mut app = build_sequence_test_app();

    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e1, 25.0));
    tick(&mut app);

    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (e1_hp.current - 20.0).abs() < f32::EPSILON,
        "e1 should be reset to 20.0 after lethal damage, got {}",
        e1_hp.current
    );
    assert!(app.world().get::<Dead>(e1).is_none());
    assert!(app.world().get_entity(e1).is_ok());

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    let e1_victims = destroyed.0.iter().filter(|m| m.victim == e1).count();
    assert_eq!(
        e1_victims, 0,
        "reset must run before DetectDeaths so no Destroyed<Cell> is written for e1"
    );
}

// Behavior 12 edge: damage exactly equal to starting HP
#[test]
fn non_active_cell_survives_damage_exactly_equal_to_starting_hp() {
    let mut app = build_sequence_test_app();

    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e1, 20.0));
    tick(&mut app);

    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (e1_hp.current - 20.0).abs() < f32::EPSILON,
        "e1 should be reset from 0.0 back to 20.0, got {}",
        e1_hp.current
    );
    assert!(app.world().get::<Dead>(e1).is_none());
}

// Behavior 12 edge: `KilledBy.dealer` is cleared by the reset
#[test]
fn reset_clears_killed_by_dealer_on_non_active_cell() {
    let mut app = build_sequence_test_app();

    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let bolt_entity = app.world_mut().spawn_empty().id();
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg_from(e1, 25.0, bolt_entity));
    tick(&mut app);

    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (e1_hp.current - 20.0).abs() < f32::EPSILON,
        "e1 should be reset to 20.0, got {}",
        e1_hp.current
    );

    let killed_by = app
        .world()
        .get::<KilledBy>(e1)
        .expect("e1 should still have KilledBy");
    assert!(
        killed_by.dealer.is_none(),
        "reset must clear KilledBy.dealer back to None, got {:?}",
        killed_by.dealer
    );
    assert!(app.world().get::<Dead>(e1).is_none());

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    let e1_victims = destroyed.0.iter().filter(|m| m.victim == e1).count();
    assert_eq!(e1_victims, 0);
}

// Behavior 12 edge: positive-path regression — next dealer recorded correctly
#[test]
fn legitimate_killing_blow_records_new_dealer_after_reset_clears_stale_dealer() {
    let mut app = build_sequence_test_app();

    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    let bolt_a = app.world_mut().spawn_empty().id();
    let bolt_b = app.world_mut().spawn_empty().id();
    advance_to_playing(&mut app);

    // Tick 1: lethal damage on e1 from bolt_a while e1 is non-active.
    push_damage(&mut app, damage_msg_from(e1, 25.0, bolt_a));
    tick(&mut app);

    // Sanity: reset restored HP and cleared the dealer.
    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!((e1_hp.current - 20.0).abs() < f32::EPSILON);
    assert!(
        app.world().get::<KilledBy>(e1).unwrap().dealer.is_none(),
        "reset should have cleared KilledBy.dealer after tick 1"
    );

    // Promote e1 to active manually so a legitimate killing blow can land.
    app.world_mut().entity_mut(e1).insert(SequenceActive);

    // Tick 2: lethal damage on e1 from a DIFFERENT dealer.
    push_damage(&mut app, damage_msg_from(e1, 25.0, bolt_b));
    tick(&mut app);

    // e1 is dead/despawned and the killer recorded is bolt_b, not bolt_a.
    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    let e1_destroyed = destroyed
        .0
        .iter()
        .find(|m| m.victim == e1)
        .expect("e1 should be in Destroyed<Cell> after the legitimate killing blow");
    assert_eq!(
        e1_destroyed.killer,
        Some(bolt_b),
        "the recorded killer must be bolt_b (new), not bolt_a (stale)"
    );
}

// Behavior 13

#[test]
fn non_sequence_cell_takes_damage_normally() {
    let mut app = build_sequence_test_app();

    let p = spawn_plain_cell(&mut app, Vec2::new(0.0, 0.0), 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(p, 5.0));
    tick(&mut app);

    let hp = app.world().get::<Hp>(p).expect("p should still have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "plain cell should take damage normally, got hp.current == {}",
        hp.current
    );
    assert!(app.world().get::<Dead>(p).is_none());
    assert!(app.world().get_entity(p).is_ok());
}

// Behavior 13 edge: lethal damage on a plain cell kills it
#[test]
fn non_sequence_cell_dies_from_lethal_damage() {
    let mut app = build_sequence_test_app();

    let p = spawn_plain_cell(&mut app, Vec2::new(0.0, 0.0), 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(p, 25.0));
    let destroyed_msgs = run_ticks_capture_destroyed(&mut app, 1);

    assert!(
        destroyed_msgs.iter().any(|m| m.victim == p),
        "plain cell should be in the Destroyed<Cell> set after lethal damage"
    );
}

// Behavior 14

#[test]
fn multiple_damage_messages_in_one_tick_still_reset_non_active_cell() {
    let mut app = build_sequence_test_app();

    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e1, 5.0));
    push_damage(&mut app, damage_msg(e1, 5.0));
    push_damage(&mut app, damage_msg(e1, 5.0));
    tick(&mut app);

    let hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "three 5.0 hits should be reverted in the same tick, got hp.current == {}",
        hp.current
    );
    assert!(app.world().get::<Dead>(e1).is_none());
}

// Behavior 14 edge: mix of active + non-active targets in the same tick
#[test]
fn active_and_non_active_cells_are_selectively_reset_in_one_tick() {
    let mut app = build_sequence_test_app();

    let e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e0, 5.0));
    push_damage(&mut app, damage_msg(e1, 5.0));
    tick(&mut app);

    let e0_hp = app.world().get::<Hp>(e0).unwrap();
    let e1_hp = app.world().get::<Hp>(e1).unwrap();
    assert!(
        (e0_hp.current - 15.0).abs() < f32::EPSILON,
        "active e0 should keep damage, got {}",
        e0_hp.current
    );
    assert!(
        (e1_hp.current - 20.0).abs() < f32::EPSILON,
        "non-active e1 should be reset to 20.0, got {}",
        e1_hp.current
    );
}

// Behavior 15

#[test]
fn lethal_damage_on_non_active_cell_does_not_emit_kill_yourself() {
    let mut app = build_sequence_test_app();
    attach_message_capture::<KillYourself<Cell>>(&mut app);

    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e1, 25.0));
    tick(&mut app);

    let kill_yourself = app
        .world()
        .resource::<MessageCollector<KillYourself<Cell>>>();
    let e1_kills = kill_yourself.0.iter().filter(|m| m.victim == e1).count();
    assert_eq!(
        e1_kills, 0,
        "no KillYourself<Cell> should be emitted for a non-active cell after reset"
    );
}

// Behavior 15 edge: three consecutive lethal ticks are all reverted
#[test]
fn three_consecutive_lethal_ticks_all_reset_non_active_cell() {
    let mut app = build_sequence_test_app();
    attach_message_capture::<KillYourself<Cell>>(&mut app);

    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    for _ in 0..3 {
        push_damage(&mut app, damage_msg(e1, 25.0));
        tick(&mut app);
        let hp = app.world().get::<Hp>(e1).unwrap();
        assert!((hp.current - 20.0).abs() < f32::EPSILON);
    }
    assert!(app.world().get::<Dead>(e1).is_none());
    assert!(app.world().get_entity(e1).is_ok());
}

// Behavior 15a
//
// Regression guard: the production implementation must gate the reset write
// behind `if hp.current < hp.starting`. To detect a buggy variant that removes
// the guard and unconditionally writes `hp.current = ceiling`, the test uses
// a non-active cell at full health (`current == starting == 20.0`) and
// pre-seeds `killed_by.dealer` with a fake entity. An unguarded implementation
// that unconditionally clears `killed_by.dealer = None` would observably clear
// the seeded dealer; the guarded implementation short-circuits at
// `if hp.current < hp.starting` and leaves both fields untouched.

#[test]
fn reset_is_noop_on_idle_non_active_cell_at_ceiling() {
    let mut app = build_sequence_test_app();

    // starting = 20.0, max = None → ceiling = 20.0, current = 20.0 at spawn
    // (genuinely "at ceiling").
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 1, 20.0);
    // Pre-seed a fake dealer so an unguarded reset would observably clear it.
    let fake_dealer = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(e1).insert(KilledBy {
        dealer: Some(fake_dealer),
    });
    advance_to_playing(&mut app);

    tick(&mut app);

    let hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "idle non-active cell must NOT be written — guard `if hp.current < hp.starting` should short-circuit, got {}",
        hp.current
    );
    let killed_by = app.world().get::<KilledBy>(e1).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(fake_dealer),
        "guarded implementation must NOT clear killed_by.dealer on idle non-active cells — an unguarded impl would land None here",
    );
    assert!(app.world().get::<Dead>(e1).is_none());
}

// Behavior 15a edge: ten idle ticks in a row.
//
// Same regression guard as the single-tick case: seeds `killed_by.dealer` so
// an unguarded implementation is observably detectable via dealer clearing.
#[test]
fn reset_is_noop_across_ten_idle_ticks() {
    let mut app = build_sequence_test_app();

    let e1 = spawn_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 1, 20.0);
    let fake_dealer = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(e1).insert(KilledBy {
        dealer: Some(fake_dealer),
    });
    advance_to_playing(&mut app);

    for i in 0..10 {
        tick(&mut app);
        let hp = app.world().get::<Hp>(e1).unwrap();
        assert!(
            (hp.current - 20.0).abs() < f32::EPSILON,
            "idle tick {i}: current must remain 20.0 (guard active), got {}",
            hp.current
        );
        let killed_by = app.world().get::<KilledBy>(e1).unwrap();
        assert_eq!(
            killed_by.dealer,
            Some(fake_dealer),
            "idle tick {i}: guarded impl must NOT clear killed_by.dealer",
        );
    }
}

// Behavior 15a regression: idle cell with `max > starting` must NOT be healed.
//
// The guard is `hp.current < hp.starting`, NOT `hp.current < ceiling`. An
// undamaged cell spawns with `current == starting`, and `apply_damage::<Cell>`
// is the only system that lowers `current`, so `current < starting` is the
// exact predicate for "was hit this tick." A buggy variant that uses `ceiling`
// as the guard would fire on an idle cell whose `max` exceeds `starting`,
// giving the cell a free heal to ceiling every tick and clearing
// `killed_by.dealer` spuriously.
#[test]
fn reset_is_noop_on_idle_cell_with_max_above_starting() {
    let mut app = build_sequence_test_app();

    // starting = 20.0, max = Some(30.0) → ceiling = 30.0, current = 20.0 at
    // spawn. Guarded impl: `20 < 20 = false`, no reset. Buggy impl using
    // `ceiling`: `20 < 30 = true`, resets hp.current to 30 and clears dealer.
    let e1 = spawn_sequence_cell_with_max(&mut app, Vec2::new(0.0, 0.0), 1, 1, 20.0, 30.0);
    let fake_dealer = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(e1).insert(KilledBy {
        dealer: Some(fake_dealer),
    });
    advance_to_playing(&mut app);

    tick(&mut app);

    let hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "idle cell with max > starting must NOT be healed — guard must use `< starting`, not `< ceiling`. got {}",
        hp.current
    );
    assert!(
        (hp.current - 30.0).abs() > f32::EPSILON,
        "must not land at ceiling (30.0) when no damage was applied, got {}",
        hp.current
    );
    let killed_by = app.world().get::<KilledBy>(e1).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(fake_dealer),
        "idle cell with max > starting must NOT have killed_by.dealer cleared",
    );
}

// Behavior 15a regression edge: damaged cell with `max > starting` heals to `max`.
//
// Confirms the OTHER half of the fix: when the guard fires (cell actually
// took damage this tick), the reset target is still `ceiling = max`, not
// just `starting`. A damaged non-active sequence cell should be restored to
// its healing ceiling, which matches the "heal to full" design intent.
#[test]
fn damaged_non_active_cell_with_max_above_starting_heals_to_max() {
    let mut app = build_sequence_test_app();

    let e1 = spawn_sequence_cell_with_max(&mut app, Vec2::new(0.0, 0.0), 1, 1, 20.0, 30.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(e1, 5.0));
    tick(&mut app);

    let hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (hp.current - 30.0).abs() < f32::EPSILON,
        "damaged non-active cell must be restored to ceiling (max = 30.0), got {}",
        hp.current
    );
}
