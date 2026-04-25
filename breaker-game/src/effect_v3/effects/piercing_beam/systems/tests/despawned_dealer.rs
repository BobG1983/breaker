//! W7 Behavior 11 (PiercingBeam): despawning the dealer between `fire()`
//! and the consumer tick must not panic; the dealer Entity id is preserved
//! verbatim on every emitted `DamageDealt<Cell>`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{bolt::components::BoltBaseDamage, effect_v3::traits::Fireable, prelude::*};

#[test]
fn dealer_despawned_after_fire_before_consumer_tick_no_panic() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);

    let dealer_entity = source;
    make_config().fire(source, "chip-source", app.world_mut());
    app.world_mut().despawn(source);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(collector.0.len(), 1);
    let msg = &collector.0[0];
    assert_eq!(
        msg.dealer,
        Some(dealer_entity),
        "dealer Entity id preserved verbatim — consumer must NOT clear it",
    );
    assert_eq!(msg.target, cell);

    let hp = app.world().get::<Hp>(cell).expect("Hp").current;
    assert!((hp - 90.0).abs() < 1e-4);
}

#[test]
fn dealer_despawned_before_fire_falls_back_no_panic() {
    let mut app = piercing_pipeline_app();
    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(999.0, 999.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();
    // Despawn BEFORE fire() — fire's `world.get::<Position2D>` returns None
    // and falls back to `Vec2::ZERO`; `world.get::<Velocity2D>` also returns
    // None, falling back to `Vec2::Y`. A cell at (0, 50) is still in the
    // beam from fallback origin/direction.
    app.world_mut().despawn(source);
    let cell = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);

    make_config().fire(source, "", app.world_mut());
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].target, cell);
}
