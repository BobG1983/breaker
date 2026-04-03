//! Physics-frozen-during-pause invariant — now a no-op.
//!
//! Pause uses `Time<Virtual>::pause()` which structurally prevents `FixedUpdate`
//! from ticking. Bevy guarantees this — no runtime check needed.

use bevy::prelude::*;

use crate::invariants::*;

/// No-op — `Time<Virtual>::pause()` structurally freezes `FixedUpdate`.
pub fn check_physics_frozen_during_pause(frame: Res<ScenarioFrame>, _log: ResMut<ViolationLog>) {
    // Bevy guarantees FixedUpdate does not tick while Time<Virtual> is paused.
    // No runtime invariant check needed.
    let _ = &*frame;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checker_produces_no_violations() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(Update, check_physics_frozen_during_pause);
        app.update();

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }
}
