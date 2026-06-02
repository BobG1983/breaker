//! W7 Behavior 4 (PiercingBeam): `DamageBoostStack` aggregate `2.0`
//! produces a final delta of `base_damage * 2.0` exactly once. Pre-W6
//! double-application would produce 40.0 (10 * 2 * 2); the correct W6/W7
//! behavior is 20.0.

use bevy::prelude::*;
use rand::SeedableRng;

use super::helpers::*;
use crate::{
    bolt::{components::BoltBaseDamage, test_utils::damage_stack},
    effect_v3::traits::Fireable,
    prelude::*,
};

#[test]
fn boost_aggregate_two_produces_final_delta_twenty_not_forty() {
    let mut app = piercing_pipeline_app();
    // Source spawned with DamageBoostStack via the canonical `damage_stack`
    // helper.
    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
            damage_stack(&[2.0]),
        ))
        .id();
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);

    make_config().fire(
        source,
        "",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("Hp").current;
    assert!(
        (hp - 80.0).abs() < 1e-4,
        "expected Hp 80.0 (delta = 10.0 * 2.0 = 20.0); got {hp} (60.0 \
         indicates pre-W6 double-application 10*2*2=40)",
    );
}

#[test]
fn boost_aggregate_two_with_single_cell_pins_exactly_eighty_hp() {
    let mut app = piercing_pipeline_app();
    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
            damage_stack(&[2.0]),
        ))
        .id();
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);

    make_config().fire(
        source,
        "",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("Hp").current;
    assert!((hp - 80.0).abs() < 1e-4, "expected 80.0, got {hp}");
    assert!((hp - 60.0).abs() > 1.0, "must NOT be 60.0 (double-applied)");
    assert!((hp - 100.0).abs() > 1.0, "must NOT be 100.0 (no damage)");
    assert!((hp - 90.0).abs() > 1.0, "must NOT be 90.0 (no boost)");
}
