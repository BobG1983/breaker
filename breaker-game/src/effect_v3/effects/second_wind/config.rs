//! `SecondWindConfig` — one-shot bottom wall.

use bevy::prelude::*;
use rantzsoft_stateflow::CleanupOnExit;
use serde::{Deserialize, Serialize};

use super::components::*;
use crate::{
    effect_v3::{
        components::EffectSourceChip,
        traits::{Fireable, Reversible},
    },
    state::types::NodeState,
};

/// Spawns an invisible one-shot bottom wall. Empty struct for trait uniformity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecondWindConfig {}

impl Fireable for SecondWindConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        let chip = EffectSourceChip(if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        });

        world.spawn((
            SecondWindWall,
            SecondWindOwner(entity),
            chip,
            CleanupOnExit::<NodeState>::default(),
        ));
    }
}

impl Reversible for SecondWindConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        // Despawn all second-wind walls owned by this entity.
        let to_despawn: Vec<Entity> = world
            .query_filtered::<(Entity, &SecondWindOwner), With<SecondWindWall>>()
            .iter(world)
            .filter(|(_, owner)| owner.0 == entity)
            .map(|(e, _)| e)
            .collect();
        for e in to_despawn {
            world.despawn(e);
        }
    }
}
