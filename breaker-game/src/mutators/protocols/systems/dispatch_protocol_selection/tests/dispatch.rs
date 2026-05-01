//! Dispatch behavior tests — B.1, B.2, B.3, B.7, B.8, B.9, B.10.

use super::helpers::{
    anchor_with_effects, deadline_with_effects, fully_seeded_registry, send_selection,
    stub_damage_boost_root, stub_piercing_root, test_app_selecting,
};
use crate::{
    effect_v3::{
        effects::PiercingConfig,
        types::{EffectType, EntityKind, RootNode, StampTarget, Tree},
    },
    mutators::protocols::{
        definition::ProtocolKind,
        resources::{ActiveProtocols, ProtocolRegistry},
    },
    prelude::*,
};

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
