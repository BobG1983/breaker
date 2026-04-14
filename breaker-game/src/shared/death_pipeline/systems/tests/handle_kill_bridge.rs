//! `effect_v3` bridge-integration tests (Behaviors 15-16).
//!
//! These tests verify that `handle_kill<T>` → `Destroyed<T>` → `effect_v3` death
//! bridge dispatch is wired end-to-end. Uses the 2-tick pattern due to the
//! known 1-frame ordering lag (D12): tick 1 runs `handle_kill<T>` which writes
//! `Destroyed<T>`; tick 2 runs `EffectV3Systems::Bridge` which reads the
//! message and dispatches triggers.
//!
//! Because the victim is despawned by `process_despawn_requests` in
//! `FixedPostUpdate` on tick 1, observable state must live on a **separate
//! listener entity** that survives across ticks. The listener has a
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

    // Tick 1: handle_kill<Cell> runs and writes Destroyed<Cell>;
    // process_despawn_requests runs in FixedPostUpdate and despawns the cell.
    tick(&mut app);

    // Assert that handle_kill<Cell> produced exactly one Destroyed<Cell> on tick 1.
    // This is the degradation fallback — it proves the handle_kill → Destroyed
    // handoff works even if the bridge observable fails on tick 2.
    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "tick 1: handle_kill<Cell> should emit exactly one Destroyed<Cell>"
    );
    assert_eq!(destroyed.0[0].victim, cell);
    assert_eq!(destroyed.0[0].victim_pos, Vec2::new(100.0, 200.0));

    // Clear pending so the second tick doesn't re-enqueue.
    app.insert_resource(PendingCellKills(vec![]));

    // Tick 2: EffectV3Systems::Bridge runs on_cell_destroyed and reads the
    // pending Destroyed<Cell>, dispatching DeathOccurred(Any) to the listener.
    //
    // NOTE: the cell entity was already despawned by tick 1's
    // FixedPostUpdate (`process_despawn_requests`). The 2-tick pattern
    // here exists solely to wait for the bridge dispatch on tick 2 — we
    // do NOT assert despawn here because that would be vacuous.
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

    // Tick 1: handle_kill<Bolt> emits Destroyed<Bolt>; process_despawn despawns bolt.
    tick(&mut app);

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Bolt>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "tick 1: handle_kill<Bolt> should emit exactly one Destroyed<Bolt>"
    );
    assert_eq!(destroyed.0[0].victim, bolt);
    assert_eq!(destroyed.0[0].victim_pos, Vec2::new(0.0, -50.0));

    app.insert_resource(PendingBoltKills(vec![]));
    // Tick 2: bridge dispatches DeathOccurred(Any).
    //
    // NOTE: the bolt entity was already despawned by tick 1's
    // FixedPostUpdate (`process_despawn_requests`). The 2-tick pattern
    // here exists solely to wait for the bridge dispatch on tick 2 — we
    // do NOT assert despawn here because that would be vacuous.
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(listener)
        .expect("listener should have EffectStack<SpeedBoostConfig> after bolt bridge dispatch");
    assert_eq!(stack.len(), 1);
}

#[test]
fn bridge_handoff_bolt_death_late_listener_still_receives_dispatch() {
    // Edge case: listener is spawned AFTER the bolt is killed but BEFORE
    // tick 2 runs. The listener still receives the dispatch because
    // on_bolt_destroyed iterates over all entities with BoundEffects at
    // the time it runs.
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

    // Tick 1: bolt killed. No listener yet.
    tick(&mut app);
    app.insert_resource(PendingBoltKills(vec![]));

    // Spawn the listener after tick 1 — before tick 2 runs.
    let listener = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Any),
            2.0,
        )]))
        .id();

    // Tick 2: bridge reads Destroyed<Bolt> and dispatches to the listener.
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(listener)
        .expect("late-joining listener should still receive dispatch");
    assert_eq!(stack.len(), 1);
}
