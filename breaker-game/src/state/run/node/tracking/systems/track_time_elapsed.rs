//! System to accumulate simulation time for the current run.

use bevy::prelude::*;

use crate::state::run::resources::RunStats;

/// Adds the fixed timestep delta to [`RunStats::time_elapsed`] each tick.
pub(crate) fn track_time_elapsed(time: Res<Time<Fixed>>, mut stats: ResMut<RunStats>) {
    stats.time_elapsed += time.delta_secs();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        use crate::shared::test_utils::TestAppBuilder;
        TestAppBuilder::new()
            .with_resource::<RunStats>()
            .with_system(FixedUpdate, track_time_elapsed)
            .build()
    }

    use crate::shared::test_utils::tick;

    #[test]
    fn accumulates_simulation_time_each_tick() {
        let mut app = test_app();

        // Run for 3 ticks
        for _ in 0..3 {
            tick(&mut app);
        }

        let stats = app.world().resource::<RunStats>();
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let expected = 3.0 * timestep.as_secs_f32();
        assert!(
            (stats.time_elapsed - expected).abs() < f32::EPSILON,
            "expected time_elapsed ~{expected}, got {}",
            stats.time_elapsed
        );
    }
}
