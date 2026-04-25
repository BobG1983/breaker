//! Scheduling tests for `tick_tether_beam`.
//!
//! Production guarantee: `tick_tether_beam` is tagged
//! `.in_set(EffectV3Systems::Tick)`, and `EffectV3Plugin` configures
//! `EffectV3Systems::Tick.before(DmgSystems::EmitDamage)`. These tests pin
//! the functional consequence: a single `tick(...)` applies dealer-side
//! `DamageBoostStack` (on the beam) and target-side `VulnerableStack` (on
//! the cell) multipliers in the same tick as the emission.
//!
//! No `BoltPlugin` is installed in these scenarios — only the tether-beam
//! emitter runs, so the pipeline's `boost × vuln` multiplies cleanly once
//! (no pre-W6 double-application is involved here).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use crate::{
    chips::definition::Rarity,
    effect_v3::effects::tether_beam::components::{
        TetherBeamDamage, TetherBeamSource, TetherBeamWidth,
    },
    prelude::*,
    shared::GameDrawLayer,
};

/// Builder-format `SourceId` used as the canonical opaque tag for the
/// `DamageBoost` / Vulnerable stack augmentation in scheduling tests.
fn test_source() -> SourceId {
    SourceId::chip("Test").rarity(Rarity::Common).build()
}

fn tether_scheduling_app() -> App {
    TestAppBuilder::new()
        .with_physics()
        .with_effects_pipeline()
        .build()
}

fn spawn_cell_with_hp(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            Hp::new(hp),
            KilledBy { killer: None },
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

fn spawn_cell_with_hp_and_vuln(app: &mut App, x: f32, y: f32, hp: f32, vuln: f32) -> Entity {
    let entity = spawn_cell_with_hp(app, x, y, hp);
    let mut stack = VulnerableStack::default();
    stack.add(test_source(), vuln);
    app.world_mut().entity_mut(entity).insert(stack);
    entity
}

/// Spawns a tether beam anchored on two `Position2D`-only entities. The
/// system reads plain `Position2D` from the endpoints; do NOT tag them with
/// `Bolt` (not needed).
fn spawn_tether_beam(app: &mut App, base_damage: f32) -> Entity {
    let bolt_a = app
        .world_mut()
        .spawn(Position2D(Vec2::new(-50.0, 0.0)))
        .id();
    let bolt_b = app.world_mut().spawn(Position2D(Vec2::new(50.0, 0.0))).id();

    app.world_mut()
        .spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(base_damage),
            TetherBeamWidth(20.0),
        ))
        .id()
}

#[test]
fn tick_tether_beam_applies_damage_boost_in_same_tick() {
    let mut app = tether_scheduling_app();

    let beam = spawn_tether_beam(&mut app, 10.0);
    app.world_mut().entity_mut(beam).insert({
        let mut stack = DamageBoostStack::default();
        stack.add(test_source(), 2.0);
        stack
    });

    let cell = spawn_cell_with_hp(&mut app, 0.0, 0.0, 100.0);

    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    assert!(
        (hp - 80.0).abs() < 1e-5,
        "final_hp = 100.0 − (10.0 × 2.0) == 80.0 (DamageBoostStack 2.0 applied \
         same-tick), got {hp}"
    );
}

#[test]
fn tick_tether_beam_applies_boost_and_vulnerability_in_same_tick() {
    let mut app = tether_scheduling_app();

    let beam = spawn_tether_beam(&mut app, 10.0);
    app.world_mut().entity_mut(beam).insert({
        let mut stack = DamageBoostStack::default();
        stack.add(test_source(), 2.0);
        stack
    });

    let cell = spawn_cell_with_hp_and_vuln(&mut app, 0.0, 0.0, 100.0, 2.0);

    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    assert!(
        (hp - 60.0).abs() < 1e-5,
        "final_hp = 100.0 − (10.0 × 2.0 × 2.0) == 60.0, got {hp}"
    );
}
