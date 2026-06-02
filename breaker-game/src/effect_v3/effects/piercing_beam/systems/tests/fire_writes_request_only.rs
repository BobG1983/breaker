//! W7 Behavior 1 (PiercingBeam): `fire()` writes ONE
//! `PiercingBeamEmissionRequested` and does NOT touch
//! `Messages<DamageDealt<Cell>>` synchronously.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::SeedableRng;

use super::super::super::{config::PiercingBeamConfig, messages::PiercingBeamEmissionRequested};
use crate::{bolt::components::BoltBaseDamage, effect_v3::traits::Fireable, prelude::*};

fn build_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_message_capture::<PiercingBeamEmissionRequested>()
        .build()
}

#[test]
fn fire_writes_one_piercing_beam_emission_requested_with_correct_geometry() {
    let mut app = build_app();
    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 150.0))));
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 300.0))));

    let config = PiercingBeamConfig {
        damage_mult: OrderedFloat(1.0),
        width:       OrderedFloat(20.0),
    };
    config.fire(
        source,
        "chip-source",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );

    // Snapshot immediately — NO `app.update()` between fire and assertions.
    let req_buf = app
        .world()
        .resource::<Messages<PiercingBeamEmissionRequested>>();
    let collected: Vec<PiercingBeamEmissionRequested> =
        req_buf.iter_current_update_messages().cloned().collect();
    assert_eq!(
        collected.len(),
        1,
        "fire() must write exactly one PiercingBeamEmissionRequested"
    );
    let req = &collected[0];
    assert_eq!(req.origin, Vec2::ZERO, "origin mirrors source Position2D");
    assert_eq!(
        req.direction,
        Vec2::Y,
        "direction is Velocity2D normalized → +Y unit vector",
    );
    assert!(
        (req.half_width - 10.0).abs() < f32::EPSILON,
        "half_width = config.width / 2.0 = 10.0, got {}",
        req.half_width,
    );
    assert!(
        (req.base_damage - 10.0).abs() < f32::EPSILON,
        "base_damage = bolt_base_damage(10.0) * damage_mult(1.0) = 10.0, got {}",
        req.base_damage,
    );
    assert_eq!(req.dealer, Some(source));
    assert_eq!(req.source, Some(SourceId::from("chip-source".to_owned())));

    // The damage stream MUST remain empty pre-tick.
    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        0,
        "fire() must NOT write any DamageDealt<Cell> synchronously",
    );
}

#[test]
fn fire_with_zero_velocity_falls_back_to_unit_y_direction() {
    let mut app = build_app();
    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::ZERO),
        ))
        .id();

    let config = PiercingBeamConfig {
        damage_mult: OrderedFloat(1.0),
        width:       OrderedFloat(20.0),
    };
    config.fire(
        source,
        "chip-source",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );

    let req_buf = app
        .world()
        .resource::<Messages<PiercingBeamEmissionRequested>>();
    let collected: Vec<PiercingBeamEmissionRequested> =
        req_buf.iter_current_update_messages().cloned().collect();
    assert_eq!(collected.len(), 1);
    assert_eq!(
        collected[0].direction,
        Vec2::Y,
        "zero velocity → normalize_or(Vec2::Y) returns Vec2::Y",
    );
}
