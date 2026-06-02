//! `PiercingBeamConfig` — fire-and-forget piercing beam line.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::super::messages::PiercingBeamEmissionRequested;
use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    effect_v3::traits::Fireable,
    prelude::*,
};

/// Fires a beam that damages all cells along a line.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PiercingBeamConfig {
    /// Multiplier applied to base damage for cells hit by the beam.
    pub damage_mult: OrderedFloat<f32>,
    /// Width of the beam rectangle in world units.
    pub width:       OrderedFloat<f32>,
}

impl Fireable for PiercingBeamConfig {
    fn fire(
        &self,
        entity: Entity,
        source: &str,
        world: &mut World,
        _rng: &mut rand_chacha::ChaCha8Rng,
    ) {
        let origin = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let direction = world
            .get::<Velocity2D>(entity)
            .map_or(Vec2::Y, |v| v.0.normalize_or(Vec2::Y));
        let bolt_base_damage = world
            .get::<BoltBaseDamage>(entity)
            .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);

        let source_id = (!source.is_empty()).then(|| SourceId::from(source.to_owned()));

        world
            .resource_mut::<Messages<PiercingBeamEmissionRequested>>()
            .write(PiercingBeamEmissionRequested {
                origin,
                direction,
                half_width: self.width.0 / 2.0,
                base_damage: bolt_base_damage * self.damage_mult.0,
                dealer: Some(entity),
                source: source_id,
            });
    }

    fn register(app: &mut App) {
        use super::super::systems::apply_piercing_beam_damage;

        app.add_message::<PiercingBeamEmissionRequested>();

        app.add_systems(
            FixedUpdate,
            apply_piercing_beam_damage.in_set(DmgSystems::EmitDamage),
        );
    }
}
