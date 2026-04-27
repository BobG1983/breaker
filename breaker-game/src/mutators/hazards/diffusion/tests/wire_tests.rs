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
