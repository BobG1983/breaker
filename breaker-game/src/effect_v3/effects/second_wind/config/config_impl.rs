//! `SecondWindConfig` — one-shot bottom wall.

use bevy::prelude::*;
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
        let playfield = world.resource::<PlayfieldConfig>().clone();

        let mut commands = world.commands();
        let wall_entity = Wall::builder().floor(&playfield).spawn(&mut commands);
        commands
            .entity(wall_entity)
            .insert((SecondWindWall, SecondWindOwner(entity), chip));
    }

    fn register(app: &mut App) {
        use super::super::systems::despawn_on_first_reflection;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            despawn_on_first_reflection.in_set(EffectV3Systems::Tick),
        );
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
