//! `PreviousDashState` — snapshot of the previous tick's [`DashState`].
//!
//! Updated each `FixedUpdate` tick by `update_previous_dash_state`, which is
//! scheduled `.after(BreakerSystems::UpdateState)` — so a system positioned
//! `.after(BreakerSystems::UpdateState).before(BreakerSystems::UpdatePreviousState)`
//! can read both the NEW `DashState` and the OLD `PreviousDashState` within a
//! single tick. Wave 5 (Reckless Dash) uses this window for transition detection.

use bevy::prelude::*;

use crate::breaker::components::DashState;

/// Snapshot of the previous tick's [`DashState`].
///
/// Updated each `FixedUpdate` tick by `update_previous_dash_state`, which is
/// scheduled `.after(BreakerSystems::UpdateState)` — so a system positioned
/// `.after(BreakerSystems::UpdateState).before(BreakerSystems::UpdatePreviousState)`
/// can read both the NEW `DashState` and the OLD `PreviousDashState` within a
/// single tick. Wave 5 (Reckless Dash) uses this window for transition detection.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PreviousDashState(pub DashState);

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;

    /// Routes `Clone` through a generic bound so the `.clone()` call is
    /// gated on the trait existing, without tripping `clippy::clone_on_copy`.
    #[must_use]
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    // ── PreviousDashState derive coverage ──

    #[test]
    fn previous_dash_state_default_is_idle_and_has_required_derives() {
        // Default is Idle.
        let default_value = PreviousDashState::default();
        assert_eq!(
            default_value.0,
            DashState::Idle,
            "PreviousDashState::default().0 should equal DashState::Idle, got {:?}",
            default_value.0,
        );

        let a = PreviousDashState(DashState::Dashing);
        // Copy
        let b = a;
        // Clone (gated through a generic to keep clippy happy on a Copy type)
        let c = require_clone(&a);
        // Field access via Copy / Clone preserves value.
        assert_eq!(b.0, DashState::Dashing);
        assert_eq!(c.0, DashState::Dashing);
        // Debug
        let dbg = format!("{a:?}");
        assert!(
            dbg.contains("PreviousDashState"),
            "Debug format should contain \"PreviousDashState\", got {dbg}",
        );
        // Component
        let mut world = World::new();
        let entity = world.spawn(a).id();
        let component = world
            .get::<PreviousDashState>(entity)
            .expect("PreviousDashState should be queryable as a Component");
        assert_eq!(component.0, DashState::Dashing);
    }
}
