//! Section F — `register` wiring + run-condition gate (Behaviours 62–65).

use bevy::prelude::*;

use super::{
    super::system::{TetherLink, register},
    helpers::{
        add_tether_stacks, build_cleanup_tether_app, build_establish_tether_app,
        canonical_tether_config, enter_playing, install_tether_config, run_fixed_update,
        spawn_cell_row, spawn_linked_pair,
    },
};
use crate::{mutators::hazards::resources::ActiveHazards, prelude::*};

// ── Behavior 62 — register wires establish_tether_links to OnEnter(Playing) ──

#[test]
fn register_wires_establish_to_on_enter_playing() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    spawn_cell_row(&mut app, 10, 50.0);

    // Register is already called inside build_establish_tether_app. Driving
    // the state transition must fire the OnEnter system and produce the same
    // 4-link outcome as Behaviour 20.
    enter_playing(&mut app);

    let mut q = app.world_mut().query_filtered::<Entity, With<TetherLink>>();
    let total = q.iter(app.world()).count();
    assert_eq!(
        total / 2,
        4,
        "register must wire establish_tether_links OnEnter(Playing); matches \
         Behaviour 20 expectation — got {} links",
        total / 2
    );
}

// ── Behavior 63 — register wires cleanup to FixedUpdate with state+hazard gates ──

#[test]
fn register_wires_cleanup_with_state_and_hazard_gates() {
    // Not-in-playing app: cleanup must NOT fire.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .build();
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    register(&mut app);

    let (a, b) = spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));
    app.world_mut().entity_mut(b).insert(Dead);
    run_fixed_update(&mut app);

    let link_a = app
        .world()
        .get::<TetherLink>(a)
        .expect("A should still hold TetherLink — state gate off");
    assert_eq!(link_a.partner, b);
}

// ── Behavior 64 — register does NOT emit DamageDealt<Cell> messages ──────────

#[test]
fn register_does_not_emit_damage_dealt_cell_messages() {
    let mut app = build_cleanup_tether_app();
    let (_a, _b) = spawn_linked_pair(&mut app, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0));
    // No damage messages pushed.
    run_fixed_update(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        0,
        "register must not schedule any system that writes DamageDealt<Cell>; \
         got {} messages",
        collector.0.len()
    );
}

// ── Behavior 65 — register does not panic when TetherConfig absent ───────────

#[test]
fn register_does_not_panic_when_tether_config_absent() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .build();
    // Zero Tether stacks, no TetherConfig.
    register(&mut app);

    run_fixed_update(&mut app);
    // Assertion: no panic above. Reaching here is success.
}

// ════════════════════════════════════════════════════════════════════
// W2 Behavior 54 — tether::register schedules tether_emit_partner in PostApplyDamage
// ════════════════════════════════════════════════════════════════════

use std::marker::PhantomData;

use crate::mutators::hazards::definition::HazardKind;

#[test]
fn tether_register_schedules_emit_partner_in_post_apply() {
    // After register(&mut app) + 1 tick with Tether active, a primary
    // DamageDealt<Cell> targeting a linked cell must produce a sibling
    // with source "hazard:tether". Proves `tether_emit_partner` was
    // scheduled into DmgSystems::PostApplyDamage.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .build();
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    register(&mut app);

    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    let _ = b;
    let bolt = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
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
    let tethered: Vec<_> = drained
        .iter()
        .filter(|m| m.source == Some(SourceId::hazard(HazardKind::Tether).build()))
        .collect();
    assert_eq!(
        tethered.len(),
        1,
        "register must wire tether_emit_partner in PostApplyDamage — got {} tether siblings",
        tethered.len()
    );
}
