//! Scheduling tests for `iron_curtain_on_bolt_lost`.
//!
//! Production guarantee: `iron_curtain_on_bolt_lost` is tagged
//! `.after(BoltSystems::BoltLost).before(EffectV3Systems::Bridge)`, and
//! `EffectV3Plugin` configures the transitive chain
//! `Bridge → Tick → DmgSystems::EmitDamage`. These tests pin the functional
//! consequence: a single `tick(...)` applies a target cell's
//! `VulnerableStack` multiplier to the Iron Curtain wave damage in the
//! same tick the wave fires. Iron Curtain emits with `dealer: None`, so
//! `DamageBoostStack` on the bolt is NOT applied — this exercises the
//! TARGET-side multiplier pathway same-tick.
//!
//! Iron Curtain fires on `BoltLost`, not on cell collision, so these tests
//! do not pair with a `bolt_cell_collision` emission — no pre-W6
//! double-apply contribution is involved here.

use bevy::{ecs::message::Messages, prelude::*};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use super::super::system::{IronCurtainConfig, register};
use crate::{
    bolt::components::BoltBaseDamage,
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolTuning},
        resources::ActiveProtocols,
    },
    shared::GameDrawLayer,
};

fn iron_curtain_scheduling_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_playfield()
        .with_bolt_registry()
        .with_breaker_registry()
        .with_cell_registry()
        .with_resource::<ActiveProtocols>()
        .with_effects_pipeline()
        .insert_resource(IronCurtainConfig {
            damage_fraction: 0.5,
            falloff_start:   1000.0,
        })
        .build();
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "IronCurtain".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::IronCurtain {
                damage_fraction: 0.5,
                falloff_start:   1000.0,
            },
        });
    register(&mut app);
    app
}

fn spawn_breaker(app: &mut App, x: f32, y: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((Breaker, Position2D(pos), GlobalPosition2D(pos), Spatial2D))
        .id()
}

fn spawn_bolt_entity(app: &mut App, x: f32, y: f32, base_damage: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Bolt,
            BoltBaseDamage(base_damage),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id()
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
    stack.add(SourceId::from("test"), vuln);
    app.world_mut().entity_mut(entity).insert(stack);
    entity
}

fn read_hp(app: &App, cell: Entity) -> Option<f32> {
    app.world().get::<Hp>(cell).map(|h| h.current)
}

fn write_bolt_lost(app: &mut App, bolt: Entity, breaker: Entity) {
    app.world_mut()
        .resource_mut::<Messages<BoltLost>>()
        .write(BoltLost { bolt, breaker });
}

#[test]
fn iron_curtain_on_bolt_lost_applies_vulnerable_stack_in_same_tick() {
    let mut app = iron_curtain_scheduling_app();
    let breaker = spawn_breaker(&mut app, 0.0, -100.0);
    let bolt = spawn_bolt_entity(&mut app, 0.0, -500.0, 10.0);
    let cell = spawn_cell_with_hp_and_vuln(&mut app, 0.0, 0.0, 100.0, 2.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp = read_hp(&app, cell).unwrap_or(f32::NAN);
    // Wave origin damage: base × damage_fraction = 10.0 × 0.5 = 5.0.
    // Vulnerability 2.0 multiplies: 5.0 × 2.0 = 10.0.
    // final_hp = 100.0 − 10.0 == 90.0.
    assert!(
        (hp - 90.0).abs() < 1e-5,
        "final_hp = 100.0 − (10.0 × 0.5 × 2.0) == 90.0, got {hp}"
    );
}

#[test]
fn iron_curtain_on_bolt_lost_without_vuln_uses_identity() {
    let mut app = iron_curtain_scheduling_app();
    let breaker = spawn_breaker(&mut app, 0.0, -100.0);
    let bolt = spawn_bolt_entity(&mut app, 0.0, -500.0, 10.0);
    let cell = spawn_cell_with_hp(&mut app, 0.0, 0.0, 100.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp = read_hp(&app, cell).unwrap_or(f32::NAN);
    // No vulnerability: final_hp = 100.0 − (10.0 × 0.5) == 95.0.
    assert!(
        (hp - 95.0).abs() < 1e-5,
        "final_hp = 100.0 − 5.0 == 95.0 (no VulnerableStack), got {hp}"
    );
}
