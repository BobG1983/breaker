//! System to tick the upgrade selection countdown timer.

use bevy::prelude::*;

use crate::{screen::upgrade_select::resources::UpgradeSelectTimer, shared::GameState};

/// Ticks the upgrade selection timer and auto-advances on expiry.
///
/// Timer expiry transitions to [`GameState::NodeTransition`] (skip, no upgrade).
pub fn tick_upgrade_timer(
    time: Res<Time>,
    mut timer: ResMut<UpgradeSelectTimer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    timer.remaining -= time.delta_secs();

    if timer.remaining <= 0.0 {
        timer.remaining = 0.0;
        next_state.set(GameState::NodeTransition);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;

    fn test_app(remaining: f32) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<GameState>();
        app.insert_resource(UpgradeSelectTimer { remaining });
        app.add_systems(Update, tick_upgrade_timer);
        app
    }

    #[test]
    fn timer_decrements_after_update() {
        let mut app = test_app(10.0);

        // First update initializes time; second gets a real delta
        app.update();
        app.update();

        let timer = app.world().resource::<UpgradeSelectTimer>();
        assert!(
            timer.remaining < 10.0,
            "expected timer to decrease, got: {}",
            timer.remaining
        );
    }

    #[test]
    fn timer_expiry_transitions_to_node_transition() {
        // Start with 0 remaining — should expire immediately
        let mut app = test_app(0.0);
        app.update();

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("NodeTransition"),
            "expected NodeTransition, got: {next:?}"
        );
    }

    #[test]
    fn timer_clamps_to_zero_on_expiry() {
        let mut app = test_app(0.0);
        app.update();

        let timer = app.world().resource::<UpgradeSelectTimer>();
        assert!(
            timer.remaining.abs() < f32::EPSILON,
            "expected 0.0, got: {}",
            timer.remaining
        );
    }

    #[test]
    fn no_transition_when_time_remains() {
        let mut app = test_app(100.0);
        app.update();

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            !format!("{next:?}").contains("NodeTransition"),
            "expected no transition, got: {next:?}"
        );
    }
}
