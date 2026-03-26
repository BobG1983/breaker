use super::helpers::*;

// -------------------------------------------------------------------------
// check_frame_limit
// -------------------------------------------------------------------------

#[derive(Resource, Default)]
struct ExitReceived(bool);

fn capture_exit(mut reader: MessageReader<AppExit>, mut received: ResMut<ExitReceived>) {
    for _ in reader.read() {
        received.0 = true;
    }
}

fn exit_test_app(current_frame: u32, max_frames: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<AppExit>()
        .insert_resource(ScenarioFrame(current_frame))
        .insert_resource(ScenarioConfig {
            definition: make_scenario(max_frames),
        })
        .init_resource::<ExitReceived>()
        .add_systems(FixedUpdate, (check_frame_limit, capture_exit).chain());
    app
}

/// When frame equals `max_frames`, `AppExit` is sent.
#[test]
fn check_frame_limit_sends_exit_at_max_frames() {
    let mut app = exit_test_app(100, 100);
    tick(&mut app);
    assert!(
        app.world().resource::<ExitReceived>().0,
        "expected AppExit when frame == max_frames"
    );
}

/// When frame exceeds `max_frames`, `AppExit` is still sent.
#[test]
fn check_frame_limit_sends_exit_when_frame_exceeds_max() {
    let mut app = exit_test_app(150, 100);
    tick(&mut app);
    assert!(
        app.world().resource::<ExitReceived>().0,
        "expected AppExit when frame > max_frames"
    );
}

/// When frame is below `max_frames`, no `AppExit` is sent.
#[test]
fn check_frame_limit_does_not_exit_before_max_frames() {
    let mut app = exit_test_app(99, 100);
    tick(&mut app);
    assert!(
        !app.world().resource::<ExitReceived>().0,
        "expected no AppExit when frame < max_frames"
    );
}
