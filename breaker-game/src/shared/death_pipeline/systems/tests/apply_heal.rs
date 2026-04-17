//! Tests for `apply_heal<T>`.
//!
//! Covers Groups A–H (Behaviors 1–25), plus Behavior 31 (per-T queue isolation
//! with `Salvo` + `TestEntity` side-by-side) and Behavior 35 (`HealCap`
//! exhaustive compile-time match) from the heal-pipeline test spec.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::helpers::{
    PendingHeal, TestEntity, build_apply_heal_app, heal_msg, spawn_test_entity,
    spawn_test_entity_dead,
};
use crate::{
    cells::behaviors::survival::salvo::components::Salvo,
    prelude::*,
    shared::death_pipeline::{
        heal_dealt::{HealCap, HealDealt},
        systems::apply_heal,
    },
};

// ── Behavior 35: HealCap exhaustive compile-time match ─────────────────────

/// Compile-time pin — `HealCap` has exactly `Starting` and `Max` variants.
/// Adding a new variant without updating this match breaks the build.
const _: fn(HealCap) = |cap| match cap {
    HealCap::Starting | HealCap::Max => (),
};

// ── Group A: Baseline heal semantics ───────────────────────────────────────

/// Behavior 1: heal increases Hp.current by amount.
#[test]
fn heal_increases_hp_current_by_amount() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 8.0).abs() < f32::EPSILON,
        "Hp should be 8.0 after 3.0 heal on 5.0-HP entity, got {}",
        hp.current
    );
    assert!(
        (hp.starting - 10.0).abs() < f32::EPSILON,
        "starting should remain 10.0, got {}",
        hp.starting
    );
    assert!(hp.max.is_none(), "max should remain None, got {:?}", hp.max);
}

/// Behavior 1 edge: heal that lands exactly on the ceiling.
#[test]
fn heal_lands_exactly_on_ceiling_starting_fallback() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should be 10.0 (exactly at ceiling) after 7.0+3.0, got {}",
        hp.current
    );
}

/// Behavior 2: multiple heals in a single tick accumulate (same cap variant).
#[test]
fn multiple_heals_same_tick_accumulate_same_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 30.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 4.0, HealCap::Max),
        heal_msg(entity, 6.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "Hp should be 15.0 after 5 + 4 + 6, got {}",
        hp.current
    );
}

/// Behavior 2 edge: three heals reach ceiling (clamp engages across messages).
#[test]
fn multiple_heals_clamp_engages_across_messages() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  27.0,
                starting: 30.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 2.0, HealCap::Max),
        heal_msg(entity, 3.0, HealCap::Max),
        heal_msg(entity, 4.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 30.0).abs() < f32::EPSILON,
        "Hp should clamp at 30.0 (ceiling), got {}",
        hp.current
    );
}

/// Behavior 3: max = None + `HealCap::Max` falls back to starting.
#[test]
fn heal_max_cap_falls_back_to_starting_when_max_is_none() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  9.5,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should clamp at starting (10.0) when max is None, got {}",
        hp.current
    );
}

/// Behavior 3 edge: already at starting with max=None — no overflow past.
#[test]
fn heal_at_starting_no_overflow_when_max_none() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  10.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 1.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should remain 10.0 (no overflow), got {}",
        hp.current
    );
}

// ── Group B: Ceiling clamping per HealCap variant ──────────────────────────

/// Behavior 4: `HealCap::Max` clamps at Hp.max when Some.
#[test]
fn heal_max_cap_clamps_at_hp_max_when_some() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  18.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Hp should clamp at max=20.0, got {}",
        hp.current
    );
}

/// Behavior 4 edge: already at max, heal is a no-op.
#[test]
fn heal_max_cap_at_max_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  20.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Hp should remain 20.0 (no-op at ceiling), got {}",
        hp.current
    );
}

/// Behavior 5: `HealCap::Max` falls back to starting with huge amount.
#[test]
fn heal_max_cap_huge_amount_falls_back_to_starting() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  8.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 500.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should clamp at starting (10.0) — NOT unbounded, got {}",
        hp.current
    );
    assert!(
        hp.current.is_finite(),
        "Hp should not be infinite, got {}",
        hp.current
    );
}

/// Behavior 5 edge: 1e9 heal on tiny current, max=None.
#[test]
fn heal_max_cap_1e9_amount_clamps_at_starting() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  0.5,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 1.0e9, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should clamp at starting (10.0), got {}",
        hp.current
    );
}

/// Behavior 6: `HealCap::Max` with max below starting — ceiling honoured verbatim.
#[test]
fn heal_max_cap_with_max_below_starting_honours_max() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      Some(5.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 100.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Hp should clamp at max=5.0 (even though starting=10.0), got {}",
        hp.current
    );
}

/// Behavior 6 edge: at max=5.0 already — no-op.
#[test]
fn heal_max_cap_at_depressed_max_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      Some(5.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 1.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Hp should remain 5.0 (already at max), got {}",
        hp.current
    );
}

/// Behavior 7: `HealCap::Starting` clamps at starting even when max > starting.
#[test]
fn heal_starting_cap_clamps_at_starting_ignoring_higher_max() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  8.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        500.0,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Starting cap must IGNORE max and clamp at starting=10.0, got {}",
        hp.current
    );
}

/// Behavior 7 edge: current at starting — Starting cap no-op.
#[test]
fn heal_starting_cap_at_starting_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  10.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Starting cap should no-op at starting, got {}",
        hp.current
    );
}

/// Behavior 7 edge: current 12.0 over-cap pre-state — Starting cap never lowers HP.
#[test]
fn heal_starting_cap_over_starting_never_lowers_hp() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  12.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 12.0).abs() < f32::EPSILON,
        "Starting cap must NEVER lower HP, got {}",
        hp.current
    );
}

/// Behavior 8: Starting cap with max=None behaves identically to Max cap.
#[test]
fn heal_starting_cap_equals_max_cap_when_max_none() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  8.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        500.0,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Starting cap with max=None should clamp at starting=10.0, got {}",
        hp.current
    );
}

/// Behavior 8 edge: small additive heal with Starting cap (identical to Max).
#[test]
fn heal_starting_cap_additive_when_under_ceiling() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Starting cap + 3.0 on current=2.0 = 5.0, got {}",
        hp.current
    );
}

/// Behavior 8 paired: Starting and Max caps give identical results when max=None.
#[test]
fn heal_cap_variants_identical_when_max_none() {
    // Run 1: HealCap::Starting
    let mut app_a = build_apply_heal_app();
    let entity_a = app_a
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  8.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();
    app_a.insert_resource(PendingHeal(vec![heal_msg(
        entity_a,
        500.0,
        HealCap::Starting,
    )]));
    tick(&mut app_a);
    let hp_a = app_a.world().get::<Hp>(entity_a).unwrap().current;

    // Run 2: HealCap::Max
    let mut app_b = build_apply_heal_app();
    let entity_b = app_b
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  8.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();
    app_b.insert_resource(PendingHeal(vec![heal_msg(entity_b, 500.0, HealCap::Max)]));
    tick(&mut app_b);
    let hp_b = app_b.world().get::<Hp>(entity_b).unwrap().current;

    assert!(
        (hp_a - 10.0).abs() < f32::EPSILON,
        "Starting cap should produce 10.0, got {hp_a}"
    );
    assert!(
        (hp_b - 10.0).abs() < f32::EPSILON,
        "Max cap should produce 10.0, got {hp_b}"
    );
    assert!(
        (hp_a - hp_b).abs() < f32::EPSILON,
        "Both caps should produce identical results when max=None, got {hp_a} vs {hp_b}"
    );
}

/// Behavior 9: over-cap pre-state — Max cap never lowers HP.
#[test]
fn heal_max_cap_over_cap_pre_state_never_lowers_hp() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  25.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 25.0).abs() < f32::EPSILON,
        "Max cap must never lower over-cap HP, got {}",
        hp.current
    );
}

/// Behavior 9 edge: same over-cap — Starting cap also never lowers HP.
#[test]
fn heal_starting_cap_over_cap_pre_state_never_lowers_hp() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  25.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 25.0).abs() < f32::EPSILON,
        "Starting cap must never lower over-cap HP, got {}",
        hp.current
    );
}

// ── Group C: Mixed-cap same-tick semantics ─────────────────────────────────

/// Behavior 10: Starting then Max — second message's higher ceiling extends.
#[test]
fn mixed_cap_starting_then_max_second_ceiling_extends() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // Step-by-step reasoning:
    // msg 1 (Starting, amount=100): current = (5 + 100).min(10) = 10
    // msg 2 (Max, amount=100):      current = (10 + 100).min(20) = 20
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 100.0, HealCap::Starting),
        heal_msg(entity, 100.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Mixed Starting->Max should reach max=20.0, got {}",
        hp.current
    );
}

/// Behavior 10 edge: reversed order (Max then Starting) — Starting is no-op.
#[test]
fn mixed_cap_max_then_starting_second_message_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // msg 1 (Max, amount=100):      current = (5 + 100).min(20) = 20
    // msg 2 (Starting, amount=100): current=20 >= starting=10, no-op (Behavior 9)
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 100.0, HealCap::Max),
        heal_msg(entity, 100.0, HealCap::Starting),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Mixed Max->Starting should stay at 20.0, got {}",
        hp.current
    );
}

/// Behavior 10 edge: small amounts, neither ceiling hit — simple accumulation.
#[test]
fn mixed_cap_small_additive_both_under_ceiling() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // msg 1 (Starting, 3.0): current = (5 + 3).min(10) = 8
    // msg 2 (Max, 3.0):      current = (8 + 3).min(20) = 11
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 3.0, HealCap::Starting),
        heal_msg(entity, 3.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 11.0).abs() < f32::EPSILON,
        "Mixed small-amount additive should give 11.0, got {}",
        hp.current
    );
}

/// Behavior 10b: per-message ceiling re-read — four-message sequence lands at 13.0.
#[test]
fn per_message_ceiling_re_read_not_cached() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // Step-by-step:
    // msg 1 (Starting, 3): current = (2 + 3).min(10) = 5
    // msg 2 (Max, 3):      current = (5 + 3).min(20) = 8
    // msg 3 (Starting, 3): current = (8 + 3).min(10) = 10
    // msg 4 (Max, 3):      current = (10 + 3).min(20) = 13
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 3.0, HealCap::Starting),
        heal_msg(entity, 3.0, HealCap::Max),
        heal_msg(entity, 3.0, HealCap::Starting),
        heal_msg(entity, 3.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 13.0).abs() < f32::EPSILON,
        "Per-message ceiling re-read should produce 13.0; cached would give 10.0, got {}",
        hp.current
    );
}

/// Behavior 10b edge: swap messages 3 and 4 — different result (11.0).
#[test]
fn per_message_ceiling_re_read_order_matters() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // Step-by-step:
    // msg 1 (Starting, 3): current = (2 + 3).min(10) = 5
    // msg 2 (Max, 3):      current = (5 + 3).min(20) = 8
    // msg 3 (Max, 3):      current = (8 + 3).min(20) = 11
    // msg 4 (Starting, 3): current=11 >= starting=10, no-op
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 3.0, HealCap::Starting),
        heal_msg(entity, 3.0, HealCap::Max),
        heal_msg(entity, 3.0, HealCap::Max),
        heal_msg(entity, 3.0, HealCap::Starting),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 11.0).abs() < f32::EPSILON,
        "Swapped order should produce 11.0 (different from 13.0), got {}",
        hp.current
    );
}

/// Behavior 11: both mixed heals fit under lower ceiling — simple accumulation.
#[test]
fn mixed_cap_both_under_lower_ceiling_accumulate() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // msg 1 (Starting, 3): current = (2 + 3).min(10) = 5
    // msg 2 (Max, 2):      current = (5 + 2).min(20) = 7
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 3.0, HealCap::Starting),
        heal_msg(entity, 2.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Mixed non-binding clamps should accumulate to 7.0, got {}",
        hp.current
    );
}

/// Behavior 11 edge: reversed order — still 7.0.
#[test]
fn mixed_cap_both_under_lower_ceiling_reversed_order() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // msg 1 (Max, 2):      current = (2 + 2).min(20) = 4
    // msg 2 (Starting, 3): current = (4 + 3).min(10) = 7
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 2.0, HealCap::Max),
        heal_msg(entity, 3.0, HealCap::Starting),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Reversed mixed additive should still give 7.0, got {}",
        hp.current
    );
}

// ── Group D: Without<Dead> filter ──────────────────────────────────────────

/// Behavior 12: heal on Dead entity is a no-op (Max cap).
#[test]
fn heal_on_dead_entity_is_noop_max_cap() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity_dead(&mut app, 10.0);

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 7.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Dead entity Hp should remain 10.0, got {}",
        hp.current
    );
}

/// Behavior 12 edge: Dead entity with negative Hp — still not healed.
#[test]
fn heal_on_dead_negative_hp_entity_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity_dead(&mut app, -5.0);

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 100.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - (-5.0)).abs() < f32::EPSILON,
        "Dead entity with negative Hp should remain -5.0, got {}",
        hp.current
    );
}

/// Behavior 12 edge: Starting cap also blocked on Dead entity.
#[test]
fn heal_on_dead_entity_is_noop_starting_cap() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity_dead(&mut app, -5.0);

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        100.0,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - (-5.0)).abs() < f32::EPSILON,
        "Dead entity with Starting cap should remain -5.0, got {}",
        hp.current
    );
}

/// Behavior 12 edge: Dead marker still present after filtered tick.
#[test]
fn heal_on_dead_entity_does_not_remove_dead_marker() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity_dead(&mut app, 10.0);

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 7.0, HealCap::Max)]));
    tick(&mut app);

    assert!(
        app.world().get::<Dead>(entity).is_some(),
        "Dead marker should still be present after the filtered tick"
    );
}

/// Behavior 13: non-Dead entity with Hp.current == 0.0 IS healed (Dead is the gate).
#[test]
fn heal_applies_to_non_dead_entity_with_zero_hp() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  0.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 4.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 4.0).abs() < f32::EPSILON,
        "non-Dead entity with 0.0 Hp should be healed to 4.0, got {}",
        hp.current
    );
}

/// Behavior 14: heal on Dead entity does not set `KilledBy`.
#[test]
fn heal_on_dead_entity_does_not_set_killed_by() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  0.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Dead,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 4.0, HealCap::Max)]));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer, None,
        "KilledBy.dealer should remain None — heal must not touch it"
    );
}

// ── Group E: Without<Invulnerable> filter ──────────────────────────────────

/// Behavior 15: heal on Invulnerable entity is a no-op (Max cap).
#[test]
fn heal_on_invulnerable_entity_is_noop_max_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Invulnerable entity Hp should remain 5.0, got {}",
        hp.current
    );
}

/// Behavior 15 edge: Starting cap also blocked on Invulnerable.
#[test]
fn heal_on_invulnerable_entity_is_noop_starting_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Invulnerable entity Hp should remain 5.0 (Starting cap), got {}",
        hp.current
    );
}

/// Behavior 15 edge: Invulnerable marker still present after filtered tick.
#[test]
fn heal_on_invulnerable_entity_preserves_marker() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    assert!(
        app.world().get::<Invulnerable>(entity).is_some(),
        "Invulnerable marker should still be present after filtered tick"
    );
}

/// Behavior 15b: Dead + Invulnerable — both filters compose (still skipped).
#[test]
fn heal_on_dead_and_invulnerable_entity_is_noop_max_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Dead,
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Dead+Invulnerable entity should remain 5.0, got {}",
        hp.current
    );
}

/// Behavior 15b edge: Starting cap also blocked on Dead+Invulnerable.
#[test]
fn heal_on_dead_and_invulnerable_entity_is_noop_starting_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Dead,
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Dead+Invulnerable entity with Starting cap should remain 5.0, got {}",
        hp.current
    );
}

/// Behavior 15b edge: both markers still present after filtered tick.
#[test]
fn heal_on_dead_and_invulnerable_entity_preserves_both_markers() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Dead,
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    assert!(
        app.world().get::<Dead>(entity).is_some(),
        "Dead marker should still be present after filtered tick"
    );
    assert!(
        app.world().get::<Invulnerable>(entity).is_some(),
        "Invulnerable marker should still be present after filtered tick"
    );
}

/// Behavior 16: mixed batch — Invulnerable A and vulnerable B.
#[test]
fn mixed_batch_invulnerable_vs_vulnerable() {
    let mut app = build_apply_heal_app();
    let entity_a = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Invulnerable,
        ))
        .id();
    let entity_b = spawn_test_entity(&mut app, 10.0);
    // Set B's current to 5.0 (spawn_test_entity creates current=starting=10.0)
    app.world_mut().get_mut::<Hp>(entity_b).unwrap().current = 5.0;

    app.insert_resource(PendingHeal(vec![
        heal_msg(entity_a, 3.0, HealCap::Max),
        heal_msg(entity_b, 3.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(entity_a).unwrap();
    let hp_b = app.world().get::<Hp>(entity_b).unwrap();
    assert!(
        (hp_a.current - 5.0).abs() < f32::EPSILON,
        "Invulnerable A should remain 5.0, got {}",
        hp_a.current
    );
    assert!(
        (hp_b.current - 8.0).abs() < f32::EPSILON,
        "Vulnerable B should be 8.0 after 3.0 heal, got {}",
        hp_b.current
    );
}

/// Behavior 16 edge: reversed message order — same outcome.
#[test]
fn mixed_batch_invulnerable_vs_vulnerable_reversed_order() {
    let mut app = build_apply_heal_app();
    let entity_a = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Invulnerable,
        ))
        .id();
    let entity_b = spawn_test_entity(&mut app, 10.0);
    app.world_mut().get_mut::<Hp>(entity_b).unwrap().current = 5.0;

    app.insert_resource(PendingHeal(vec![
        heal_msg(entity_b, 3.0, HealCap::Max),
        heal_msg(entity_a, 3.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(entity_a).unwrap();
    let hp_b = app.world().get::<Hp>(entity_b).unwrap();
    assert!(
        (hp_a.current - 5.0).abs() < f32::EPSILON,
        "Invulnerable A should remain 5.0 (order-independent), got {}",
        hp_a.current
    );
    assert!(
        (hp_b.current - 8.0).abs() < f32::EPSILON,
        "Vulnerable B should be 8.0 (order-independent), got {}",
        hp_b.current
    );
}

/// Behavior 16 edge: different caps per entity.
#[test]
fn mixed_batch_invulnerable_starting_vulnerable_max() {
    let mut app = build_apply_heal_app();
    let entity_a = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Invulnerable,
        ))
        .id();
    let entity_b = spawn_test_entity(&mut app, 10.0);
    app.world_mut().get_mut::<Hp>(entity_b).unwrap().current = 5.0;

    app.insert_resource(PendingHeal(vec![
        heal_msg(entity_a, 3.0, HealCap::Starting),
        heal_msg(entity_b, 3.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(entity_a).unwrap();
    let hp_b = app.world().get::<Hp>(entity_b).unwrap();
    assert!(
        (hp_a.current - 5.0).abs() < f32::EPSILON,
        "Invulnerable A filtered before cap considered, got {}",
        hp_a.current
    );
    assert!(
        (hp_b.current - 8.0).abs() < f32::EPSILON,
        "Vulnerable B should be 8.0, got {}",
        hp_b.current
    );
}

// ── Group F: Target query edge cases (silent skips) ────────────────────────

/// Behavior 17: entity without `TestEntity` marker is skipped.
#[test]
fn heal_targeting_entity_without_marker_is_skipped() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Entity without TestEntity should remain at 5.0, got {}",
        hp.current
    );
}

/// Behavior 18: entity without Hp is silently skipped.
#[test]
fn heal_targeting_entity_without_hp_does_not_panic() {
    let mut app = build_apply_heal_app();
    let entity = app.world_mut().spawn(TestEntity).id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    assert!(
        app.world().get::<Hp>(entity).is_none(),
        "Entity without Hp should still have no Hp"
    );
}

/// Behavior 19: heal targeting despawned entity is silently skipped.
#[test]
fn heal_targeting_despawned_entity_does_not_panic() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity(&mut app, 5.0);
    app.world_mut().entity_mut(entity).despawn();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app); // must not panic

    assert!(
        app.world().get_entity(entity).is_err(),
        "entity should remain despawned"
    );
}

// ── Group G: Numeric edge cases on amount ──────────────────────────────────

/// Behavior 21: heal with amount == 0.0 is a no-op (Max cap).
#[test]
fn heal_with_zero_amount_is_noop_max_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 0.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Zero-amount heal should leave Hp unchanged at 7.0, got {}",
        hp.current
    );
}

/// Behavior 21 edge: zero amount, Starting cap.
#[test]
fn heal_with_zero_amount_is_noop_starting_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 0.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Zero-amount heal with Starting cap should leave Hp at 7.0, got {}",
        hp.current
    );
}

/// Behavior 22: heal with negative amount is a no-op (never damages).
#[test]
fn heal_with_negative_amount_is_noop_max_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, -3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Negative-amount heal must NOT damage — Hp should remain 7.0, got {}",
        hp.current
    );
}

/// Behavior 22 edge: large negative amount — still no-op.
#[test]
fn heal_with_large_negative_amount_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, -1000.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Large-negative amount guard must be absolute, got {}",
        hp.current
    );
}

/// Behavior 22 edge: negative zero.
#[test]
fn heal_with_negative_zero_amount_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, -0.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Negative-zero heal should be no-op, got {}",
        hp.current
    );
}

/// Behavior 22 edge: negative amount with Starting cap (guard is cap-agnostic).
#[test]
fn heal_with_negative_amount_is_noop_starting_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, -3.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Negative-amount heal with Starting cap must not damage, got {}",
        hp.current
    );
}

/// Behavior 23: NaN amount does not corrupt Hp.current (Max cap).
#[test]
fn heal_with_nan_amount_does_not_corrupt_hp_max_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, f32::NAN, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        hp.current.is_finite(),
        "Hp should remain finite after NaN heal, got {}",
        hp.current
    );
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Hp should remain 7.0 after NaN heal, got {}",
        hp.current
    );
}

/// Behavior 23 edge: NaN with Starting cap.
#[test]
fn heal_with_nan_amount_does_not_corrupt_hp_starting_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        f32::NAN,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        hp.current.is_finite() && (hp.current - 7.0).abs() < f32::EPSILON,
        "Hp should remain finite and 7.0 after NaN heal (Starting cap), got {}",
        hp.current
    );
}

/// Behavior 23 edge: INFINITY + Max cap, max=None → clamps at starting.
#[test]
fn heal_with_infinity_max_cap_no_max_clamps_at_starting() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        f32::INFINITY,
        HealCap::Max,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "INFINITY + Max cap + max=None should clamp at starting=10.0, got {}",
        hp.current
    );
}

/// Behavior 23 edge: INFINITY + Starting cap with max=Some → clamps at starting.
#[test]
fn heal_with_infinity_starting_cap_ignores_max() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        f32::INFINITY,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "INFINITY + Starting cap should clamp at starting=10.0 ignoring max=20.0, got {}",
        hp.current
    );
}

/// Behavior 23 edge: INFINITY + Max cap with max=Some → clamps at max.
#[test]
fn heal_with_infinity_max_cap_with_max_clamps_at_max() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        f32::INFINITY,
        HealCap::Max,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "INFINITY + Max cap with max=20.0 should clamp at 20.0, got {}",
        hp.current
    );
}

/// Behavior 23 edge: `NEG_INFINITY` is blocked by negative-amount guard.
#[test]
fn heal_with_neg_infinity_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        f32::NEG_INFINITY,
        HealCap::Max,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "NEG_INFINITY should be blocked by <= 0.0 guard, got {}",
        hp.current
    );
}

// ── Group H: Heal attribution (healer / source) ───────────────────────────

/// Behavior 24: heal does NOT write to `KilledBy`.
#[test]
fn heal_does_not_write_to_killed_by() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity(&mut app, 10.0);
    app.world_mut().get_mut::<Hp>(entity).unwrap().current = 5.0;
    let some_entity = app.world_mut().spawn_empty().id();

    let msg = HealDealt::<TestEntity> {
        healer:  Some(some_entity),
        target:  entity,
        amount:  3.0,
        source:  None,
        cap:     HealCap::Max,
        _marker: PhantomData,
    };
    app.insert_resource(PendingHeal(vec![msg]));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer, None,
        "heal must never touch KilledBy.dealer"
    );
}

/// Behavior 25: heal payload fields are read-only — no leakage into unrelated components.
#[test]
fn heal_payload_fields_do_not_leak_into_other_components() {
    let mut app = build_apply_heal_app();
    let healer = app.world_mut().spawn_empty().id();
    let victim = app
        .world_mut()
        .spawn((TestEntity, Hp::new(10.0), KilledBy::default()))
        .id();
    // Pre-damage victim to 4.0.
    app.world_mut().get_mut::<Hp>(victim).unwrap().current = 4.0;

    let msg = HealDealt::<TestEntity> {
        healer:  Some(healer),
        target:  victim,
        amount:  2.0,
        source:  Some("cascade".to_string()),
        cap:     HealCap::Max,
        _marker: PhantomData,
    };
    app.insert_resource(PendingHeal(vec![msg]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(victim).unwrap();
    assert!(
        (hp.current - 6.0).abs() < f32::EPSILON,
        "Hp should be 6.0 after 2.0 heal, got {}",
        hp.current
    );
    assert!(
        app.world().get::<Dead>(victim).is_none(),
        "Dead should remain absent after a heal"
    );
    assert!(
        app.world().get::<Invulnerable>(victim).is_none(),
        "Invulnerable should remain absent after a heal"
    );
    let killed_by = app.world().get::<KilledBy>(victim).unwrap();
    assert_eq!(
        killed_by.dealer, None,
        "KilledBy.dealer should remain None after a heal"
    );
}

// ── Behavior 31: Per-T queue isolation (TestEntity and Salvo side-by-side) ─

/// Behavior 31: `HealDealt`<TestEntity> does not heal a Salvo entity.
/// Both monomorphizations are registered in the same app.
#[test]
fn heal_wrong_monomorphization_test_entity_message_does_not_heal_salvo() {
    // One-off harness — both monomorphizations registered.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<HealDealt<TestEntity>>();
    app.add_message::<HealDealt<Salvo>>();
    app.add_systems(FixedUpdate, apply_heal::<TestEntity>);
    app.add_systems(FixedUpdate, apply_heal::<Salvo>);

    // Spawn a Salvo-only entity (NO TestEntity marker on it).
    let salvo_entity = app
        .world_mut()
        .spawn((Salvo, Hp::new(10.0), KilledBy::default()))
        .id();
    // Pre-damage salvo to 5.0 so a potential heal would be visible.
    app.world_mut().get_mut::<Hp>(salvo_entity).unwrap().current = 5.0;

    // Write HealDealt<TestEntity> targeting the Salvo entity.
    app.world_mut()
        .resource_mut::<Messages<HealDealt<TestEntity>>>()
        .write(HealDealt::<TestEntity> {
            healer:  None,
            target:  salvo_entity,
            amount:  5.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        });

    tick(&mut app);

    let hp = app.world().get::<Hp>(salvo_entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Salvo entity should not be healed by HealDealt<TestEntity>, got {}",
        hp.current
    );
}

/// Behavior 31 converse: `HealDealt`<Salvo> does not heal a `TestEntity`.
#[test]
fn heal_wrong_monomorphization_salvo_message_does_not_heal_test_entity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<HealDealt<TestEntity>>();
    app.add_message::<HealDealt<Salvo>>();
    app.add_systems(FixedUpdate, apply_heal::<TestEntity>);
    app.add_systems(FixedUpdate, apply_heal::<Salvo>);

    let test_entity = app
        .world_mut()
        .spawn((TestEntity, Hp::new(10.0), KilledBy::default()))
        .id();
    app.world_mut().get_mut::<Hp>(test_entity).unwrap().current = 5.0;

    app.world_mut()
        .resource_mut::<Messages<HealDealt<Salvo>>>()
        .write(HealDealt::<Salvo> {
            healer:  None,
            target:  test_entity,
            amount:  5.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        });

    tick(&mut app);

    let hp = app.world().get::<Hp>(test_entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "TestEntity should not be healed by HealDealt<Salvo>, got {}",
        hp.current
    );
}
