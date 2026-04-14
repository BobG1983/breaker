//! Stage effect command — sugar for `route_effect` with `RouteType::Staged`.

use bevy::prelude::*;

use crate::effect_v3::{
    storage::{BoundEffects, StagedEffects},
    types::Tree,
};

/// Deferred command that stages (one-shot installs) a tree on an entity.
/// Sugar for `RouteEffectCommand` with `RouteType::Staged`.
pub struct StageEffectCommand {
    /// The entity to install the tree on.
    pub entity: Entity,
    /// The name identifying the source of the tree.
    pub name:   String,
    /// The tree to install.
    pub tree:   Tree,
}

impl Command for StageEffectCommand {
    fn apply(self, world: &mut World) {
        // Guard against despawned entities — `entity_mut` panics on a
        // missing entity, and a despawn can be queued in the same tick as
        // the stage command.
        if world.get_entity(self.entity).is_err() {
            return;
        }

        // Enforce the invariant that any entity with `StagedEffects` also
        // has `BoundEffects`. Bridges query
        // `(&BoundEffects, Option<&StagedEffects>)` and would otherwise
        // skip entities that only have staged entries. Inserting an empty
        // `BoundEffects` component keeps staged-only entities reachable
        // through the bridge bound query without requiring callers to
        // seed it manually.
        let has_bound = world.get::<BoundEffects>(self.entity).is_some();
        if !has_bound {
            world
                .entity_mut(self.entity)
                .insert(BoundEffects::default());
        }

        let has_staged = world.get::<StagedEffects>(self.entity).is_some();
        if !has_staged {
            world
                .entity_mut(self.entity)
                .insert(StagedEffects::default());
        }
        if let Some(mut staged) = world.get_mut::<StagedEffects>(self.entity) {
            staged.0.push((self.name, self.tree));
        }
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::{BumpForceConfig, SpeedBoostConfig},
        types::EffectType,
    };

    #[test]
    fn stage_effect_command_inserts_tree_into_staged_effects() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        StageEffectCommand {
            entity,
            name: "stage_chip".to_owned(),
            tree: Tree::Fire(EffectType::BumpForce(BumpForceConfig {
                multiplier: OrderedFloat(1.8),
            })),
        }
        .apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(staged.0[0].0, "stage_chip");
    }

    #[test]
    fn stage_effect_command_appends_to_existing_staged_effects() {
        let mut world = World::new();
        let existing_tree = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let entity = world
            .spawn(StagedEffects(vec![("existing".to_owned(), existing_tree)]))
            .id();

        StageEffectCommand {
            entity,
            name: "stage_chip".to_owned(),
            tree: Tree::Fire(EffectType::BumpForce(BumpForceConfig {
                multiplier: OrderedFloat(1.8),
            })),
        }
        .apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 2);
    }

    #[test]
    fn stage_effect_command_does_not_panic_when_entity_despawned() {
        // Regression: StageEffectCommand::apply used to call
        // world.entity_mut(self.entity) unconditionally, which panics on a
        // despawned entity. A despawn can be queued in the same tick as
        // the stage command (e.g., by a Bridge-set system that runs before
        // Conditions). The guard at the top of apply() must short-circuit.
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.despawn(entity);

        StageEffectCommand {
            entity,
            name: "stage_chip".to_owned(),
            tree: Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
        }
        .apply(&mut world);
        // No panic — pass. No observable state change: the entity is
        // gone, so no BoundEffects or StagedEffects component lingers.
    }
}
