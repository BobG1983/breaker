//! Remove effect command — removes matching effect trees from an entity.

use bevy::prelude::*;

use crate::effect_v3::storage::{BoundEffects, StagedEffects};

/// Deferred command that removes all effect trees matching a given name
/// from an entity's `BoundEffects` and `StagedEffects`.
pub struct RemoveEffectCommand {
    /// The entity to remove effect trees from.
    pub entity: Entity,
    /// The name to match against.
    pub name:   String,
}

impl Command for RemoveEffectCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
            bound.0.retain(|(name, _)| name != &self.name);
        }
        if let Some(mut staged) = world.get_mut::<StagedEffects>(self.entity) {
            staged.0.retain(|(name, _)| name != &self.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::{
            DamageBoostConfig, FlashStepConfig, PiercingConfig, QuickStopConfig, SpeedBoostConfig,
        },
        types::{EffectType, Tree},
    };

    #[test]
    fn remove_command_removes_matching_entries_from_bound_effects() {
        let mut world = World::new();
        let tree_a = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let tree_b = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }));
        let tree_c = Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 }));

        let entity = world
            .spawn(BoundEffects(vec![
                ("chip_a".to_owned(), tree_a),
                ("chip_b".to_owned(), tree_b.clone()),
                ("chip_a".to_owned(), tree_c),
            ]))
            .id();

        RemoveEffectCommand {
            entity,
            name: "chip_a".to_owned(),
        }
        .apply(&mut world);

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0].0, "chip_b");
        assert_eq!(bound.0[0].1, tree_b);
    }

    #[test]
    fn remove_command_no_matching_name_leaves_bound_effects_unchanged() {
        let mut world = World::new();
        let tree = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let entity = world
            .spawn(BoundEffects(vec![("chip_a".to_owned(), tree)]))
            .id();

        RemoveEffectCommand {
            entity,
            name: "nonexistent".to_owned(),
        }
        .apply(&mut world);

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1);
    }

    #[test]
    fn remove_command_removes_matching_entries_from_staged_effects() {
        let mut world = World::new();
        let tree_x = Tree::Fire(EffectType::QuickStop(QuickStopConfig {
            multiplier: OrderedFloat(1.2),
        }));
        let tree_y = Tree::Fire(EffectType::FlashStep(FlashStepConfig {}));

        let entity = world
            .spawn(StagedEffects(vec![
                ("chip_x".to_owned(), tree_x),
                ("chip_y".to_owned(), tree_y.clone()),
            ]))
            .id();

        RemoveEffectCommand {
            entity,
            name: "chip_x".to_owned(),
        }
        .apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(staged.0[0].0, "chip_y");
        assert_eq!(staged.0[0].1, tree_y);
    }

    #[test]
    fn remove_command_entity_with_no_staged_effects_does_not_panic() {
        let mut world = World::new();
        let tree = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let entity = world
            .spawn(BoundEffects(vec![("chip_a".to_owned(), tree)]))
            .id();

        // Entity has BoundEffects but no StagedEffects — should not panic.
        RemoveEffectCommand {
            entity,
            name: "chip_a".to_owned(),
        }
        .apply(&mut world);
    }

    #[test]
    fn remove_command_removes_from_both_bound_and_staged() {
        let mut world = World::new();
        let tree_a = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let tree_b = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }));

        let entity = world
            .spawn((
                BoundEffects(vec![("shared_chip".to_owned(), tree_a)]),
                StagedEffects(vec![("shared_chip".to_owned(), tree_b)]),
            ))
            .id();

        RemoveEffectCommand {
            entity,
            name: "shared_chip".to_owned(),
        }
        .apply(&mut world);

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 0);
        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 0);
    }

    #[test]
    fn remove_command_on_entity_with_no_effect_components_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        RemoveEffectCommand {
            entity,
            name: "nonexistent".to_owned(),
        }
        .apply(&mut world);

        // If we reach here, no panic occurred.
        assert!(world.get::<BoundEffects>(entity).is_none());
        assert!(world.get::<StagedEffects>(entity).is_none());
    }
}
