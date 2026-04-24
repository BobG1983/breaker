//! Scheduling tests for `debt_collector_on_impact`.
//!
//! Production guarantee: `debt_collector_on_impact` is tagged
//! `.after(BoltSystems::CellCollision).before(EffectV3Systems::Bridge)`,
//! and `EffectV3Plugin` configures the transitive chain
//! `Bridge → Tick → DmgSystems::EmitDamage`. These tests pin the functional
//! consequence: when the bolt carries `DamageBoostStack` and `DebtCashOut`,
//! a single `tick(...)` applies the dealer's `DamageBoostStack` multiplier
//! to the amplified `DamageDealt<Cell>` in the same tick as the emission.
//! The spec explicitly excludes `DebtStack` from the bolt — production
//! never carries BOTH `DebtStack` and `DebtCashOut`.
//!
//! These scenarios pair the protocol emission with `bolt_cell_collision`'s
//! own emission in the same tick. Post-W6 both emissions are single-apply:
//! `starting_hp − (base × boost) − (base × cashout × boost)`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use super::super::system::{DebtCashOut, DebtCollectorConfig, register};
use crate::{
    bolt::{
        BoltPlugin,
        test_utils::{damage_stack, default_bolt_definition, spawn_bolt},
    },
    cells::resources::CellConfig,
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolTuning},
        resources::ActiveProtocols,
    },
    shared::GameDrawLayer,
};

fn debt_collector_scheduling_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_playfield()
        .with_bolt_registry()
        .with_breaker_registry()
        .with_cell_registry()
        .with_resource::<ActiveProtocols>()
        .with_resource::<crate::input::resources::InputActions>()
        .with_effects_pipeline()
        .insert_resource(DebtCollectorConfig {
            stack_per_bump: 1.0,
        })
        .build();
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "DebtCollector".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::DebtCollector {
                stack_per_bump: 1.0,
            },
        });
    app.add_plugins(BoltPlugin);
    register(&mut app);
    app
}

fn spawn_cell_with_hp(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
    let cc = CellConfig::default();
    let half_extents = Vec2::new(cc.width / 2.0, cc.height / 2.0);
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            crate::cells::components::CellWidth::new(cc.width),
            crate::cells::components::CellHeight::new(cc.height),
            Hp::new(hp),
            KilledBy { killer: None },
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

fn read_hp(app: &App, cell: Entity) -> Option<f32> {
    app.world().get::<Hp>(cell).map(|h| h.current)
}

#[test]
fn debt_collector_on_impact_applies_damage_boost_in_same_tick() {
    let mut app = debt_collector_scheduling_app();
    let bc = default_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell = spawn_cell_with_hp(&mut app, 0.0, cell_y, 200.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[2.0]))
        // Perfect-bump primed — DebtCashOut carries captured stack value 2.0.
        // Do NOT add DebtStack — production never has both simultaneously.
        .insert(DebtCashOut(2.0));

    tick(&mut app);

    let hp = read_hp(&app, cell).unwrap_or(f32::NAN);
    // Post-W6 formula (single-apply on baseline):
    //   200.0 − (bolt_base × boost) − (bolt_base × cashout × boost)
    //     = 200.0 − (10.0 × 2.0) − (10.0 × 2.0 × 2.0)
    //     = 200.0 − 20.0 − 40.0
    //     = 140.0
    assert!(
        (hp - 140.0).abs() < 1e-5,
        "final_hp = 140.0 (baseline 20.0 + debt 40.0 dropped from 200.0), got {hp}"
    );
}

#[test]
fn debt_collector_on_impact_without_boost_uses_identity() {
    let mut app = debt_collector_scheduling_app();
    let bc = default_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell = spawn_cell_with_hp(&mut app, 0.0, cell_y, 200.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(DebtCashOut(2.0));

    tick(&mut app);

    let hp = read_hp(&app, cell).unwrap_or(f32::NAN);
    // Without boost: baseline 10.0 (no double-apply) + debt single-apply 20.0
    // = 30.0 damage; final_hp = 200.0 − 30.0 == 170.0.
    assert!(
        (hp - 170.0).abs() < 1e-5,
        "final_hp = 200.0 − 10.0 − 20.0 == 170.0 (no boost), got {hp}"
    );
}
