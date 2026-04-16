//! Tests for `check_portal_entry` — behaviors 9-12.
//!
//! The system reads `BoltImpactCell` messages, checks if the impacted cell
//! has `PortalCell`, and writes `PortalEntered`.

use bevy::prelude::*;

use super::system::check_portal_entry;
use crate::{
    cells::{
        behaviors::portal::components::{PortalCell, PortalConfig},
        messages::PortalEntered,
    },
    prelude::*,
};

// ── Pending message injection ─────────────────────────────────────────────

/// Seeded `BoltImpactCell` messages drained into the queue before
/// `check_portal_entry` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
struct PendingBoltImpacts(Vec<BoltImpactCell>);

/// Drains `PendingBoltImpacts` into the `BoltImpactCell` message queue.
fn enqueue_bolt_impacts(
    mut pending: ResMut<PendingBoltImpacts>,
    mut writer: MessageWriter<BoltImpactCell>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

/// Pushes a `BoltImpactCell` into the per-tick pending queue.
fn push_bolt_impact(app: &mut App, msg: BoltImpactCell) {
    app.world_mut()
        .resource_mut::<PendingBoltImpacts>()
        .0
        .push(msg);
}

/// Constructs a `BoltImpactCell` message with default normal and zero piercing.
fn bolt_impact(bolt: Entity, cell: Entity) -> BoltImpactCell {
    BoltImpactCell {
        cell,
        bolt,
        impact_normal: Vec2::Y,
        piercing_remaining: 0,
    }
}

// ── Test app builder ──────────────────────────────────────────────────────

fn build_check_portal_entry_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_message::<BoltImpactCell>()
        .with_message_capture::<PortalEntered>()
        .with_system(FixedUpdate, enqueue_bolt_impacts.before(check_portal_entry))
        .with_system(FixedUpdate, check_portal_entry)
        .build();
    app.init_resource::<PendingBoltImpacts>();
    app
}

/// Spawns a minimal portal cell entity with `PortalCell` and `PortalConfig`.
fn spawn_portal_cell(app: &mut App, tier_offset: i32) -> Entity {
    app.world_mut()
        .spawn((Cell, PortalCell, PortalConfig { tier_offset }))
        .id()
}

/// Spawns a minimal non-portal cell entity.
fn spawn_plain_cell(app: &mut App) -> Entity {
    app.world_mut().spawn(Cell).id()
}

/// Spawns a minimal bolt entity.
fn spawn_bolt(app: &mut App) -> Entity {
    app.world_mut().spawn(Bolt).id()
}

// ── Behavior 9: Bolt hitting portal cell emits PortalEntered ──────────────

#[test]
fn bolt_hitting_portal_cell_emits_portal_entered() {
    let mut app = build_check_portal_entry_app();
    let portal = spawn_portal_cell(&mut app, 2);
    let bolt = spawn_bolt(&mut app);

    push_bolt_impact(&mut app, bolt_impact(bolt, portal));
    tick(&mut app);

    let collected = app.world().resource::<MessageCollector<PortalEntered>>();
    assert_eq!(
        collected.0.len(),
        1,
        "expected exactly 1 PortalEntered message, got {}",
        collected.0.len()
    );
    assert_eq!(
        collected.0[0].portal, portal,
        "PortalEntered.portal should be the portal cell entity"
    );
    assert_eq!(
        collected.0[0].bolt, bolt,
        "PortalEntered.bolt should be the bolt entity"
    );
}

// ── Behavior 10: Bolt hitting non-portal cell does NOT emit PortalEntered ─
//
// Uses a companion portal cell impact to prove the system ran (otherwise the
// no-op stub trivially passes by writing nothing).

#[test]
fn bolt_hitting_non_portal_cell_does_not_emit_portal_entered() {
    let mut app = build_check_portal_entry_app();
    let plain_cell = spawn_plain_cell(&mut app);
    let portal_cell = spawn_portal_cell(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);

    // Two impacts: one on a plain cell, one on a portal cell.
    push_bolt_impact(&mut app, bolt_impact(bolt_a, plain_cell));
    push_bolt_impact(&mut app, bolt_impact(bolt_b, portal_cell));
    tick(&mut app);

    let collected = app.world().resource::<MessageCollector<PortalEntered>>();
    // The portal cell impact MUST produce a PortalEntered (proves the system ran).
    // The plain cell impact must NOT.
    assert_eq!(
        collected.0.len(),
        1,
        "only the portal cell impact should emit PortalEntered, got {}",
        collected.0.len()
    );
    assert_eq!(
        collected.0[0].portal, portal_cell,
        "the single PortalEntered should reference the portal cell, not the plain cell"
    );
}

// ── Behavior 11: Multiple portal cells hit each emit PortalEntered ────────

#[test]
fn multiple_portal_cells_hit_each_emit_portal_entered() {
    let mut app = build_check_portal_entry_app();
    let portal_a = spawn_portal_cell(&mut app, 1);
    let portal_b = spawn_portal_cell(&mut app, -1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);

    push_bolt_impact(&mut app, bolt_impact(bolt_a, portal_a));
    push_bolt_impact(&mut app, bolt_impact(bolt_b, portal_b));
    tick(&mut app);

    let collected = app.world().resource::<MessageCollector<PortalEntered>>();
    assert_eq!(
        collected.0.len(),
        2,
        "expected 2 PortalEntered messages (one per portal), got {}",
        collected.0.len()
    );

    let portals: Vec<Entity> = collected.0.iter().map(|m| m.portal).collect();
    assert!(
        portals.contains(&portal_a),
        "PortalEntered should include portal_a"
    );
    assert!(
        portals.contains(&portal_b),
        "PortalEntered should include portal_b"
    );
}

// ── Behavior 12: Missing cell entity does not panic ───────────────────────
//
// Uses a companion portal cell impact to prove the system ran and processes
// all messages in the batch (otherwise the no-op stub trivially passes).

#[test]
fn missing_cell_entity_does_not_panic() {
    let mut app = build_check_portal_entry_app();
    let portal = spawn_portal_cell(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);

    // Two impacts: one referencing a missing entity, one on a real portal.
    push_bolt_impact(&mut app, bolt_impact(bolt_a, Entity::PLACEHOLDER));
    push_bolt_impact(&mut app, bolt_impact(bolt_b, portal));
    tick(&mut app);

    let collected = app.world().resource::<MessageCollector<PortalEntered>>();
    // The real portal impact MUST produce a PortalEntered (proves system ran
    // and processed both messages without panicking on the missing entity).
    assert_eq!(
        collected.0.len(),
        1,
        "only the real portal impact should produce PortalEntered, got {}",
        collected.0.len()
    );
    assert_eq!(
        collected.0[0].portal, portal,
        "the PortalEntered should reference the real portal cell"
    );
}
