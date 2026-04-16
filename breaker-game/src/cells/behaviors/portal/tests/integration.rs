//! Integration test — behavior 18.
//!
//! Verifies that portal cells with `RequiredToClear` block node completion
//! until the full portal pipeline (`check_portal_entry` -> `handle_portal_entered`
//! -> `handle_portal_completed`) destroys them via `KillYourself<Cell>`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::{
        behaviors::portal::{
            components::{PortalCell, PortalConfig},
            systems::{check_portal_entry, handle_portal_completed, handle_portal_entered},
        },
        components::RequiredToClear,
        messages::{PortalCompleted, PortalEntered},
    },
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
    state::run::node::{
        ClearRemainingCount, messages::NodeCleared, systems::track_node_completion,
    },
};

// ── Pending message injection ─────────────────────────────────────────────

/// Seeded `BoltImpactCell` messages drained into the queue before
/// `check_portal_entry` runs. One-shot per `tick()`.
#[derive(Resource, Default)]
struct PendingBoltImpacts(Vec<BoltImpactCell>);

fn enqueue_bolt_impacts(
    mut pending: ResMut<PendingBoltImpacts>,
    mut writer: MessageWriter<BoltImpactCell>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

fn push_bolt_impact(app: &mut App, msg: BoltImpactCell) {
    app.world_mut()
        .resource_mut::<PendingBoltImpacts>()
        .0
        .push(msg);
}

/// Seeded `Destroyed<Cell>` messages for regular (non-portal) cell kills
/// injected directly to feed `track_node_completion`.
#[derive(Resource, Default)]
struct PendingCellDestroyed(Vec<Destroyed<Cell>>);

fn enqueue_cell_destroyed(
    mut pending: ResMut<PendingCellDestroyed>,
    mut writer: MessageWriter<Destroyed<Cell>>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

fn push_cell_destroyed(app: &mut App, msg: Destroyed<Cell>) {
    app.world_mut()
        .resource_mut::<PendingCellDestroyed>()
        .0
        .push(msg);
}

fn make_destroyed(victim: Entity) -> Destroyed<Cell> {
    Destroyed {
        victim,
        killer: None,
        victim_pos: Vec2::ZERO,
        killer_pos: None,
        _marker: PhantomData,
    }
}

fn bolt_impact(bolt: Entity, cell: Entity) -> BoltImpactCell {
    BoltImpactCell {
        cell,
        bolt,
        impact_normal: Vec2::Y,
        piercing_remaining: 0,
    }
}

// ── Test app builder ──────────────────────────────────────────────────────

fn build_integration_app() -> App {
    let mut app = TestAppBuilder::new().with_effects_pipeline().build();

    // Register portal messages
    app.add_message::<PortalEntered>();
    app.add_message::<PortalCompleted>();

    // Register capture for NodeCleared
    attach_message_capture::<NodeCleared>(&mut app);

    // Inject resources
    app.init_resource::<PendingBoltImpacts>();
    app.init_resource::<PendingCellDestroyed>();

    // Wire portal pipeline systems in FixedUpdate, chained:
    // enqueue_bolt_impacts -> check_portal_entry -> handle_portal_entered
    //   -> handle_portal_completed (before death pipeline applies the KillYourself)
    app.add_systems(
        FixedUpdate,
        (
            enqueue_bolt_impacts,
            check_portal_entry,
            handle_portal_entered,
            handle_portal_completed,
        )
            .chain()
            .before(DeathPipelineSystems::ApplyDamage),
    );

    // Wire node completion tracking: reads Destroyed<Cell>, decrements count.
    // Must run after HandleKill so it sees Destroyed<Cell> from the death pipeline.
    app.add_systems(
        FixedUpdate,
        (
            enqueue_cell_destroyed.before(track_node_completion),
            track_node_completion.after(DeathPipelineSystems::HandleKill),
        ),
    );

    app
}

// ── Behavior 18: Portal cell with RequiredToClear blocks node completion ──

#[test]
fn portal_cell_with_required_to_clear_blocks_node_completion() {
    let mut app = build_integration_app();

    // Two RequiredToClear cells: one regular, one portal.
    let regular_cell = app
        .world_mut()
        .spawn((Cell, RequiredToClear, Hp::new(20.0), KilledBy::default()))
        .id();
    let portal_cell = app
        .world_mut()
        .spawn((
            Cell,
            PortalCell,
            PortalConfig { tier_offset: 1 },
            RequiredToClear,
            Hp::new(100.0),
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();
    let bolt = app.world_mut().spawn(Bolt).id();

    app.insert_resource(ClearRemainingCount { remaining: 2 });

    // Step 1: Destroy the regular cell — ClearRemainingCount should go to 1,
    // NodeCleared should NOT be emitted.
    push_cell_destroyed(&mut app, make_destroyed(regular_cell));
    tick(&mut app);

    let remaining = app.world().resource::<ClearRemainingCount>();
    assert_eq!(
        remaining.remaining, 1,
        "after regular cell destroyed, ClearRemainingCount should be 1, got {}",
        remaining.remaining
    );
    let cleared = app.world().resource::<MessageCollector<NodeCleared>>();
    assert!(
        cleared.0.is_empty(),
        "NodeCleared should NOT be emitted with 1 cell remaining"
    );

    // Step 2: Bolt hits portal cell — the full pipeline should emit
    // PortalEntered -> PortalCompleted -> KillYourself<Cell>,
    // which flows through the death pipeline producing Destroyed<Cell>,
    // which track_node_completion reads to decrement remaining to 0.
    // All systems run within one FixedUpdate tick when correctly ordered.
    push_bolt_impact(&mut app, bolt_impact(bolt, portal_cell));
    tick(&mut app);

    let remaining = app.world().resource::<ClearRemainingCount>();
    assert_eq!(
        remaining.remaining, 0,
        "after portal cell pipeline completes, ClearRemainingCount should be 0, got {}",
        remaining.remaining
    );
    let cleared = app.world().resource::<MessageCollector<NodeCleared>>();
    assert!(
        !cleared.0.is_empty(),
        "NodeCleared should be emitted when ClearRemainingCount reaches 0"
    );
}
