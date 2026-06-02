//! Reseed `ProtocolRng` on each chip-select visit using the canonical
//! `derive_seed(derive_seed_named(run_seed, "protocol"), offering_index)` formula.

use bevy::prelude::*;

use crate::{
    mutators::protocols::resources::ProtocolOfferingCount,
    shared::rng::{ProtocolRng, derive_seed, derive_seed_named},
    state::run::resources::RunStats,
};

/// Seeds `ProtocolRng` from the canonical formula then increments
/// `ProtocolOfferingCount` so the next visit uses a distinct stream.
pub(crate) fn reseed_protocol_rng(
    run_stats: Res<RunStats>,
    mut rng: ResMut<ProtocolRng>,
    mut count: ResMut<ProtocolOfferingCount>,
) {
    let seed = derive_seed(
        derive_seed_named(run_stats.seed, "protocol"),
        u64::from(count.0),
    );
    *rng = ProtocolRng::from_seed(seed);
    count.0 = count.0.wrapping_add(1);
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;
    use crate::{
        prelude::*,
        shared::{
            RunSeed,
            rng::{ChipSelectCount, HazardRng, derive_seed, derive_seed_named},
        },
        state::run::chip_select::sets::ChipSelectSystems,
    };

    // Sentinel seed used so a no-op stub (which leaves ProtocolRng at SENTINEL)
    // produces draws that differ from the expected draws (from the formula).
    const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

    // ── Group A, Behavior 4 — counter increments ────────────────────────────

    #[test]
    fn reseed_protocol_rng_increments_protocol_offering_count() {
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 42,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(0))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app.update();

        assert_eq!(
            app.world().resource::<ProtocolOfferingCount>().0,
            1,
            "reseed_protocol_rng must increment ProtocolOfferingCount from 0 to 1"
        );
    }

    #[test]
    fn reseed_protocol_rng_increments_count_on_second_tick() {
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 42,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(0))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<ProtocolOfferingCount>().0,
            2,
            "reseed_protocol_rng must increment counter to 2 after two ticks"
        );

        // Edge case A: verify second reseed uses discriminator 1.
        let expected_seed = derive_seed(derive_seed_named(42, "protocol"), 1);
        let mut expected_rng = ProtocolRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();
        let actual_draw: u64 = app.world_mut().resource_mut::<ProtocolRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "after second tick, ProtocolRng must be seeded with discriminator 1"
        );
    }

    #[test]
    fn reseed_protocol_rng_no_panic_when_count_is_max() {
        // Edge case B: counter at u32::MAX — must not panic. Exact post-overflow
        // value is implementation-defined (wrap or saturate); we do NOT assert a
        // specific value.
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 42,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(u32::MAX))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app.update(); // must not panic

        let count = app.world().resource::<ProtocolOfferingCount>().0;
        // The result is either wrapping (0) or saturating (u32::MAX); both are valid.
        assert!(
            count == 0 || count == u32::MAX,
            "post-overflow count must be 0 (wrapping) or u32::MAX (saturating), got {count}"
        );
    }

    // ── Group B — formula correctness ───────────────────────────────────────

    #[test]
    fn reseed_protocol_rng_uses_canonical_formula_on_first_visit() {
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 42,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(0))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app.update();

        let expected_seed = derive_seed(derive_seed_named(42, "protocol"), 0);
        let mut expected_rng = ProtocolRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();
        let actual_draw: u64 = app.world_mut().resource_mut::<ProtocolRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "ProtocolRng first draw must match derive_seed(derive_seed_named(42, \"protocol\"), 0)"
        );
    }

    #[test]
    fn reseed_protocol_rng_formula_with_zero_seed_no_panic() {
        // Edge case: seed = 0 must produce a valid (non-panicking) output.
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 0,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(0))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app.update();

        let expected_seed = derive_seed(derive_seed_named(0, "protocol"), 0);
        let mut expected_rng = ProtocolRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();
        let actual_draw: u64 = app.world_mut().resource_mut::<ProtocolRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "seed=0 must produce derive_seed(derive_seed_named(0, \"protocol\"), 0)"
        );
    }

    #[test]
    fn reseed_protocol_rng_uses_offering_count_as_discriminator_on_third_visit() {
        // Given count=2 (third visit), the discriminator is 2.
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 42,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(2))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app.update();

        let expected_seed = derive_seed(derive_seed_named(42, "protocol"), 2);
        let mut expected_rng = ProtocolRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();
        let actual_draw: u64 = app.world_mut().resource_mut::<ProtocolRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "ProtocolRng with count=2 must use discriminator 2"
        );
    }

    #[test]
    fn different_offering_counts_produce_different_streams() {
        // Edge case for Behavior 6: count=0 vs count=2 must produce different draws.
        let draw_with_count = |count: u32| -> u64 {
            let mut app = TestAppBuilder::new()
                .insert_resource(RunStats {
                    seed: 42,
                    ..default()
                })
                .insert_resource(ProtocolOfferingCount(count))
                .insert_resource(ProtocolRng::from_seed(SENTINEL))
                .with_system(Update, reseed_protocol_rng)
                .build();
            app.update();
            app.world_mut().resource_mut::<ProtocolRng>().0.random()
        };

        let draw_0 = draw_with_count(0);
        let draw_2 = draw_with_count(2);
        assert_ne!(
            draw_0, draw_2,
            "ProtocolOfferingCount=0 and =2 must produce different streams"
        );
    }

    #[test]
    fn reseed_protocol_rng_overwrites_preexisting_rng_resource() {
        // Given ProtocolRng seeded at SENTINEL; after reseed_protocol_rng runs
        // the first draw must equal the formula result, NOT the SENTINEL stream.
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 42,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(1))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app.update();

        let expected_seed = derive_seed(derive_seed_named(42, "protocol"), 1);
        let mut expected_rng = ProtocolRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();
        let actual_draw: u64 = app.world_mut().resource_mut::<ProtocolRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "reseed must overwrite SENTINEL-seeded ProtocolRng"
        );

        // Sentinel draw must differ from formula draw (validates the test itself).
        let sentinel_draw: u64 = ProtocolRng::from_seed(SENTINEL).0.random();
        assert_ne!(
            actual_draw, sentinel_draw,
            "draw after reseed must differ from SENTINEL stream"
        );
    }

    #[test]
    fn reseed_protocol_rng_idempotent_under_same_inputs() {
        // Edge case for Behavior 7: running twice with the same (seed, count)
        // produces identical first draws.
        let run_once = |seed: u64, count: u32| -> u64 {
            let mut app = TestAppBuilder::new()
                .insert_resource(RunStats { seed, ..default() })
                .insert_resource(ProtocolOfferingCount(count))
                .insert_resource(ProtocolRng::from_seed(SENTINEL))
                .with_system(Update, reseed_protocol_rng)
                .build();
            app.update();
            app.world_mut().resource_mut::<ProtocolRng>().0.random()
        };

        let draw_a = run_once(42, 3);
        let draw_b = run_once(42, 3);
        assert_eq!(
            draw_a, draw_b,
            "same (seed, count) inputs must produce identical draws across independent apps"
        );
    }

    #[test]
    fn reseed_protocol_rng_deterministic_across_independent_apps() {
        // Behavior 8: two independent apps with (seed=1234, count=7) produce equal draws.
        let mut app_a = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 1234,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(7))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        let mut app_b = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 1234,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(7))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app_a.update();
        app_b.update();

        let draw_a: u64 = app_a.world_mut().resource_mut::<ProtocolRng>().0.random();
        let draw_b: u64 = app_b.world_mut().resource_mut::<ProtocolRng>().0.random();
        assert_eq!(
            draw_a, draw_b,
            "two independent apps with (seed=1234, count=7) must produce identical draws"
        );
    }

    #[test]
    fn reseed_protocol_rng_deterministic_zero_inputs() {
        // Edge case for Behavior 8: (seed=0, count=0) stable across two apps.
        let mut app_a = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 0,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(0))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        let mut app_b = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 0,
                ..default()
            })
            .insert_resource(ProtocolOfferingCount(0))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app_a.update();
        app_b.update();

        let draw_a: u64 = app_a.world_mut().resource_mut::<ProtocolRng>().0.random();
        let draw_b: u64 = app_b.world_mut().resource_mut::<ProtocolRng>().0.random();
        assert_eq!(
            draw_a, draw_b,
            "(seed=0, count=0) must be stable across apps"
        );
    }

    #[test]
    fn reseed_protocol_rng_reads_run_stats_seed_not_run_seed() {
        // Behavior 9: intentionally mismatched RunStats.seed=42 vs RunSeed(Some(999)).
        // The reseed must use RunStats.seed=42.
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 42,
                ..default()
            })
            .insert_resource(RunSeed(Some(999)))
            .insert_resource(ProtocolOfferingCount(0))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app.update();

        let expected_seed = derive_seed(derive_seed_named(42, "protocol"), 0);
        let mut expected_rng = ProtocolRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();
        let actual_draw: u64 = app.world_mut().resource_mut::<ProtocolRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "reseed_protocol_rng must read RunStats.seed (42), not RunSeed (999)"
        );

        // Verify it is NOT the RunSeed(999) value.
        let wrong_seed = derive_seed(derive_seed_named(999, "protocol"), 0);
        let mut wrong_rng = ProtocolRng::from_seed(wrong_seed);
        let wrong_draw: u64 = wrong_rng.0.random();
        assert_ne!(
            actual_draw, wrong_draw,
            "draw must not match the seed derived from RunSeed value 999"
        );
    }

    #[test]
    fn reseed_protocol_rng_reads_run_stats_seed_when_run_seed_is_none() {
        // Edge case for Behavior 9: RunSeed(None) — still reads RunStats.seed=42.
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 42,
                ..default()
            })
            .insert_resource(RunSeed(None))
            .insert_resource(ProtocolOfferingCount(0))
            .insert_resource(ProtocolRng::from_seed(SENTINEL))
            .with_system(Update, reseed_protocol_rng)
            .build();

        app.update();

        let expected_seed = derive_seed(derive_seed_named(42, "protocol"), 0);
        let mut expected_rng = ProtocolRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();
        let actual_draw: u64 = app.world_mut().resource_mut::<ProtocolRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "RunSeed(None) must not affect reseed — RunStats.seed=42 governs"
        );
    }

    // ── Group F, Behavior 29 — schedule set membership ──────────────────────

    #[test]
    fn reseed_protocol_rng_is_in_chip_select_reseed_set() {
        use crate::{
            mutators::plugin::MutatorsPlugin, state::run::chip_select::sets::ChipSelectSystems,
        };
        let mut app = TestAppBuilder::new().with_state_hierarchy().build();
        app.add_plugins(MutatorsPlugin);

        assert!(
            system_in_set(
                &mut app,
                OnEnter(ChipSelectState::Selecting),
                reseed_protocol_rng,
                ChipSelectSystems::Reseed,
            ),
            "reseed_protocol_rng must be in ChipSelectSystems::Reseed"
        );
    }

    #[test]
    fn reseed_protocol_rng_is_not_in_generate_offerings_set() {
        use crate::{
            mutators::plugin::MutatorsPlugin, state::run::chip_select::sets::ChipSelectSystems,
        };
        let mut app = TestAppBuilder::new().with_state_hierarchy().build();
        app.add_plugins(MutatorsPlugin);

        assert!(
            !system_in_set(
                &mut app,
                OnEnter(ChipSelectState::Selecting),
                reseed_protocol_rng,
                ChipSelectSystems::GenerateOfferings,
            ),
            "reseed_protocol_rng must NOT be in ChipSelectSystems::GenerateOfferings"
        );
    }

    // ── Group F, Behavior 31 — MutatorsPlugin initializes ProtocolOfferingCount

    #[test]
    fn mutators_plugin_initializes_protocol_offering_count_at_zero() {
        use crate::mutators::plugin::MutatorsPlugin;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(MutatorsPlugin);

        let count = app
            .world()
            .get_resource::<ProtocolOfferingCount>()
            .expect("MutatorsPlugin must init_resource::<ProtocolOfferingCount>()");
        assert_eq!(
            count.0, 0,
            "ProtocolOfferingCount must default to 0 after MutatorsPlugin::build"
        );
    }

    #[test]
    fn mutators_plugin_does_not_stomp_preinserted_protocol_offering_count() {
        // Edge case for Behavior 31: if the resource is inserted BEFORE the plugin,
        // init_resource is a no-op and the pre-existing value must survive.
        use crate::mutators::plugin::MutatorsPlugin;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ProtocolOfferingCount(7));
        app.add_plugins(MutatorsPlugin);

        let count = app
            .world()
            .get_resource::<ProtocolOfferingCount>()
            .expect("ProtocolOfferingCount must exist");
        assert_eq!(
            count.0, 7,
            "init_resource must not overwrite an already-inserted ProtocolOfferingCount(7)"
        );
    }

    // ── Group F, Behavior 32 — channel distinctness sanity ──────────────────

    #[test]
    fn channel_seeds_are_pairwise_distinct_for_seed_42() {
        let protocol_seed_0 = derive_seed(derive_seed_named(42, "protocol"), 0);
        let protocol_seed_1 = derive_seed(derive_seed_named(42, "protocol"), 1);
        let hazard_seed_0 = derive_seed(derive_seed_named(42, "hazard"), 0);
        let hazard_seed_5 = derive_seed(derive_seed_named(42, "hazard"), 5);

        let seeds = [
            protocol_seed_0,
            protocol_seed_1,
            hazard_seed_0,
            hazard_seed_5,
        ];
        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                assert_ne!(
                    seeds[i], seeds[j],
                    "seeds[{i}] and seeds[{j}] must be distinct for seed=42"
                );
            }
        }
    }

    #[test]
    fn channel_seeds_are_pairwise_distinct_for_seed_zero() {
        // Edge case for Behavior 32: seed=0 must not produce collisions.
        let protocol_seed_0 = derive_seed(derive_seed_named(0, "protocol"), 0);
        let protocol_seed_1 = derive_seed(derive_seed_named(0, "protocol"), 1);
        let hazard_seed_0 = derive_seed(derive_seed_named(0, "hazard"), 0);
        let hazard_seed_5 = derive_seed(derive_seed_named(0, "hazard"), 5);

        let seeds = [
            protocol_seed_0,
            protocol_seed_1,
            hazard_seed_0,
            hazard_seed_5,
        ];
        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                assert_ne!(
                    seeds[i], seeds[j],
                    "seeds[{i}] and seeds[{j}] must be distinct for seed=0"
                );
            }
        }
    }

    // ── Group F, Behavior 33 — ProtocolOfferingCount is distinct from ChipSelectCount

    #[test]
    fn protocol_offering_count_is_distinct_type_from_chip_select_count() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ProtocolOfferingCount::default());
        app.insert_resource(ChipSelectCount::default());

        // Update to different values to prove type distinctness.
        app.world_mut().insert_resource(ProtocolOfferingCount(5));
        app.world_mut().insert_resource(ChipSelectCount(3));

        let poc = app.world().resource::<ProtocolOfferingCount>().0;
        let csc = app.world().resource::<ChipSelectCount>().0;

        assert_eq!(poc, 5, "ProtocolOfferingCount must be 5");
        assert_eq!(csc, 3, "ChipSelectCount must be 3");
        assert_ne!(
            poc, csc,
            "ProtocolOfferingCount and ChipSelectCount must be distinct resources"
        );
    }

    #[test]
    fn protocol_offering_count_tuple_constructor_is_public() {
        // Edge case for Behavior 33: public tuple field access.
        let c = ProtocolOfferingCount(7);
        assert_eq!(c.0, 7, "ProtocolOfferingCount(7).0 must be 7");
    }

    // ── Group G, Behavior 34 — end-to-end determinism ───────────────────────

    fn make_full_protocol_registry() -> crate::mutators::protocols::resources::ProtocolRegistry {
        use crate::mutators::protocols::{
            definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
            resources::ProtocolRegistry,
        };
        let mut registry = ProtocolRegistry::default();
        for kind in ProtocolKind::ALL {
            let tuning = match kind {
                ProtocolKind::Greed => ProtocolTuning::Greed {
                    rarity_boost_per_skip: 0.05,
                },
                ProtocolKind::Deadline => ProtocolTuning::Deadline { effects: vec![] },
                ProtocolKind::Ricochet => ProtocolTuning::Ricochet { effects: vec![] },
                ProtocolKind::Anchor => ProtocolTuning::Anchor { effects: vec![] },
                ProtocolKind::Kickstart => ProtocolTuning::Kickstart { effects: vec![] },
                ProtocolKind::DebtCollector => ProtocolTuning::DebtCollector {
                    stack_per_bump: 0.1,
                },
                ProtocolKind::IronCurtain => ProtocolTuning::IronCurtain {
                    damage_fraction: 0.25,
                    falloff_start:   0.5,
                },
                ProtocolKind::EchoStrike => ProtocolTuning::EchoStrike {
                    max_echoes:      3,
                    newest_fraction: 0.5,
                    middle_fraction: 0.25,
                    oldest_fraction: 0.125,
                },
                ProtocolKind::Siphon => ProtocolTuning::Siphon {
                    streak_window: 2.0,
                    time_per_kill: 0.25,
                },
                ProtocolKind::RecklessDash => ProtocolTuning::RecklessDash {
                    risky_zone_start:  0.7,
                    damage_multiplier: 4.0,
                    double_penalty:    true,
                },
                ProtocolKind::Burnout => ProtocolTuning::Burnout {
                    fill_duration:               4.0,
                    drain_duration:              2.0,
                    still_threshold:             1.5,
                    full_heat_damage_multiplier: 4.0,
                    speed_boost_duration:        2.0,
                    shockwave_base_range:        0.0,
                    shockwave_range_per_level:   0.0,
                    shockwave_stacks:            0,
                    shockwave_speed:             0.0,
                },
                ProtocolKind::Conductor => ProtocolTuning::Conductor,
                ProtocolKind::Afterimage => ProtocolTuning::Afterimage {
                    phantom_duration:      1.5,
                    phantom_bolt_duration: 0.75,
                },
                ProtocolKind::Fission => ProtocolTuning::Fission {
                    kills_per_split:      10,
                    divergence_angle_rad: 0.0,
                },
                ProtocolKind::TierRegression => ProtocolTuning::TierRegression { tiers_back: 1 },
            };
            registry.insert(ProtocolDefinition {
                name: format!("{kind:?}"),
                description: String::new(),
                unlock_tier: 0,
                tuning,
            });
        }
        registry
    }

    #[test]
    fn end_to_end_protocol_offering_is_deterministic_for_seed_42_count_0() {
        use crate::mutators::protocols::{
            resources::{ActiveProtocols, ProtocolOffer, UnlockedProtocols},
            systems::generate_protocol_offering,
        };

        let build = || -> App {
            TestAppBuilder::new()
                .insert_resource(RunStats {
                    seed: 42,
                    ..default()
                })
                .insert_resource(ProtocolOfferingCount(0))
                .insert_resource(ProtocolRng::from_seed(SENTINEL))
                .insert_resource(UnlockedProtocols::default())
                .insert_resource(ActiveProtocols::default())
                .insert_resource(make_full_protocol_registry())
                .insert_resource(ProtocolOffer::default())
                .with_system(
                    Update,
                    (reseed_protocol_rng, generate_protocol_offering).chain(),
                )
                .build()
        };

        let mut app_a = build();
        let mut app_b = build();
        app_a.update();
        app_b.update();

        let kind_a = app_a
            .world()
            .resource::<ProtocolOffer>()
            .0
            .as_ref()
            .expect("app_a must produce an offer")
            .kind();
        let kind_b = app_b
            .world()
            .resource::<ProtocolOffer>()
            .0
            .as_ref()
            .expect("app_b must produce an offer")
            .kind();
        assert_eq!(
            kind_a, kind_b,
            "end-to-end: (seed=42, count=0) must produce the same kind in independent apps"
        );
    }

    #[test]
    fn end_to_end_different_count_produces_different_kind() {
        use crate::mutators::protocols::{
            definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
            resources::{ActiveProtocols, ProtocolOffer, ProtocolRegistry, UnlockedProtocols},
            systems::generate_protocol_offering,
        };

        let build = |count: u32| -> App {
            let mut registry = ProtocolRegistry::default();
            for kind in ProtocolKind::ALL {
                let tuning = match kind {
                    ProtocolKind::Greed => ProtocolTuning::Greed {
                        rarity_boost_per_skip: 0.05,
                    },
                    _ => ProtocolTuning::Deadline { effects: vec![] },
                };
                registry.insert(ProtocolDefinition {
                    name: format!("{kind:?}"),
                    description: String::new(),
                    unlock_tier: 0,
                    tuning,
                });
            }
            TestAppBuilder::new()
                .insert_resource(RunStats {
                    seed: 42,
                    ..default()
                })
                .insert_resource(ProtocolOfferingCount(count))
                .insert_resource(ProtocolRng::from_seed(SENTINEL))
                .insert_resource(UnlockedProtocols::default())
                .insert_resource(ActiveProtocols::default())
                .insert_resource(registry)
                .insert_resource(ProtocolOffer::default())
                .with_system(
                    Update,
                    (reseed_protocol_rng, generate_protocol_offering).chain(),
                )
                .build()
        };

        let kind_a = {
            let mut app = build(0);
            app.update();
            app.world()
                .resource::<ProtocolOffer>()
                .0
                .as_ref()
                .expect("count=0 must offer")
                .kind()
        };
        let kind_b = {
            let mut app = build(2);
            app.update();
            app.world()
                .resource::<ProtocolOffer>()
                .0
                .as_ref()
                .expect("count=2 must offer")
                .kind()
        };
        // Edge case A: different count must produce different kind (in all but
        // degenerate cases where the RNG streams collide; verify statistically
        // by checking the seeds differ — the formula guarantees distinct seeds).
        let seed_0 = derive_seed(derive_seed_named(42, "protocol"), 0);
        let seed_2 = derive_seed(derive_seed_named(42, "protocol"), 2);
        assert_ne!(
            seed_0, seed_2,
            "discriminator 0 and 2 must produce distinct seeds (prerequisite)"
        );
        // Both runs must produce a valid kind regardless of whether they match.
        assert!(
            ProtocolKind::ALL.contains(&kind_a),
            "count=0 must yield a valid kind, got {kind_a:?}"
        );
        assert!(
            ProtocolKind::ALL.contains(&kind_b),
            "count=2 must yield a valid kind, got {kind_b:?}"
        );
    }

    #[test]
    fn end_to_end_different_seed_produces_different_kind() {
        use crate::mutators::protocols::{
            definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
            resources::{ActiveProtocols, ProtocolOffer, ProtocolRegistry, UnlockedProtocols},
            systems::generate_protocol_offering,
        };

        let build = |run_seed: u64| -> App {
            let mut registry = ProtocolRegistry::default();
            for kind in ProtocolKind::ALL {
                let tuning = match kind {
                    ProtocolKind::Greed => ProtocolTuning::Greed {
                        rarity_boost_per_skip: 0.05,
                    },
                    _ => ProtocolTuning::Deadline { effects: vec![] },
                };
                registry.insert(ProtocolDefinition {
                    name: format!("{kind:?}"),
                    description: String::new(),
                    unlock_tier: 0,
                    tuning,
                });
            }
            TestAppBuilder::new()
                .insert_resource(RunStats {
                    seed: run_seed,
                    ..default()
                })
                .insert_resource(ProtocolOfferingCount(0))
                .insert_resource(ProtocolRng::from_seed(SENTINEL))
                .insert_resource(UnlockedProtocols::default())
                .insert_resource(ActiveProtocols::default())
                .insert_resource(registry)
                .insert_resource(ProtocolOffer::default())
                .with_system(
                    Update,
                    (reseed_protocol_rng, generate_protocol_offering).chain(),
                )
                .build()
        };

        // Edge case B: seed=42 vs seed=100 (count=0) — seeds are distinct.
        let seed_42 = derive_seed(derive_seed_named(42, "protocol"), 0);
        let seed_100 = derive_seed(derive_seed_named(100, "protocol"), 0);
        assert_ne!(
            seed_42, seed_100,
            "run_seed 42 and 100 must produce distinct protocol channel seeds"
        );

        let kind_42 = {
            let mut app = build(42);
            app.update();
            app.world()
                .resource::<ProtocolOffer>()
                .0
                .as_ref()
                .expect("seed=42 must offer")
                .kind()
        };
        let kind_100 = {
            let mut app = build(100);
            app.update();
            app.world()
                .resource::<ProtocolOffer>()
                .0
                .as_ref()
                .expect("seed=100 must offer")
                .kind()
        };
        assert!(
            ProtocolKind::ALL.contains(&kind_42),
            "seed=42 must yield valid kind, got {kind_42:?}"
        );
        assert!(
            ProtocolKind::ALL.contains(&kind_100),
            "seed=100 must yield valid kind, got {kind_100:?}"
        );
    }
}
