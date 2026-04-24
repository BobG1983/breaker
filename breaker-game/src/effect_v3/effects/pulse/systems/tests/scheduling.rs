//! Scheduling tests for `apply_pulse_damage`.
//!
//! Production guarantee: `apply_pulse_damage` is tagged
//! `.in_set(EffectV3Systems::Tick)`, and `EffectV3Plugin` configures
//! `EffectV3Systems::Tick.before(DmgSystems::EmitDamage)`. These tests pin
//! the functional consequence: a single `tick(...)` applies the dealer's
//! `DamageBoostStack` multiplier to the ring emission same-tick. Target-side
//! `VulnerableStack` goes through the identical `DmgSystems::ApplyVulnerable`
//! pipeline stage — already pinned by `shockwave` and `tether_beam`
//! scheduling tests, so not re-asserted here.
//!
//! No `BoltPlugin` is installed in these scenarios — only the pulse emitter
//! runs, so the pipeline's `boost × vuln` multiplies cleanly once (no pre-W6
//! double-application is involved here).

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use crate::{
    effect_v3::effects::pulse::components::{
        PulseRing, PulseRingBaseDamage, PulseRingDamageMultiplier, PulseRingDamaged,
        PulseRingMaxRadius, PulseRingRadius, PulseRingSpeed,
    },
    prelude::*,
    shared::GameDrawLayer,
};

/// Shared app builder for behavior 3 — `with_effects_pipeline()` installs
/// `RantzDmgPlugin`, registers all game `Dmgable`s, and adds `EffectV3Plugin`
/// (which schedules `apply_pulse_damage` in `EffectV3Systems::Tick`).
fn pulse_scheduling_app() -> App {
    TestAppBuilder::new()
        .with_physics()
        .with_effects_pipeline()
        .build()
}

/// Spawns a cell at `(x, y)` with `Hp(hp)` and the `Cell` marker.
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

/// Spawns a stable pulse-ring entity with `radius = 100.0`, `speed = 0.0` so
/// the ring does not move during the test tick.
fn spawn_pulse_ring(app: &mut App, pos: Vec2, base_damage: f32) -> Entity {
    app.world_mut()
        .spawn((
            PulseRing,
            PulseRingRadius(100.0),
            PulseRingMaxRadius(200.0),
            PulseRingSpeed(0.0),
            PulseRingDamaged(HashSet::new()),
            PulseRingBaseDamage(base_damage),
            PulseRingDamageMultiplier(1.0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id()
}

fn read_hp(app: &App, cell: Entity) -> Option<f32> {
    app.world().get::<Hp>(cell).map(|h| h.current)
}

// ── Behavior 3 HP delta: dealer-side DamageBoostStack same-tick ────────────

#[test]
fn apply_pulse_damage_applies_damage_boost_in_same_tick() {
    let mut app = pulse_scheduling_app();

    let ring = spawn_pulse_ring(&mut app, Vec2::ZERO, 10.0);
    app.world_mut().entity_mut(ring).insert({
        let mut stack = DamageBoostStack::default();
        stack.add(SourceId::from("test"), 2.0);
        stack
    });

    let cell = spawn_cell_with_hp(&mut app, 50.0, 0.0, 100.0);

    tick(&mut app);

    let hp = read_hp(&app, cell).unwrap_or(f32::NAN);
    assert!(
        (hp - 80.0).abs() < 1e-5,
        "final_hp = 100.0 − (10.0 × 1.0 × 2.0) == 80.0 (DamageBoostStack 2.0 \
         applied same-tick), got {hp}"
    );
}

// ── Behavior 3 edge: identity (no DamageBoostStack) ────────────────────────

#[test]
fn apply_pulse_damage_without_damage_boost_uses_identity() {
    let mut app = pulse_scheduling_app();

    let _ring = spawn_pulse_ring(&mut app, Vec2::ZERO, 10.0);
    let cell = spawn_cell_with_hp(&mut app, 50.0, 0.0, 100.0);

    tick(&mut app);

    let hp = read_hp(&app, cell).unwrap_or(f32::NAN);
    assert!(
        (hp - 90.0).abs() < 1e-5,
        "final_hp = 100.0 − (10.0 × 1.0 × 1.0) == 90.0 (no DamageBoostStack = \
         identity), got {hp}"
    );
}
