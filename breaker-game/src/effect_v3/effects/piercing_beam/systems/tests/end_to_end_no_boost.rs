//! W7 Behavior 3 (PiercingBeam): end-to-end damage parity with no boosts.

use bevy::prelude::*;
use rand::SeedableRng;

use super::helpers::*;
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn two_cells_in_beam_each_take_ten_damage() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    let c1 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);
    let c2 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 150.0), 100.0);

    make_config().fire(
        source,
        "",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let hp1 = app.world().get::<Hp>(c1).expect("c1 Hp").current;
    let hp2 = app.world().get::<Hp>(c2).expect("c2 Hp").current;
    assert!((hp1 - 90.0).abs() < 1e-4, "c1 expected 90.0, got {hp1}");
    assert!((hp2 - 90.0).abs() < 1e-4, "c2 expected 90.0, got {hp2}");
}

#[test]
fn three_cells_in_beam_each_take_ten_damage_no_double_application() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    let c1 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);
    let c2 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 150.0), 100.0);
    let c3 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 300.0), 100.0);

    make_config().fire(
        source,
        "",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    for (entity, label) in [(c1, "c1"), (c2, "c2"), (c3, "c3")] {
        let hp = app.world().get::<Hp>(entity).expect("Hp").current;
        assert!(
            (hp - 90.0).abs() < 1e-4,
            "{label} expected 90.0 (no double-application), got {hp}",
        );
    }
}
