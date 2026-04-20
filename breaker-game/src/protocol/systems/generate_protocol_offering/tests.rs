//! Tests for `generate_protocol_offering` — pool selection, determinism, and
//! the `ProtocolOffer` resource write path.

use bevy::prelude::*;

use super::generate_protocol_offering;
use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        resources::{ActiveProtocols, ProtocolOffer, ProtocolRegistry, UnlockedProtocols},
    },
};

/// Local helper — builds a `ProtocolDefinition` for a given kind with a
/// human-readable name and canonical tuning values (mirrors
/// `protocol/resources.rs::tests::def_for`).
fn def_for(kind: ProtocolKind, name: &str) -> ProtocolDefinition {
    let tuning = match kind {
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
        ProtocolKind::Greed => ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
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
        },
        ProtocolKind::Conductor => ProtocolTuning::Conductor {
            primary_swap_window: 0.2,
        },
        ProtocolKind::Afterimage => ProtocolTuning::Afterimage {
            phantom_duration:      1.5,
            phantom_bolt_duration: 0.75,
        },
        ProtocolKind::Fission => ProtocolTuning::Fission {
            kills_per_split: 10,
        },
        ProtocolKind::TierRegression => ProtocolTuning::TierRegression { tiers_back: 1 },
    };
    ProtocolDefinition {
        name: name.to_string(),
        description: String::new(),
        unlock_tier: 0,
        tuning,
    }
}

/// Build a registry seeded with every `ProtocolKind::ALL` variant.
fn fully_seeded_registry() -> ProtocolRegistry {
    let mut registry = ProtocolRegistry::default();
    for kind in ProtocolKind::ALL {
        registry.insert(def_for(*kind, &format!("{kind:?}")));
    }
    registry
}

/// Build an `ActiveProtocols` holding every kind in `ProtocolKind::ALL`.
fn fully_active_protocols() -> ActiveProtocols {
    let mut active = ActiveProtocols::default();
    for kind in ProtocolKind::ALL {
        active.insert(def_for(*kind, &format!("{kind:?}")));
    }
    active
}

/// Build an `ActiveProtocols` holding every `ProtocolKind::ALL` variant
/// except the given `omit` kind.
fn active_except(omit: ProtocolKind) -> ActiveProtocols {
    let mut active = ActiveProtocols::default();
    for kind in ProtocolKind::ALL {
        if *kind != omit {
            active.insert(def_for(*kind, &format!("{kind:?}")));
        }
    }
    active
}

/// Convenience: build a test app at Update with the system wired and
/// all required resources pre-seeded. The caller inserts `GameRng`,
/// `UnlockedProtocols`, `ActiveProtocols`, `ProtocolRegistry`, and
/// `ProtocolOffer` before calling this — this helper simply assembles the
/// result and registers the system.
fn build_app(
    unlocked: UnlockedProtocols,
    active: ActiveProtocols,
    registry: ProtocolRegistry,
    rng: GameRng,
    offer: ProtocolOffer,
) -> App {
    TestAppBuilder::new()
        .insert_resource(unlocked)
        .insert_resource(active)
        .insert_resource(registry)
        .insert_resource(rng)
        .insert_resource(offer)
        .with_system(Update, generate_protocol_offering)
        .build()
}

// ── Behavior 1: Offer populates when every unlocked kind is eligible ───────

#[test]
fn offer_populates_when_every_unlocked_kind_is_eligible() {
    let mut app = build_app(
        UnlockedProtocols::default(),
        ActiveProtocols::default(),
        fully_seeded_registry(),
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );

    app.update();

    let offer = app.world().resource::<ProtocolOffer>();
    assert!(
        offer.0.is_some(),
        "expected ProtocolOffer to be Some after generate_protocol_offering"
    );
    let kind = offer.0.as_ref().unwrap().kind();
    assert!(
        ProtocolKind::ALL.contains(&kind),
        "expected offered kind to be one of ProtocolKind::ALL, got {kind:?}"
    );
}

#[test]
fn offer_is_deterministic_across_two_runs_with_same_seed() {
    // Edge case of Behavior 1: re-running with a fresh GameRng::from_seed(42)
    // produces the same kind.
    let mut app = build_app(
        UnlockedProtocols::default(),
        ActiveProtocols::default(),
        fully_seeded_registry(),
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );
    app.update();
    let first_kind = app
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("first run should populate offer")
        .kind();

    // Reseed RNG and clear the offer, then run again.
    app.world_mut().insert_resource(GameRng::from_seed(42));
    app.world_mut().insert_resource(ProtocolOffer::default());
    app.update();
    let second_kind = app
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("second run should populate offer")
        .kind();

    assert_eq!(
        first_kind, second_kind,
        "same seed must produce the same offered kind"
    );
}

// ── Behavior 2: Offer is None when all unlocked kinds are already active ──

#[test]
fn offer_is_none_when_all_unlocked_kinds_are_already_active() {
    let mut app = build_app(
        UnlockedProtocols::default(),
        fully_active_protocols(),
        fully_seeded_registry(),
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );

    app.update();

    let offer = app.world().resource::<ProtocolOffer>();
    assert!(
        offer.0.is_none(),
        "expected ProtocolOffer to be None when every unlocked kind is already active, got {:?}",
        offer.0.as_ref().map(ProtocolDefinition::kind)
    );
}

#[test]
fn offer_overwrites_stale_prior_offer_with_none_when_pool_is_empty() {
    // Edge case of Behavior 2: stale ProtocolOffer(Some(...)) is cleared.
    let stale = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    let mut app = build_app(
        UnlockedProtocols::default(),
        fully_active_protocols(),
        fully_seeded_registry(),
        GameRng::from_seed(42),
        stale,
    );

    app.update();

    let offer = app.world().resource::<ProtocolOffer>();
    assert!(
        offer.0.is_none(),
        "system must overwrite any stale prior offer with None when eligible pool is empty"
    );
}

// ── Behavior 3: Offer selects the single eligible kind when 14 are active ─

#[test]
fn offer_selects_the_single_eligible_kind_when_14_are_active_anchor() {
    let mut app = build_app(
        UnlockedProtocols::default(),
        active_except(ProtocolKind::Anchor),
        fully_seeded_registry(),
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );

    app.update();

    let kind = app
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("expected ProtocolOffer::Some when one kind is still eligible")
        .kind();
    assert_eq!(kind, ProtocolKind::Anchor);
}

#[test]
fn offer_selects_the_single_eligible_kind_when_14_are_active_tier_regression() {
    // Edge case: proves the pick is not biased toward a specific slot in ALL.
    let mut app = build_app(
        UnlockedProtocols::default(),
        active_except(ProtocolKind::TierRegression),
        fully_seeded_registry(),
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );

    app.update();

    let kind = app
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("expected ProtocolOffer::Some when one kind is still eligible")
        .kind();
    assert_eq!(kind, ProtocolKind::TierRegression);
}

// ── Behavior 4: Offer skips locked kinds ──────────────────────────────────

/// Build a registry that ONLY contains the given kinds — equivalent to
/// restricting the "unlocked" set when `UnlockedProtocols::default()` still
/// contains all 15. This is a test-only proxy for `unlocked_with` since
/// `UnlockedProtocols` has no public mutation helpers beyond `empty()`.
fn registry_with(kinds: &[ProtocolKind]) -> ProtocolRegistry {
    let mut registry = ProtocolRegistry::default();
    for kind in kinds {
        registry.insert(def_for(*kind, &format!("{kind:?}")));
    }
    registry
}

#[test]
fn offer_skips_locked_kinds_five_unlocked() {
    // Proxy for the "5 kinds unlocked" test: seed registry with only those
    // 5 kinds. The union of unlocked ∩ registry ∩ !active is exactly the 5.
    let allowed = [
        ProtocolKind::Deadline,
        ProtocolKind::Ricochet,
        ProtocolKind::Anchor,
        ProtocolKind::Kickstart,
        ProtocolKind::Greed,
    ];
    let mut app = build_app(
        UnlockedProtocols::default(),
        ActiveProtocols::default(),
        registry_with(&allowed),
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );

    app.update();

    let kind = app
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("expected ProtocolOffer::Some when 5 kinds are eligible")
        .kind();
    assert!(
        allowed.contains(&kind),
        "expected offered kind to be one of {allowed:?}, got {kind:?}"
    );
}

#[test]
fn offer_skips_locked_kinds_one_unlocked_burnout() {
    // Edge case: exactly one kind eligible.
    let allowed = [ProtocolKind::Burnout];
    let mut app = build_app(
        UnlockedProtocols::default(),
        ActiveProtocols::default(),
        registry_with(&allowed),
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );

    app.update();

    let kind = app
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("expected ProtocolOffer::Some when Burnout alone is eligible")
        .kind();
    assert_eq!(kind, ProtocolKind::Burnout);
}

// ── Behavior 5: Offer is None when the registry is missing a definition ───

#[test]
fn offer_is_none_when_sole_eligible_kind_missing_from_registry() {
    // Registry only holds Deadline — nothing else is eligible because the
    // other 14 kinds aren't in the registry.
    //
    // To exercise "eligibility requires registry", we mark 14 of 15 kinds
    // as active, leaving only Burnout eligible-by-unlocked. But the
    // registry holds ONLY Deadline, so Burnout lookups fail and the pool
    // is empty.
    let mut active = ActiveProtocols::default();
    for kind in ProtocolKind::ALL {
        if *kind != ProtocolKind::Burnout {
            active.insert(def_for(*kind, &format!("{kind:?}")));
        }
    }
    let registry = registry_with(&[ProtocolKind::Deadline]);

    let mut app = build_app(
        UnlockedProtocols::default(),
        active,
        registry,
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );

    app.update();

    let offer = app.world().resource::<ProtocolOffer>();
    assert!(
        offer.0.is_none(),
        "eligibility must require the registry to hold the definition — \
         Burnout is unlocked-and-not-active but missing from registry, so pool is empty"
    );
}

#[test]
fn offer_returns_present_kind_even_when_other_eligible_kinds_missing_from_registry() {
    // Edge case: Burnout and Deadline are both unlocked-and-not-active;
    // the registry holds only Deadline. The system must return Deadline
    // (not panic, not offer a missing kind).
    let mut active = ActiveProtocols::default();
    for kind in ProtocolKind::ALL {
        if *kind != ProtocolKind::Burnout && *kind != ProtocolKind::Deadline {
            active.insert(def_for(*kind, &format!("{kind:?}")));
        }
    }
    let registry = registry_with(&[ProtocolKind::Deadline]);

    let mut app = build_app(
        UnlockedProtocols::default(),
        active,
        registry,
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );

    app.update();

    let kind = app
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("expected ProtocolOffer::Some when Deadline is the only registry-present kind")
        .kind();
    assert_eq!(kind, ProtocolKind::Deadline);
}

// ── Behavior 6: Determinism under seed across independent apps ────────────

#[test]
fn offer_is_deterministic_across_two_independent_apps_with_same_seed_7() {
    let mut app_a = build_app(
        UnlockedProtocols::default(),
        ActiveProtocols::default(),
        fully_seeded_registry(),
        GameRng::from_seed(7),
        ProtocolOffer::default(),
    );
    let mut app_b = build_app(
        UnlockedProtocols::default(),
        ActiveProtocols::default(),
        fully_seeded_registry(),
        GameRng::from_seed(7),
        ProtocolOffer::default(),
    );

    app_a.update();
    app_b.update();

    let kind_a = app_a
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("app_a should have ProtocolOffer::Some")
        .kind();
    let kind_b = app_b
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("app_b should have ProtocolOffer::Some")
        .kind();

    assert_eq!(
        kind_a, kind_b,
        "independent apps with the same seed must produce the same offered kind"
    );
}

#[test]
fn offer_is_deterministic_across_two_independent_apps_with_same_seed_8() {
    let mut app_a = build_app(
        UnlockedProtocols::default(),
        ActiveProtocols::default(),
        fully_seeded_registry(),
        GameRng::from_seed(8),
        ProtocolOffer::default(),
    );
    let mut app_b = build_app(
        UnlockedProtocols::default(),
        ActiveProtocols::default(),
        fully_seeded_registry(),
        GameRng::from_seed(8),
        ProtocolOffer::default(),
    );

    app_a.update();
    app_b.update();

    let kind_a = app_a
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("app_a should have ProtocolOffer::Some")
        .kind();
    let kind_b = app_b
        .world()
        .resource::<ProtocolOffer>()
        .0
        .as_ref()
        .expect("app_b should have ProtocolOffer::Some")
        .kind();

    assert_eq!(kind_a, kind_b);
}

// ── Behavior 7: Offer is None when ProtocolRegistry is empty ──────────────

#[test]
fn offer_is_none_when_protocol_registry_is_empty() {
    let mut app = build_app(
        UnlockedProtocols::default(),
        ActiveProtocols::default(),
        ProtocolRegistry::default(),
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );

    app.update();

    let offer = app.world().resource::<ProtocolOffer>();
    assert!(
        offer.0.is_none(),
        "empty ProtocolRegistry must produce ProtocolOffer(None), not panic"
    );
}

// ── Behavior 8: Offer is None when UnlockedProtocols is empty ─────────────

#[test]
fn offer_is_none_when_unlocked_protocols_is_empty() {
    let mut app = build_app(
        UnlockedProtocols::empty(),
        ActiveProtocols::default(),
        fully_seeded_registry(),
        GameRng::from_seed(42),
        ProtocolOffer::default(),
    );

    app.update();

    let offer = app.world().resource::<ProtocolOffer>();
    assert!(
        offer.0.is_none(),
        "UnlockedProtocols::empty() must produce ProtocolOffer(None)"
    );
}
