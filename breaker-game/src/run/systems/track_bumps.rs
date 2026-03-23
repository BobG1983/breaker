//! System to track bump stats for the current run.

use bevy::prelude::*;

use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    run::resources::{HighlightTracker, RunStats},
};

/// Reads [`BumpPerformed`] messages and updates [`RunStats`] counters
/// and [`HighlightTracker`] streak tracking.
pub(crate) fn track_bumps(
    mut reader: MessageReader<BumpPerformed>,
    mut stats: ResMut<RunStats>,
    mut tracker: ResMut<HighlightTracker>,
) {
    for msg in reader.read() {
        stats.bumps_performed += 1;
        tracker.total_bumps_this_node += 1;
        if msg.grade == BumpGrade::Perfect {
            stats.perfect_bumps += 1;
            tracker.consecutive_perfect_bumps += 1;
        } else {
            tracker.non_perfect_bumps_this_node += 1;
            tracker.best_perfect_streak = tracker
                .best_perfect_streak
                .max(tracker.consecutive_perfect_bumps);
            tracker.consecutive_perfect_bumps = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::messages::BumpGrade;

    #[derive(Resource)]
    struct TestMessages(Vec<BumpPerformed>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<BumpPerformed>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .add_systems(FixedUpdate, (enqueue_messages, track_bumps).chain());
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn increments_bumps_performed_for_any_grade() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.bumps_performed, 1,
            "any bump grade should increment bumps_performed"
        );
    }

    #[test]
    fn increments_perfect_bumps_only_for_perfect_grade() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.perfect_bumps, 1,
            "Perfect grade should increment perfect_bumps"
        );
        assert_eq!(
            stats.bumps_performed, 1,
            "Perfect grade should also increment bumps_performed"
        );
    }

    #[test]
    fn tracks_consecutive_perfect_bumps() {
        let mut app = test_app();
        // Send 3 consecutive perfect bumps across 3 ticks
        for _ in 0..3 {
            app.insert_resource(TestMessages(vec![BumpPerformed {
                grade: BumpGrade::Perfect,
                bolt: Entity::PLACEHOLDER,
            }]));
            tick(&mut app);
        }

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.consecutive_perfect_bumps, 3,
            "3 consecutive Perfect bumps should set consecutive_perfect_bumps to 3"
        );
    }

    #[test]
    fn non_perfect_bump_resets_consecutive_and_records_streak() {
        let mut app = test_app();
        // Manually set up the tracker with 6 consecutive perfect bumps
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .consecutive_perfect_bumps = 6;

        app.insert_resource(TestMessages(vec![BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.consecutive_perfect_bumps, 0,
            "non-perfect bump should reset consecutive counter"
        );
        assert_eq!(
            tracker.best_perfect_streak, 6,
            "streak of 6 should be recorded as best_perfect_streak"
        );
    }

    #[test]
    fn non_perfect_bump_records_streak_below_threshold() {
        let mut app = test_app();
        // Set up 3 consecutive perfect bumps (below perfect_streak_count of 5)
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .consecutive_perfect_bumps = 3;

        app.insert_resource(TestMessages(vec![BumpPerformed {
            grade: BumpGrade::Late,
            bolt: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.consecutive_perfect_bumps, 0,
            "non-perfect bump should reset consecutive counter"
        );
        assert_eq!(
            tracker.best_perfect_streak, 3,
            "streak of 3 is recorded but will not become a highlight (3 < 5)"
        );
    }

    // --- Behavior 20: non-perfect bumps increment non_perfect_bumps_this_node ---

    #[test]
    fn early_bump_increments_non_perfect_bumps_this_node() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![
            BumpPerformed {
                grade: BumpGrade::Early,
                bolt: Entity::PLACEHOLDER,
            },
            BumpPerformed {
                grade: BumpGrade::Late,
                bolt: Entity::PLACEHOLDER,
            },
        ]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.non_perfect_bumps_this_node, 2,
            "Early and Late bumps should both increment non_perfect_bumps_this_node"
        );
    }

    // --- Behavior 21: all bumps increment total_bumps_this_node ---

    #[test]
    fn all_grades_increment_total_bumps_this_node() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![
            BumpPerformed {
                grade: BumpGrade::Perfect,
                bolt: Entity::PLACEHOLDER,
            },
            BumpPerformed {
                grade: BumpGrade::Early,
                bolt: Entity::PLACEHOLDER,
            },
            BumpPerformed {
                grade: BumpGrade::Late,
                bolt: Entity::PLACEHOLDER,
            },
        ]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.total_bumps_this_node, 3,
            "all bump grades should increment total_bumps_this_node"
        );
    }
}
