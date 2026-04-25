//! Shared test fixtures for `apply_piercing_beam_damage` integration tests.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use crate::{
    bolt::components::BoltBaseDamage, chips::definition::Rarity,
    effect_v3::effects::piercing_beam::config::PiercingBeamConfig, prelude::*,
};

/// Builder-format `SourceId` for the chip-namespaced source string.
pub(super) fn piercing_beam_chip_source() -> SourceId {
    SourceId::chip("Piercing Beam").rarity(Rarity::Rare).build()
}

pub(super) fn piercing_beam_chip_source_str() -> String {
    piercing_beam_chip_source().0.into_owned()
}

/// `with_effects_pipeline()` test app that registers `PiercingBeamConfig`
/// (registers the message + consumer system once GREEN lands), plus message
/// captures for `DamageDealt<Cell>` and `PiercingBeamEmissionRequested`.
pub(super) fn piercing_pipeline_app() -> App {
    use super::super::super::messages::PiercingBeamEmissionRequested;

    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_message_capture::<PiercingBeamEmissionRequested>()
        .build()
}

/// Beam-friendly source: `BoltBaseDamage(10.0)`, `Position2D(Vec2::ZERO)`,
/// `Velocity2D(Vec2::new(0.0, 400.0))` (i.e. firing along +Y).
pub(super) fn spawn_beam_source(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id()
}

/// Spawns a cell with `Hp::new(hp)` plus the rest of the components a
/// fully-indexed game-side cell carries. The piercing beam consumer iterates
/// the cells query directly (no quadtree dependency), but cells still need
/// these standard components for the dmg pipeline (Hp + `KilledBy`).
pub(super) fn spawn_cell_with_hp(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Hp::new(hp),
            KilledBy { killer: None },
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id()
}

/// Default no-multiplier piercing beam config (`damage_mult = 1.0`,
/// `width = 20.0`) — `base_damage = bolt_base_damage * 1.0 = 10.0`.
pub(super) fn make_config() -> PiercingBeamConfig {
    PiercingBeamConfig {
        damage_mult: OrderedFloat(1.0),
        width:       OrderedFloat(20.0),
    }
}
