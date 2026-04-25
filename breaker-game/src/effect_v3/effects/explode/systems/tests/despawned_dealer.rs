//! W7 Behavior 11 (Explode): the consumer reads a buffered request whose
//! `dealer` was despawned between `fire()` and the consumer tick. The test
//! must not panic; `dealer` is preserved verbatim on the emitted
//! `DamageDealt<Cell>` (the despawned entity id is still present — kill
//! attribution / further pipeline stages own that semantics).

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{super::super::config::ExplodeConfig, helpers::*};
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn dealer_despawned_after_fire_before_consumer_tick_no_panic() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    let dealer_entity = source;
    config.fire(source, "chip-source", app.world_mut());
    // Despawn the dealer AFTER fire() but BEFORE the consumer tick.
    app.world_mut().despawn(source);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        1,
        "consumer must produce one DamageDealt<Cell> from buffered request",
    );
    let msg = &collector.0[0];
    assert_eq!(
        msg.dealer,
        Some(dealer_entity),
        "dealer entity id is preserved verbatim — consumer does NOT clear it",
    );
    assert_eq!(msg.target, cell);

    let hp = app.world().get::<Hp>(cell).expect("Hp").current;
    assert!(
        (hp - 90.0).abs() < 1e-4,
        "damage still applied — dealer liveness has no bearing on amount",
    );
}

#[test]
fn dealer_despawned_before_fire_falls_back_to_zero_position_no_panic() {
    let mut app = explode_pipeline_app();
    let source = app
        .world_mut()
        .spawn(Position2D(Vec2::new(999.0, 999.0)))
        .id();
    // Despawn BEFORE fire() — fire's `world.get::<Position2D>(source)`
    // returns None and the production code falls back to `Vec2::ZERO`. A
    // cell at (20, 0) is then within range from the fallback origin.
    app.world_mut().despawn(source);
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(source, "", app.world_mut());
    tick(&mut app);

    // No panic. The cell at (20, 0) is in range from fallback (0, 0).
    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].target, cell);
}
