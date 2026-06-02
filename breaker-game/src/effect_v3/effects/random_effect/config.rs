//! `RandomEffectConfig` — fire-and-forget random effect selection.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::effect_v3::{dispatch::fire_dispatch_with_rng, traits::Fireable};

/// Picks a random effect from a weighted pool and fires it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RandomEffectConfig {
    /// Weighted list of effects — each entry is (weight, effect). Fires exactly one per activation.
    pub pool: Vec<(OrderedFloat<f32>, Box<crate::effect_v3::types::EffectType>)>,
}

impl Fireable for RandomEffectConfig {
    fn fire(
        &self,
        entity: Entity,
        source: &str,
        world: &mut World,
        rng: &mut rand_chacha::ChaCha8Rng,
    ) {
        if self.pool.is_empty() {
            return;
        }

        let total_weight: f32 = self.pool.iter().map(|(w, _)| w.0).sum();
        if total_weight <= 0.0 {
            return;
        }

        let roll: f32 = rng.random::<f32>() * total_weight;
        let mut accumulated = 0.0_f32;
        let mut selected: Option<&crate::effect_v3::types::EffectType> = None;
        for (weight, effect) in &self.pool {
            accumulated += weight.0;
            if roll < accumulated {
                selected = Some(effect.as_ref());
                break;
            }
        }
        // Fallback to last entry on f32 imprecision (the previous loop only
        // exits early on a strict-less-than match, so a roll equal to
        // total_weight lands here).
        let selected = selected.unwrap_or_else(|| self.pool[self.pool.len() - 1].1.as_ref());

        fire_dispatch_with_rng(selected, entity, source, world, rng);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

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
        prelude::*,
        shared::rng::{EffectBaseSeed, EffectEventCounter},
    };

    fn world_with_rng(seed: u64) -> (World, ChaCha8Rng) {
        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(seed));
        world.insert_resource(EffectEventCounter::default());
        let rng = ChaCha8Rng::seed_from_u64(seed);
        (world, rng)
    }

    // ── Tests ─────────────────────────────────────────────────────────────

    #[test]
    fn single_entry_pool_always_selects_that_entry() {
        let (mut world, mut rng) = world_with_rng(42);
        let entity = world.spawn_empty().id();

        let config = RandomEffectConfig {
            pool: vec![(
                OrderedFloat(1.0),
                Box::new(EffectType::FlashStep(FlashStepConfig {})),
            )],
        };
        config.fire(entity, "flux_chip", &mut world, &mut rng);

        assert!(
            world.get::<FlashStepActive>(entity).is_some(),
            "single-entry pool should always fire FlashStep"
        );
    }

    #[test]
    fn empty_pool_is_noop() {
        let (mut world, mut rng) = world_with_rng(42);
        let entity = world.spawn_empty().id();

        let config = RandomEffectConfig { pool: vec![] };
        config.fire(entity, "flux_chip", &mut world, &mut rng);

        // No panic, no components added.
        assert!(world.get::<FlashStepActive>(entity).is_none());
        assert!(world.get::<Dead>(entity).is_none());
    }

    #[test]
    fn zero_total_weight_is_noop() {
        let (mut world, mut rng) = world_with_rng(42);
        let entity = world.spawn_empty().id();

        let config = RandomEffectConfig {
            pool: vec![(
                OrderedFloat(0.0),
                Box::new(EffectType::FlashStep(FlashStepConfig {})),
            )],
        };
        config.fire(entity, "flux_chip", &mut world, &mut rng);

        assert!(
            world.get::<FlashStepActive>(entity).is_none(),
            "zero-weight pool should trigger early return"
        );
    }

    #[test]
    fn weighted_selection_is_deterministic_with_seeded_rng() {
        let (mut world1, mut rng1) = world_with_rng(42);
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
        config.fire(entity1, "flux_chip", &mut world1, &mut rng1);

        let has_flash_1 = world1.get::<FlashStepActive>(entity1).is_some();
        let has_dead_1 = world1.get::<Dead>(entity1).is_some();
        assert!(has_flash_1 || has_dead_1, "exactly one effect should fire");
        assert!(!(has_flash_1 && has_dead_1), "should not fire both effects");

        // Second world with same seed produces identical outcome.
        let (mut world2, mut rng2) = world_with_rng(42);
        let entity2 = world2.spawn_empty().id();
        config.fire(entity2, "flux_chip", &mut world2, &mut rng2);

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
            let (mut world, mut rng) = world_with_rng(seed);
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
            config.fire(entity, "flux_chip", &mut world, &mut rng);

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

    // ── B11 — fire() uses passed &mut ChaCha8Rng, not a GameRng resource ──

    #[test]
    fn random_effect_fire_uses_passed_rng_not_game_rng() {
        // World has NO GameRng resource — must not panic.
        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(0));
        world.insert_resource(EffectEventCounter::default());
        let entity = world.spawn_empty().id();

        let config = RandomEffectConfig {
            pool: vec![
                (
                    OrderedFloat(0.9),
                    Box::new(EffectType::FlashStep(FlashStepConfig {})),
                ),
                (OrderedFloat(0.1), Box::new(EffectType::Die(DieConfig {}))),
            ],
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        config.fire(entity, "src", &mut world, &mut rng);

        // Must have applied exactly one of the two effects.
        let has_flash = world.get::<FlashStepActive>(entity).is_some();
        let has_dead = world.get::<Dead>(entity).is_some();
        assert!(
            has_flash || has_dead,
            "exactly one effect must have fired without GameRng present"
        );
        assert!(
            !(has_flash && has_dead),
            "both effects must not fire in the same call"
        );
    }

    #[test]
    fn random_effect_100_sequential_seeds_produce_expected_distribution() {
        // 100 fires with sequential seeds 0..100 over [(0.9, FlashStep), (0.1, Die)]
        // must produce a Flash:Die ratio close to 9:1.
        let mut flash_count = 0;
        let mut die_count = 0;

        for seed in 0_u64..100 {
            let mut world = World::new();
            let entity = world.spawn_empty().id();
            let config = RandomEffectConfig {
                pool: vec![
                    (
                        OrderedFloat(0.9),
                        Box::new(EffectType::FlashStep(FlashStepConfig {})),
                    ),
                    (OrderedFloat(0.1), Box::new(EffectType::Die(DieConfig {}))),
                ],
            };
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            config.fire(entity, "src", &mut world, &mut rng);

            if world.get::<FlashStepActive>(entity).is_some() {
                flash_count += 1;
            } else if world.get::<Dead>(entity).is_some() {
                die_count += 1;
            }
        }

        assert!(
            flash_count >= 80,
            "expected flash_count >= 80, got {flash_count} (die_count = {die_count})"
        );
        assert!(
            die_count >= 1,
            "expected at least one Die fire across 100 seeds, got 0"
        );
    }

    // ── B20 — single_entry_pool preserved under new signature ─────────────

    #[test]
    fn random_effect_single_entry_pool_always_selects_that_entry_new_sig() {
        // B20: same as single_entry_pool_always_selects_that_entry but with
        // explicit new-signature call.
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = RandomEffectConfig {
            pool: vec![(
                OrderedFloat(1.0),
                Box::new(EffectType::FlashStep(FlashStepConfig {})),
            )],
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        config.fire(entity, "flux_chip", &mut world, &mut rng);

        assert!(
            world.get::<FlashStepActive>(entity).is_some(),
            "single-entry pool must always select FlashStep regardless of RNG draw"
        );
        assert!(
            world.get::<Dead>(entity).is_none(),
            "single-entry pool must not add Dead"
        );
    }
}
