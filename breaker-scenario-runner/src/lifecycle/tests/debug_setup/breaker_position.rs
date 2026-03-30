//! Tests for breaker teleport via `apply_debug_setup`.

use super::super::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_setup — teleport breaker to breaker_position
// -------------------------------------------------------------------------

/// When `debug_setup` has `breaker_position: Some((100.0, -50.0))`,
/// `apply_debug_setup` must move the `ScenarioTagBreaker` entity's
/// `Position2D` to `(100.0, -50.0)`.
#[test]
fn apply_debug_setup_teleports_breaker_to_breaker_position() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        debug_setup: Some(DebugSetup {
            bolt_position: None,
            breaker_position: Some((100.0, -50.0)),
            disable_physics: false,
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, Position2D(Vec2::new(0.0, 0.0))))
        .id();

    // First update: system runs and mutates position directly (no commands needed)
    app.update();
    // Second update: flush any pending commands
    app.update();

    let position = app
        .world()
        .entity(entity)
        .get::<Position2D>()
        .expect("breaker entity must still have Position2D");

    assert!(
        (position.0.x - 100.0_f32).abs() < f32::EPSILON,
        "expected x = 100.0 after breaker_position teleport, got {}",
        position.0.x
    );
    assert!(
        (position.0.y - (-50.0_f32)).abs() < f32::EPSILON,
        "expected y = -50.0 after breaker_position teleport, got {}",
        position.0.y
    );
}
