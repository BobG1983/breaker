//! Tests for `handle_portal_entered` — behaviors 13-14.
//!
//! The mock handler reads `PortalEntered` messages and immediately writes
//! `PortalCompleted`. Will be replaced with real sub-node logic in the
//! node refactor.

use bevy::prelude::*;

use super::system::handle_portal_entered;
use crate::{
    cells::messages::{PortalCompleted, PortalEntered},
    prelude::*,
};

// ── Pending message injection ─────────────────────────────────────────────

/// Seeded `PortalEntered` messages drained into the queue before
/// `handle_portal_entered` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
struct PendingPortalEntered(Vec<PortalEntered>);

/// Drains `PendingPortalEntered` into the `PortalEntered` message queue.
fn enqueue_portal_entered(
    mut pending: ResMut<PendingPortalEntered>,
    mut writer: MessageWriter<PortalEntered>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

/// Pushes a `PortalEntered` into the per-tick pending queue.
fn push_portal_entered(app: &mut App, msg: PortalEntered) {
    app.world_mut()
        .resource_mut::<PendingPortalEntered>()
        .0
        .push(msg);
}

// ── Test app builder ──────────────────────────────────────────────────────

fn build_handle_portal_entered_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_message::<PortalEntered>()
        .with_message_capture::<PortalCompleted>()
        .with_system(
            FixedUpdate,
            enqueue_portal_entered.before(handle_portal_entered),
        )
        .with_system(FixedUpdate, handle_portal_entered)
        .build();
    app.init_resource::<PendingPortalEntered>();
    app
}

// ── Behavior 13: Mock handler converts PortalEntered to PortalCompleted ───

#[test]
fn portal_entered_produces_portal_completed() {
    let mut app = build_handle_portal_entered_app();

    let entity_a = app.world_mut().spawn_empty().id();
    let entity_b = app.world_mut().spawn_empty().id();

    push_portal_entered(
        &mut app,
        PortalEntered {
            portal: entity_a,
            bolt:   entity_b,
        },
    );
    tick(&mut app);

    let collected = app.world().resource::<MessageCollector<PortalCompleted>>();
    assert_eq!(
        collected.0.len(),
        1,
        "expected exactly 1 PortalCompleted message, got {}",
        collected.0.len()
    );
    assert_eq!(
        collected.0[0].portal, entity_a,
        "PortalCompleted.portal should match the portal from PortalEntered"
    );
}

// ── Behavior 14: Multiple PortalEntered each produce PortalCompleted ──────

#[test]
fn multiple_portal_entered_each_produce_portal_completed() {
    let mut app = build_handle_portal_entered_app();

    let portal_a = app.world_mut().spawn_empty().id();
    let portal_b = app.world_mut().spawn_empty().id();
    let bolt = app.world_mut().spawn_empty().id();

    push_portal_entered(
        &mut app,
        PortalEntered {
            portal: portal_a,
            bolt,
        },
    );
    push_portal_entered(
        &mut app,
        PortalEntered {
            portal: portal_b,
            bolt,
        },
    );
    tick(&mut app);

    let collected = app.world().resource::<MessageCollector<PortalCompleted>>();
    assert_eq!(
        collected.0.len(),
        2,
        "expected 2 PortalCompleted messages, got {}",
        collected.0.len()
    );

    let portals: Vec<Entity> = collected.0.iter().map(|m| m.portal).collect();
    assert!(
        portals.contains(&portal_a),
        "PortalCompleted should include portal_a"
    );
    assert!(
        portals.contains(&portal_b),
        "PortalCompleted should include portal_b"
    );
}
