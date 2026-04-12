use bevy::prelude::*;
use breaker::effect_v3::effects::second_wind::SecondWindWall;

use crate::{invariants::*, types::InvariantKind};

/// Checks that at most one [`SecondWindWall`] entity exists at any frame.
///
/// The Second Wind effect spawns a single protective wall at the bottom of the
/// playfield. Having more than one simultaneously indicates the effect fired
/// twice without the first wall being consumed — likely a missing despawn or a
/// double-trigger bug.
///
/// Fires immediately on the first frame where count > 1. Does not throttle —
/// the bug is severe enough to report every frame it persists.
pub fn check_second_wind_wall_at_most_one(
    walls: Query<Entity, With<SecondWindWall>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    let count = walls.iter().count();
    if count > 1 {
        log.0.push(ViolationEntry {
            frame:     frame.0,
            invariant: InvariantKind::SecondWindWallAtMostOne,
            entity:    None,
            message:   format!(
                "SecondWindWallAtMostOne FAIL frame={} count={count} (expected ≤ 1)",
                frame.0,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            .add_systems(FixedUpdate, check_second_wind_wall_at_most_one);
        app
    }

    #[test]
    fn no_violation_when_no_walls_exist() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when no SecondWindWall entities exist"
        );
    }

    #[test]
    fn no_violation_when_exactly_one_wall_exists() {
        let mut app = test_app();
        app.world_mut().spawn(SecondWindWall);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when exactly one SecondWindWall exists"
        );
    }

    #[test]
    fn fires_when_two_walls_exist() {
        let mut app = test_app();
        app.world_mut().spawn(SecondWindWall);
        app.world_mut().spawn(SecondWindWall);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one violation when 2 SecondWindWall entities exist, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::SecondWindWallAtMostOne,);
    }

    #[test]
    fn fires_when_three_walls_exist() {
        let mut app = test_app();
        for _ in 0..3 {
            app.world_mut().spawn(SecondWindWall);
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0
                .iter()
                .any(|v| v.invariant == InvariantKind::SecondWindWallAtMostOne),
            "expected SecondWindWallAtMostOne violation with 3 walls"
        );
    }

    #[test]
    fn message_includes_count() {
        let mut app = test_app();
        app.world_mut().spawn(SecondWindWall);
        app.world_mut().spawn(SecondWindWall);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0[0].message.contains("count=2"),
            "violation message should include count=2, got: {}",
            log.0[0].message
        );
    }
}
