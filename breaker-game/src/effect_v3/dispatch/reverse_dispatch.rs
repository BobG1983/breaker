//! `reverse_dispatch` — match `ReversibleEffectType` variant to `config.reverse()` call.

use bevy::prelude::*;

use crate::effect_v3::{
    traits::{Fireable, Reversible},
    types::ReversibleEffectType,
};

/// Dispatch a `ReversibleEffectType` to the appropriate config's `reverse()` method.
///
/// This is a mechanical match — each arm unwraps the config and calls
/// `config.reverse(entity, source, world)`.
pub fn reverse_dispatch(
    effect: &ReversibleEffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
) {
    match effect {
        ReversibleEffectType::SpeedBoost(config) => config.reverse(entity, source, world),
        ReversibleEffectType::SizeBoost(config) => config.reverse(entity, source, world),
        ReversibleEffectType::DamageBoost(config) => config.reverse(entity, source, world),
        ReversibleEffectType::BumpForce(config) => config.reverse(entity, source, world),
        ReversibleEffectType::QuickStop(config) => config.reverse(entity, source, world),
        ReversibleEffectType::FlashStep(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Piercing(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Vulnerable(config) => config.reverse(entity, source, world),
        ReversibleEffectType::RampingDamage(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Attraction(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Anchor(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Pulse(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Shield(config) => config.reverse(entity, source, world),
        ReversibleEffectType::SecondWind(config) => config.reverse(entity, source, world),
        ReversibleEffectType::CircuitBreaker(config) => config.reverse(entity, source, world),
        ReversibleEffectType::EntropyEngine(config) => config.reverse(entity, source, world),
    }
}

/// Helper to fire a `ReversibleEffectType` (all reversible effects are also fireable).
pub fn fire_reversible_dispatch(
    effect: &ReversibleEffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
) {
    match effect {
        ReversibleEffectType::SpeedBoost(config) => config.fire(entity, source, world),
        ReversibleEffectType::SizeBoost(config) => config.fire(entity, source, world),
        ReversibleEffectType::DamageBoost(config) => config.fire(entity, source, world),
        ReversibleEffectType::BumpForce(config) => config.fire(entity, source, world),
        ReversibleEffectType::QuickStop(config) => config.fire(entity, source, world),
        ReversibleEffectType::FlashStep(config) => config.fire(entity, source, world),
        ReversibleEffectType::Piercing(config) => config.fire(entity, source, world),
        ReversibleEffectType::Vulnerable(config) => config.fire(entity, source, world),
        ReversibleEffectType::RampingDamage(config) => config.fire(entity, source, world),
        ReversibleEffectType::Attraction(config) => config.fire(entity, source, world),
        ReversibleEffectType::Anchor(config) => config.fire(entity, source, world),
        ReversibleEffectType::Pulse(config) => config.fire(entity, source, world),
        ReversibleEffectType::Shield(config) => config.fire(entity, source, world),
        ReversibleEffectType::SecondWind(config) => config.fire(entity, source, world),
        ReversibleEffectType::CircuitBreaker(config) => config.fire(entity, source, world),
        ReversibleEffectType::EntropyEngine(config) => config.fire(entity, source, world),
    }
}

/// Dispatch a `ReversibleEffectType` to the appropriate config's
/// `reverse_all_by_source()` method.
///
/// Removes ALL instances of the effect fired from the given source, rather
/// than matching on config value like `reverse_dispatch` does.
pub fn reverse_all_by_source_dispatch(
    effect: &ReversibleEffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
) {
    match effect {
        ReversibleEffectType::SpeedBoost(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::SizeBoost(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::DamageBoost(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::BumpForce(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::QuickStop(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::FlashStep(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::Piercing(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::Vulnerable(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::RampingDamage(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::Attraction(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::Anchor(config) => config.reverse_all_by_source(entity, source, world),
        ReversibleEffectType::Pulse(config) => config.reverse_all_by_source(entity, source, world),
        ReversibleEffectType::Shield(config) => config.reverse_all_by_source(entity, source, world),
        ReversibleEffectType::SecondWind(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::CircuitBreaker(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
        ReversibleEffectType::EntropyEngine(config) => {
            config.reverse_all_by_source(entity, source, world);
        }
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::{
            AnchorConfig, AttractionConfig, FlashStepConfig, RampingDamageConfig, SpeedBoostConfig,
            anchor::components::{AnchorActive, AnchorPlanted, AnchorTimer},
            attraction::components::ActiveAttractions,
            flash_step::FlashStepActive,
            piercing::PiercingConfig,
            ramping_damage::components::RampingDamageAccumulator,
        },
        stacking::EffectStack,
        traits::Fireable,
        types::AttractionType,
    };

    #[test]
    fn reverse_dispatch_speed_boost_removes_from_stack() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        };

        // Fire first so there's something to reverse.
        config.fire(entity, "test_chip", &mut world);
        assert_eq!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity)
                .unwrap()
                .len(),
            1
        );

        let effect = ReversibleEffectType::SpeedBoost(config);
        reverse_dispatch(&effect, entity, "test_chip", &mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert!(stack.is_empty());
    }

    // ── reverse_all_by_source_dispatch ─────────────────────────────────

    #[test]
    fn reverse_all_by_source_dispatch_routes_speed_boost() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, "test_chip", &mut world);
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, "test_chip", &mut world);

        let effect = ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });
        reverse_all_by_source_dispatch(&effect, entity, "test_chip", &mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert!(
            stack.is_empty(),
            "all entries from test_chip should be removed"
        );
    }

    #[test]
    fn reverse_all_by_source_dispatch_routes_ramping_damage_with_accumulator_cleanup() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .fire(entity, "test_chip", &mut world);
        RampingDamageConfig {
            increment: OrderedFloat(1.0),
        }
        .fire(entity, "test_chip", &mut world);

        world
            .entity_mut(entity)
            .insert(RampingDamageAccumulator(OrderedFloat(3.0)));

        let effect = ReversibleEffectType::RampingDamage(RampingDamageConfig {
            increment: OrderedFloat(0.5),
        });
        reverse_all_by_source_dispatch(&effect, entity, "test_chip", &mut world);

        let stack = world
            .get::<EffectStack<RampingDamageConfig>>(entity)
            .unwrap();
        assert!(stack.is_empty());
        assert!(
            world.get::<RampingDamageAccumulator>(entity).is_none(),
            "accumulator should be removed when stack is empty"
        );
    }

    #[test]
    fn reverse_all_by_source_dispatch_routes_attraction() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .fire(entity, "test_chip", &mut world);
        AttractionConfig {
            attraction_type: AttractionType::Wall,
            force:           OrderedFloat(50.0),
            max_force:       None,
        }
        .fire(entity, "test_chip", &mut world);

        let effect = ReversibleEffectType::Attraction(AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        });
        reverse_all_by_source_dispatch(&effect, entity, "test_chip", &mut world);

        let active = world.get::<ActiveAttractions>(entity).unwrap();
        assert!(active.0.is_empty(), "all entries should be removed");
    }

    #[test]
    fn reverse_all_by_source_dispatch_routes_flash_step_via_default() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        FlashStepConfig {}.fire(entity, "dash_chip", &mut world);
        assert!(world.get::<FlashStepActive>(entity).is_some());

        let effect = ReversibleEffectType::FlashStep(FlashStepConfig {});
        reverse_all_by_source_dispatch(&effect, entity, "dash_chip", &mut world);

        assert!(
            world.get::<FlashStepActive>(entity).is_none(),
            "FlashStepActive should be removed"
        );
    }

    #[test]
    fn reverse_all_by_source_dispatch_routes_anchor_with_parameterized_source() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        AnchorConfig {
            bump_force_multiplier:     OrderedFloat(2.0),
            perfect_window_multiplier: OrderedFloat(1.5),
            plant_delay:               OrderedFloat(0.5),
        }
        .fire(entity, "dispatch_chip", &mut world);
        world.entity_mut(entity).insert(AnchorPlanted);

        PiercingConfig { charges: 1 }.fire(entity, "dispatch_chip", &mut world);
        PiercingConfig { charges: 2 }.fire(entity, "dispatch_chip", &mut world);

        let effect = ReversibleEffectType::Anchor(AnchorConfig {
            bump_force_multiplier:     OrderedFloat(2.0),
            perfect_window_multiplier: OrderedFloat(1.5),
            plant_delay:               OrderedFloat(0.5),
        });
        reverse_all_by_source_dispatch(&effect, entity, "dispatch_chip", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert!(
            stack.is_empty(),
            "all piercing entries from dispatch_chip should be removed"
        );
        assert!(world.get::<AnchorActive>(entity).is_none());
        assert!(world.get::<AnchorTimer>(entity).is_none());
        assert!(world.get::<AnchorPlanted>(entity).is_none());
    }
}
