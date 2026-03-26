use super::helpers::*;

// -------------------------------------------------------------------------
// tick_scenario_frame — gated on entered_playing
// -------------------------------------------------------------------------

/// `tick_scenario_frame` must NOT increment `ScenarioFrame` when
/// `ScenarioStats::entered_playing` is `false`.
///
/// Given: `ScenarioStats { entered_playing: false }`, `ScenarioFrame(0)`,
///        `tick_scenario_frame` registered with `run_if(entered_playing)`.
/// When:  5 fixed-update ticks run.
/// Then:  `ScenarioFrame` is still 0.
///
/// Edge case: when `ScenarioStats` is absent (`Option` is `None`), the
/// guard must also block execution — frame should not tick.
#[test]
fn tick_scenario_frame_gated_before_playing_entered() {
    use crate::invariants::ScenarioStats;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioFrame(0))
        .insert_resource(ScenarioStats {
            entered_playing: false,
            ..Default::default()
        })
        .add_systems(FixedUpdate, tick_scenario_frame.run_if(entered_playing));

    for _ in 0..5 {
        tick(&mut app);
    }

    let frame = app.world().resource::<ScenarioFrame>();
    assert_eq!(
        frame.0, 0,
        "expected ScenarioFrame == 0 after 5 ticks with entered_playing=false, got {}",
        frame.0
    );
}

/// When `ScenarioStats` is absent (not inserted as a resource), the
/// `entered_playing` guard must return `false` — `tick_scenario_frame`
/// must not run.
#[test]
fn tick_scenario_frame_gated_when_scenario_stats_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioFrame(0))
        // Intentionally do NOT insert ScenarioStats
        .add_systems(FixedUpdate, tick_scenario_frame.run_if(entered_playing));

    for _ in 0..5 {
        tick(&mut app);
    }

    let frame = app.world().resource::<ScenarioFrame>();
    assert_eq!(
        frame.0, 0,
        "expected ScenarioFrame == 0 after 5 ticks with no ScenarioStats, got {}",
        frame.0
    );
}

// -------------------------------------------------------------------------
// tick_scenario_frame — ticks normally after Playing entered
// -------------------------------------------------------------------------

/// `tick_scenario_frame` must increment normally when
/// `ScenarioStats::entered_playing` is `true`.
///
/// Given: `ScenarioStats { entered_playing: true }`, `ScenarioFrame(0)`.
/// When:  3 fixed-update ticks run.
/// Then:  `ScenarioFrame` is 3.
#[test]
fn tick_scenario_frame_ticks_after_playing_entered() {
    use crate::invariants::ScenarioStats;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioFrame(0))
        .insert_resource(ScenarioStats {
            entered_playing: true,
            ..Default::default()
        })
        .add_systems(FixedUpdate, tick_scenario_frame.run_if(entered_playing));

    for _ in 0..3 {
        tick(&mut app);
    }

    let frame = app.world().resource::<ScenarioFrame>();
    assert_eq!(
        frame.0, 3,
        "expected ScenarioFrame == 3 after 3 ticks with entered_playing=true, got {}",
        frame.0
    );
}

// -------------------------------------------------------------------------
// check_frame_limit — gated on entered_playing
// -------------------------------------------------------------------------

#[derive(Resource, Default)]
struct ExitReceived(bool);

fn capture_exit(mut reader: MessageReader<AppExit>, mut received: ResMut<ExitReceived>) {
    for _ in reader.read() {
        received.0 = true;
    }
}

/// `check_frame_limit` must NOT send `AppExit` when
/// `ScenarioStats::entered_playing` is `false`, even when
/// `ScenarioFrame` exceeds `max_frames`.
///
/// Given: `ScenarioStats { entered_playing: false }`, `ScenarioFrame(0)`,
///        `max_frames: 5`, both `tick_scenario_frame` and
///        `check_frame_limit` registered with the `run_if(entered_playing)`
///        guard.
/// When:  10 fixed-update ticks run.
/// Then:  No `AppExit` message sent (frame never reached 5 because
///        `tick_scenario_frame` was also gated).
#[test]
fn check_frame_limit_gated_before_playing_entered() {
    use crate::invariants::ScenarioStats;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<AppExit>()
        .insert_resource(ScenarioFrame(0))
        .insert_resource(ScenarioConfig {
            definition: make_scenario(5),
        })
        .insert_resource(ScenarioStats {
            entered_playing: false,
            ..Default::default()
        })
        .init_resource::<ExitReceived>()
        .add_systems(
            FixedUpdate,
            (
                tick_scenario_frame.run_if(entered_playing),
                check_frame_limit.run_if(entered_playing),
                capture_exit,
            )
                .chain(),
        );

    for _ in 0..10 {
        tick(&mut app);
    }

    assert!(
        !app.world().resource::<ExitReceived>().0,
        "expected no AppExit when entered_playing=false, even after 10 ticks with max_frames=5"
    );

    let frame = app.world().resource::<ScenarioFrame>();
    assert_eq!(
        frame.0, 0,
        "expected ScenarioFrame == 0 (gated), got {}",
        frame.0
    );
}
