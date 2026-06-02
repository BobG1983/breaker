//! W7 Behavior 2 (PiercingBeam): the consumer emits the raw `base_damage`
//! per cell (pre-multiplication, W6 invariant).

use bevy::prelude::*;
use rand::SeedableRng;

use super::helpers::*;
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn consumer_emits_one_damage_dealt_with_raw_base_damage() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);

    make_config().fire(
        source,
        "chip-source",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(collector.0.len(), 1);
    let msg = &collector.0[0];
    assert!(
        (msg.amount - 10.0).abs() < f32::EPSILON,
        "amount must equal raw base_damage = 10.0 (NOT pre-multiplied), got {}",
        msg.amount,
    );
    assert_eq!(msg.target, cell);
    assert_eq!(msg.dealer, Some(source));
    assert_eq!(msg.source, Some(SourceId::from("chip-source".to_owned())));
}

#[test]
fn consumer_emits_raw_for_each_of_two_cells_in_beam() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    let _c1 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);
    let _c2 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 150.0), 100.0);

    make_config().fire(
        source,
        "chip-source",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(collector.0.len(), 2);
    for msg in &collector.0 {
        assert!(
            (msg.amount - 10.0).abs() < f32::EPSILON,
            "each amount must be raw 10.0, NEVER pre-multiplied to 20.0, got {}",
            msg.amount,
        );
    }
}
