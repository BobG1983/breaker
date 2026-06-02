//! W7 Behavior 8 (PiercingBeam): two `fire()` calls in the same tick
//! produce two requests; the single consumer invocation reads BOTH and
//! emits damage independently.

use std::collections::HashMap;

use bevy::prelude::*;
use rand::SeedableRng;

use super::helpers::*;
use crate::{bolt::components::BoltBaseDamage, effect_v3::traits::Fireable, prelude::*};

#[test]
fn two_independent_fires_produce_two_emission_sets_no_request_dropped() {
    let mut app = piercing_pipeline_app();
    let source_a = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();
    let source_b = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(200.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();
    let cell_a = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);
    let cell_b = spawn_cell_with_hp(&mut app, Vec2::new(200.0, 50.0), 100.0);

    make_config().fire(
        source_a,
        "chip-a",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    make_config().fire(
        source_b,
        "chip-b",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        2,
        "consumer reads both requests and emits independently",
    );

    let mut by_target: HashMap<Entity, (Option<Entity>, Option<SourceId>)> = HashMap::new();
    for msg in &collector.0 {
        by_target.insert(msg.target, (msg.dealer, msg.source.clone()));
    }
    assert_eq!(
        by_target.get(&cell_a),
        Some(&(Some(source_a), Some(SourceId::from("chip-a".to_owned())))),
    );
    assert_eq!(
        by_target.get(&cell_b),
        Some(&(Some(source_b), Some(SourceId::from("chip-b".to_owned())))),
    );

    let hp_a = app.world().get::<Hp>(cell_a).expect("Hp").current;
    let hp_b = app.world().get::<Hp>(cell_b).expect("Hp").current;
    assert!((hp_a - 90.0).abs() < 1e-4);
    assert!((hp_b - 90.0).abs() < 1e-4);
}

#[test]
fn cell_inside_both_beams_receives_one_message_per_source() {
    let mut app = piercing_pipeline_app();
    let source_a = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();
    // Source B at (5, 0) so the beam axes overlap a single cell at (5, 100).
    // Both beams point +Y; cell is forward of both origins (along >= 0) and
    // perp distance 5 ≤ half_width 10 from beam A (which is anchored at x=0)
    // and perp distance 0 from beam B (anchored at x=5).
    let source_b = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(5.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(5.0, 100.0), 100.0);

    make_config().fire(
        source_a,
        "chip-a",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    make_config().fire(
        source_b,
        "chip-b",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        2,
        "cell inside both beams receives one DamageDealt per source",
    );
    let dealers: Vec<Option<Entity>> = collector.0.iter().map(|m| m.dealer).collect();
    assert!(dealers.contains(&Some(source_a)));
    assert!(dealers.contains(&Some(source_b)));

    let hp = app.world().get::<Hp>(cell).expect("Hp").current;
    assert!(
        (hp - 80.0).abs() < 1e-4,
        "cell took 10.0 + 10.0 = 20.0 → expected Hp 80.0, got {hp}",
    );
}
