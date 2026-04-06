//! Tests for `MoveBolt` and `TogglePause` frame mutations.

use crate::lifecycle::tests::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_frame_mutations -- MoveBolt at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `MoveBolt(999.0, 999.0)` at frame 5 and
/// the current frame is 5, the tagged bolt must be moved to (999.0, 999.0).
#[test]
fn apply_debug_frame_mutations_move_bolt_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::MoveBolt(999.0, 999.0),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))))
        .id();

    app.update();

    let position = app
        .world()
        .entity(entity)
        .get::<Position2D>()
        .expect("entity must still have Position2D");
    assert_eq!(
        position.0,
        Vec2::new(999.0, 999.0),
        "expected bolt at (999.0, 999.0), got {:?}",
        position.0
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations -- TogglePause pauses virtual time
// -------------------------------------------------------------------------

/// When `frame_mutations` has `TogglePause` at frame 3, the current frame
/// is 3, and the game is not paused, the system must pause `Time<Virtual>`.
#[test]
fn apply_debug_frame_mutations_toggle_pause_pauses_virtual_time() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 3,
            mutation: MutationKind::TogglePause,
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(3))
        .add_systems(Update, apply_debug_frame_mutations);

    // Verify not paused initially
    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should not be paused initially"
    );

    app.update();

    // Check that Time<Virtual> is now paused
    assert!(
        app.world().resource::<Time<Virtual>>().is_paused(),
        "expected Time<Virtual> to be paused after TogglePause"
    );
}
