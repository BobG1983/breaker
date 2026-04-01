//! Tests for `MoveBolt` and `TogglePause` frame mutations.

use super::super::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — MoveBolt at matching frame
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
// apply_debug_frame_mutations — TogglePause sets NextState to Paused
// -------------------------------------------------------------------------

/// When `frame_mutations` has `TogglePause` at frame 3, the current frame
/// is 3, and the game is in `PlayingState::Active`, the system must set
/// `NextState<PlayingState>` to `Paused`.
#[test]
fn apply_debug_frame_mutations_toggle_pause_sets_paused() {
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
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .add_sub_state::<PlayingState>()
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(3))
        .add_systems(Update, apply_debug_frame_mutations);

    // Drive into GameState::Playing so PlayingState becomes active.
    // Single update: state transition activates PlayingState, then the mutation
    // system runs (frame=3 matches) and sets NextState<PlayingState> to Paused.
    // A second update would process that transition, then the system would toggle
    // back to Active (because TogglePause runs again at the same frame).
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Check that NextState<PlayingState> is set to Paused
    let next = app.world().resource::<NextState<PlayingState>>();
    assert!(
        matches!(next, NextState::Pending(PlayingState::Paused)),
        "expected NextState::Pending(Paused), got: {next:?}"
    );
}
