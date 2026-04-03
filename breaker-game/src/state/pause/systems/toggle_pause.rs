//! System to toggle pause with Escape key.

use bevy::prelude::*;

use crate::input::resources::{GameAction, InputActions};

/// Toggles `Time<Virtual>` between paused and unpaused on `TogglePause`.
///
/// Gated on `NodeState::Playing` — only active during node gameplay.
/// `Time<Virtual>::pause()` freezes `FixedUpdate` (gameplay) while leaving
/// `Update` (UI, input) running.
pub(crate) fn toggle_pause(actions: Res<InputActions>, mut time: ResMut<Time<Virtual>>) {
    if !actions.active(GameAction::TogglePause) {
        return;
    }

    if time.is_paused() {
        time.unpause();
    } else {
        time.pause();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::resources::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<InputActions>()
            .add_systems(Update, toggle_pause);
        app
    }

    fn inject_toggle_pause(app: &mut App) {
        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::TogglePause);
        app.update();
    }

    #[test]
    fn toggle_pauses_virtual_time() {
        let mut app = test_app();
        inject_toggle_pause(&mut app);

        let time = app.world().resource::<Time<Virtual>>();
        assert!(time.is_paused(), "Time<Virtual> should be paused");
    }

    #[test]
    fn toggle_again_unpauses_virtual_time() {
        let mut app = test_app();
        inject_toggle_pause(&mut app);

        // Clear and toggle again
        app.world_mut().resource_mut::<InputActions>().0.clear();
        inject_toggle_pause(&mut app);

        let time = app.world().resource::<Time<Virtual>>();
        assert!(!time.is_paused(), "Time<Virtual> should be unpaused");
    }

    #[test]
    fn no_toggle_no_change() {
        let mut app = test_app();
        app.update();

        let time = app.world().resource::<Time<Virtual>>();
        assert!(!time.is_paused(), "Time<Virtual> should remain unpaused");
    }
}
