//! Section C — register (no-op / inert).

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{hazard::resources::ActiveHazards, prelude::*};

// Behavior 21 — register does NOT emit any DamageDealt<Cell> messages.
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
        KilledBy::default(),
    ));
    app.world_mut().spawn((
        Cell,
        Position2D(Vec2::new(50.0, 0.0)),
        Hp::new(100.0),
        KilledBy::default(),
    ));

    register(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        0,
        "register must not schedule any system that writes DamageDealt<Cell>, got {}",
        collector.0.len()
    );
}

// Behavior 22 — register does not panic when DiffusionConfig is absent.
#[test]
fn register_does_not_panic_without_diffusion_config() {
    let mut app = test_app_playing();
    register(&mut app);
    tick(&mut app);
}

// Behavior 23 — register does not panic when ActiveHazards is absent.
#[test]
fn register_does_not_panic_without_active_hazards() {
    // Deliberately omit .with_resource::<ActiveHazards>(). If register wired
    // a system gated on `hazard_active(...)` (which takes a non-optional
    // Res<ActiveHazards>), Bevy 0.18 would panic during schedule execution.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_message::<DamageDealt<Cell>>()
        .build();

    register(&mut app);
    tick(&mut app);
}
