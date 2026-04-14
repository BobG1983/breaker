//! Route effect command — installs a tree on a target entity.

use bevy::prelude::*;

use crate::effect_v3::{
    storage::{BoundEffects, StagedEffects},
    types::{RouteType, Tree},
};

/// Deferred command that routes a tree to an entity.
pub struct RouteEffectCommand {
    /// The entity to install the tree on.
    pub entity:     Entity,
    /// The name identifying the source of the tree.
    pub name:       String,
    /// The tree to install.
    pub tree:       Tree,
    /// Whether the tree is permanent (Bound) or one-shot (Staged).
    pub route_type: RouteType,
}

impl Command for RouteEffectCommand {
    fn apply(self, world: &mut World) {
        match self.route_type {
            RouteType::Bound => {
                let has_bound = world.get::<BoundEffects>(self.entity).is_some();
                if !has_bound {
                    world
                        .entity_mut(self.entity)
                        .insert(BoundEffects::default());
                }
                if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
                    bound.0.push((self.name, self.tree));
                }
            }
            RouteType::Staged => {
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
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        types::EffectType,
    };

    // ── RouteType::Bound ──────────────────────────────────────────────

    #[test]
    fn route_bound_inserts_tree_into_bound_effects() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let tree = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        RouteEffectCommand {
            entity,
            name: "test_chip".to_owned(),
            tree: tree.clone(),
            route_type: RouteType::Bound,
        }
        .apply(&mut world);

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0].0, "test_chip");
        assert_eq!(bound.0[0].1, tree);
    }

    #[test]
    fn route_bound_appends_to_existing_bound_effects() {
        let mut world = World::new();
        let existing_tree = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }));
        let entity = world
            .spawn(BoundEffects(vec![(
                "existing_chip".to_owned(),
                existing_tree,
            )]))
            .id();

        let new_tree = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        RouteEffectCommand {
            entity,
            name: "test_chip".to_owned(),
            tree: new_tree,
            route_type: RouteType::Bound,
        }
        .apply(&mut world);

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 2);
    }

    // ── RouteType::Staged ─────────────────────────────────────────────

    #[test]
    fn route_staged_inserts_tree_into_staged_effects() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let tree = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }));

        RouteEffectCommand {
            entity,
            name: "test_chip".to_owned(),
            tree: tree.clone(),
            route_type: RouteType::Staged,
        }
        .apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(staged.0[0].0, "test_chip");
        assert_eq!(staged.0[0].1, tree);
    }

    #[test]
    fn route_staged_appends_to_existing_staged_effects() {
        let mut world = World::new();
        let existing_tree = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let entity = world
            .spawn(StagedEffects(vec![(
                "existing_chip".to_owned(),
                existing_tree,
            )]))
            .id();

        let new_tree = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }));

        RouteEffectCommand {
            entity,
            name: "test_chip".to_owned(),
            tree: new_tree,
            route_type: RouteType::Staged,
        }
        .apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 2);
    }
}
