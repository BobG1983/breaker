//! `AttractionConfig` — attraction steering toward entities.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::components::{ActiveAttractions, AttractionEntry};
use crate::effect_v3::{
    traits::{Fireable, Reversible},
    types::AttractionType,
};

/// Configuration for bolt attraction toward a target entity type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttractionConfig {
    /// Which entity type the bolt steers toward.
    pub attraction_type: AttractionType,
    /// Attraction strength per tick.
    pub force:           OrderedFloat<f32>,
    /// Optional cap on the per-tick steering delta (None = uncapped).
    pub max_force:       Option<OrderedFloat<f32>>,
}

impl Fireable for AttractionConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }

        let entry = AttractionEntry {
            source:          source.to_owned(),
            attraction_type: self.attraction_type,
            force:           self.force.0,
            max_force:       self.max_force.map(|f| f.0),
        };

        if let Some(mut active) = world.get_mut::<ActiveAttractions>(entity) {
            active.0.push(entry);
        } else {
            world
                .entity_mut(entity)
                .insert(ActiveAttractions(vec![entry]));
        }
    }

    fn register(app: &mut App) {
        use super::systems::apply_attraction_forces;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            apply_attraction_forces.in_set(EffectV3Systems::Tick),
        );
    }
}

impl Reversible for AttractionConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut active) = world.get_mut::<ActiveAttractions>(entity)
            && let Some(idx) = active
                .0
                .iter()
                .position(|e| e.source == source && e.attraction_type == self.attraction_type)
        {
            active.0.remove(idx);
        }
    }
}
