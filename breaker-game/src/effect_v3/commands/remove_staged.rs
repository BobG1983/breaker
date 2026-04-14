//! Remove a specific staged effect entry by (source, tree) identity.

use bevy::prelude::*;

use crate::effect_v3::{storage::StagedEffects, types::Tree};

/// Deferred command that removes the FIRST `StagedEffects` entry whose
/// source name AND tree both match the given values. Removes exactly ONE
/// entry — not a source sweep. Does NOT touch `BoundEffects`.
///
/// This is the consume primitive used by `walk_staged_effects` after a
/// staged entry fires or arms its inner. Entry-specific so a fresh stage
/// queued later in the same command flush under the same source name is
/// preserved (it is a different entry — different tuple — so the scan
/// leaves it alone).
///
/// Compare: `RemoveEffectCommand` (existing) sweeps ALL matching source
/// entries from BOTH `BoundEffects` and `StagedEffects`. Use that for
/// explicit disarm/cleanup. Use `RemoveStagedEffectCommand` for the
/// one-shot consume contract of `walk_staged_effects`.
///
/// Visibility is restricted to `effect_v3` — this is an internal consume
/// primitive paired with `walk_staged_effects`. External domains must go
/// through `EffectCommandsExt::remove_staged_effect`.
pub(in crate::effect_v3) struct RemoveStagedEffectCommand {
    pub entity: Entity,
    pub name:   String,
    pub tree:   Tree,
}

impl Command for RemoveStagedEffectCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut staged) = world.get_mut::<StagedEffects>(self.entity)
            && let Some(pos) = staged
                .0
                .iter()
                .position(|(n, t)| n == &self.name && t == &self.tree)
        {
            staged.0.remove(pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::SpeedBoostConfig,
        storage::{BoundEffects, StagedEffects},
        types::EffectType,
    };

    fn speed_tree(mult: f32) -> Tree {
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(mult),
        }))
    }

    #[test]
    fn removes_first_matching_name_and_tree_entry() {
        let mut world = World::new();
        let target = speed_tree(1.5);
        let other = speed_tree(2.0);
        let entity = world
            .spawn(StagedEffects(vec![
                ("chip_a".to_owned(), other.clone()),
                ("chip_a".to_owned(), target.clone()),
                ("chip_a".to_owned(), target.clone()),
            ]))
            .id();

        RemoveStagedEffectCommand {
            entity,
            name: "chip_a".to_owned(),
            tree: target.clone(),
        }
        .apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 2);
        assert_eq!(staged.0[0], ("chip_a".to_owned(), other));
        assert_eq!(staged.0[1], ("chip_a".to_owned(), target));
    }

    #[test]
    fn no_op_when_entry_not_present() {
        let mut world = World::new();
        let kept = speed_tree(1.5);
        let entity = world
            .spawn(StagedEffects(vec![("chip_a".to_owned(), kept.clone())]))
            .id();

        RemoveStagedEffectCommand {
            entity,
            name: "chip_a".to_owned(),
            tree: speed_tree(2.0),
        }
        .apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(staged.0[0], ("chip_a".to_owned(), kept));
    }

    #[test]
    fn no_op_when_entity_has_no_staged_effects_component() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        RemoveStagedEffectCommand {
            entity,
            name: "chip_a".to_owned(),
            tree: speed_tree(1.5),
        }
        .apply(&mut world);

        assert!(world.get::<StagedEffects>(entity).is_none());
    }

    #[test]
    fn does_not_touch_bound_effects() {
        let mut world = World::new();
        let tree = speed_tree(1.5);
        let entity = world
            .spawn((
                BoundEffects(vec![("chip_a".to_owned(), tree.clone())]),
                StagedEffects(vec![("chip_a".to_owned(), tree.clone())]),
            ))
            .id();

        RemoveStagedEffectCommand {
            entity,
            name: "chip_a".to_owned(),
            tree: tree.clone(),
        }
        .apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert!(staged.0.is_empty());

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0], ("chip_a".to_owned(), tree));
    }
}
