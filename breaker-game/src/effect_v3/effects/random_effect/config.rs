//! `RandomEffectConfig` — fire-and-forget random effect selection.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    effect_v3::{dispatch::fire_dispatch, traits::Fireable, types::EffectType},
    shared::rng::GameRng,
};

/// Picks a random effect from a weighted pool and fires it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RandomEffectConfig {
    /// Weighted list of effects — each entry is (weight, effect). Fires exactly one per activation.
    pub pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
}

impl Fireable for RandomEffectConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        if self.pool.is_empty() {
            return;
        }

        let total_weight: f32 = self.pool.iter().map(|(w, _)| w.0).sum();
        if total_weight <= 0.0 {
            return;
        }

        // Pick a weighted random effect from the pool.
        let roll: f32 = world.resource_mut::<GameRng>().0.random::<f32>() * total_weight;
        let mut accumulated = 0.0;
        let mut chosen: Option<&EffectType> = None;
        for (weight, effect) in &self.pool {
            accumulated += weight.0;
            if roll < accumulated {
                chosen = Some(effect);
                break;
            }
        }

        // Fallback to last entry if floating-point imprecision causes no match.
        let Some(fallback) = self.pool.last() else {
            return;
        };
        let effect = chosen.unwrap_or(&fallback.1);
        fire_dispatch(effect, entity, source, world);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::{
        effect_v3::{
            effects::{
                die::DieConfig,
                flash_step::{FlashStepActive, FlashStepConfig},
            },
            traits::Fireable,
            types::EffectType,
        },
        shared::{death_pipeline::Dead, rng::GameRng},
    };

    fn world_with_rng(seed: u64) -> World {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(seed));
        world
    }

    // ── Tests ─────────────────────────────────────────────────────────────

    #[test]
    fn single_entry_pool_always_selects_that_entry() {
        let mut world = world_with_rng(42);
        let entity = world.spawn_empty().id();

        let config = RandomEffectConfig {
            pool: vec![(
                OrderedFloat(1.0),
                Box::new(EffectType::FlashStep(FlashStepConfig {})),
            )],
        };
        config.fire(entity, "flux_chip", &mut world);

        assert!(
            world.get::<FlashStepActive>(entity).is_some(),
            "single-entry pool should always fire FlashStep"
        );
    }

    #[test]
    fn empty_pool_is_noop() {
        let mut world = world_with_rng(42);
        let entity = world.spawn_empty().id();

        let config = RandomEffectConfig { pool: vec![] };
        config.fire(entity, "flux_chip", &mut world);

        // No panic, no components added.
        assert!(world.get::<FlashStepActive>(entity).is_none());
        assert!(world.get::<Dead>(entity).is_none());
    }

    #[test]
    fn zero_total_weight_is_noop() {
        let mut world = world_with_rng(42);
        let entity = world.spawn_empty().id();

        let config = RandomEffectConfig {
            pool: vec![(
                OrderedFloat(0.0),
                Box::new(EffectType::FlashStep(FlashStepConfig {})),
            )],
        };
        config.fire(entity, "flux_chip", &mut world);

        assert!(
            world.get::<FlashStepActive>(entity).is_none(),
            "zero-weight pool should trigger early return"
        );
    }

    #[test]
    fn weighted_selection_is_deterministic_with_seeded_rng() {
        let mut world1 = world_with_rng(42);
        let entity1 = world1.spawn_empty().id();

        let config = RandomEffectConfig {
            pool: vec![
                (
                    OrderedFloat(0.9),
                    Box::new(EffectType::FlashStep(FlashStepConfig {})),
                ),
                (OrderedFloat(0.1), Box::new(EffectType::Die(DieConfig {}))),
            ],
        };
        config.fire(entity1, "flux_chip", &mut world1);

        let has_flash_1 = world1.get::<FlashStepActive>(entity1).is_some();
        let has_dead_1 = world1.get::<Dead>(entity1).is_some();
        assert!(has_flash_1 || has_dead_1, "exactly one effect should fire");
        assert!(!(has_flash_1 && has_dead_1), "should not fire both effects");

        // Second world with same seed produces identical outcome.
        let mut world2 = world_with_rng(42);
        let entity2 = world2.spawn_empty().id();
        config.fire(entity2, "flux_chip", &mut world2);

        let has_flash_2 = world2.get::<FlashStepActive>(entity2).is_some();
        let has_dead_2 = world2.get::<Dead>(entity2).is_some();
        assert_eq!(
            has_flash_1, has_flash_2,
            "same seed should produce same selection"
        );
        assert_eq!(
            has_dead_1, has_dead_2,
            "same seed should produce same selection"
        );
    }

    #[test]
    fn zero_weight_entry_in_multi_entry_pool_is_never_selected() {
        // Try several seeds — the zero-weight entry should never be selected.
        for seed in [0, 1, 42, 99, 123, 255, 999] {
            let mut world = world_with_rng(seed);
            let entity = world.spawn_empty().id();

            let config = RandomEffectConfig {
                pool: vec![
                    (OrderedFloat(0.0), Box::new(EffectType::Die(DieConfig {}))),
                    (
                        OrderedFloat(1.0),
                        Box::new(EffectType::FlashStep(FlashStepConfig {})),
                    ),
                ],
            };
            config.fire(entity, "flux_chip", &mut world);

            assert!(
                world.get::<FlashStepActive>(entity).is_some(),
                "seed {seed}: non-zero-weight FlashStep should always be selected"
            );
            assert!(
                world.get::<Dead>(entity).is_none(),
                "seed {seed}: zero-weight Die should never be selected"
            );
        }
    }
}
