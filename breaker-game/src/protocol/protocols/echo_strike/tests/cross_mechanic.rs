//! W2 Behavior 46: Echo Strike fires on tether- and diffusion-sourced ripples.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::echo_strike_emit_siblings,
    helpers::{
        canonical_echo_strike_config, install_echo_strike_config,
        seed_active_protocols_with_echo_strike, spawn_bolt_primed_with_network, spawn_cell_empty,
    },
};
use crate::{
    hazard::definition::HazardKind,
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

fn build_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveProtocols>()
        .build();
    install_echo_strike_config(&mut app, canonical_echo_strike_config());
    seed_active_protocols_with_echo_strike(&mut app, 3, 0.5, 0.25, 0.1);
    app.add_systems(
        FixedUpdate,
        echo_strike_emit_siblings
            .in_set(DmgSystems::PostApplyDamage)
            .run_if(protocol_active(ProtocolKind::EchoStrike))
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

// ── W2 Behavior 46: echoes fire on tether-sourced ripples ──

#[test]
fn echo_strike_fires_on_tether_sourced_ripple() {
    let mut app = build_app();
    let c_newest = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 100.0, vec![c_newest]);
    let c1 = spawn_cell_empty(&mut app);

    // Simulate a tether-sourced ripple arriving at c1 — `dealer: None`,
    // `attributed_to: Some(bolt)`. Echo Strike resolves the bolt via
    // `msg.dealer.or(msg.attributed_to)`.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: Some(bolt),
            target:        c1,
            amount:        25.0,
            source:        Some(SourceId::hazard(HazardKind::Tether).build()),
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    let echoes: Vec<_> = drained
        .iter()
        .filter(|m| m.source == Some(SourceId::protocol(ProtocolKind::EchoStrike).build()))
        .collect();
    assert_eq!(echoes.len(), 1, "one echo sibling to c_newest expected");
    assert_eq!(echoes[0].target, c_newest);
    assert_f32_eq(echoes[0].amount, 25.0 * 0.5);
    assert_eq!(echoes[0].dealer, None);
    assert_eq!(echoes[0].attributed_to, Some(bolt));
}

#[test]
fn echo_strike_fires_on_diffusion_sourced_ripple() {
    let mut app = build_app();
    let c_newest = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 100.0, vec![c_newest]);
    let c1 = spawn_cell_empty(&mut app);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: Some(bolt),
            target:        c1,
            amount:        50.0,
            source:        Some(SourceId::hazard(HazardKind::Diffusion).instance(0).build()),
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    let echoes: Vec<_> = drained
        .iter()
        .filter(|m| m.source == Some(SourceId::protocol(ProtocolKind::EchoStrike).build()))
        .collect();
    assert_eq!(echoes.len(), 1);
    assert_f32_eq(echoes[0].amount, 50.0 * 0.5);
}

#[test]
fn echo_strike_does_not_fire_on_ripple_without_echo_primed() {
    let mut app = build_app();
    let c_newest = spawn_cell_empty(&mut app);
    // Bolt lacks EchoPrimed.
    let bolt = app
        .world_mut()
        .spawn((
            crate::bolt::components::Bolt,
            crate::bolt::components::BoltBaseDamage(100.0),
            super::super::system::EchoNetwork {
                echoes: vec![c_newest].into_iter().collect(),
            },
        ))
        .id();
    let c1 = spawn_cell_empty(&mut app);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: Some(bolt),
            target:        c1,
            amount:        25.0,
            source:        Some(SourceId::hazard(HazardKind::Tether).build()),
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
            .any(|m| { m.source == Some(SourceId::protocol(ProtocolKind::EchoStrike).build(),) }),
        "EchoPrimed gate must fail for ripples too"
    );
}

#[test]
fn echo_strike_does_not_fire_on_environmental_primary() {
    // dealer: None, attributed_to: None → resolver finds no bolt → no echoes.
    let mut app = build_app();
    let c1 = spawn_cell_empty(&mut app);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c1,
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
            .any(|m| { m.source == Some(SourceId::protocol(ProtocolKind::EchoStrike).build(),) })
    );
}
