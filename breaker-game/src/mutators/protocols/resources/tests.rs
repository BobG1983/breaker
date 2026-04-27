use bevy::prelude::*;

use super::types::*;
use crate::mutators::protocols::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// Helper: build a `ProtocolDefinition` for a given `kind` with a
// human-readable name suitable for assertions.
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
        ProtocolKind::Conductor => ProtocolTuning::Conductor,
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

// ── Behavior 5: ProtocolRegistry::default() is empty ──────────────────

#[test]
fn protocol_registry_default_is_empty() {
    let registry = ProtocolRegistry::default();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

// ── Behavior 6: ProtocolRegistry::insert + get roundtrip ──────────────

#[test]
fn protocol_registry_insert_then_get_returns_definition() {
    let mut registry = ProtocolRegistry::default();
    registry.insert(def_for(ProtocolKind::Deadline, "Deadline"));

    let fetched = registry.get(ProtocolKind::Deadline);
    assert!(
        fetched.is_some(),
        "expected Some after inserting Deadline definition"
    );
    assert_eq!(fetched.unwrap().name, "Deadline");
}

#[test]
fn protocol_registry_get_missing_kind_returns_none() {
    let mut registry = ProtocolRegistry::default();
    registry.insert(def_for(ProtocolKind::Deadline, "Deadline"));
    assert!(
        registry.get(ProtocolKind::Greed).is_none(),
        "unseen kind should return None"
    );
}

// ── Behavior 7: insert replaces on same kind ──────────────────────────

#[test]
fn protocol_registry_insert_replaces_existing_definition() {
    let mut registry = ProtocolRegistry::default();
    registry.insert(def_for(ProtocolKind::Deadline, "old"));
    registry.insert(def_for(ProtocolKind::Deadline, "new"));

    assert_eq!(registry.len(), 1, "second insert must replace, not append");
    assert_eq!(registry.get(ProtocolKind::Deadline).unwrap().name, "new");
}

// ── Behavior 8: ActiveProtocols::default() is empty ───────────────────

#[test]
fn active_protocols_default_is_empty() {
    let active = ActiveProtocols::default();
    assert!(active.is_empty());
}

// ── Behavior 9: ActiveProtocols insert + contains ─────────────────────

#[test]
fn active_protocols_insert_then_contains_returns_true() {
    let mut active = ActiveProtocols::default();
    active.insert(def_for(ProtocolKind::Greed, "Greed"));

    assert!(active.contains(ProtocolKind::Greed));
    assert!(
        !active.contains(ProtocolKind::Deadline),
        "other kinds must remain absent"
    );
}

// ── Behavior 10: clear() empties the set ──────────────────────────────

#[test]
fn active_protocols_clear_empties_the_set() {
    let mut active = ActiveProtocols::default();
    active.insert(def_for(ProtocolKind::Deadline, "Deadline"));
    active.insert(def_for(ProtocolKind::Greed, "Greed"));

    active.clear();

    assert!(active.is_empty());
    assert!(!active.contains(ProtocolKind::Deadline));
    assert!(!active.contains(ProtocolKind::Greed));
}

// ── Behavior 11: UnlockedProtocols::default() contains every kind ─────

#[test]
fn unlocked_protocols_default_length_is_15() {
    let unlocked = UnlockedProtocols::default();
    assert_eq!(unlocked.len(), 15);
}

#[test]
fn unlocked_protocols_default_contains_every_kind() {
    // Independent list (not `ProtocolKind::ALL`) so the assertion cannot
    // pass vacuously if `ALL` is accidentally empty.
    let every_kind = [
        ProtocolKind::Deadline,
        ProtocolKind::Ricochet,
        ProtocolKind::Anchor,
        ProtocolKind::Kickstart,
        ProtocolKind::DebtCollector,
        ProtocolKind::IronCurtain,
        ProtocolKind::EchoStrike,
        ProtocolKind::Siphon,
        ProtocolKind::Greed,
        ProtocolKind::RecklessDash,
        ProtocolKind::Burnout,
        ProtocolKind::Conductor,
        ProtocolKind::Afterimage,
        ProtocolKind::Fission,
        ProtocolKind::TierRegression,
    ];

    let unlocked = UnlockedProtocols::default();
    for kind in every_kind {
        assert!(
            unlocked.contains(kind),
            "UnlockedProtocols::default() should contain {kind:?}"
        );
    }
}

// ── Behavior 12: ProtocolOffer::default() is ProtocolOffer(None) ──────

#[test]
fn protocol_offer_default_is_none() {
    let offer = ProtocolOffer::default();
    assert!(offer.0.is_none());
}

#[test]
fn protocol_offer_can_carry_a_definition() {
    // Proves the inner type is `Option<ProtocolDefinition>` so callers
    // can actually carry offer data.
    let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
    assert!(offer.0.is_some());
    assert_eq!(offer.0.as_ref().unwrap().kind(), ProtocolKind::Greed);
}

// ── Behavior 13: protocol_active() run condition ──────────────────────

/// Counter resource that systems increment to prove they ran.
#[derive(Resource, Default)]
struct RanCounter(u32);

fn increment_counter(mut counter: ResMut<RanCounter>) {
    counter.0 += 1;
}

#[test]
fn protocol_active_gates_system_on_active_protocols() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ActiveProtocols>()
        .init_resource::<RanCounter>()
        .add_systems(
            Update,
            increment_counter.run_if(protocol_active(ProtocolKind::Greed)),
        );

    // First update: Greed is not active, system should NOT run.
    app.update();
    assert_eq!(
        app.world().resource::<RanCounter>().0,
        0,
        "protocol_active(Greed) should gate off when ActiveProtocols is empty"
    );

    // Insert Greed → system should run next frame.
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(def_for(ProtocolKind::Greed, "Greed"));

    app.update();
    assert_eq!(
        app.world().resource::<RanCounter>().0,
        1,
        "protocol_active(Greed) should pass once Greed is in ActiveProtocols"
    );
}

#[test]
fn protocol_active_does_not_gate_on_other_kinds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ActiveProtocols>()
        .init_resource::<RanCounter>()
        .add_systems(
            Update,
            increment_counter.run_if(protocol_active(ProtocolKind::Deadline)),
        );

    // Only Greed inserted — Deadline predicate should stay false.
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(def_for(ProtocolKind::Greed, "Greed"));

    app.update();
    assert_eq!(
        app.world().resource::<RanCounter>().0,
        0,
        "protocol_active(Deadline) should stay false while only Greed is active"
    );
}
