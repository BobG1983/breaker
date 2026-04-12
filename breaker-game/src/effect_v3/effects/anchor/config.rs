//! `AnchorConfig` — anchor bolt in place with enhanced bump.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::components::{AnchorActive, AnchorPlanted, AnchorTimer};
use crate::effect_v3::traits::{Fireable, Reversible};

/// Configuration for the anchor effect on the breaker.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnchorConfig {
    /// How much the bump force is multiplied when planted.
    pub bump_force_multiplier:     OrderedFloat<f32>,
    /// How much wider the perfect timing window becomes when planted.
    pub perfect_window_multiplier: OrderedFloat<f32>,
    /// Seconds the breaker must stand still before planting.
    pub plant_delay:               OrderedFloat<f32>,
}

impl Fireable for AnchorConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }
        world.entity_mut(entity).insert((
            AnchorActive {
                bump_force_multiplier:     self.bump_force_multiplier.0,
                perfect_window_multiplier: self.perfect_window_multiplier.0,
                plant_delay:               self.plant_delay.0,
            },
            AnchorTimer(self.plant_delay.0),
        ));
    }

    fn register(app: &mut App) {
        use super::systems::tick_anchor;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(FixedUpdate, tick_anchor.in_set(EffectV3Systems::Tick));
    }
}

impl Reversible for AnchorConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }
        world
            .entity_mut(entity)
            .remove::<AnchorActive>()
            .remove::<AnchorTimer>()
            .remove::<AnchorPlanted>();
    }
}
