use bevy::prelude::*;
use rand::Rng;

use super::system::reseed_chip_rng;
use crate::{
    prelude::*,
    shared::{
        RunSeed,
        rng::{ChipRng, ChipSelectCount, GameRng, derive_seed, derive_seed_named},
    },
    state::run::resources::RunStats,
};

const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

fn test_app() -> App {
    TestAppBuilder::new()
        .with_resource::<RunStats>()
        .with_resource::<RunSeed>()
        .with_resource::<ChipSelectCount>()
        .with_resource::<ChipRng>()
        .with_system(Update, reseed_chip_rng)
        .build()
}

// ── Behavior 6 — first visit (count = 0) ────────────────────────────────

#[test]
fn reseed_chip_rng_seeds_from_canonical_formula_on_first_visit() {
    let mut app = test_app();
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(ChipSelectCount(0));
    app.insert_resource(ChipRng::from_seed(SENTINEL));
    app.update();

    let expected_seed = derive_seed(derive_seed_named(42, "chip"), 0);
    let mut expected_rng = ChipRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();

    let actual_draw: u64 = app.world_mut().resource_mut::<ChipRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "ChipRng first draw must match derive_seed(derive_seed_named(42,\"chip\"),0)"
    );
}

#[test]
fn reseed_chip_rng_seeds_from_canonical_formula_first_visit_zero_seed() {
    let mut app = test_app();
    app.insert_resource(RunStats {
        seed: 0,
        ..default()
    });
    app.insert_resource(ChipSelectCount(0));
    app.insert_resource(ChipRng::from_seed(SENTINEL));
    app.update();

    let expected_seed = derive_seed(derive_seed_named(0, "chip"), 0);
    let mut expected_rng = ChipRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();

    let actual_draw: u64 = app.world_mut().resource_mut::<ChipRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "reseed_chip_rng with run_seed=0, count=0 must not panic and must be deterministic"
    );
}

// ── Behavior 7 — third visit (count = 2) ────────────────────────────────

#[test]
fn reseed_chip_rng_seeds_from_canonical_formula_on_third_visit() {
    let mut app = test_app();
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(ChipSelectCount(2));
    app.insert_resource(ChipRng::from_seed(SENTINEL));
    app.update();

    let expected_seed = derive_seed(derive_seed_named(42, "chip"), 2);
    let mut expected_rng = ChipRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();

    let actual_draw: u64 = app.world_mut().resource_mut::<ChipRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "ChipRng first draw must match derive_seed(derive_seed_named(42,\"chip\"),2)"
    );
}

#[test]
fn reseed_chip_rng_different_count_produces_different_draw() {
    let draw_for_count = |count: u32| -> u64 {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(ChipSelectCount(count));
        app.insert_resource(ChipRng::from_seed(SENTINEL));
        app.update();
        app.world_mut().resource_mut::<ChipRng>().0.random()
    };
    let draw_0 = draw_for_count(0);
    let draw_2 = draw_for_count(2);
    assert_ne!(
        draw_0, draw_2,
        "different chip_select_count discriminators must produce different first draws"
    );
}

// ── Behavior 8 — overwrites pre-existing ChipRng ────────────────────────

#[test]
fn reseed_chip_rng_overwrites_preexisting_chip_rng() {
    let mut app = test_app();
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(ChipSelectCount(1));
    app.insert_resource(ChipRng::from_seed(SENTINEL));
    app.update();

    let expected_seed = derive_seed(derive_seed_named(42, "chip"), 1);
    let mut expected_rng = ChipRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();

    let actual_draw: u64 = app.world_mut().resource_mut::<ChipRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "reseed_chip_rng must overwrite the sentinel-seeded ChipRng, not skip it"
    );
}

#[test]
fn reseed_chip_rng_idempotent_for_same_inputs() {
    // Running twice with identical (seed, count) produces the same post-reseed draw.
    let run_once = || -> u64 {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(ChipSelectCount(1));
        app.insert_resource(ChipRng::from_seed(SENTINEL));
        app.update();
        app.world_mut().resource_mut::<ChipRng>().0.random()
    };
    assert_eq!(
        run_once(),
        run_once(),
        "reseed_chip_rng is idempotent — same (seed, count) always yields the same post-reseed draw"
    );
}

// ── Behavior 9 — stable across independent App instances ─────────────────

#[test]
fn reseed_chip_rng_stable_across_independent_apps() {
    let draw = || -> u64 {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 1234,
            ..default()
        });
        app.insert_resource(ChipSelectCount(7));
        app.insert_resource(ChipRng::from_seed(SENTINEL));
        app.update();
        app.world_mut().resource_mut::<ChipRng>().0.random()
    };
    assert_eq!(
        draw(),
        draw(),
        "ChipRng first draw must be identical across independent apps with same (seed, count)"
    );
}

#[test]
fn reseed_chip_rng_stable_for_zero_seed_zero_count() {
    let draw = || -> u64 {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 0,
            ..default()
        });
        app.insert_resource(ChipSelectCount(0));
        app.insert_resource(ChipRng::from_seed(SENTINEL));
        app.update();
        app.world_mut().resource_mut::<ChipRng>().0.random()
    };
    assert_eq!(
        draw(),
        draw(),
        "reseed_chip_rng(seed=0, count=0) must be stable and not panic"
    );
}

// ── Behavior 10 — reads RunStats.seed, NOT RunSeed ──────────────────────

#[test]
fn reseed_chip_rng_reads_run_stats_seed_not_run_seed() {
    let mut app = TestAppBuilder::new()
        .with_resource::<RunStats>()
        .with_resource::<RunSeed>()
        .with_resource::<ChipSelectCount>()
        .with_resource::<ChipRng>()
        .with_system(Update, reseed_chip_rng)
        .build();
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(RunSeed(Some(999))); // intentionally mismatched
    app.insert_resource(ChipSelectCount(0));
    app.insert_resource(ChipRng::from_seed(SENTINEL));
    app.update();

    // Must use RunStats.seed=42, not RunSeed=999.
    let expected_seed = derive_seed(derive_seed_named(42, "chip"), 0);
    let mut expected_rng = ChipRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();

    let wrong_seed = derive_seed(derive_seed_named(999, "chip"), 0);
    let mut wrong_rng = ChipRng::from_seed(wrong_seed);
    let wrong_draw: u64 = wrong_rng.0.random();

    let actual_draw: u64 = app.world_mut().resource_mut::<ChipRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "reseed_chip_rng must read RunStats.seed (42), not RunSeed (999)"
    );
    assert_ne!(
        actual_draw, wrong_draw,
        "draw must not match the RunSeed=999 path"
    );
}

#[test]
fn reseed_chip_rng_reads_run_stats_seed_when_run_seed_is_none() {
    let mut app = TestAppBuilder::new()
        .with_resource::<RunStats>()
        .with_resource::<RunSeed>()
        .with_resource::<ChipSelectCount>()
        .with_resource::<ChipRng>()
        .with_system(Update, reseed_chip_rng)
        .build();
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(RunSeed(None));
    app.insert_resource(ChipSelectCount(0));
    app.insert_resource(ChipRng::from_seed(SENTINEL));
    app.update();

    let expected_seed = derive_seed(derive_seed_named(42, "chip"), 0);
    let mut expected_rng = ChipRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();

    let actual_draw: u64 = app.world_mut().resource_mut::<ChipRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "reseed_chip_rng must use RunStats.seed even when RunSeed is None"
    );
}

// ── Behavior 11 — does NOT touch ChipSelectCount ────────────────────────

#[test]
fn reseed_chip_rng_does_not_touch_chip_select_count() {
    let mut app = test_app();
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(ChipSelectCount(3));
    app.insert_resource(ChipRng::from_seed(SENTINEL));
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        3,
        "reseed_chip_rng must not modify ChipSelectCount"
    );
}

#[test]
fn reseed_chip_rng_repeated_invocations_leave_chip_select_count_unchanged() {
    let mut app = test_app();
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(ChipSelectCount(3));
    app.insert_resource(ChipRng::from_seed(SENTINEL));

    for _ in 0..5 {
        app.update();
    }

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        3,
        "ChipSelectCount must remain 3 after 5 reseed_chip_rng invocations"
    );
}

// ── Behavior 11b — does NOT advance GameRng ──────────────────────────────

#[test]
fn reseed_chip_rng_does_not_advance_game_rng() {
    let mut app = TestAppBuilder::new()
        .with_resource::<RunStats>()
        .with_resource::<RunSeed>()
        .with_resource::<ChipSelectCount>()
        .with_resource::<ChipRng>()
        .with_resource::<GameRng>()
        .with_system(Update, reseed_chip_rng)
        .build();
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(ChipSelectCount(0));
    app.insert_resource(ChipRng::from_seed(SENTINEL));
    app.insert_resource(GameRng::from_seed(99));
    app.update();

    // GameRng state must be identical to a fresh GameRng::from_seed(99).
    let mut fresh_game_rng = GameRng::from_seed(99);
    let expected_game_draw: u64 = fresh_game_rng.0.random();

    let actual_game_draw: u64 = app.world_mut().resource_mut::<GameRng>().0.random();
    assert_eq!(
        actual_game_draw, expected_game_draw,
        "reseed_chip_rng must NOT advance GameRng state (no GameRng read)"
    );
}

#[test]
fn reseed_chip_rng_five_invocations_leave_game_rng_untouched() {
    let mut app = TestAppBuilder::new()
        .with_resource::<RunStats>()
        .with_resource::<RunSeed>()
        .with_resource::<ChipSelectCount>()
        .with_resource::<ChipRng>()
        .with_resource::<GameRng>()
        .with_system(Update, reseed_chip_rng)
        .build();
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(ChipSelectCount(0));
    app.insert_resource(ChipRng::from_seed(SENTINEL));
    app.insert_resource(GameRng::from_seed(99));

    for _ in 0..5 {
        app.update();
    }

    let mut fresh_game_rng = GameRng::from_seed(99);
    let expected_game_draw: u64 = fresh_game_rng.0.random();

    let actual_game_draw: u64 = app.world_mut().resource_mut::<GameRng>().0.random();
    assert_eq!(
        actual_game_draw, expected_game_draw,
        "after 5 reseed_chip_rng calls, GameRng must still match a fresh from_seed(99)"
    );
}
