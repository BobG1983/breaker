//! `ShieldConfig` — shield wall protection.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
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

        world.spawn((
            ShieldWall,
            ShieldOwner(entity),
            ShieldDuration(self.duration.0),
            ShieldReflectionCost(self.reflection_cost.0),
            chip,
            CleanupOnExit::<NodeState>::default(),
        ));
    }

    fn register(app: &mut App) {
        use super::systems::tick_shield_duration;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            tick_shield_duration.in_set(EffectV3Systems::Tick),
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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::traits::Fireable;

    fn make_config() -> ShieldConfig {
        ShieldConfig {
            duration:        OrderedFloat(5.0),
            reflection_cost: OrderedFloat(0.5),
        }
    }

    // ── C5: Shield resets existing shield timer instead of spawning second ──

    #[test]
    fn shield_fire_resets_existing_shield_duration() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();

        // Spawn an existing shield with duration 1.0.
        world.spawn((
            ShieldWall,
            ShieldOwner(owner),
            ShieldDuration(1.0),
            ShieldReflectionCost(0.5),
        ));

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        // There should still be exactly 1 shield.
        let shields: Vec<(Entity, &ShieldDuration)> = world
            .query_filtered::<(Entity, &ShieldDuration), With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(shields.len(), 1, "should have exactly 1 shield wall, not 2");
        assert!(
            (shields[0].1.0 - 5.0).abs() < f32::EPSILON,
            "shield duration should be reset to 5.0, got {}",
            shields[0].1.0,
        );
    }

    #[test]
    fn shield_fire_resets_nearly_expired_shield() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();

        world.spawn((
            ShieldWall,
            ShieldOwner(owner),
            ShieldDuration(0.01),
            ShieldReflectionCost(0.5),
        ));

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let shields: Vec<&ShieldDuration> = world
            .query_filtered::<&ShieldDuration, With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(shields.len(), 1, "should have exactly 1 shield wall");
        assert!(
            (shields[0].0 - 5.0).abs() < f32::EPSILON,
            "nearly-expired shield should be reset to 5.0, got {}",
            shields[0].0,
        );
    }

    #[test]
    fn shield_fire_spawns_new_when_none_exists() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let shields: Vec<(&ShieldOwner, &ShieldDuration, &ShieldReflectionCost)> = world
            .query_filtered::<(&ShieldOwner, &ShieldDuration, &ShieldReflectionCost), With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(shields.len(), 1, "should spawn exactly 1 shield wall");
        assert_eq!(shields[0].0.0, owner);
        assert!(
            (shields[0].1.0 - 5.0).abs() < f32::EPSILON,
            "shield duration should be 5.0, got {}",
            shields[0].1.0,
        );
        assert!(
            (shields[0].2.0 - 0.5).abs() < f32::EPSILON,
            "shield reflection cost should be 0.5, got {}",
            shields[0].2.0,
        );
    }

    #[test]
    fn shield_fire_does_not_reset_another_owners_shield() {
        let mut world = World::new();
        let owner_a = world.spawn_empty().id();
        let owner_b = world.spawn_empty().id();

        // B's existing shield with duration 2.0.
        world.spawn((
            ShieldWall,
            ShieldOwner(owner_b),
            ShieldDuration(2.0),
            ShieldReflectionCost(0.5),
        ));

        make_config().fire(owner_a, "aegis", &mut world);
        world.flush();

        let shields: Vec<(&ShieldOwner, &ShieldDuration)> = world
            .query_filtered::<(&ShieldOwner, &ShieldDuration), With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(
            shields.len(),
            2,
            "should have 2 shield walls (A's new + B's existing)"
        );

        // Find B's shield and check it's unchanged.
        let b_shield = shields.iter().find(|(o, _)| o.0 == owner_b);
        assert!(b_shield.is_some(), "B's shield should still exist");
        assert!(
            (b_shield.unwrap().1.0 - 2.0).abs() < f32::EPSILON,
            "B's shield duration should remain 2.0, got {}",
            b_shield.unwrap().1.0,
        );

        // Find A's shield.
        let a_shield = shields.iter().find(|(o, _)| o.0 == owner_a);
        assert!(a_shield.is_some(), "A's new shield should exist");
        assert!(
            (a_shield.unwrap().1.0 - 5.0).abs() < f32::EPSILON,
            "A's shield duration should be 5.0, got {}",
            a_shield.unwrap().1.0,
        );
    }
}
