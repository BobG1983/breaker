use bevy::prelude::*;
use breaker::state::run::node::resources::NodeTimer;

use crate::{invariants::*, types::InvariantKind};

/// Checks that [`NodeTimer::remaining`] never goes negative.
///
/// Only runs when the `NodeTimer` resource exists (Chrono breaker).
pub fn check_timer_non_negative(
    timer: Option<Res<NodeTimer>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    let Some(timer) = timer else { return };
    if timer.remaining < 0.0 {
        log.0.push(ViolationEntry {
            frame:     frame.0,
            invariant: InvariantKind::TimerNonNegative,
            entity:    None,
            message:   format!(
                "TimerNonNegative FAIL frame={} remaining={:.3}",
                frame.0, timer.remaining,
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

    #[test]
    fn timer_non_negative_fires_when_remaining_is_negative() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(NodeTimer {
                remaining: -1.0,
                total:     60.0,
            })
            .add_systems(FixedUpdate, check_timer_non_negative);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::TimerNonNegative);
    }

    #[test]
    fn timer_non_negative_does_not_fire_when_remaining_is_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(NodeTimer {
                remaining: 0.0,
                total:     60.0,
            })
            .add_systems(FixedUpdate, check_timer_non_negative);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn timer_non_negative_skips_when_no_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default());
        // NodeTimer not inserted
        app.add_systems(FixedUpdate, check_timer_non_negative);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }
}
