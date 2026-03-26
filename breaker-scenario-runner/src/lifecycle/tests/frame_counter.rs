use super::helpers::*;

// -------------------------------------------------------------------------
// tick_scenario_frame
// -------------------------------------------------------------------------

/// Each fixed-update tick increments [`ScenarioFrame`] by 1.
#[test]
fn tick_scenario_frame_increments_by_one_per_tick() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioFrame(0))
        .add_systems(FixedUpdate, tick_scenario_frame);

    tick(&mut app);
    assert_eq!(app.world().resource::<ScenarioFrame>().0, 1);

    tick(&mut app);
    assert_eq!(app.world().resource::<ScenarioFrame>().0, 2);
}

// -------------------------------------------------------------------------
// ScenarioStats — max_frame tracked by tick_scenario_frame
// -------------------------------------------------------------------------

/// After 10 ticks, `ScenarioStats::max_frame` must equal 10.
/// `tick_scenario_frame` must update both [`ScenarioFrame`] and `stats.max_frame`.
#[test]
fn scenario_stats_max_frame_tracked_by_tick_scenario_frame() {
    use crate::invariants::ScenarioStats;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioFrame(0))
        .init_resource::<ScenarioStats>()
        .add_systems(FixedUpdate, tick_scenario_frame);

    for _ in 0..10 {
        tick(&mut app);
    }

    let stats = app.world().resource::<ScenarioStats>();
    assert_eq!(
        stats.max_frame, 10,
        "expected max_frame == 10 after 10 ticks, got {}",
        stats.max_frame
    );
}
