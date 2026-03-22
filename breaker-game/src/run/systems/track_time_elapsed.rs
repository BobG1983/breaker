//! System to accumulate simulation time for the current run.

use bevy::prelude::*;

use crate::run::resources::RunStats;

/// Adds the fixed timestep delta to [`RunStats::time_elapsed`] each tick.
pub(crate) fn track_time_elapsed(time: Res<Time<Fixed>>, mut stats: ResMut<RunStats>) {
    stats.time_elapsed += time.delta_secs();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<RunStats>()
            .add_systems(FixedUpdate, track_time_elapsed);
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
