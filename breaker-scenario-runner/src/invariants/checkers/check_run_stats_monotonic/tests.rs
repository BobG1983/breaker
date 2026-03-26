use bevy::prelude::*;
use breaker::run::RunStats;

use super::checker::*;
use crate::{invariants::*, types::InvariantKind};

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame::default())
        .add_systems(FixedUpdate, check_run_stats_monotonic);
    app
}

fn run_stats_with(
    nodes_cleared: u32,
    cells_destroyed: u32,
    bumps_performed: u32,
    perfect_bumps: u32,
    bolts_lost: u32,
) -> RunStats {
    RunStats {
        nodes_cleared,
        cells_destroyed,
        bumps_performed,
        perfect_bumps,
        bolts_lost,
        ..Default::default()
    }
}

#[test]
fn skips_when_no_run_stats_resource() {
    let mut app = test_app();
    // RunStats not inserted — system should skip gracefully

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violations when RunStats resource is absent"
    );
}

#[test]
fn no_violation_on_first_tick_with_run_stats() {
    let mut app = test_app();
    app.insert_resource(RunStats::default());

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation on the first tick — seeds Local snapshot"
    );
}

#[test]
fn counters_increasing_no_violation() {
    let mut app = test_app();
    app.insert_resource(run_stats_with(0, 5, 10, 2, 1));

    // Tick 1: seeds Local
    tick(&mut app);

    // Increase all counters (legal)
    *app.world_mut().resource_mut::<RunStats>() = run_stats_with(1, 8, 15, 3, 1);

    // Tick 2: all increased — no violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violations when all counters increase: {:?}",
        log.0
    );
}

#[test]
fn counters_unchanged_no_violation() {
    let mut app = test_app();
    app.insert_resource(run_stats_with(3, 12, 20, 5, 2));

    // Tick 1: seeds Local
    tick(&mut app);

    // Tick 2: no change — no violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violations when counters are unchanged"
    );
}

#[test]
fn nodes_cleared_decrease_fires_violation() {
    let mut app = test_app();
    app.insert_resource(run_stats_with(3, 0, 0, 0, 0));

    // Tick 1: seeds Local with nodes_cleared=3
    tick(&mut app);

    // Decrease nodes_cleared (illegal reset)
    app.world_mut().resource_mut::<RunStats>().nodes_cleared = 1;

    // Tick 2: 1 < 3 → violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::RunStatsMonotonic)
            .count(),
        1,
        "expected exactly 1 RunStatsMonotonic violation when nodes_cleared decreases 3→1"
    );
}

#[test]
fn cells_destroyed_decrease_fires_violation() {
    let mut app = test_app();
    app.insert_resource(run_stats_with(0, 20, 0, 0, 0));

    tick(&mut app); // seeds Local

    app.world_mut().resource_mut::<RunStats>().cells_destroyed = 10;

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::RunStatsMonotonic)
            .count(),
        1,
        "expected exactly 1 RunStatsMonotonic violation when cells_destroyed decreases 20→10"
    );
}

#[test]
fn bumps_performed_decrease_fires_violation() {
    let mut app = test_app();
    app.insert_resource(run_stats_with(0, 0, 50, 0, 0));

    tick(&mut app); // seeds Local

    app.world_mut().resource_mut::<RunStats>().bumps_performed = 49;

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::RunStatsMonotonic)
            .count(),
        1,
        "expected exactly 1 RunStatsMonotonic violation when bumps_performed decreases 50→49"
    );
}

#[test]
fn perfect_bumps_decrease_fires_violation() {
    let mut app = test_app();
    app.insert_resource(run_stats_with(0, 0, 0, 7, 0));

    tick(&mut app); // seeds Local

    app.world_mut().resource_mut::<RunStats>().perfect_bumps = 6;

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::RunStatsMonotonic)
            .count(),
        1,
        "expected exactly 1 RunStatsMonotonic violation when perfect_bumps decreases 7→6"
    );
}

#[test]
fn bolts_lost_decrease_fires_violation() {
    let mut app = test_app();
    app.insert_resource(run_stats_with(0, 0, 0, 0, 3));

    tick(&mut app); // seeds Local

    app.world_mut().resource_mut::<RunStats>().bolts_lost = 2;

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::RunStatsMonotonic)
            .count(),
        1,
        "expected exactly 1 RunStatsMonotonic violation when bolts_lost decreases 3→2"
    );
}

#[test]
fn multiple_counters_decrease_each_fires_violation() {
    let mut app = test_app();
    app.insert_resource(run_stats_with(5, 30, 40, 10, 3));

    tick(&mut app); // seeds Local

    // Decrease all counters simultaneously
    *app.world_mut().resource_mut::<RunStats>() = run_stats_with(4, 28, 38, 9, 2);

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    let violations: Vec<_> = log
        .0
        .iter()
        .filter(|v| v.invariant == InvariantKind::RunStatsMonotonic)
        .collect();
    assert_eq!(
        violations.len(),
        5,
        "expected 5 RunStatsMonotonic violations (one per counter), got: {:?}",
        violations.iter().map(|v| &v.message).collect::<Vec<_>>()
    );
}

#[test]
fn resets_local_when_run_stats_removed() {
    let mut app = test_app();
    app.insert_resource(run_stats_with(3, 10, 5, 2, 1));

    tick(&mut app); // seeds Local

    // Remove RunStats (e.g., run ended)
    app.world_mut().remove_resource::<RunStats>();

    tick(&mut app); // should reset Local, no violation

    let log_after_removal = app.world().resource::<ViolationLog>();
    assert!(
        log_after_removal.0.is_empty(),
        "expected no violation when RunStats is absent"
    );

    // Re-insert RunStats at 0 (new run)
    app.insert_resource(RunStats::default());

    tick(&mut app); // seeds Local fresh — no violation

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation when RunStats reappears after reset"
    );
}

#[test]
fn run_restart_to_default_resets_snapshot_no_violation() {
    let mut app = test_app();
    app.insert_resource(run_stats_with(3, 15, 20, 5, 1));

    tick(&mut app); // seeds Local with non-default values

    // Simulate run restart: stats reset to all-zero (new run)
    *app.world_mut().resource_mut::<RunStats>() = RunStats::default();

    tick(&mut app); // all-zero after non-zero — reset snapshot, no violation

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation when stats reset to default (run restart), got: {:?}",
        log.0.iter().map(|v| &v.message).collect::<Vec<_>>()
    );
}
