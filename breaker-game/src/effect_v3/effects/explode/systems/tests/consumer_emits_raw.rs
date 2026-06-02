//! W7 Behavior 2 (Explode): the consumer system (`apply_explode_damage`)
//! emits one `DamageDealt<Cell>` per hit cell carrying the raw
//! `request.base_damage` — pre-multiplication, per the W6 invariant. The
//! pipeline's `MutateDamage` set later applies any boosts.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::SeedableRng;

use super::{super::super::config::ExplodeConfig, helpers::*};
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn consumer_emits_one_damage_dealt_with_raw_base_damage() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(
        source,
        "chip-source",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        1,
        "consumer must emit exactly one DamageDealt<Cell> for the in-range cell"
    );
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
fn consumer_emits_raw_for_each_of_two_cells_inside_range() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let _c1 = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);
    let _c2 = spawn_cell_with_hp(&mut app, Vec2::new(40.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(
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
