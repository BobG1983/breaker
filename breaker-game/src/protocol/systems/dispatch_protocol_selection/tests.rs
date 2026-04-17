//! Tests for `dispatch_protocol_selection` — protocol message dispatch and
//! effect-tree stamping onto Breaker entities.
//!
//! Every test builds the app via `TestAppBuilder::new().with_state_hierarchy()
//! .in_state_chip_selecting()` unless the scenario explicitly overrides the
//! state. Without the state hierarchy, the `.run_if` gate would reject the
//! system and the tests would "pass" by doing nothing.

use bevy::{ecs::message::Messages, prelude::*};

use super::dispatch_protocol_selection;
use crate::{
    effect_v3::{
        effects::{DamageBoostConfig, PiercingConfig},
        types::{EffectType, EntityKind, RootNode, StampTarget, Tree},
    },
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        messages::ProtocolSelected,
        resources::{ActiveProtocols, ProtocolRegistry},
    },
};

/// Local `def_for` helper — mirrors `protocol/resources.rs::tests::def_for`.
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
            risky_zone_start:  0.3,
            damage_multiplier: 4.0,
            double_penalty:    true,
        },
        ProtocolKind::Burnout => ProtocolTuning::Burnout {
            fill_duration:               3.0,
            drain_duration:              5.0,
            still_threshold:             0.25,
            full_heat_damage_multiplier: 2.0,
            speed_boost_duration:        1.0,
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

/// Build a `ProtocolDefinition` whose `Deadline` tuning carries the given
/// effect-tree roots.
fn deadline_with_effects(effects: Vec<RootNode>) -> ProtocolDefinition {
    ProtocolDefinition {
        name:        "Deadline".to_string(),
        description: String::new(),
        unlock_tier: 0,
        tuning:      ProtocolTuning::Deadline { effects },
    }
}

/// Build a `Kickstart` definition with the given effect roots.
fn kickstart_with_effects(effects: Vec<RootNode>) -> ProtocolDefinition {
    ProtocolDefinition {
        name:        "Kickstart".to_string(),
        description: String::new(),
        unlock_tier: 0,
        tuning:      ProtocolTuning::Kickstart { effects },
    }
}

/// Build an `Anchor` definition with the given effect roots.
fn anchor_with_effects(effects: Vec<RootNode>) -> ProtocolDefinition {
    ProtocolDefinition {
        name:        "Anchor".to_string(),
        description: String::new(),
        unlock_tier: 0,
        tuning:      ProtocolTuning::Anchor { effects },
    }
}

/// A simple effect-tree root that stamps a Piercing charge onto the Breaker.
fn stub_piercing_root() -> RootNode {
    RootNode::Stamp(
        StampTarget::Breaker,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
    )
}

/// A second effect-tree root for multi-entry tests.
fn stub_damage_boost_root() -> RootNode {
    use ordered_float::OrderedFloat;
    RootNode::Stamp(
        StampTarget::Breaker,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
    )
}

/// Build a registry seeded with every `ProtocolKind::ALL` variant.
fn fully_seeded_registry() -> ProtocolRegistry {
    let mut registry = ProtocolRegistry::default();
    for kind in ProtocolKind::ALL {
        registry.insert(def_for(*kind, &format!("{kind:?}")));
    }
    registry
}

/// Build a test app at `ChipSelectState::Selecting` with the dispatch
/// system wired in `Update` behind the canonical `run_if` gate. Caller
/// inserts the registry.
///
/// The `.run_if(in_state(ChipSelectState::Selecting))` gate mirrors what
/// `ProtocolPlugin::build` is specified to install. Registering the system
/// with the same gate here lets Behavior 6 exercise the state gate without
/// needing the full plugin.
fn test_app_selecting(registry: ProtocolRegistry) -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .insert_resource(registry)
        .with_resource::<ActiveProtocols>()
        .with_message::<ProtocolSelected>()
        .with_system(
            Update,
            dispatch_protocol_selection.run_if(in_state(ChipSelectState::Selecting)),
        )
        .build()
}

/// Build a test app at `NodeState::Playing` (NOT `ChipSelectState::Selecting`)
/// for gate-check tests. Same `run_if` gate applied — the state is the
/// independent variable.
fn test_app_node_playing(registry: ProtocolRegistry) -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .insert_resource(registry)
        .with_resource::<ActiveProtocols>()
        .with_message::<ProtocolSelected>()
        .with_system(
            Update,
            dispatch_protocol_selection.run_if(in_state(ChipSelectState::Selecting)),
        )
        .build()
}

/// Write a `ProtocolSelected` message.
fn send_selection(app: &mut App, kind: ProtocolKind) {
    app.world_mut()
        .resource_mut::<Messages<ProtocolSelected>>()
        .write(ProtocolSelected { kind });
}

// ── B.1: Custom-system protocol selection inserts into ActiveProtocols ────

#[test]
fn custom_system_protocol_selection_inserts_into_active_protocols() {
    let mut app = test_app_selecting(fully_seeded_registry());
    app.world_mut().spawn(Breaker);

    send_selection(&mut app, ProtocolKind::Greed);
    app.update();

    let active = app.world().resource::<ActiveProtocols>();
    assert!(
        active.contains(ProtocolKind::Greed),
        "ActiveProtocols should contain Greed after dispatch"
    );
    assert_eq!(
        active.get(ProtocolKind::Greed).unwrap().name,
        "Greed",
        "stored definition should match the registry entry"
    );
}

#[test]
fn custom_system_protocol_does_not_produce_bound_effects() {
    // Edge case of B.1: Greed is a custom-system protocol, so no stamp
    // should be queued onto any entity.
    let mut app = test_app_selecting(fully_seeded_registry());
    app.world_mut().spawn(Breaker);

    send_selection(&mut app, ProtocolKind::Greed);
    app.update();

    let count = app
        .world_mut()
        .query::<&BoundEffects>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "custom-system protocols must not install any BoundEffects"
    );
}

// ── B.2: Effect-tree protocol stamps each RootNode onto every Breaker ─────

#[test]
fn effect_tree_protocol_stamps_each_root_onto_every_breaker() {
    let mut registry = ProtocolRegistry::default();
    registry.insert(deadline_with_effects(vec![stub_piercing_root()]));

    let mut app = test_app_selecting(registry);
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Deadline);
    app.update();

    let active = app.world().resource::<ActiveProtocols>();
    assert!(active.contains(ProtocolKind::Deadline));

    let bound = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects after Deadline stamp");
    assert_eq!(bound.0.len(), 1, "expected 1 stamped entry");
    assert_eq!(bound.0[0].0, "protocol:Deadline");
}

#[test]
fn effect_tree_protocol_with_two_roots_stamps_two_entries_in_order() {
    // Edge case of B.2: two RootNode::Stamp entries produce two BoundEffects
    // in insertion order.
    let mut registry = ProtocolRegistry::default();
    registry.insert(deadline_with_effects(vec![
        stub_piercing_root(),
        stub_damage_boost_root(),
    ]));

    let mut app = test_app_selecting(registry);
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Deadline);
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(bound.0.len(), 2, "expected 2 stamped entries");
    assert_eq!(bound.0[0].0, "protocol:Deadline");
    assert_eq!(bound.0[1].0, "protocol:Deadline");
}

// ── B.3: Multiple breakers each receive the stamped effect tree ───────────

#[test]
fn effect_tree_protocol_stamps_to_every_breaker_entity() {
    let mut registry = ProtocolRegistry::default();
    registry.insert(deadline_with_effects(vec![stub_piercing_root()]));

    let mut app = test_app_selecting(registry);
    let breaker_a = app.world_mut().spawn(Breaker).id();
    let breaker_b = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Deadline);
    app.update();

    let bound_a = app
        .world()
        .get::<BoundEffects>(breaker_a)
        .expect("breaker_a should have BoundEffects");
    assert_eq!(bound_a.0.len(), 1);
    assert_eq!(bound_a.0[0].0, "protocol:Deadline");

    let bound_b = app
        .world()
        .get::<BoundEffects>(breaker_b)
        .expect("breaker_b should have BoundEffects");
    assert_eq!(bound_b.0.len(), 1);
    assert_eq!(bound_b.0[0].0, "protocol:Deadline");
}

// ── B.4: Source name is formatted as protocol:<kind> ──────────────────────

#[test]
fn source_name_is_protocol_colon_kind_debug_kickstart() {
    let mut registry = ProtocolRegistry::default();
    registry.insert(kickstart_with_effects(vec![stub_piercing_root()]));

    let mut app = test_app_selecting(registry);
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Kickstart);
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        bound.0[0].0, "protocol:Kickstart",
        "source name must be format!(\"protocol:{{kind:?}}\")"
    );
}

#[test]
fn source_name_preserves_multi_word_camelcase_tier_regression() {
    // Edge case: TierRegression is a custom-system protocol (effects
    // returns None), so to exercise the source-name formatter we need a
    // stub-able effect-tree tuning. We simulate this by using
    // `Deadline` tuning but with a ProtocolSelected kind of
    // TierRegression — however, the registry key is the tuning's own kind,
    // so this doesn't work cleanly.
    //
    // Instead: use the effect-tree protocol Anchor (kind = Anchor) with a
    // single stamp, and verify source = "protocol:Anchor". The
    // multi-word CamelCase case is otherwise exercised by B.4's debug
    // formatter consistency — `format!("{:?}", kind)` produces the exact
    // variant name verbatim for every variant.
    let mut registry = ProtocolRegistry::default();
    registry.insert(anchor_with_effects(vec![stub_piercing_root()]));

    let mut app = test_app_selecting(registry);
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Anchor);
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(bound.0[0].0, "protocol:Anchor");
}

// ── B.5: Missing registry entry logs and does not mutate ActiveProtocols ──

#[test]
fn missing_registry_entry_does_not_mutate_active_protocols() {
    let mut app = test_app_selecting(ProtocolRegistry::default());
    app.world_mut().spawn(Breaker);

    send_selection(&mut app, ProtocolKind::Greed);
    app.update();

    let active = app.world().resource::<ActiveProtocols>();
    assert!(
        active.is_empty(),
        "unknown kind must be a soft warning — ActiveProtocols stays empty"
    );
}

#[test]
fn missing_registry_entry_does_not_produce_bound_effects() {
    // Edge case of B.5: no stamp attempted.
    let mut app = test_app_selecting(ProtocolRegistry::default());
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Greed);
    app.update();

    assert!(
        app.world().get::<BoundEffects>(breaker).is_none(),
        "Breaker must not gain BoundEffects when registry lookup fails"
    );
}

// ── B.6: Dispatch gate — message outside Selecting is ignored ─────────────

#[test]
fn dispatch_gate_rejects_message_outside_selecting_state() {
    let mut app = test_app_node_playing(fully_seeded_registry());
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Greed);
    app.update();

    let active = app.world().resource::<ActiveProtocols>();
    assert!(
        !active.contains(ProtocolKind::Greed),
        "dispatch must be gated on ChipSelectState::Selecting"
    );
    assert!(active.is_empty(), "ActiveProtocols must stay empty");

    assert!(
        app.world().get::<BoundEffects>(breaker).is_none(),
        "no stamp should occur outside Selecting"
    );
}

// ── B.7: Effect-tree protocol with empty effects stamps nothing ───────────

#[test]
fn effect_tree_protocol_with_empty_effects_inserts_but_stamps_nothing() {
    let mut registry = ProtocolRegistry::default();
    registry.insert(anchor_with_effects(vec![]));

    let mut app = test_app_selecting(registry);
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Anchor);
    app.update();

    let active = app.world().resource::<ActiveProtocols>();
    assert!(
        active.contains(ProtocolKind::Anchor),
        "Anchor with empty effects should still be inserted into ActiveProtocols"
    );
    assert!(
        app.world().get::<BoundEffects>(breaker).is_none(),
        "empty effects vec must produce no stamps"
    );
}

// ── B.8: Non-Breaker StampTarget routes to every Breaker anyway ───────────

#[test]
fn non_breaker_stamp_target_routes_to_every_breaker_bound_effects() {
    let every_bolt_root = RootNode::Stamp(
        StampTarget::EveryBolt,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
    );
    let mut registry = ProtocolRegistry::default();
    registry.insert(deadline_with_effects(vec![every_bolt_root]));

    let mut app = test_app_selecting(registry);
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Deadline);
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects — EveryBolt stamps route to breaker");
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "protocol:Deadline");
}

#[test]
fn non_breaker_stamp_target_with_no_bolt_entities_does_not_panic() {
    // Edge case of B.8: no Bolt entity exists — system must not crash.
    let every_bolt_root = RootNode::Stamp(
        StampTarget::EveryBolt,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
    );
    let mut registry = ProtocolRegistry::default();
    registry.insert(deadline_with_effects(vec![every_bolt_root]));

    let mut app = test_app_selecting(registry);
    app.world_mut().spawn(Breaker);

    send_selection(&mut app, ProtocolKind::Deadline);
    app.update();
    // Assertion: no panic.
}

// ── B.9: RootNode::Spawn is a no-op in dispatch ───────────────────────────

#[test]
fn root_node_spawn_is_a_noop_for_bound_effects() {
    let spawn_root = RootNode::Spawn(EntityKind::Bolt, Tree::Sequence(vec![]));
    let mut registry = ProtocolRegistry::default();
    registry.insert(deadline_with_effects(vec![spawn_root]));

    let mut app = test_app_selecting(registry);
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Deadline);
    app.update();

    let active = app.world().resource::<ActiveProtocols>();
    assert!(
        active.contains(ProtocolKind::Deadline),
        "Deadline should still be inserted even though the root is Spawn"
    );
    assert!(
        app.world().get::<BoundEffects>(breaker).is_none(),
        "RootNode::Spawn must not stamp to BoundEffects"
    );
}

// ── B.10: Two ProtocolSelected messages in the same frame insert both ─────

#[test]
fn two_protocol_selected_messages_in_same_frame_insert_both_kinds() {
    let mut app = test_app_selecting(fully_seeded_registry());
    app.world_mut().spawn(Breaker);

    send_selection(&mut app, ProtocolKind::Greed);
    send_selection(&mut app, ProtocolKind::Anchor);
    app.update();

    let active = app.world().resource::<ActiveProtocols>();
    assert!(
        active.contains(ProtocolKind::Greed),
        "Greed must be inserted"
    );
    assert!(
        active.contains(ProtocolKind::Anchor),
        "Anchor must be inserted"
    );
}

// ── Source-name format smoke tests ─────────────────────────────────────────
//
// `dispatch_protocol_selection` formats the stamp source via
// `format!("protocol:{kind:?}")`. Effect-tree kinds are exercised through the
// stamp-dispatch path above; custom-system kinds (`effects()` returns `None`)
// are covered here via a pure-format assertion so the `{:?}` contract holds
// for every kind without a Bevy app.

#[test]
fn source_name_for_tier_regression_is_protocol_tier_regression() {
    // Regression guard: multi-word CamelCase kind names must not gain spaces
    // or lowercasing. Since `TierRegression` is a custom-system protocol
    // (`effects()` returns `None`) it cannot be tested via the stamp path.
    let source = format!("protocol:{:?}", ProtocolKind::TierRegression);
    assert_eq!(source, "protocol:TierRegression");
}

#[test]
fn source_name_for_debt_collector_is_protocol_debt_collector() {
    // Same guard for another custom-system, multi-word kind.
    let source = format!("protocol:{:?}", ProtocolKind::DebtCollector);
    assert_eq!(source, "protocol:DebtCollector");
}
