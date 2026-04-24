//! W2 Behaviors 39–44: `echo_strike_emit_siblings` in `DmgSystems::PostApplyDamage`.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::{EchoNetwork, EchoPrimed, EchoStrikeConfig, echo_strike_emit_siblings},
    helpers::{
        canonical_echo_strike_config, install_echo_strike_config,
        seed_active_protocols_with_echo_strike, spawn_bolt_primed_with_network,
        spawn_bolt_with_echo_network, spawn_cell_empty,
    },
};
use crate::{
    prelude::*,
    protocol::{
        definition::ProtocolKind,
        resources::{ActiveProtocols, protocol_active},
    },
};

#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "expected {expected}, got {actual}"
    );
}

fn build_app(active: bool) -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveProtocols>()
        .build();
    install_echo_strike_config(&mut app, canonical_echo_strike_config());
    if active {
        seed_active_protocols_with_echo_strike(&mut app, 3, 0.5, 0.3, 0.1);
    }
    app.add_systems(
        FixedUpdate,
        echo_strike_emit_siblings
            .in_set(DmgSystems::PostApplyDamage)
            .run_if(protocol_active(ProtocolKind::EchoStrike))
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

// ── W2 Behavior 39: system runs in PostApplyDamage, inactive gate ──

#[test]
fn echo_strike_inactive_emits_nothing() {
    let mut app = build_app(false);
    let c_newest = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 100.0, vec![c_newest]);
    let c_primary = spawn_cell_empty(&mut app);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        c_primary,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert!(
        !drained
            .iter()
            .any(|m| m.source == Some(SourceId::from("protocol:echo_strike")))
    );
}

// ── W2 Behavior 40: emits to every entry in EchoNetwork when primed ──

#[test]
fn echo_strike_emits_siblings_for_3_echo_network() {
    let mut app = build_app(true);
    // EchoStrikeConfig: newest 0.5, middle 0.3, oldest 0.1.
    install_echo_strike_config(
        &mut app,
        EchoStrikeConfig {
            max_echoes:      3,
            newest_fraction: 0.5,
            middle_fraction: 0.3,
            oldest_fraction: 0.1,
        },
    );
    let c_old = spawn_cell_empty(&mut app);
    let c_mid = spawn_cell_empty(&mut app);
    let c_newest = spawn_cell_empty(&mut app);
    // Front = oldest, back = newest.
    let bolt = spawn_bolt_primed_with_network(&mut app, 100.0, vec![c_old, c_mid, c_newest]);
    let c_primary = spawn_cell_empty(&mut app);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        c_primary,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    let echoes: Vec<&DamageDealt<Cell>> = drained
        .iter()
        .filter(|m| m.source == Some(SourceId::from("protocol:echo_strike")))
        .collect();
    assert_eq!(echoes.len(), 3);

    let find = |t: Entity| echoes.iter().find(|m| m.target == t);
    assert_f32_eq(find(c_old).expect("oldest").amount, 10.0);
    assert_f32_eq(find(c_mid).expect("middle").amount, 30.0);
    assert_f32_eq(find(c_newest).expect("newest").amount, 50.0);
    assert_eq!(find(c_old).unwrap().dealer, None);
    assert_eq!(find(c_old).unwrap().attributed_to, Some(bolt));
}

#[test]
fn echo_strike_empty_network_emits_nothing() {
    let mut app = build_app(true);
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            crate::bolt::components::BoltBaseDamage(100.0),
            EchoPrimed,
            EchoNetwork::default(),
        ))
        .id();
    let c_primary = spawn_cell_empty(&mut app);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        c_primary,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert!(
        !drained
            .iter()
            .any(|m| m.source == Some(SourceId::from("protocol:echo_strike")))
    );
}

// ── W2 Behavior 41: EchoPrimed gate fail — no emission when bolt lacks it ──

#[test]
fn echo_strike_does_not_emit_when_bolt_lacks_echo_primed() {
    let mut app = build_app(true);
    let c_newest = spawn_cell_empty(&mut app);
    // Bolt has network but NOT EchoPrimed.
    let bolt = spawn_bolt_with_echo_network(&mut app, 100.0, vec![c_newest]);
    let c_primary = spawn_cell_empty(&mut app);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        c_primary,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert!(
        !drained
            .iter()
            .any(|m| m.source == Some(SourceId::from("protocol:echo_strike"))),
        "no echoes without EchoPrimed gate"
    );
}

#[test]
fn echo_strike_despawned_bolt_emits_nothing() {
    // Edge case: bolts.get(bolt) fails → no siblings, no panic.
    let mut app = build_app(true);
    let c_newest = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 100.0, vec![c_newest]);
    let c_primary = spawn_cell_empty(&mut app);
    app.world_mut().despawn(bolt);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        c_primary,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app); // must not panic
}

// ── W2 Behavior 43: skip when msg.amount == 0 (invulnerable source) ──

#[test]
fn echo_strike_skips_when_amount_zero() {
    let mut app = build_app(true);
    let c_newest = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 100.0, vec![c_newest]);
    let c_primary = spawn_cell_empty(&mut app);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        c_primary,
            amount:        0.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert!(
        !drained
            .iter()
            .any(|m| m.source == Some(SourceId::from("protocol:echo_strike")))
    );
}

// ── W2 Behavior 44: loop protection — source "protocol:echo_strike" → skip ──

#[test]
fn echo_strike_loop_protection_skips_on_own_source() {
    let mut app = build_app(true);
    let c_newest = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 100.0, vec![c_newest]);
    let c_primary = spawn_cell_empty(&mut app);

    // An echo-sourced message arrives — must not re-emit.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: Some(bolt),
            target:        c_primary,
            amount:        50.0,
            source:        Some(SourceId::from("protocol:echo_strike")),
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert_eq!(
        drained
            .iter()
            .filter(|m| m.source == Some(SourceId::from("protocol:echo_strike")))
            .count(),
        1,
        "only the original echo-sourced message; no re-emission"
    );
}
