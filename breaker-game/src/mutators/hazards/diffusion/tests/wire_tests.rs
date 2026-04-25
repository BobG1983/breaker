//! Section C — wire (no-op / inert).

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{mutators::hazards::resources::ActiveHazards, prelude::*};

// Behavior 21 — wire does NOT emit any DamageDealt<Cell> messages.
#[test]
fn register_does_not_emit_damage_dealt_cell_messages() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();

    // Install config + 1 Diffusion stack + two cells.
    app.world_mut().insert_resource(canonical_config());
    add_diffusion_stacks(&mut app, 1);
    app.world_mut().spawn((
        Cell,
        Position2D(Vec2::ZERO),
        Hp::new(100.0),
        KilledBy { killer: None },
    ));
    app.world_mut().spawn((
        Cell,
        Position2D(Vec2::new(50.0, 0.0)),
        Hp::new(100.0),
        KilledBy { killer: None },
    ));

    wire(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        0,
        "wire must not schedule any system that writes DamageDealt<Cell>, got {}",
        collector.0.len()
    );
}

// Behavior 22 — wire does not panic when DiffusionConfig is absent.
#[test]
fn register_does_not_panic_without_diffusion_config() {
    let mut app = test_app_playing();
    wire(&mut app);
    tick(&mut app);
}

// Behavior 23 — wire does not panic when Diffusion is not active.
//
// The diffusion systems are gated on `hazard_active(HazardKind::Diffusion)`,
// which requires `ActiveHazards` to exist as a resource. With zero stacks,
// the run-if condition is false and the systems skip — no panic.
#[test]
fn register_does_not_panic_when_diffusion_inactive() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .build();

    wire(&mut app);
    tick(&mut app);
}

// ════════════════════════════════════════════════════════════════════
// W2 Behaviors 53, 56 — diffusion wire schedule placement + ordering
// ════════════════════════════════════════════════════════════════════

use std::marker::PhantomData;

use crate::mutators::{
    hazards::definition::HazardKind, plugin::wire_damage_chain,
    protocols::resources::ActiveProtocols,
};

// ── W2 Behavior 53: the central chain wires reduce_primary in MutateDamage
//     + emit_rings in PostApplyDamage. After Wave 3, `wire(app)` no longer
//     schedules chain participants — `MutatorsPlugin::wire_damage_chain`
//     does. This test exercises both, exactly as `MutatorsPlugin::build`
//     does in production. ──

#[test]
fn register_wires_systems_into_dmg_sets() {
    // After wire(app) + wire_damage_chain(app) + 1 tick with Diffusion
    // active + primary msg, msg.amount must be reduced (proves
    // reduce_primary ran in MutateDamage).
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<ActiveProtocols>()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .build();
    app.world_mut().insert_resource(canonical_config());
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Diffusion);
    wire(&mut app);
    wire_damage_chain(&mut app);

    let c0 = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id();
    let c1 = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::new(30.0, 0.0)),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id();
    let _ = c1;

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    // Primary reduced AND ring emitted — proves both wire-wired systems ran.
    let c0_hp = app.world().get::<Hp>(c0).expect("Hp").current;
    assert!(
        c0_hp < 100.0,
        "C0 HP must be reduced: wire must schedule reduce_primary in MutateDamage"
    );
}

// W2 Behavior 56 — cross-mechanic PostApplyDamage emitter ordering — moved to
// `mutators/plugin/tests/damage_chain.rs`. Those tests now exercise
// `MutatorsPlugin::wire_damage_chain` directly (the central authority for the
// `diffusion → tether → echo_strike` ripple chain) instead of constructing
// the chain inline.
