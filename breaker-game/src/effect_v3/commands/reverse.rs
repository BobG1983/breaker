//! Reverse effect command — deferred execution of `Reversible::reverse`.

use bevy::prelude::*;

use crate::effect_v3::{dispatch::reverse_dispatch, types::ReversibleEffectType};

/// Deferred command that reverses a reversible effect on an entity.
pub struct ReverseEffectCommand {
    /// The entity to reverse the effect on.
    pub entity: Entity,
    /// The reversible effect to reverse.
    pub effect: ReversibleEffectType,
    /// The chip or definition name that originated this effect.
    pub source: String,
}

impl Command for ReverseEffectCommand {
    fn apply(self, world: &mut World) {
        reverse_dispatch(&self.effect, self.entity, &self.source, world);
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::SpeedBoostConfig, stacking::EffectStack, traits::Fireable,
        types::ReversibleEffectType,
    };

    #[test]
    fn reverse_effect_command_removes_fired_effect_from_stack() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        };

        // Fire first to set up the stack.
        config.fire(entity, "test_chip", &mut world);
        assert_eq!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity)
                .unwrap()
                .len(),
            1
        );

        ReverseEffectCommand {
            entity,
            effect: ReversibleEffectType::SpeedBoost(config),
            source: "test_chip".to_owned(),
        }
        .apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert!(stack.is_empty());
    }

    #[test]
    fn reverse_effect_command_does_not_panic_when_effect_was_never_fired() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        ReverseEffectCommand {
            entity,
            effect: ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }),
            source: "test_chip".to_owned(),
        }
        .apply(&mut world);

        // No stack at all — command should not panic.
        assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
    }
}
