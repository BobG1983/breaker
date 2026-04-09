//! System to toggle pause with Escape key.

use bevy::prelude::*;

/// Toggles `Time<Virtual>` between paused and unpaused on Escape.
///
/// Reads `ButtonInput<KeyCode>` directly instead of `InputActions` because
/// pause is a UI action in `Update`, not a gameplay action in `FixedUpdate`.
/// `InputActions` is cleared in `FixedPostUpdate` (before `Update`) and
/// stales when `Time<Virtual>` is paused (`FixedPostUpdate` stops running).
///
/// Gated on `NodeState::Playing` — only active during node gameplay.
/// `Time<Virtual>::pause()` freezes `FixedUpdate` (gameplay) while leaving
/// `Update` (UI, input) running.
pub(crate) fn toggle_pause(keyboard: Res<ButtonInput<KeyCode>>, mut time: ResMut<Time<Virtual>>) {
    if !keyboard.just_pressed(KeyCode::Escape) {
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
    use crate::shared::test_utils::TestAppBuilder;

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_resource::<ButtonInput<KeyCode>>()
            .with_system(Update, toggle_pause)
            .build()
    }

    /// Simulates a single Escape key press-and-release cycle.
    fn tap_escape(app: &mut App) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        // Release and clear so next frame sees a clean state
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release(KeyCode::Escape);
        input.clear();
    }

    #[test]
    fn toggle_pauses_virtual_time() {
        let mut app = test_app();
        tap_escape(&mut app);

        let time = app.world().resource::<Time<Virtual>>();
        assert!(time.is_paused(), "Time<Virtual> should be paused");
    }

    #[test]
    fn toggle_again_unpauses_virtual_time() {
        let mut app = test_app();
        tap_escape(&mut app);
        tap_escape(&mut app);

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

    #[test]
    fn held_escape_does_not_double_toggle() {
        let mut app = test_app();
        tap_escape(&mut app);

        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "should be paused after first press"
        );

        // Next frame with no new press — should stay paused
        app.update();

        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "should stay paused when Escape is not pressed again"
        );
    }
}
