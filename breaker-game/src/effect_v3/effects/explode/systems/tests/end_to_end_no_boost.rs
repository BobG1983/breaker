//! W7 Behavior 3 (Explode): end-to-end damage parity with no boosts.
//! `fire()` + 1 `tick(&mut app)` reduces each in-range cell's HP by exactly
//! `base_damage` (10.0 → 100 - 10 = 90). The single fixed-update advance
//! runs the full `EmitDamage → MutateDamage → ApplyDamage → PostApplyDamage`
//! chain, so `Hp.current` reflects the damage immediately.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::SeedableRng;

use super::{super::super::config::ExplodeConfig, helpers::*};
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn two_cells_inside_range_each_take_ten_damage() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let c1 = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);
    let c2 = spawn_cell_with_hp(&mut app, Vec2::new(30.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(
        source,
        "",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let hp1 = app.world().get::<Hp>(c1).expect("c1 has Hp").current;
    let hp2 = app.world().get::<Hp>(c2).expect("c2 has Hp").current;
    assert!(
        (hp1 - 90.0).abs() < 1e-4,
        "c1 should be at 90.0 (took raw 10.0 damage), got {hp1}",
    );
    assert!(
        (hp2 - 90.0).abs() < 1e-4,
        "c2 should be at 90.0 (took raw 10.0 damage), got {hp2}",
    );
}

#[test]
fn three_cells_with_boundary_each_take_ten_damage_no_double_application() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let c1 = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);
    let c2 = spawn_cell_with_hp(&mut app, Vec2::new(30.0, 0.0), 100.0);
    let c3 = spawn_cell_with_hp(&mut app, Vec2::new(50.0, 0.0), 100.0); // boundary

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(
        source,
        "",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    for (entity, label) in [(c1, "c1"), (c2, "c2"), (c3, "c3")] {
        let hp = app.world().get::<Hp>(entity).expect("Hp present").current;
        assert!(
            (hp - 90.0).abs() < 1e-4,
            "{label} expected Hp 90.0 (no double-application), got {hp}",
        );
    }
}
