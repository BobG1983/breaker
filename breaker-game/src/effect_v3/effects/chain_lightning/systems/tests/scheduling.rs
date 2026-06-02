//! Scheduling tests for `tick_chain_lightning`.
//!
//! Production guarantee: `tick_chain_lightning` is tagged
//! `.in_set(EffectV3Systems::Tick)`, and `EffectV3Plugin` configures
//! `EffectV3Systems::Tick.before(DmgSystems::EmitDamage)`. These tests pin
//! the functional consequence: when an arc arrives at a target cell, a
//! single `tick(...)` applies the chain entity's `DamageBoostStack`
//! multiplier in the same tick as the emission.
//!
//! No `BoltPlugin` is installed in these scenarios — only the chain-lightning
//! emitter runs, so the pipeline multiplies cleanly once (no pre-W6
//! double-application is involved here).

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use crate::{
    bolt::test_utils::damage_stack,
    effect_v3::effects::chain_lightning::components::{ChainLightningChain, ChainState},
    prelude::*,
    shared::GameDrawLayer,
};

fn chain_scheduling_app() -> App {
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

#[test]
fn tick_chain_lightning_applies_damage_boost_in_same_tick() {
    let mut app = chain_scheduling_app();

    let cell = spawn_cell_with_hp(&mut app, 50.0, 0.0, 100.0);

    // Live scratch entity for the `arc_entity` field — never panics in debug
    // the way `Entity::PLACEHOLDER` might when component requirements exist.
    let arc_entity = app.world_mut().spawn_empty().id();

    // Spawn the chain with arc already at target_pos — so the FIRST
    // `FixedUpdate` tick transitions `ArcTraveling → Idle` AND writes the
    // `DamageDealt<Cell>` in the same tick.
    let chain_entity = app
        .world_mut()
        .spawn((
            ChainLightningChain {
                remaining_jumps: 1,
                damage:          10.0,
                hit_set:         HashSet::new(),
                state:           ChainState::ArcTraveling {
                    target: cell,
                    target_pos: Vec2::new(50.0, 0.0),
                    arc_entity,
                    arc_pos: Vec2::new(50.0, 0.0),
                },
                range:           100.0,
                arc_speed:       9999.0,
                source_pos:      Vec2::ZERO,
                tick:            0,
            },
            Position2D(Vec2::ZERO),
            GlobalPosition2D(Vec2::ZERO),
            Spatial2D,
        ))
        .id();

    app.world_mut()
        .entity_mut(chain_entity)
        .insert(damage_stack(&[2.0]));

    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    assert!(
        (hp - 80.0).abs() < 1e-5,
        "final_hp = 100.0 − (10.0 × 2.0) == 80.0 (chain damage 10.0 × \
         DamageBoostStack 2.0 applied same-tick), got {hp}"
    );
}

#[test]
fn tick_chain_lightning_without_damage_boost_uses_identity() {
    let mut app = chain_scheduling_app();

    let cell = spawn_cell_with_hp(&mut app, 50.0, 0.0, 100.0);

    let arc_entity = app.world_mut().spawn_empty().id();

    app.world_mut().spawn((
        ChainLightningChain {
            remaining_jumps: 1,
            damage:          10.0,
            hit_set:         HashSet::new(),
            state:           ChainState::ArcTraveling {
                target: cell,
                target_pos: Vec2::new(50.0, 0.0),
                arc_entity,
                arc_pos: Vec2::new(50.0, 0.0),
            },
            range:           100.0,
            arc_speed:       9999.0,
            source_pos:      Vec2::ZERO,
            tick:            0,
        },
        Position2D(Vec2::ZERO),
        GlobalPosition2D(Vec2::ZERO),
        Spatial2D,
    ));

    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    assert!(
        (hp - 90.0).abs() < 1e-5,
        "final_hp = 100.0 − 10.0 == 90.0 (identity), got {hp}"
    );
}
