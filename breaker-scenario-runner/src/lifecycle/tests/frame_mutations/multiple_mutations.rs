//! Tests for multiple frame mutations on the same frame.

use super::super::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — multiple mutations on same frame
// -------------------------------------------------------------------------

/// When two mutations are scheduled for the same frame, both must apply.
/// Given `SetDashState(Braking)` and `MoveBolt(100.0, 200.0)` both at
/// frame 5, the breaker state must change AND the bolt must move.
#[test]
fn apply_debug_frame_mutations_multiple_mutations_on_same_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        frame_mutations: Some(vec![
            FrameMutation {
                frame: 5,
                mutation: MutationKind::SetDashState(ScenarioDashState::Braking),
            },
            FrameMutation {
                frame: 5,
                mutation: MutationKind::MoveBolt(100.0, 200.0),
            },
        ]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    let breaker_entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, DashState::Idle))
        .id();
    let bolt_entity = app
        .world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))))
        .id();

    app.update();

    let state = app
        .world()
        .entity(breaker_entity)
        .get::<DashState>()
        .expect("breaker entity must still have DashState");
    assert_eq!(
        *state,
        DashState::Braking,
        "expected DashState::Braking from first mutation, got {state:?}"
    );

    let position = app
        .world()
        .entity(bolt_entity)
        .get::<Position2D>()
        .expect("bolt entity must still have Position2D");
    assert_eq!(
        position.0,
        Vec2::new(100.0, 200.0),
        "expected bolt at (100.0, 200.0) from second mutation, got {:?}",
        position.0
    );
}
