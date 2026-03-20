//! System to regenerate HP on cells with the `CellRegen` component.

use bevy::prelude::*;

use crate::cells::components::{Cell, CellHealth, CellRegen};

/// Regenerates HP on cells with [`CellRegen`] each fixed timestep.
///
/// Adds `rate * dt` to the cell's current HP, clamped to max.
/// Destroyed cells (HP == 0) are skipped.
pub(crate) fn tick_cell_regen(
    time: Res<Time<Fixed>>,
    mut query: Query<(&mut CellHealth, &CellRegen), With<Cell>>,
) {
    let dt = time.delta_secs();
    for (mut health, regen) in &mut query {
        if health.is_destroyed() {
            continue;
        }
        health.current = regen.rate.mul_add(dt, health.current).min(health.max);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::cells::components::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, tick_cell_regen);
        app
    }

    /// Sets the fixed timestep to `dt` and accumulates one step, then runs update.
    fn tick_with_dt(app: &mut App, dt: Duration) {
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(dt);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(dt);
        app.update();
    }

    fn spawn_regen_cell(app: &mut App, current: f32, max: f32, rate: f32) -> Entity {
        app.world_mut()
            .spawn((Cell, CellHealth { current, max }, CellRegen { rate }))
            .id()
    }

    #[test]
    fn regen_cell_regenerates_hp_over_time() {
        // Given: Cell with 5.0/20.0 HP, regen rate 2.0
        // When: tick with dt = 1.0s
        // Then: current increases to ~7.0 (5.0 + 2.0 * 1.0)
        let mut app = test_app();
        let entity = spawn_regen_cell(&mut app, 5.0, 20.0, 2.0);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let health = app.world().get::<CellHealth>(entity).unwrap();
        assert!(
            (health.current - 7.0).abs() < f32::EPSILON,
            "cell with 5.0 HP and regen rate 2.0 after 1.0s should have 7.0 HP, got {}",
            health.current
        );
    }

    #[test]
    fn regen_does_not_exceed_max_hp() {
        // Given: Cell with 19.5/20.0 HP, regen rate 2.0
        // When: tick with dt = 1.0s (would add 2.0, total 21.5)
        // Then: current is clamped to 20.0
        let mut app = test_app();
        let entity = spawn_regen_cell(&mut app, 19.5, 20.0, 2.0);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let health = app.world().get::<CellHealth>(entity).unwrap();
        assert!(
            (health.current - 20.0).abs() < f32::EPSILON,
            "regen should clamp to max HP 20.0, got {}",
            health.current
        );
    }

    #[test]
    fn destroyed_cell_does_not_regen() {
        // Given: Cell with 0.0/20.0 HP (destroyed), regen rate 2.0
        // When: tick
        // Then: current stays at 0.0
        let mut app = test_app();
        let entity = spawn_regen_cell(&mut app, 0.0, 20.0, 2.0);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let health = app.world().get::<CellHealth>(entity).unwrap();
        assert!(
            (health.current - 0.0).abs() < f32::EPSILON,
            "destroyed cell (0 HP) should not regenerate, got {}",
            health.current
        );
    }

    #[test]
    fn full_hp_cell_stays_unchanged() {
        // Given: Cell at full HP 20.0/20.0, regen rate 2.0
        // When: tick
        // Then: current stays at 20.0
        let mut app = test_app();
        let entity = spawn_regen_cell(&mut app, 20.0, 20.0, 2.0);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let health = app.world().get::<CellHealth>(entity).unwrap();
        assert!(
            (health.current - 20.0).abs() < f32::EPSILON,
            "cell at full HP should stay at 20.0, got {}",
            health.current
        );
    }

    #[test]
    fn zero_regen_rate_produces_no_change() {
        // Given: Cell with 5.0/20.0 HP, regen rate 0.0
        // When: tick
        // Then: current stays at 5.0
        let mut app = test_app();
        let entity = spawn_regen_cell(&mut app, 5.0, 20.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let health = app.world().get::<CellHealth>(entity).unwrap();
        assert!(
            (health.current - 5.0).abs() < f32::EPSILON,
            "cell with zero regen rate should stay at 5.0 HP, got {}",
            health.current
        );
    }
}
