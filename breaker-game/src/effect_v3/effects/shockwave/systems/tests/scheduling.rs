//! Scheduling tests for `apply_shockwave_damage`.
//!
//! Production guarantee: `apply_shockwave_damage` is tagged
//! `.in_set(EffectV3Systems::Tick)`, and `EffectV3Plugin` configures
//! `EffectV3Systems::Tick.before(DmgSystems::EmitDamage)`. These tests pin
//! the functional consequence: a single `tick(...)` applies dealer-side
//! `DamageBoostStack` and target-side `VulnerableStack` multipliers in the
//! same tick as the emission.
//!
//! No `BoltPlugin` is installed in these scenarios — only the shockwave
//! emitter runs, so the pipeline's `boost × vuln` multiplies cleanly once
//! (no pre-W6 double-application is involved here).

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use crate::{
    chips::definition::Rarity,
    effect_v3::effects::shockwave::components::{
        ShockwaveBaseDamage, ShockwaveDamageMultiplier, ShockwaveDamaged, ShockwaveMaxRadius,
        ShockwaveRadius, ShockwaveSource, ShockwaveSpeed,
    },
    prelude::*,
    shared::GameDrawLayer,
};

/// Builder-format `SourceId` used as the canonical opaque tag for the
/// `DamageBoost` / Vulnerable stack augmentation in scheduling tests.
fn test_source() -> SourceId {
    SourceId::chip("Test").rarity(Rarity::Common).build()
}

fn shockwave_scheduling_app() -> App {
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

fn spawn_shockwave(app: &mut App, pos: Vec2, base_damage: f32) -> Entity {
    app.world_mut()
        .spawn((
            ShockwaveSource,
            ShockwaveRadius(100.0),
            ShockwaveMaxRadius(200.0),
            ShockwaveSpeed(0.0),
            ShockwaveDamaged(HashSet::new()),
            ShockwaveBaseDamage(base_damage),
            ShockwaveDamageMultiplier(1.0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id()
}

#[test]
fn apply_shockwave_damage_applies_damage_boost_in_same_tick() {
    let mut app = shockwave_scheduling_app();

    let sw = spawn_shockwave(&mut app, Vec2::ZERO, 10.0);
    app.world_mut().entity_mut(sw).insert({
        let mut stack = DamageBoostStack::default();
        stack.add(test_source(), 2.0);
        stack
    });

    let cell = spawn_cell_with_hp(&mut app, 50.0, 0.0, 100.0);

    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    assert!(
        (hp - 80.0).abs() < 1e-5,
        "final_hp = 100.0 − (10.0 × 1.0 × 2.0) == 80.0, got {hp}"
    );
}

#[test]
fn apply_shockwave_damage_applies_vulnerable_stack_in_same_tick() {
    let mut app = shockwave_scheduling_app();

    let _sw = spawn_shockwave(&mut app, Vec2::ZERO, 10.0);
    let cell = spawn_cell_with_hp_and_vuln(&mut app, 50.0, 0.0, 100.0, 1.5);

    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    assert!(
        (hp - 85.0).abs() < 1e-5,
        "final_hp = 100.0 − (10.0 × 1.0 × 1.5) == 85.0 (VulnerableStack 1.5 \
         applied same-tick), got {hp}"
    );
}
