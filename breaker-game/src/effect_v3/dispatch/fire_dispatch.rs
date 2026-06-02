//! `fire_dispatch` — match `EffectType` variant to `config.fire()` call.

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{
    effect_v3::{traits::Fireable, types::EffectType},
    shared::rng::{EffectBaseSeed, EffectEventCounter, derive_seed},
};

/// Dispatch an `EffectType` to the appropriate config's `fire()` method.
///
/// Increments `EffectEventCounter` exactly once per call, constructs an
/// ephemeral `ChaCha8Rng` seeded from `derive_seed(EffectBaseSeed, counter)`,
/// and passes it to the config's `fire()`.
pub fn fire_dispatch(effect: &EffectType, entity: Entity, source: &str, world: &mut World) {
    let base = world.get_resource_or_insert_with(EffectBaseSeed::default).0;
    let counter_before = world
        .get_resource_or_insert_with(EffectEventCounter::default)
        .0;
    world.resource_mut::<EffectEventCounter>().0 = counter_before.wrapping_add(1);

    let seed = derive_seed(base, counter_before);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    fire_dispatch_with_rng(effect, entity, source, world, &mut rng);
}

/// Dispatch without incrementing `EffectEventCounter`. Used by effects that
/// recurse into nested effects (e.g., `RandomEffectConfig`) to avoid
/// double-counting a single fire event.
///
/// Contract: callers are responsible for ensuring the counter was already
/// incremented for this fire event (i.e., the outer `fire_dispatch` did it,
/// and this entry point intentionally skips a second increment). Do not call
/// this directly from a non-recursive context — call `fire_dispatch` instead.
pub(in crate::effect_v3) fn fire_dispatch_with_rng(
    effect: &EffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
    rng: &mut ChaCha8Rng,
) {
    match effect {
        EffectType::SpeedBoost(config) => config.fire(entity, source, world, rng),
        EffectType::SizeBoost(config) => config.fire(entity, source, world, rng),
        EffectType::DamageBoost(config) => config.fire(entity, source, world, rng),
        EffectType::BumpForce(config) => config.fire(entity, source, world, rng),
        EffectType::QuickStop(config) => config.fire(entity, source, world, rng),
        EffectType::FlashStep(config) => config.fire(entity, source, world, rng),
        EffectType::Piercing(config) => config.fire(entity, source, world, rng),
        EffectType::Vulnerable(config) => config.fire(entity, source, world, rng),
        EffectType::RampingDamage(config) => config.fire(entity, source, world, rng),
        EffectType::Attraction(config) => config.fire(entity, source, world, rng),
        EffectType::Anchor(config) => config.fire(entity, source, world, rng),
        EffectType::Pulse(config) => config.fire(entity, source, world, rng),
        EffectType::Shield(config) => config.fire(entity, source, world, rng),
        EffectType::SecondWind(config) => config.fire(entity, source, world, rng),
        EffectType::Shockwave(config) => config.fire(entity, source, world, rng),
        EffectType::Explode(config) => config.fire(entity, source, world, rng),
        EffectType::ChainLightning(config) => config.fire(entity, source, world, rng),
        EffectType::PiercingBeam(config) => config.fire(entity, source, world, rng),
        EffectType::SpawnBolts(config) => config.fire(entity, source, world, rng),
        EffectType::SpawnPhantom(config) => config.fire(entity, source, world, rng),
        EffectType::ChainBolt(config) => config.fire(entity, source, world, rng),
        EffectType::MirrorProtocol(config) => config.fire(entity, source, world, rng),
        EffectType::TetherBeam(config) => config.fire(entity, source, world, rng),
        EffectType::GravityWell(config) => config.fire(entity, source, world, rng),
        EffectType::LoseLife(config) => config.fire(entity, source, world, rng),
        EffectType::TimePenalty(config) => config.fire(entity, source, world, rng),
        EffectType::Die(config) => config.fire(entity, source, world, rng),
        EffectType::CircuitBreaker(config) => config.fire(entity, source, world, rng),
        EffectType::EntropyEngine(config) => config.fire(entity, source, world, rng),
        EffectType::RandomEffect(config) => config.fire(entity, source, world, rng),
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    use super::*;
    use crate::{
        effect_v3::{
            effects::{
                AnchorConfig, AttractionConfig, BumpForceConfig, ChainBoltConfig,
                ChainLightningConfig, CircuitBreakerConfig, DamageBoostConfig, DieConfig,
                EntropyConfig, ExplodeConfig, FlashStepConfig, GravityWellConfig, LoseLifeConfig,
                MirrorConfig, PiercingBeamConfig, PiercingConfig, PulseConfig, QuickStopConfig,
                RampingDamageConfig, RandomEffectConfig, SecondWindConfig, ShieldConfig,
                ShockwaveConfig, SizeBoostConfig, SpawnBoltsConfig, SpawnPhantomConfig,
                SpeedBoostConfig, TetherBeamConfig, TetherMode, TimePenaltyConfig,
                VulnerableConfig, explode::messages::ExplodeEmissionRequested,
                piercing_beam::messages::PiercingBeamEmissionRequested,
            },
            stacking::EffectStack,
            types::AttractionType,
        },
        prelude::*,
        shared::{
            PlayfieldConfig,
            rng::{EffectBaseSeed, EffectEventCounter, derive_seed},
        },
        state::run::node::messages::ReduceNodeTimer,
    };

    #[test]
    fn fire_dispatch_speed_boost_creates_stack() {
        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(0));
        world.insert_resource(EffectEventCounter::default());
        let entity = world.spawn_empty().id();
        let effect = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });

        fire_dispatch(&effect, entity, "test_chip", &mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }

    fn passive_effect_types() -> Vec<EffectType> {
        vec![
            EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }),
            EffectType::SizeBoost(SizeBoostConfig {
                multiplier: OrderedFloat(1.2),
            }),
            EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            }),
            EffectType::BumpForce(BumpForceConfig {
                multiplier: OrderedFloat(1.3),
            }),
            EffectType::QuickStop(QuickStopConfig {
                multiplier: OrderedFloat(1.1),
            }),
            EffectType::FlashStep(FlashStepConfig {}),
            EffectType::Piercing(PiercingConfig { charges: 3 }),
            EffectType::Vulnerable(VulnerableConfig {
                multiplier: OrderedFloat(1.5),
            }),
            EffectType::RampingDamage(RampingDamageConfig {
                increment: OrderedFloat(0.5),
            }),
            EffectType::Attraction(AttractionConfig {
                attraction_type: AttractionType::Cell,
                force:           OrderedFloat(100.0),
                max_force:       None,
            }),
            EffectType::Anchor(AnchorConfig {
                bump_force_multiplier:     OrderedFloat(2.0),
                perfect_window_multiplier: OrderedFloat(1.5),
                plant_delay:               OrderedFloat(0.5),
            }),
        ]
    }

    fn active_effect_types() -> Vec<EffectType> {
        vec![
            EffectType::Pulse(PulseConfig {
                base_range:      OrderedFloat(64.0),
                range_per_level: OrderedFloat(16.0),
                stacks:          1,
                speed:           OrderedFloat(200.0),
                interval:        OrderedFloat(1.0),
            }),
            EffectType::Shield(ShieldConfig {
                duration:        OrderedFloat(5.0),
                reflection_cost: OrderedFloat(1.0),
            }),
            EffectType::SecondWind(SecondWindConfig {}),
            EffectType::Shockwave(ShockwaveConfig {
                base_range:      OrderedFloat(64.0),
                range_per_level: OrderedFloat(16.0),
                stacks:          1,
                speed:           OrderedFloat(200.0),
            }),
            EffectType::Explode(ExplodeConfig {
                range:  OrderedFloat(48.0),
                damage: OrderedFloat(10.0),
            }),
            EffectType::ChainLightning(ChainLightningConfig {
                arcs:        3,
                range:       OrderedFloat(80.0),
                damage_mult: OrderedFloat(1.0),
                arc_speed:   OrderedFloat(400.0),
            }),
            EffectType::PiercingBeam(PiercingBeamConfig {
                damage_mult: OrderedFloat(1.0),
                width:       OrderedFloat(8.0),
            }),
            EffectType::SpawnBolts(SpawnBoltsConfig {
                count:    2,
                lifespan: None,
                inherit:  false,
            }),
            EffectType::SpawnPhantom(SpawnPhantomConfig {
                duration:   OrderedFloat(5.0),
                max_active: 3,
            }),
            EffectType::ChainBolt(ChainBoltConfig {
                tether_distance: OrderedFloat(100.0),
            }),
            EffectType::MirrorProtocol(MirrorConfig { inherit: false }),
            EffectType::TetherBeam(TetherBeamConfig {
                damage_mult: OrderedFloat(1.0),
                mode:        TetherMode::SpawnBolt,
                width:       OrderedFloat(4.0),
            }),
            EffectType::GravityWell(GravityWellConfig {
                strength: OrderedFloat(50.0),
                duration: OrderedFloat(3.0),
                radius:   OrderedFloat(64.0),
                max:      2,
            }),
            EffectType::LoseLife(LoseLifeConfig {}),
            EffectType::TimePenalty(TimePenaltyConfig {
                seconds: OrderedFloat(5.0),
            }),
            EffectType::Die(DieConfig {}),
            EffectType::CircuitBreaker(CircuitBreakerConfig {
                bumps_required:  5,
                spawn_count:     2,
                inherit:         false,
                shockwave_range: OrderedFloat(64.0),
                shockwave_speed: OrderedFloat(200.0),
            }),
            EffectType::EntropyEngine(EntropyConfig {
                max_effects: 3,
                pool:        vec![],
            }),
            EffectType::RandomEffect(RandomEffectConfig { pool: vec![] }),
        ]
    }

    /// Returns one instance of every `EffectType` variant with sensible test values.
    fn all_effect_types() -> Vec<EffectType> {
        let mut types = passive_effect_types();
        types.extend(active_effect_types());
        types
    }

    // B22a — migrated smoke test: inserts EffectBaseSeed + EffectEventCounter directly.
    #[test]
    fn fire_dispatch_does_not_panic_for_any_effect_type_variant() {
        let mut app = App::new();
        app.world_mut().insert_resource(EffectBaseSeed(0));
        app.world_mut()
            .insert_resource(EffectEventCounter::default());
        app.world_mut().insert_resource(PlayfieldConfig::default());
        app.world_mut()
            .init_resource::<rantzsoft_physics2d::resources::CollisionQuadtree>();
        app.world_mut().init_resource::<Assets<Mesh>>();
        app.world_mut().init_resource::<Assets<ColorMaterial>>();
        app.add_message::<ExplodeEmissionRequested>();
        app.add_message::<PiercingBeamEmissionRequested>();
        app.add_message::<ReduceNodeTimer>();

        let types = all_effect_types();
        assert_eq!(
            types.len(),
            30,
            "update all_effect_types when EffectType gains variants"
        );
        for effect in &types {
            let entity = app
                .world_mut()
                .spawn((
                    Position2D(Vec2::new(100.0, 200.0)),
                    Velocity2D(Vec2::new(0.0, 300.0)),
                ))
                .id();

            fire_dispatch(effect, entity, "smoke_test", app.world_mut());
        }

        let counter = app.world().resource::<EffectEventCounter>().0;
        assert_eq!(
            counter, 30,
            "EffectEventCounter must increment once per fire_dispatch call; \
             got {counter} after 30 calls"
        );
    }

    // ── Group A — Bridge dispatch counter + ephemeral RNG construction ────────

    #[test]
    fn fire_dispatch_increments_effect_event_counter_exactly_once() {
        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(0));
        world.insert_resource(EffectEventCounter(0));
        let entity = world.spawn_empty().id();

        let effect = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });
        fire_dispatch(&effect, entity, "test", &mut world);

        assert_eq!(
            world.resource::<EffectEventCounter>().0,
            1,
            "counter must be exactly 1 after one fire_dispatch call"
        );
    }

    #[test]
    fn fire_dispatch_counter_wraps_at_u64_max_without_panic() {
        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(0));
        world.insert_resource(EffectEventCounter(u64::MAX - 1));
        let entity = world.spawn_empty().id();

        let effect = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });

        // First call: counter goes from MAX-1 to MAX.
        fire_dispatch(&effect, entity, "test", &mut world);
        assert_eq!(
            world.resource::<EffectEventCounter>().0,
            u64::MAX,
            "counter must be u64::MAX after first call starting at MAX-1"
        );

        // Second call: wraps from MAX to 0.
        fire_dispatch(&effect, entity, "test", &mut world);
        assert_eq!(
            world.resource::<EffectEventCounter>().0,
            0,
            "counter must wrap to 0 from u64::MAX without panic"
        );
    }

    #[test]
    fn fire_dispatch_noop_fire_body_still_increments_counter() {
        // SpawnBoltsConfig with count:0 early-returns from fire() body,
        // but the bridge must still increment the counter.
        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(0));
        world.insert_resource(EffectEventCounter(0));
        let entity = world.spawn_empty().id();

        let effect = EffectType::SpawnBolts(SpawnBoltsConfig {
            count:    0,
            lifespan: None,
            inherit:  false,
        });
        fire_dispatch(&effect, entity, "test", &mut world);

        assert_eq!(
            world.resource::<EffectEventCounter>().0,
            1,
            "counter must increment even when fire() body is a no-op"
        );
    }

    #[test]
    fn fire_dispatch_increments_counter_n_times_for_n_calls() {
        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(0));
        world.insert_resource(EffectEventCounter(0));
        let entity = world.spawn_empty().id();

        let effect = EffectType::FlashStep(FlashStepConfig {});

        for _ in 0..5 {
            fire_dispatch(&effect, entity, "test", &mut world);
        }

        assert_eq!(
            world.resource::<EffectEventCounter>().0,
            5,
            "counter must be 5 after 5 fire_dispatch calls"
        );
    }

    #[test]
    fn fire_dispatch_rng_derived_from_pre_increment_counter() {
        // The RNG inside fire must be seeded from derive_seed(base, counter_before_increment).
        // We verify by using a RandomEffect pool whose selection is predictable
        // from a known counter value.
        // Seed with counter=5: derive_seed(0, 5) → some deterministic seed.
        // Verify by reconstructing the expected draw externally.
        let mut world_a = World::new();
        world_a.insert_resource(EffectBaseSeed(0));
        world_a.insert_resource(EffectEventCounter(5));
        let entity_a = world_a.spawn_empty().id();

        let pool = vec![
            (
                OrderedFloat(0.5_f32),
                Box::new(EffectType::FlashStep(FlashStepConfig {})),
            ),
            (
                OrderedFloat(0.5_f32),
                Box::new(EffectType::Die(DieConfig {})),
            ),
        ];
        let effect = EffectType::RandomEffect(RandomEffectConfig { pool });

        fire_dispatch(&effect, entity_a, "probe", &mut world_a);

        // Reconstruct: what would derive_seed(0, 5) → rng select?
        let expected_seed = derive_seed(0, 5);
        let mut expected_rng = ChaCha8Rng::seed_from_u64(expected_seed);
        let draw: f32 = expected_rng.random::<f32>();
        // pool is [(0.5, FlashStep), (0.5, Die)]
        // cumulative: FlashStep if draw < 0.5, Die otherwise
        let expected_flash = draw < 0.5;

        let has_flash = world_a.get::<FlashStepActive>(entity_a).is_some();
        let has_dead = world_a.get::<Dead>(entity_a).is_some();

        if expected_flash {
            assert!(
                has_flash,
                "expected FlashStep from pre-increment seed derive_seed(0,5), draw={draw}"
            );
        } else {
            assert!(
                has_dead,
                "expected Die from pre-increment seed derive_seed(0,5), draw={draw}"
            );
        }

        // Sanity: counter must be 6 now.
        assert_eq!(world_a.resource::<EffectEventCounter>().0, 6);
    }

    #[test]
    fn fire_dispatch_same_base_and_counter_produce_identical_state() {
        // Two independent apps with the same seed+counter produce identical post-state.
        let pool = || {
            vec![
                (
                    OrderedFloat(0.5_f32),
                    Box::new(EffectType::FlashStep(FlashStepConfig {})),
                ),
                (
                    OrderedFloat(0.5_f32),
                    Box::new(EffectType::Die(DieConfig {})),
                ),
            ]
        };
        let effect_a = EffectType::RandomEffect(RandomEffectConfig { pool: pool() });
        let effect_b = EffectType::RandomEffect(RandomEffectConfig { pool: pool() });

        let mut world_a = World::new();
        world_a.insert_resource(EffectBaseSeed(42));
        world_a.insert_resource(EffectEventCounter(0));
        let entity_a = world_a.spawn_empty().id();
        fire_dispatch(&effect_a, entity_a, "test", &mut world_a);

        let mut world_b = World::new();
        world_b.insert_resource(EffectBaseSeed(42));
        world_b.insert_resource(EffectEventCounter(0));
        let entity_b = world_b.spawn_empty().id();
        fire_dispatch(&effect_b, entity_b, "test", &mut world_b);

        let flash_a = world_a.get::<FlashStepActive>(entity_a).is_some();
        let flash_b = world_b.get::<FlashStepActive>(entity_b).is_some();
        assert_eq!(
            flash_a, flash_b,
            "identical seed+counter must produce identical FlashStep outcome"
        );

        let dead_a = world_a.get::<Dead>(entity_a).is_some();
        let dead_b = world_b.get::<Dead>(entity_b).is_some();
        assert_eq!(
            dead_a, dead_b,
            "identical seed+counter must produce identical Die outcome"
        );
    }

    #[test]
    fn fire_dispatch_different_base_seeds_can_diverge() {
        // At least one of several counter values must produce a different outcome
        // when base seed changes by 1.
        let pool = || {
            vec![
                (
                    OrderedFloat(0.5_f32),
                    Box::new(EffectType::FlashStep(FlashStepConfig {})),
                ),
                (
                    OrderedFloat(0.5_f32),
                    Box::new(EffectType::Die(DieConfig {})),
                ),
            ]
        };

        let mut found_divergence = false;
        for counter in 0_u64..6 {
            let effect_a = EffectType::RandomEffect(RandomEffectConfig { pool: pool() });
            let effect_b = EffectType::RandomEffect(RandomEffectConfig { pool: pool() });

            let mut world_a = World::new();
            world_a.insert_resource(EffectBaseSeed(42));
            world_a.insert_resource(EffectEventCounter(counter));
            let entity_a = world_a.spawn_empty().id();
            fire_dispatch(&effect_a, entity_a, "test", &mut world_a);

            let mut world_b = World::new();
            world_b.insert_resource(EffectBaseSeed(43));
            world_b.insert_resource(EffectEventCounter(counter));
            let entity_b = world_b.spawn_empty().id();
            fire_dispatch(&effect_b, entity_b, "test", &mut world_b);

            let flash_a = world_a.get::<FlashStepActive>(entity_a).is_some();
            let flash_b = world_b.get::<FlashStepActive>(entity_b).is_some();
            if flash_a != flash_b {
                found_divergence = true;
                break;
            }
        }
        assert!(
            found_divergence,
            "different base seeds (42 vs 43) must produce at least one diverging outcome \
             across 6 counter values"
        );
    }

    #[test]
    fn fire_dispatch_does_not_mutate_effect_base_seed() {
        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(0xABCD_1234));
        world.insert_resource(EffectEventCounter(0));
        let entity = world.spawn_empty().id();

        let effect = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });
        fire_dispatch(&effect, entity, "test", &mut world);

        assert_eq!(
            world.resource::<EffectBaseSeed>().0,
            0xABCD_1234,
            "EffectBaseSeed must not be mutated by fire_dispatch"
        );
        assert_eq!(
            world.resource::<EffectEventCounter>().0,
            1,
            "only EffectEventCounter must change"
        );
    }

    // B3 — ephemeral seed is derived from spec-pinned values EffectBaseSeed(0xABCD_1234) + EffectEventCounter(7)
    #[test]
    fn fire_dispatch_ephemeral_seed_derived_from_spec_values() {
        // B3: seed passed to fire() must equal derive_seed(base, counter_before_increment).
        // Pin the exact spec values so the formula is locked by this test.
        let base = 0xABCD_1234_u64;
        let counter = 7_u64;
        let expected_seed = derive_seed(base, counter);

        // Use a RandomEffect pool whose selection outcome depends on the RNG seed.
        // By reconstructing the expected draw from derive_seed(base, counter) we
        // confirm the bridge uses the correct formula.
        let pool = vec![
            (
                OrderedFloat(0.5_f32),
                Box::new(EffectType::FlashStep(FlashStepConfig {})),
            ),
            (
                OrderedFloat(0.5_f32),
                Box::new(EffectType::Die(DieConfig {})),
            ),
        ];
        let effect = EffectType::RandomEffect(RandomEffectConfig { pool });

        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(base));
        world.insert_resource(EffectEventCounter(counter));
        let entity = world.spawn_empty().id();

        fire_dispatch(&effect, entity, "b3_spec_check", &mut world);

        // Reconstruct expected draw using the spec-mandated formula.
        let mut expected_rng = ChaCha8Rng::seed_from_u64(expected_seed);
        let draw: f32 = expected_rng.random::<f32>();
        let expected_flash = draw < 0.5;

        let has_flash = world.get::<FlashStepActive>(entity).is_some();
        let has_dead = world.get::<Dead>(entity).is_some();

        if expected_flash {
            assert!(
                has_flash,
                "B3 spec: expected FlashStep with base=0xABCD_1234 counter=7 \
                 (seed={expected_seed}, draw={draw})"
            );
        } else {
            assert!(
                has_dead,
                "B3 spec: expected Die with base=0xABCD_1234 counter=7 \
                 (seed={expected_seed}, draw={draw})"
            );
        }

        assert_eq!(
            world.resource::<EffectEventCounter>().0,
            counter + 1,
            "B3: counter must be 8 after one fire_dispatch with counter=7"
        );
    }

    #[test]
    fn fire_dispatch_inserts_default_resources_when_absent() {
        // When EffectBaseSeed and EffectEventCounter are absent, fire_dispatch
        // must not panic and must insert them with defaults.
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let effect = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });
        fire_dispatch(&effect, entity, "test", &mut world);

        let counter = world.resource::<EffectEventCounter>().0;
        let base = world.resource::<EffectBaseSeed>().0;
        assert_eq!(
            counter, 1,
            "EffectEventCounter must be 1 after first call with absent resources"
        );
        assert_eq!(base, 0, "EffectBaseSeed must default to 0 when absent");
    }

    // ── Group B (partial — B9 and B12) ────────────────────────────────────────

    #[test]
    fn fire_dispatch_speed_boost_signature_takes_ephemeral_rng() {
        // B9: the fire() call compiles and executes with a &mut ChaCha8Rng
        // parameter, producing the same post-state as before.
        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(0));
        world.insert_resource(EffectEventCounter(0));
        let entity = world.spawn_empty().id();

        let effect = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });
        fire_dispatch(&effect, entity, "test", &mut world);

        let stack_len = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .unwrap()
            .len();
        assert_eq!(stack_len, 1);

        // Two identical-seed calls produce identical stacks.
        let mut world_b = World::new();
        world_b.insert_resource(EffectBaseSeed(0));
        world_b.insert_resource(EffectEventCounter(0));
        let entity_b = world_b.spawn_empty().id();
        let effect_b = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });
        fire_dispatch(&effect_b, entity_b, "test", &mut world_b);
        let stack_b_len = world_b
            .get::<EffectStack<SpeedBoostConfig>>(entity_b)
            .unwrap()
            .len();
        assert_eq!(
            stack_len, stack_b_len,
            "identical-seed calls must produce identical stack lengths"
        );
    }

    #[test]
    fn fire_dispatch_random_effect_increments_counter_exactly_once() {
        // B12: RandomEffect internally calls fire_dispatch_with_rng (not fire_dispatch),
        // so the outer counter increments exactly once.
        let mut world = World::new();
        world.insert_resource(EffectBaseSeed(0));
        world.insert_resource(EffectEventCounter(0));
        let entity = world.spawn_empty().id();

        let pool = vec![
            (
                OrderedFloat(0.5_f32),
                Box::new(EffectType::FlashStep(FlashStepConfig {})),
            ),
            (
                OrderedFloat(0.5_f32),
                Box::new(EffectType::Die(DieConfig {})),
            ),
        ];
        let effect = EffectType::RandomEffect(RandomEffectConfig { pool });
        fire_dispatch(&effect, entity, "test", &mut world);

        assert_eq!(
            world.resource::<EffectEventCounter>().0,
            1,
            "EffectEventCounter must be exactly 1 after one RandomEffect fire_dispatch \
             (inner effect must NOT re-increment the counter)"
        );
    }

    // Behavior 6 (FAILS at RED): fire_dispatch.rs must not reference the old monolithic
    // RNG type anywhere — neither the dead insert_resource harness line (around line 530)
    // nor the stale migration comment (around line 244), nor the import.
    // Writer-code removes both cleanup targets at GREEN.
    //
    // Self-referential guard: built at compile time to avoid false-positive here.
    const STALE_DISPATCH_RNG: &str = concat!("Game", "Rng");

    #[test]
    fn fire_dispatch_rs_has_no_game_rng_reference_after_wave3_sweep() {
        let source = include_str!("fire_dispatch.rs");
        assert!(
            !source.contains(STALE_DISPATCH_RNG),
            "effect_v3/dispatch/fire_dispatch.rs must not reference the old monolithic RNG type; \
             remove the dead insert_resource harness line, the stale migration comment, \
             and the import"
        );
    }
}
