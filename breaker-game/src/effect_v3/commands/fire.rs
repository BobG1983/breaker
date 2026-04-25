//! Fire effect command — deferred execution of `Fireable::fire`.

use bevy::prelude::*;

use crate::effect_v3::{dispatch::fire_dispatch, types::EffectType};

/// Deferred command that fires an effect on an entity. Constructed only
/// inside `effect_v3` — call sites use `EffectCommandsExt::fire_effect`.
pub(in crate::effect_v3) struct FireEffectCommand {
    /// The entity to apply the effect to.
    pub(in crate::effect_v3) entity: Entity,
    /// The effect to fire.
    pub(in crate::effect_v3) effect: EffectType,
    /// The chip or definition name that originated this effect.
    pub(in crate::effect_v3) source: String,
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
    use crate::{
        chips::definition::Rarity,
        effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
        prelude::{SourceId, SourceIdExt},
    };

    /// Builder-format `SourceId` for a generic test chip — used as the
    /// canonical fixture across `FireEffectCommand` tests.
    fn test_chip_source() -> SourceId {
        SourceId::chip("TestChip").rarity(Rarity::Common).build()
    }

    /// Distinct second builder-format `SourceId` for the
    /// preserves-source-string test.
    fn my_source_chip() -> SourceId {
        SourceId::chip("MySourceChip")
            .rarity(Rarity::Common)
            .build()
    }

    #[test]
    fn fire_effect_command_delegates_to_fire_dispatch() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        FireEffectCommand {
            entity,
            effect: EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }),
            source: test_chip_source().0.into_owned(),
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
            source: test_chip_source().0.into_owned(),
        }
        .apply(&mut world);

        FireEffectCommand {
            entity,
            effect,
            source: test_chip_source().0.into_owned(),
        }
        .apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn fire_effect_command_preserves_source_string() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let expected = my_source_chip();

        FireEffectCommand {
            entity,
            effect: EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            }),
            source: expected.0.clone().into_owned(),
        }
        .apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        let (source, _config) = stack.iter().next().unwrap();
        assert_eq!(source, &expected);
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
