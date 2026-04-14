//! Stamp effect command — sugar for `route_effect` with `RouteType::Bound`.

use bevy::prelude::*;

use crate::effect_v3::{storage::BoundEffects, types::Tree};

/// Deferred command that stamps (permanently installs) a tree on an entity.
/// Sugar for `RouteEffectCommand` with `RouteType::Bound`.
pub struct StampEffectCommand {
    /// The entity to install the tree on.
    pub entity: Entity,
    /// The name identifying the source of the tree.
    pub name:   String,
    /// The tree to install.
    pub tree:   Tree,
}

impl Command for StampEffectCommand {
    fn apply(self, world: &mut World) {
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
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::{PiercingConfig, SpeedBoostConfig},
        types::EffectType,
    };

    #[test]
    fn stamp_effect_command_inserts_tree_into_bound_effects() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        StampEffectCommand {
            entity,
            name: "stamp_chip".to_owned(),
            tree: Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 3 })),
        }
        .apply(&mut world);

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0].0, "stamp_chip");
    }

    #[test]
    fn stamp_effect_command_appends_to_existing_bound_effects() {
        let mut world = World::new();
        let existing_tree = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let entity = world
            .spawn(BoundEffects(vec![("existing".to_owned(), existing_tree)]))
            .id();

        StampEffectCommand {
            entity,
            name: "stamp_chip".to_owned(),
            tree: Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 3 })),
        }
        .apply(&mut world);

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 2);
    }
}
