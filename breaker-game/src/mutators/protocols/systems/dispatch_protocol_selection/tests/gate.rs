//! Gate tests — B.5 (missing registry entry) and B.6 (state gate).

use super::helpers::{
    fully_seeded_registry, send_selection, test_app_node_playing, test_app_selecting,
};
use crate::{
    mutators::protocols::{
        definition::ProtocolKind,
        resources::{ActiveProtocols, ProtocolRegistry},
    },
    prelude::*,
};

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
