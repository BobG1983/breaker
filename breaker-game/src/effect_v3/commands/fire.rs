//! Fire effect command — deferred execution of `Fireable::fire`.

use bevy::prelude::*;

use crate::effect_v3::{dispatch::fire_dispatch, types::EffectType};

/// Deferred command that fires an effect on an entity.
pub struct FireEffectCommand {
    /// The entity to apply the effect to.
    pub entity: Entity,
    /// The effect to fire.
    pub effect: EffectType,
    /// The chip or definition name that originated this effect.
    pub source: String,
}

impl Command for FireEffectCommand {
    fn apply(self, world: &mut World) {
        fire_dispatch(&self.effect, self.entity, &self.source, world);
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

    #[test]
    fn fire_effect_command_delegates_to_fire_dispatch() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        FireEffectCommand {
            entity,
            effect: EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }),
            source: "test_chip".to_owned(),
        }
        .apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn fire_effect_command_stacks_when_fired_twice() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let effect = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });

        FireEffectCommand {
            entity,
            effect: effect.clone(),
            source: "test_chip".to_owned(),
        }
        .apply(&mut world);

        FireEffectCommand {
            entity,
            effect,
            source: "test_chip".to_owned(),
        }
        .apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn fire_effect_command_preserves_source_string() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        FireEffectCommand {
            entity,
            effect: EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            }),
            source: "my_source_chip".to_owned(),
        }
        .apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        let (source, _config) = stack.iter().next().unwrap();
        assert_eq!(source, "my_source_chip");
    }

    #[test]
    fn fire_effect_command_empty_source_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        FireEffectCommand {
            entity,
            effect: EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            }),
            source: String::new(),
        }
        .apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }
}
