//! W7 Behavior 4 (Explode): the W6 invariant under the new request flow —
//! `DamageBoostStack` aggregate `2.0` produces a final delta `=
//! base_damage * 2.0` (NOT `base_damage * 2.0 * 2.0`). The pipeline applies
//! the boost exactly once in `MutateDamage`; the consumer emits raw.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{super::super::config::ExplodeConfig, helpers::*};
use crate::{bolt::test_utils::damage_stack, effect_v3::traits::Fireable, prelude::*};

#[test]
fn boost_aggregate_two_produces_final_delta_twenty_not_forty() {
    let mut app = explode_pipeline_app();
    // Source spawned with DamageBoostStack via the canonical `damage_stack`
    // helper (no `Clone`, no `with_persistent` constructor — see
    // bolt/test_utils.rs:68-74).
    let source = app
        .world_mut()
        .spawn((Position2D(Vec2::ZERO), damage_stack(&[2.0])))
        .id();
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(source, "", app.world_mut());
    tick(&mut app);

    // Final HP delta proof:
    //   100.0 - Hp.current == 10.0 * 2.0 (NOT 10.0 * 2.0 * 2.0 == 40.0,
    //   which would indicate pre-W6 double-application).
    // The MessageCollector observes the post-mutator value (collector pumps
    // at `Last`, after MutateDamage), so we cannot directly assert the
    // emission-time `amount == 10.0` here — see test spec note on
    // collector capture timing. The mathematical proof IS: HP delta of
    // exactly 20 ⟹ emitted amount was 10 and was multiplied exactly once.
    let hp = app.world().get::<Hp>(cell).expect("Hp present").current;
    assert!(
        (hp - 80.0).abs() < 1e-4,
        "expected Hp 80.0 (delta = 10.0 * 2.0 = 20.0); got {hp} (60.0 \
         would mean pre-W6 double-application 10*2*2=40)",
    );
}

#[test]
fn boost_aggregate_two_with_single_cell_pins_exactly_eighty_hp() {
    // Edge: confirm against alternative "wrong" outcomes — 60.0 (double-
    // applied), 100.0 (no damage), 90.0 (no boost).
    let mut app = explode_pipeline_app();
    let source = app
        .world_mut()
        .spawn((Position2D(Vec2::ZERO), damage_stack(&[2.0])))
        .id();
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(source, "", app.world_mut());
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("Hp present").current;
    assert!((hp - 80.0).abs() < 1e-4, "expected 80.0, got {hp}");
    assert!((hp - 60.0).abs() > 1.0, "must NOT be 60.0 (double-applied)");
    assert!((hp - 100.0).abs() > 1.0, "must NOT be 100.0 (no damage)");
    assert!((hp - 90.0).abs() > 1.0, "must NOT be 90.0 (no boost)");
}
