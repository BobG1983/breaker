//! Tests for `handle_portal_completed` — behaviors 15-17.
//!
//! The system reads `PortalCompleted` messages and kills the portal cell
//! by writing `KillYourself<Cell>`.

use bevy::prelude::*;

use super::system::handle_portal_completed;
use crate::{
    cells::{behaviors::portal::components::PortalCell, messages::PortalCompleted},
    prelude::*,
    shared::death_pipeline::kill_yourself::KillYourself,
};

// ── Pending message injection ─────────────────────────────────────────────

/// Seeded `PortalCompleted` messages drained into the queue before
/// `handle_portal_completed` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
struct PendingPortalCompleted(Vec<PortalCompleted>);

/// Drains `PendingPortalCompleted` into the `PortalCompleted` message queue.
fn enqueue_portal_completed(
    mut pending: ResMut<PendingPortalCompleted>,
    mut writer: MessageWriter<PortalCompleted>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

/// Pushes a `PortalCompleted` into the per-tick pending queue.
fn push_portal_completed(app: &mut App, msg: PortalCompleted) {
    app.world_mut()
        .resource_mut::<PendingPortalCompleted>()
        .0
        .push(msg);
}

// ── Test app builder ──────────────────────────────────────────────────────

fn build_handle_portal_completed_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_message::<PortalCompleted>()
        .with_message_capture::<KillYourself<Cell>>()
        .with_system(
            FixedUpdate,
            enqueue_portal_completed.before(handle_portal_completed),
        )
        .with_system(FixedUpdate, handle_portal_completed)
        .build();
    app.init_resource::<PendingPortalCompleted>();
    app
}

// ── Behavior 15: PortalCompleted kills the portal cell via KillYourself ───

#[test]
fn portal_completed_emits_kill_yourself_for_portal_cell() {
    let mut app = build_handle_portal_completed_app();

    let portal = app
        .world_mut()
        .spawn((Cell, PortalCell, Hp::new(100.0), KilledBy::default()))
        .id();

    push_portal_completed(&mut app, PortalCompleted { portal });
    tick(&mut app);

    let collected = app
        .world()
        .resource::<MessageCollector<KillYourself<Cell>>>();
    assert_eq!(
        collected.0.len(),
        1,
        "expected exactly 1 KillYourself<Cell> message, got {}",
        collected.0.len()
    );
    assert_eq!(
        collected.0[0].victim, portal,
        "KillYourself<Cell>.victim should be the portal entity"
    );
    assert_eq!(
        collected.0[0].killer, None,
        "KillYourself<Cell>.killer should be None (environmental kill)"
    );
}

// ── Behavior 16: PortalCompleted for already-dead cell does not panic ─────
//
// Uses a companion live portal to prove the system ran and processed all
// messages (otherwise the no-op stub trivially passes).

#[test]
fn portal_completed_for_dead_cell_does_not_panic() {
    let mut app = build_handle_portal_completed_app();

    let dead_portal = app
        .world_mut()
        .spawn((Cell, PortalCell, Hp::new(100.0), KilledBy::default(), Dead))
        .id();
    let live_portal = app
        .world_mut()
        .spawn((Cell, PortalCell, Hp::new(100.0), KilledBy::default()))
        .id();

    push_portal_completed(
        &mut app,
        PortalCompleted {
            portal: dead_portal,
        },
    );
    push_portal_completed(
        &mut app,
        PortalCompleted {
            portal: live_portal,
        },
    );
    tick(&mut app);

    // The live portal MUST produce a KillYourself<Cell> (proves system ran
    // and processed both messages without panicking on the dead entity).
    let collected = app
        .world()
        .resource::<MessageCollector<KillYourself<Cell>>>();
    assert!(
        collected.0.iter().any(|m| m.victim == live_portal),
        "live portal should produce KillYourself<Cell>, proves system ran"
    );
}

// ── Behavior 17: PortalCompleted for missing entity does not panic ────────
//
// Uses a companion real portal to prove the system ran.

#[test]
fn portal_completed_for_missing_entity_does_not_panic() {
    let mut app = build_handle_portal_completed_app();

    let real_portal = app
        .world_mut()
        .spawn((Cell, PortalCell, Hp::new(100.0), KilledBy::default()))
        .id();

    push_portal_completed(
        &mut app,
        PortalCompleted {
            portal: Entity::PLACEHOLDER,
        },
    );
    push_portal_completed(
        &mut app,
        PortalCompleted {
            portal: real_portal,
        },
    );
    tick(&mut app);

    // The real portal MUST produce a KillYourself<Cell> (proves system ran
    // and processed both messages without panicking on the missing entity).
    let collected = app
        .world()
        .resource::<MessageCollector<KillYourself<Cell>>>();
    assert!(
        collected.0.iter().any(|m| m.victim == real_portal),
        "real portal should produce KillYourself<Cell>, proves system ran"
    );
}
