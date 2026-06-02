//! W7 Behavior 1 (Explode): `fire()` writes ONLY an
//! `ExplodeEmissionRequested` and does NOT touch `Messages<DamageDealt<Cell>>`
//! synchronously. The consumer system (registered in `EmitDamage`) later
//! produces the per-cell damage messages — but at the snapshot moment
//! immediately after `fire()`, only the request is observable.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::SeedableRng;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use super::super::super::{config::ExplodeConfig, messages::ExplodeEmissionRequested};
use crate::{effect_v3::traits::Fireable, prelude::*};

fn build_app() -> App {
    TestAppBuilder::new()
        .with_physics()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_message_capture::<ExplodeEmissionRequested>()
        .build()
}

fn spawn_indexed_cell(app: &mut App, pos: Vec2) -> Entity {
    let e = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            Aabb2D::new(Vec2::ZERO, Vec2::splat(5.0)),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
        ))
        .id();
    app.world_mut().run_schedule(FixedUpdate);
    e
}

#[test]
fn fire_writes_one_explode_emission_requested_with_correct_geometry() {
    let mut app = build_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let _c1 = spawn_indexed_cell(&mut app, Vec2::new(20.0, 0.0));
    let _c2 = spawn_indexed_cell(&mut app, Vec2::new(30.0, 0.0));
    let _c3 = spawn_indexed_cell(&mut app, Vec2::new(50.0, 0.0));

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

    // Snapshot immediately — NO `app.update()` between fire and assertions.
    let req_buf = app.world().resource::<Messages<ExplodeEmissionRequested>>();
    let collected: Vec<ExplodeEmissionRequested> =
        req_buf.iter_current_update_messages().cloned().collect();

    assert_eq!(
        collected.len(),
        1,
        "fire() must write exactly one ExplodeEmissionRequested"
    );
    let req = &collected[0];
    assert_eq!(req.center, Vec2::ZERO, "center mirrors source Position2D");
    assert!(
        (req.radius - 50.0).abs() < f32::EPSILON,
        "radius mirrors config.range, got {}",
        req.radius,
    );
    assert!(
        (req.base_damage - 10.0).abs() < f32::EPSILON,
        "base_damage mirrors config.damage RAW (no boost), got {}",
        req.base_damage,
    );
    assert_eq!(req.dealer, Some(source));
    assert_eq!(req.source, Some(SourceId::from("chip-source".to_owned())));

    // The damage stream MUST remain empty pre-tick — fire() must not have
    // touched `Messages<DamageDealt<Cell>>`.
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
fn fire_with_zero_range_writes_request_with_zero_radius_and_no_damage_messages() {
    let mut app = build_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let _cell = spawn_indexed_cell(&mut app, Vec2::ZERO);

    let config = ExplodeConfig {
        range:  OrderedFloat(0.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(
        source,
        "chip-source",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );

    let req_buf = app.world().resource::<Messages<ExplodeEmissionRequested>>();
    let collected: Vec<ExplodeEmissionRequested> =
        req_buf.iter_current_update_messages().cloned().collect();
    assert_eq!(collected.len(), 1);
    assert!((collected[0].radius - 0.0).abs() < f32::EPSILON);

    // Even with a coincident cell, fire() must NOT inspect the cell graph.
    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(collector.0.len(), 0);
}
