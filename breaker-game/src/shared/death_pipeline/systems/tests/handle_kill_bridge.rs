//! `effect_v3` bridge-integration tests (Behaviors 15-16).
//!
//! These tests verify that `handle_kill<T>` → `Destroyed<T>` → `effect_v3` death
//! bridge dispatch is wired end-to-end. `handle_kill<T>` writes `Destroyed<T>`
//! in `DeathPipelineSystems::HandleKill`, and the death bridge runs immediately
//! after via `.after(DeathPipelineSystems::HandleKill)` within the same tick,
//! dispatching triggers in the same fixed update as the kill.
//!
//! Because the victim is despawned by `process_despawn_requests` in
//! `FixedPostUpdate` on the same tick, observable state must live on a
//! **separate listener entity** that survives despawn. The listener has a
//! `BoundEffects` tree binding `DeathOccurred(EntityKind::Any)` →
//! `Tree::Fire(EffectType::SpeedBoost(..))`, and the observable is an
//! `EffectStack<SpeedBoostConfig>` on that listener.

use std::marker::PhantomData;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{
    PendingBoltKills, PendingCellKills, attach_bolt_destroyed_collector,
    attach_cell_destroyed_collector, build_plugin_integration_app, enqueue_bolt_kills,
    enqueue_cell_kills,
};
use crate::{
    bolt::components::Bolt,
    cells::components::Cell,
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        types::{EffectType, EntityKind, Tree, Trigger},
    },
    shared::{
        death_pipeline::{
            destroyed::Destroyed, hp::Hp, kill_yourself::KillYourself, killed_by::KilledBy,
            sets::DeathPipelineSystems,
        },
        test_utils::{MessageCollector, tick},
    },
};

/// Builds a `(String, Tree)` entry for a `When(trigger, Fire(SpeedBoost))` tree.
fn death_speed_tree(name: &str, trigger: Trigger, multiplier: f32) -> (String, Tree) {
    (
        name.to_string(),
        Tree::When(
            trigger,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(multiplier),
            }))),
        ),
    )
}

// ── Behavior 15: Cell death → effect_v3 bridge handoff ─────────────────

#[test]
fn bridge_handoff_cell_death_fires_death_occurred_any_on_listener() {
    let mut app = build_plugin_integration_app();
    attach_cell_destroyed_collector(&mut app);
    app.init_resource::<PendingCellKills>();
    app.add_systems(
        FixedUpdate,
        enqueue_cell_kills.before(DeathPipelineSystems::HandleKill),
    );

    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(1.0),
            KilledBy::default(),
            Position2D(Vec2::new(100.0, 200.0)),
        ))
        .id();

    let listener = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Any),
            1.5,
        )]))
        .id();

    app.insert_resource(PendingCellKills(vec![KillYourself::<Cell> {
        victim:  cell,
        killer:  None,
        _marker: PhantomData,
    }]));

    // Tick 1: handle_kill<Cell> runs and writes Destroyed<Cell>; the death
    // bridge runs same-tick via .after(HandleKill) and dispatches
    // DeathOccurred(Any) to the listener; process_despawn_requests runs in
    // FixedPostUpdate and despawns the cell.
    tick(&mut app);

    // handle_kill<Cell> should have produced exactly one Destroyed<Cell>.
    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "handle_kill<Cell> should emit exactly one Destroyed<Cell>"
    );
    assert_eq!(destroyed.0[0].victim, cell);
    assert_eq!(destroyed.0[0].victim_pos, Vec2::new(100.0, 200.0));

    // Clear pending so the second tick doesn't re-enqueue.
    app.insert_resource(PendingCellKills(vec![]));

    // Tick 2: extra tick for historical safety — under the current schedule
    // the bridge has already dispatched during tick 1, but a second tick
    // confirms no retroactive state changes.
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(listener)
        .expect("listener should have EffectStack<SpeedBoostConfig> after bridge dispatch");
    assert_eq!(
        stack.len(),
        1,
        "listener should have exactly one SpeedBoost entry"
    );
}

#[test]
fn bridge_handoff_cell_death_dispatch_not_gated_on_killer_presence() {
    // Edge case: inject the KillYourself<Cell> message with a killer —
    // the listener still receives DeathOccurred(Any) regardless of killer presence.
    let mut app = build_plugin_integration_app();
    attach_cell_destroyed_collector(&mut app);
    app.init_resource::<PendingCellKills>();
    app.add_systems(
        FixedUpdate,
        enqueue_cell_kills.before(DeathPipelineSystems::HandleKill),
    );

    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(1.0),
            KilledBy::default(),
            Position2D(Vec2::new(100.0, 200.0)),
        ))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 50.0)))
        .id();

    let listener = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Any),
            1.5,
        )]))
        .id();

    app.insert_resource(PendingCellKills(vec![KillYourself::<Cell> {
        victim:  cell,
        killer:  Some(killer),
        _marker: PhantomData,
    }]));

    tick(&mut app);
    app.insert_resource(PendingCellKills(vec![]));
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(listener)
        .expect("listener should receive dispatch regardless of killer presence");
    assert_eq!(stack.len(), 1);
}

// ── Behavior 16: Bolt death → effect_v3 bridge handoff ─────────────────

#[test]
fn bridge_handoff_bolt_death_fires_death_occurred_any_on_listener() {
    let mut app = build_plugin_integration_app();
    attach_bolt_destroyed_collector(&mut app);
    app.init_resource::<PendingBoltKills>();
    app.add_systems(
        FixedUpdate,
        enqueue_bolt_kills.before(DeathPipelineSystems::HandleKill),
    );

    // Bolt does NOT have #[require(Spatial2D)], so Position2D MUST be explicit.
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Hp::new(1.0),
            KilledBy::default(),
            Position2D(Vec2::new(0.0, -50.0)),
        ))
        .id();

    let listener = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Any),
            2.0,
        )]))
        .id();

    app.insert_resource(PendingBoltKills(vec![KillYourself::<Bolt> {
        victim:  bolt,
        killer:  None,
        _marker: PhantomData,
    }]));

    // Tick 1: handle_kill<Bolt> emits Destroyed<Bolt>; the death bridge runs
    // same-tick via .after(HandleKill) and dispatches DeathOccurred(Any);
    // process_despawn despawns bolt.
    tick(&mut app);

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Bolt>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "handle_kill<Bolt> should emit exactly one Destroyed<Bolt>"
    );
    assert_eq!(destroyed.0[0].victim, bolt);
    assert_eq!(destroyed.0[0].victim_pos, Vec2::new(0.0, -50.0));

    app.insert_resource(PendingBoltKills(vec![]));
    // Tick 2: extra tick for historical safety — the bridge has already
    // dispatched during tick 1 under the current schedule.
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(listener)
        .expect("listener should have EffectStack<SpeedBoostConfig> after bolt bridge dispatch");
    assert_eq!(stack.len(), 1);
}

#[test]
fn bridge_handoff_bolt_death_late_listener_does_not_retroactively_dispatch() {
    // Edge case: listener is spawned AFTER the bolt has already died and the
    // bridge has already dispatched. Under the same-tick schedule
    // (bridge .after(HandleKill)), the bridge runs during tick 1 — the tick
    // when the bolt dies. Any listener that appears only on tick 2 MUST NOT
    // retroactively receive that dispatch, because the Destroyed<Bolt>
    // message was already consumed on tick 1.
    let mut app = build_plugin_integration_app();
    attach_bolt_destroyed_collector(&mut app);
    app.init_resource::<PendingBoltKills>();
    app.add_systems(
        FixedUpdate,
        enqueue_bolt_kills.before(DeathPipelineSystems::HandleKill),
    );

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Hp::new(1.0),
            KilledBy::default(),
            Position2D(Vec2::new(0.0, -50.0)),
        ))
        .id();

    app.insert_resource(PendingBoltKills(vec![KillYourself::<Bolt> {
        victim:  bolt,
        killer:  None,
        _marker: PhantomData,
    }]));

    // Tick 1: handle_kill<Bolt> runs, Destroyed<Bolt> is written, the bridge
    // runs same-tick and sees no listener entities, so nothing is dispatched.
    // No listener has been spawned yet.
    tick(&mut app);
    app.insert_resource(PendingBoltKills(vec![]));

    // Spawn the listener AFTER the dispatch tick has already completed.
    let listener = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Any),
            2.0,
        )]))
        .id();

    // Tick 2: Destroyed<Bolt> has already been consumed on tick 1; the bridge
    // has nothing new to dispatch. The late-joining listener must NOT
    // retroactively receive dispatch.
    tick(&mut app);

    // The listener MUST NOT have received any SpeedBoost effect — either the
    // component is absent entirely (nothing was dispatched to create it) or,
    // if present, the stack is empty.
    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(listener);
    match stack {
        None => {
            // Nothing was dispatched — component was never created. This is
            // the expected shape: no retroactive dispatch.
        }
        Some(stack) => {
            assert_eq!(
                stack.len(),
                0,
                "late-joining listener must not retroactively receive dispatch — stack should be empty, got {}",
                stack.len()
            );
        }
    }
}
