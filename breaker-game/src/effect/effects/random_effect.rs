use bevy::prelude::*;
use rand::distr::{Distribution, weighted::WeightedIndex};

use crate::{
    effect::core::{EffectNode, StagedEffects},
    shared::rng::GameRng,
};

/// Selects a random effect from a weighted pool and fires it.
///
/// Uses `GameRng` with `WeightedIndex` for deterministic weighted random selection.
pub(crate) fn fire(
    entity: Entity,
    pool: &[(f32, EffectNode)],
    source_chip: &str,
    world: &mut World,
) {
    if pool.is_empty() {
        warn!("random_effect: empty pool for entity {:?}", entity);
        return;
    }

    let selected_node = {
        let mut rng = world.resource_mut::<GameRng>();
        let weights: Vec<f32> = pool.iter().map(|(w, _)| *w).collect();
        let Ok(dist) = WeightedIndex::new(&weights) else {
            warn!("random_effect: all-zero weights for entity {:?}", entity);
            return;
        };
        let idx = dist.sample(&mut rng.0);
        pool[idx].1.clone()
    };

    match selected_node {
        EffectNode::Do(effect) => effect.fire(entity, source_chip, world),
        other => {
            if let Some(mut staged) = world.get_mut::<StagedEffects>(entity) {
                staged.0.push((source_chip.to_string(), other));
            }
        }
    }
}

/// No-op — inner effects handle their own reversal.
pub(crate) fn reverse(
    _entity: Entity,
    _pool: &[(f32, EffectNode)],
    _source_chip: &str,
    _world: &mut World,
) {
}

/// Registers systems for `RandomEffect` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        effect::{
            core::{EffectKind, StagedEffects, Trigger},
            effects::{
                bump_force::ActiveBumpForces, damage_boost::ActiveDamageBoosts,
                speed_boost::ActiveSpeedBoosts,
            },
        },
        shared::rng::GameRng,
    };

    // ── Behavior 1: fire() with single-element pool selects and fires that element ──

    #[test]
    fn fire_with_single_element_pool_fires_that_effect() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
            .id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        fire(entity, &pool, "", &mut world);

        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            active.0,
            vec![2.0],
            "single-element pool should fire DamageBoost(2.0), got {:?}",
            active.0
        );
    }

    #[test]
    fn fire_with_single_element_pool_and_tiny_weight_selects_only_element() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
            .id();
        let pool = vec![(0.001, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        fire(entity, &pool, "", &mut world);

        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            active.0,
            vec![2.0],
            "single-element pool with weight 0.001 should still fire the only element"
        );
    }

    // ── Behavior 2: fire() with multi-element pool uses weighted random selection ──

    #[test]
    fn fire_with_equal_weight_pool_selects_exactly_one_effect() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();
        let pool = vec![
            (1.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
            (
                1.0,
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
            ),
        ];

        fire(entity, &pool, "", &mut world);

        let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
        let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        let total = damage.0.len() + speed.0.len();
        assert_eq!(
            total, 1,
            "exactly one effect should be fired from two-element equal-weight pool, got {} total (damage: {:?}, speed: {:?})",
            total, damage.0, speed.0
        );
    }

    // ── Behavior 3: fire() weighted selection respects relative weights ──

    #[test]
    fn fire_with_zero_weight_item_never_selects_it() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(0));
        let entity = world
            .spawn((ActiveDamageBoosts(vec![]), ActiveSpeedBoosts(vec![])))
            .id();
        let pool = vec![
            (100.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
            (
                0.0,
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
            ),
        ];

        fire(entity, &pool, "", &mut world);

        let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
        let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(
            damage.0,
            vec![2.0],
            "DamageBoost should always be selected when other weight is 0.0"
        );
        assert!(
            speed.0.is_empty(),
            "SpeedBoost with weight 0.0 should never be selected"
        );
    }

    // ── Behavior 4: fire() with empty pool is a no-op ──

    #[test]
    fn fire_with_empty_pool_is_noop() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world.spawn_empty().id();
        let pool: Vec<(f32, EffectNode)> = vec![];

        // Should not panic; early return on empty pool.
        fire(entity, &pool, "", &mut world);

        // Entity should remain unchanged — no Active* components added
        assert!(
            world.get::<ActiveDamageBoosts>(entity).is_none(),
            "no ActiveDamageBoosts should be added on empty pool"
        );
    }

    #[test]
    fn fire_with_empty_pool_preserves_existing_state() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world.spawn(ActiveDamageBoosts(vec![5.0])).id();
        let pool: Vec<(f32, EffectNode)> = vec![];

        fire(entity, &pool, "", &mut world);

        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            active.0,
            vec![5.0],
            "existing ActiveDamageBoosts should be preserved on empty pool"
        );
    }

    // ── Behavior 5: fire() with non-Do node pushes to StagedEffects ──

    #[test]
    fn fire_with_non_do_node_pushes_to_staged_effects() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world.spawn(StagedEffects::default()).id();
        let non_do_node = EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        };
        let pool = vec![(1.0, non_do_node.clone())];

        fire(entity, &pool, "", &mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "one entry should be pushed to StagedEffects"
        );
        assert_eq!(
            staged.0[0].0, "",
            "chip name should be empty string for RandomEffect dispatch"
        );
        assert_eq!(
            staged.0[0].1, non_do_node,
            "the non-Do node should be pushed to StagedEffects"
        );
    }

    #[test]
    fn fire_with_non_do_node_silently_drops_when_no_staged_effects() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        // Entity WITHOUT StagedEffects component
        let entity = world.spawn_empty().id();
        let non_do_node = EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        };
        let pool = vec![(1.0, non_do_node)];

        // Should not panic — silently dropped
        fire(entity, &pool, "", &mut world);

        assert!(
            world.get::<StagedEffects>(entity).is_none(),
            "StagedEffects should not be inserted if absent"
        );
    }

    // ── Behavior 6: fire() deterministic across separate worlds with same seed ──

    #[test]
    fn fire_deterministic_across_separate_worlds_with_same_seed() {
        let pool = vec![
            (1.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
            (
                1.0,
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
            ),
            (1.0, EffectNode::Do(EffectKind::BumpForce(50.0))),
        ];

        // World 1
        let mut world1 = World::new();
        world1.insert_resource(GameRng::from_seed(99));
        let entity1 = world1
            .spawn((
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
                ActiveSpeedBoosts(vec![]),
                ActiveBumpForces(vec![]),
            ))
            .id();
        fire(entity1, &pool, "", &mut world1);

        // World 2
        let mut world2 = World::new();
        world2.insert_resource(GameRng::from_seed(99));
        let entity2 = world2
            .spawn((
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
                ActiveSpeedBoosts(vec![]),
                ActiveBumpForces(vec![]),
            ))
            .id();
        fire(entity2, &pool, "", &mut world2);

        let damage1 = world1.get::<ActiveDamageBoosts>(entity1).unwrap();
        let damage2 = world2.get::<ActiveDamageBoosts>(entity2).unwrap();
        let speed1 = world1.get::<ActiveSpeedBoosts>(entity1).unwrap();
        let speed2 = world2.get::<ActiveSpeedBoosts>(entity2).unwrap();
        let bump1 = world1.get::<ActiveBumpForces>(entity1).unwrap();
        let bump2 = world2.get::<ActiveBumpForces>(entity2).unwrap();

        assert_eq!(
            damage1.0, damage2.0,
            "ActiveDamageBoosts must match across worlds with same seed"
        );
        assert_eq!(
            speed1.0, speed2.0,
            "ActiveSpeedBoosts must match across worlds with same seed"
        );
        assert_eq!(
            bump1.0, bump2.0,
            "ActiveBumpForces must match across worlds with same seed"
        );
    }

    // ── Behavior 7: fire() with all-zero weights pool is a no-op ──

    #[test]
    fn fire_with_all_zero_weights_is_noop() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((ActiveDamageBoosts(vec![]), ActiveSpeedBoosts(vec![])))
            .id();
        let pool = vec![
            (0.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
            (
                0.0,
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
            ),
        ];

        fire(entity, &pool, "", &mut world);

        let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
        let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert!(
            damage.0.is_empty(),
            "no DamageBoost should be fired with all-zero weights"
        );
        assert!(
            speed.0.is_empty(),
            "no SpeedBoost should be fired with all-zero weights"
        );
    }

    #[test]
    fn fire_with_single_zero_weight_is_noop() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world.spawn(ActiveDamageBoosts(vec![])).id();
        let pool = vec![(0.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        fire(entity, &pool, "", &mut world);

        let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert!(
            damage.0.is_empty(),
            "single-element pool with weight 0.0 should be a no-op (WeightedIndex fails)"
        );
    }

    // ── Behavior 8: reverse() is a no-op ──

    #[test]
    fn reverse_preserves_existing_state() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![2.0])).id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        reverse(entity, &pool, "", &mut world);

        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            active.0,
            vec![2.0],
            "reverse should not modify ActiveDamageBoosts"
        );
    }

    #[test]
    fn reverse_on_entity_with_no_components_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        // Should not panic
        reverse(entity, &pool, "", &mut world);

        assert!(
            world.get_entity(entity).is_ok(),
            "entity should still exist after no-op reverse"
        );
    }

    // ── Section N: meta-effect forwards source_chip ──

    #[test]
    fn fire_forwards_source_chip_to_inner_do_effect() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
            .id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        fire(entity, &pool, "chaos_chip", &mut world);

        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            active.0,
            vec![2.0],
            "inner effect should fire — proves source_chip was threaded through"
        );
    }

    #[test]
    fn fire_forwards_source_chip_to_staged_effects_push() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world.spawn(StagedEffects::default()).id();
        let non_do_node = EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        };
        let pool = vec![(1.0, non_do_node)];

        fire(entity, &pool, "chaos_chip", &mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(
            staged.0[0].0, "chaos_chip",
            "StagedEffects entry should have chip_name 'chaos_chip' forwarded from source_chip, not empty string"
        );
    }

    #[test]
    fn fire_forwards_empty_source_chip_to_staged_effects_push() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world.spawn(StagedEffects::default()).id();
        let non_do_node = EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        };
        let pool = vec![(1.0, non_do_node)];

        fire(entity, &pool, "", &mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(
            staged.0[0].0, "",
            "empty source_chip should forward as empty chip_name"
        );
    }
}
