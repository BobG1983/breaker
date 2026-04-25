//! Shared test fixtures for `apply_explode_damage` integration tests.
//!
//! Mirrors the spawn helpers in
//! `breaker-game/src/effect_v3/effects/explode/config.rs` (the inline `mod
//! tests`) but reuses `with_effects_pipeline()` so `RantzDmgPlugin` +
//! `EffectV3Plugin` (which invokes `ExplodeConfig::register`) are wired in.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use crate::{chips::definition::Rarity, prelude::*};

/// Builder-format `SourceId` used for the chip-namespaced source string in
/// behaviors 1, 5, 8 (Explode side).
pub(super) fn explode_chip_source() -> SourceId {
    SourceId::chip("Devastating Splinter")
        .rarity(Rarity::Rare)
        .build()
}

/// String form of [`explode_chip_source`] suitable for passing to
/// `ExplodeConfig::fire(.., &source_str, ..)`.
pub(super) fn explode_chip_source_str() -> String {
    explode_chip_source().0.into_owned()
}

/// `TestAppBuilder::new().with_physics().with_state_hierarchy()
/// .in_state_node_playing().with_effects_pipeline()
/// .with_message_capture::<DamageDealt<Cell>>()
/// .with_message_capture::<ExplodeEmissionRequested>().build()`.
///
/// Used by behaviors 2, 3, 4, 6, 7, 8, 10, 11. `with_effects_pipeline()`
/// installs `EffectV3Plugin`, which calls `ExplodeConfig::register(app)` —
/// in the GREEN phase that registration adds `apply_explode_damage` to
/// `DmgSystems::EmitDamage`. At RED time the registration is missing and
/// these tests are expected to fail.
pub(super) fn explode_pipeline_app() -> App {
    use super::super::super::messages::ExplodeEmissionRequested;

    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_effects_pipeline()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_message_capture::<ExplodeEmissionRequested>()
        .build()
}

/// Spawns a cell with quadtree-indexable components, then runs one
/// `FixedUpdate` schedule so `maintain_quadtree` indexes it. Used by
/// quadtree-dependent behaviors (Explode broad phase).
pub(super) fn spawn_cell_with_hp(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    let e = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(hp),
            KilledBy { killer: None },
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

/// Spawns a `Dead` cell at `pos`. Used by behavior 7's edge-case to confirm
/// the consumer's `Without<Dead>` filter excludes it.
pub(super) fn spawn_dead_cell_at(app: &mut App, pos: Vec2) -> Entity {
    let e = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(100.0),
            KilledBy { killer: None },
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            Aabb2D::new(Vec2::ZERO, Vec2::splat(5.0)),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Dead,
        ))
        .id();
    app.world_mut().run_schedule(FixedUpdate);
    e
}
