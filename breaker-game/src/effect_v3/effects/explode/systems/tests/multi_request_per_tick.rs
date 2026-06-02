//! W7 Behavior 8 (Explode): two `fire()` calls in the same synchronous
//! block produce two `ExplodeEmissionRequested` messages, and the single
//! `apply_explode_damage` invocation reads BOTH and emits independent damage
//! per (request, cell) pair. Each carries its own `dealer` and `source`.

use std::collections::HashMap;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::SeedableRng;

use super::{super::super::config::ExplodeConfig, helpers::*};
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn two_independent_fires_produce_two_emission_sets_no_request_dropped() {
    let mut app = explode_pipeline_app();
    let source_a = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let source_b = app
        .world_mut()
        .spawn(Position2D(Vec2::new(200.0, 0.0)))
        .id();
    let cell_a = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);
    let cell_b = spawn_cell_with_hp(&mut app, Vec2::new(220.0, 0.0), 100.0);
    // Cell at (100, 0) is OUTSIDE both A's and B's ranges (range = 50.0).
    let _outside = spawn_cell_with_hp(&mut app, Vec2::new(100.0, 0.0), 100.0);

    let config_a = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    let config_b = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config_a.fire(
        source_a,
        "chip-a",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    config_b.fire(
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
        "consumer must read both requests and emit one DamageDealt<Cell> each",
    );

    // Map by target → (dealer, source) for assertion order-independence.
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
    assert!(
        (hp_a - 90.0).abs() < 1e-4,
        "cell_a HP expected 90.0, got {hp_a}"
    );
    assert!(
        (hp_b - 90.0).abs() < 1e-4,
        "cell_b HP expected 90.0, got {hp_b}"
    );
}

#[test]
fn cell_inside_both_sources_receives_one_message_per_source() {
    let mut app = explode_pipeline_app();
    let source_a = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let source_b = app.world_mut().spawn(Position2D(Vec2::new(50.0, 0.0))).id();
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(25.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(100.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(
        source_a,
        "chip-a",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    config.fire(
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
        "cell inside both sources receives one DamageDealt per source",
    );
    let dealers: Vec<Option<Entity>> = collector.0.iter().map(|m| m.dealer).collect();
    assert!(dealers.contains(&Some(source_a)));
    assert!(dealers.contains(&Some(source_b)));

    let hp = app.world().get::<Hp>(cell).expect("Hp").current;
    assert!(
        (hp - 80.0).abs() < 1e-4,
        "cell took 10.0 + 10.0 = 20.0 damage → expected Hp 80.0, got {hp}",
    );
}
