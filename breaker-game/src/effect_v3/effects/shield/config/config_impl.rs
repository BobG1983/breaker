//! `ShieldConfig` — shield wall protection.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::super::components::*;
use crate::{
    effect_v3::{
        components::EffectSourceChip,
        traits::{Fireable, Reversible},
    },
    shared::PlayfieldConfig,
    walls::components::Wall,
};

/// Configuration for a temporary shield wall that reflects bolts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShieldConfig {
    /// How long the shield wall lasts in seconds.
    pub duration:        OrderedFloat<f32>,
    /// Seconds subtracted from the shield's remaining time each time a bolt bounces off it.
    pub reflection_cost: OrderedFloat<f32>,
}

impl Fireable for ShieldConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // Check for existing shield with same owner — reset duration instead of spawning.
        let existing: Option<Entity> = world
            .query_filtered::<(Entity, &ShieldOwner), With<ShieldWall>>()
            .iter(world)
            .find(|(_, owner)| owner.0 == entity)
            .map(|(e, _)| e);

        if let Some(existing_shield) = existing {
            if let Some(mut duration) = world.get_mut::<ShieldDuration>(existing_shield) {
                duration.0 = self.duration.0;
            }
            return;
        }

        let chip = EffectSourceChip(if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        });
        let playfield = world.resource::<PlayfieldConfig>().clone();

        let mut commands = world.commands();
        let wall_entity = Wall::builder().floor(&playfield).spawn(&mut commands);
        commands.entity(wall_entity).insert((
            ShieldWall,
            ShieldOwner(entity),
            ShieldDuration(self.duration.0),
            ShieldReflectionCost(self.reflection_cost.0),
            chip,
        ));
    }

    fn register(app: &mut App) {
        use super::super::systems::{apply_shield_reflection_cost, tick_shield_duration};
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            (tick_shield_duration, apply_shield_reflection_cost)
                .chain()
                .in_set(EffectV3Systems::Tick),
        );
    }
}

impl Reversible for ShieldConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        // Despawn all shield walls owned by this entity.
        let to_despawn: Vec<Entity> = world
            .query_filtered::<(Entity, &ShieldOwner), With<ShieldWall>>()
            .iter(world)
            .filter(|(_, owner)| owner.0 == entity)
            .map(|(e, _)| e)
            .collect();
        for e in to_despawn {
            world.despawn(e);
        }
    }
}
