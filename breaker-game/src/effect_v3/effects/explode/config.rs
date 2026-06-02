//! `ExplodeConfig` — fire-and-forget area explosion.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::messages::ExplodeEmissionRequested;
use crate::{effect_v3::traits::Fireable, prelude::*};

/// Area explosion dealing flat damage to all cells within range.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExplodeConfig {
    /// Radius of the explosion in world units.
    pub range:  OrderedFloat<f32>,
    /// Flat damage dealt to every cell within range.
    pub damage: OrderedFloat<f32>,
}

impl Fireable for ExplodeConfig {
    fn fire(
        &self,
        entity: Entity,
        source: &str,
        world: &mut World,
        _rng: &mut rand_chacha::ChaCha8Rng,
    ) {
        let center = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let source_id = (!source.is_empty()).then(|| SourceId::from(source.to_owned()));

        world
            .resource_mut::<Messages<ExplodeEmissionRequested>>()
            .write(ExplodeEmissionRequested {
                center,
                radius: self.range.0,
                base_damage: self.damage.0,
                dealer: Some(entity),
                source: source_id,
            });
    }

    fn register(app: &mut App) {
        use super::systems::apply_explode_damage;

        app.add_message::<ExplodeEmissionRequested>();

        app.add_systems(
            FixedUpdate,
            apply_explode_damage.in_set(DmgSystems::EmitDamage),
        );
    }
}
