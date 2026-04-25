//! Tests for `generate_hazard_offerings` — uniform random draw-without-replacement
//! of up to 3 hazard definitions into `HazardOffers`.

use std::collections::HashSet;

use bevy::prelude::*;

use super::*;
use crate::mutators::hazards::{
    definition::{HazardDefinition, HazardKind, HazardTuning},
    resources::{ActiveHazards, HazardOffers, HazardRegistry},
};

// Local duplication of `hazard/definition.rs::tests::tuning_variants_*` —
// avoids exposing a crate-visible test helper. Mirrors the protocol-test
// duplication convention.
fn tuning_variants_first_half() -> Vec<(HazardTuning, HazardKind)> {
    vec![
        (
            HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
            HazardKind::Decay,
        ),
        (
            HazardTuning::Drift {
                force:           100.0,
                period_secs:     8.0,
                per_level_force: 33.3,
            },
            HazardKind::Drift,
        ),
        (
            HazardTuning::Haste {
                base_percent:      0.1,
                per_level_percent: 0.05,
            },
            HazardKind::Haste,
        ),
        (
            HazardTuning::EchoCells {
                delay_secs:           1.5,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
            HazardKind::EchoCells,
        ),
        (
            HazardTuning::Erosion {
                shrink_rate:      0.05,
                min_width_frac:   0.35,
                restore_nonwhiff: 0.25,
                restore_perfect:  0.5,
            },
            HazardKind::Erosion,
        ),
        (
            HazardTuning::Cascade {
                base_heal:      1.0,
                per_level_heal: 0.5,
            },
            HazardKind::Cascade,
        ),
        (
            HazardTuning::Fracture {
                base_splits:      1,
                per_level_splits: 1,
            },
            HazardKind::Fracture,
        ),
        (
            HazardTuning::Renewal {
                base_period_secs:         10.0,
                per_level_reduction_frac: 0.2,
            },
            HazardKind::Renewal,
        ),
    ]
}

fn tuning_variants_second_half() -> Vec<(HazardTuning, HazardKind)> {
    vec![
        (
            HazardTuning::Volatility {
                hp_per_interval: 1.0,
                interval_secs:   5.0,
                max_multiplier:  2.0,
            },
            HazardKind::Volatility,
        ),
        (
            HazardTuning::GravitySurge {
                base_duration_secs:      2.0,
                per_level_duration_secs: 1.0,
                base_strength:           100.0,
                per_level_strength_frac: 0.2,
            },
            HazardKind::GravitySurge,
        ),
        (
            HazardTuning::Overcharge {
                base_frac:      0.05,
                per_level_frac: 0.03,
            },
            HazardKind::Overcharge,
        ),
        (
            HazardTuning::Resonance {
                kills_to_trigger:      2,
                base_window:           0.5,
                window_per_level:      0.3,
                wave_speed:            200.0,
                base_slow_duration:    1.5,
                base_slow_strength:    0.5,
                slow_duration_scaling: 0.2,
                slow_strength_scaling: 0.15,
                contact_threshold:     16.0,
                wave_max_lifetime:     10.0,
            },
            HazardKind::Resonance,
        ),
        (
            HazardTuning::Diffusion {
                base_share_frac:      0.2,
                per_level_share_frac: 0.1,
                depth_every_levels:   5,
            },
            HazardKind::Diffusion,
        ),
        (
            HazardTuning::Tether {
                base_share_frac:         0.25,
                per_level_share_frac:    0.1,
                base_coverage_frac:      0.4,
                per_level_coverage_frac: 0.1,
            },
            HazardKind::Tether,
        ),
        (
            HazardTuning::Momentum {
                base_hp_per_hit:      10.0,
                per_level_hp_per_hit: 10.0,
            },
            HazardKind::Momentum,
        ),
        (
            HazardTuning::Sympathy {
                base_heal_frac:      0.25,
                per_level_heal_frac: 0.05,
                depth_every_levels:  5,
            },
            HazardKind::Sympathy,
        ),
    ]
}

fn tuning_for_kind(kind: HazardKind) -> HazardTuning {
    let mut all = tuning_variants_first_half();
    all.extend(tuning_variants_second_half());
    all.into_iter()
        .find(|(_, k)| *k == kind)
        .map(|(t, _)| t)
        .expect("every HazardKind must have a tuning in the local helper")
}

fn make_hazard_def(kind: HazardKind, name: &str) -> HazardDefinition {
    HazardDefinition {
        name:        name.to_owned(),
        description: format!("{kind:?} hazard"),
        unlock_tier: 0,
        tuning:      tuning_for_kind(kind),
    }
}

fn make_registry_with(kinds: &[HazardKind]) -> HazardRegistry {
    let mut registry = HazardRegistry::default();
    for kind in kinds {
        registry.insert(make_hazard_def(*kind, &format!("{kind:?}")));
    }
    registry
}

fn make_full_registry() -> HazardRegistry {
    make_registry_with(HazardKind::ALL)
}

fn test_app_with(
    registry: HazardRegistry,
    active: ActiveHazards,
    seed: u64,
    offers: HazardOffers,
) -> App {
    TestAppBuilder::new()
        .insert_resource(registry)
        .insert_resource(active)
        .insert_resource(GameRng::from_seed(seed))
        .insert_resource(offers)
        .with_system(Update, generate_hazard_offerings)
        .build()
}

// ── Domain B.1: exactly 3 offers when registry has ≥ 3 definitions ──────

#[test]
fn writes_exactly_three_offers_when_registry_has_three_definitions() {
    let registry = make_registry_with(&[HazardKind::Decay, HazardKind::Drift, HazardKind::Haste]);
    let mut app = test_app_with(
        registry,
        ActiveHazards::default(),
        42,
        HazardOffers::default(),
    );
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    assert_eq!(
        offers.0.len(),
        3,
        "expected exactly 3 offers with 3-kind registry, got {}",
        offers.0.len()
    );
}

// ── Domain B.2: 3 distinct kinds when registry has ≥ 3 kinds ─────────────

#[test]
fn all_three_offers_are_distinct_kinds_seed_42() {
    let mut app = test_app_with(
        make_full_registry(),
        ActiveHazards::default(),
        42,
        HazardOffers::default(),
    );
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    let kinds: HashSet<HazardKind> = offers.0.iter().map(HazardDefinition::kind).collect();
    assert_eq!(
        kinds.len(),
        3,
        "expected 3 distinct kinds, got {kinds:?} from offers {:?}",
        offers
            .0
            .iter()
            .map(HazardDefinition::kind)
            .collect::<Vec<_>>()
    );
}

#[test]
fn all_three_offers_are_distinct_kinds_seed_100() {
    let mut app = test_app_with(
        make_full_registry(),
        ActiveHazards::default(),
        100,
        HazardOffers::default(),
    );
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    let kinds: HashSet<HazardKind> = offers.0.iter().map(HazardDefinition::kind).collect();
    assert_eq!(kinds.len(), 3, "expected 3 distinct kinds at seed 100");
}

#[test]
fn all_three_offers_are_distinct_kinds_seed_9999() {
    let mut app = test_app_with(
        make_full_registry(),
        ActiveHazards::default(),
        9999,
        HazardOffers::default(),
    );
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    let kinds: HashSet<HazardKind> = offers.0.iter().map(HazardDefinition::kind).collect();
    assert_eq!(kinds.len(), 3, "expected 3 distinct kinds at seed 9999");
}

// ── Domain B.3: offers are determined by the seed ────────────────────────

#[test]
fn offers_are_determined_by_seed_42() {
    let mut app_a = test_app_with(
        make_full_registry(),
        ActiveHazards::default(),
        42,
        HazardOffers::default(),
    );
    app_a.update();

    let mut app_b = test_app_with(
        make_full_registry(),
        ActiveHazards::default(),
        42,
        HazardOffers::default(),
    );
    app_b.update();

    let kinds_a: Vec<HazardKind> = app_a
        .world()
        .resource::<HazardOffers>()
        .0
        .iter()
        .map(HazardDefinition::kind)
        .collect();
    let kinds_b: Vec<HazardKind> = app_b
        .world()
        .resource::<HazardOffers>()
        .0
        .iter()
        .map(HazardDefinition::kind)
        .collect();

    assert_eq!(
        kinds_a, kinds_b,
        "two apps with same seed 42 must produce the same ordered kinds"
    );
}

#[test]
fn offers_are_determined_by_seed_7() {
    let mut app_a = test_app_with(
        make_full_registry(),
        ActiveHazards::default(),
        7,
        HazardOffers::default(),
    );
    app_a.update();

    let mut app_b = test_app_with(
        make_full_registry(),
        ActiveHazards::default(),
        7,
        HazardOffers::default(),
    );
    app_b.update();

    let kinds_a: Vec<HazardKind> = app_a
        .world()
        .resource::<HazardOffers>()
        .0
        .iter()
        .map(HazardDefinition::kind)
        .collect();
    let kinds_b: Vec<HazardKind> = app_b
        .world()
        .resource::<HazardOffers>()
        .0
        .iter()
        .map(HazardDefinition::kind)
        .collect();

    assert_eq!(
        kinds_a, kinds_b,
        "two apps with same seed 7 must produce the same ordered kinds"
    );
}

// ── Domain B.4: empty registry produces empty HazardOffers (no panic) ─────

#[test]
fn empty_registry_produces_empty_hazard_offers() {
    let mut app = test_app_with(
        HazardRegistry::default(),
        ActiveHazards::default(),
        42,
        HazardOffers::default(),
    );
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    assert!(
        offers.0.is_empty(),
        "empty registry must yield empty HazardOffers, got {} entries",
        offers.0.len()
    );
}

// ── Domain B.5: registry with N < 3 kinds yields N offers ────────────────

#[test]
fn registry_with_one_kind_produces_one_offer_of_that_kind() {
    let registry = make_registry_with(&[HazardKind::Decay]);
    let mut app = test_app_with(
        registry,
        ActiveHazards::default(),
        42,
        HazardOffers::default(),
    );
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    assert_eq!(
        offers.0.len(),
        1,
        "registry with 1 kind must yield exactly 1 offer, got {}",
        offers.0.len()
    );
    assert_eq!(offers.0[0].kind(), HazardKind::Decay);
}

#[test]
fn registry_with_two_kinds_produces_two_distinct_offers() {
    let registry = make_registry_with(&[HazardKind::Decay, HazardKind::Drift]);
    let mut app = test_app_with(
        registry,
        ActiveHazards::default(),
        42,
        HazardOffers::default(),
    );
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    assert_eq!(
        offers.0.len(),
        2,
        "registry with 2 kinds must yield 2 offers"
    );
    let kinds: HashSet<HazardKind> = offers.0.iter().map(HazardDefinition::kind).collect();
    assert!(kinds.contains(&HazardKind::Decay));
    assert!(kinds.contains(&HazardKind::Drift));
    assert_eq!(kinds.len(), 2);
}

// ── Domain B.6: HazardOffers is overwritten, not appended ────────────────

#[test]
fn hazard_offers_is_always_overwritten_not_appended() {
    // Pre-seed HazardOffers with a stale Sympathy entry. The system must
    // produce exactly 3 offers — total — not 4.
    let stale = HazardOffers(vec![make_hazard_def(HazardKind::Sympathy, "Sympathy")]);
    let mut app = test_app_with(make_full_registry(), ActiveHazards::default(), 42, stale);
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    assert_eq!(
        offers.0.len(),
        3,
        "pre-seeded offers must be fully overwritten, not extended; got {}",
        offers.0.len()
    );
}

// ── Domain B.7: ActiveHazards does NOT filter offers ─────────────────────

#[test]
fn active_hazards_state_does_not_filter_offers() {
    let mut active = ActiveHazards::default();
    for kind in HazardKind::ALL {
        active.add_stack(*kind);
    }

    let mut app = test_app_with(make_full_registry(), active, 42, HazardOffers::default());
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    assert_eq!(
        offers.0.len(),
        3,
        "hazards stack — fully-active ActiveHazards must not filter out offers; got {}",
        offers.0.len()
    );
}

#[test]
fn empty_active_hazards_also_yields_three_offers() {
    let mut app = test_app_with(
        make_full_registry(),
        ActiveHazards::default(),
        42,
        HazardOffers::default(),
    );
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    assert_eq!(offers.0.len(), 3);
}

// ── Domain B.8: HazardOffers is present after the system runs ────────────

#[test]
fn hazard_offers_is_present_after_system_runs_without_prior_insertion() {
    // No HazardOffers resource inserted; the system must create it.
    let mut app = TestAppBuilder::new()
        .insert_resource(make_full_registry())
        .insert_resource(ActiveHazards::default())
        .insert_resource(GameRng::from_seed(11))
        .with_system(Update, generate_hazard_offerings)
        .build();
    app.update();

    let offers = app
        .world()
        .get_resource::<HazardOffers>()
        .expect("HazardOffers resource must be present after generate_hazard_offerings runs");
    assert_eq!(offers.0.len(), 3, "expected 3 offers after generation");
}

#[test]
fn hazard_offers_present_after_run_with_default_pre_inserted() {
    let mut app = test_app_with(
        make_full_registry(),
        ActiveHazards::default(),
        11,
        HazardOffers::default(),
    );
    app.update();

    let offers = app.world().resource::<HazardOffers>();
    assert_eq!(offers.0.len(), 3);
}
