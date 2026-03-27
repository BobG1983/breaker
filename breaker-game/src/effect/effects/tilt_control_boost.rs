//! Tilt control boost chip effect observer — adds sensitivity to breaker.

use bevy::prelude::*;

use super::stack_f32;
use crate::{breaker::components::Breaker, chips::components::TiltControlBoost};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a tilt control passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct TiltControlApplied {
    /// Tilt control sensitivity increase per stack.
    pub per_stack: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    // FUTURE: may be used for upcoming phases
    // /// Name of the chip that applied this effect.
    // pub chip_name: String,
}

/// Observer: applies tilt control boost stacking to all breaker entities.
pub(crate) fn handle_tilt_control_boost(
    trigger: On<TiltControlApplied>,
    mut query: Query<(Entity, Option<&mut TiltControlBoost>), With<Breaker>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let per_stack = event.per_stack;
    let max_stacks = event.max_stacks;
    for (entity, mut existing) in &mut query {
        stack_f32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            TiltControlBoost,
        );
    }
}

/// Registers all observers and systems for the tilt control boost effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_tilt_control_boost);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_tilt_control_boost);
        app
    }

    #[test]
    fn inserts_tilt_control_boost_on_breaker() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn(Breaker).id();

        app.world_mut().commands().trigger(TiltControlApplied {
            per_stack: 5.0,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let t = app
            .world()
            .entity(breaker)
            .get::<TiltControlBoost>()
            .unwrap();
        assert!((t.0 - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_tilt_control_boost() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn((Breaker, TiltControlBoost(5.0))).id();

        app.world_mut().commands().trigger(TiltControlApplied {
            per_stack: 5.0,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let t = app
            .world()
            .entity(breaker)
            .get::<TiltControlBoost>()
            .unwrap();
        assert!(
            (t.0 - 10.0).abs() < f32::EPSILON,
            "TiltControlBoost should stack from 5.0 to 10.0, got {}",
            t.0
        );
    }

    #[test]
    fn respects_max_stacks_tilt_control_boost() {
        let mut app = test_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, TiltControlBoost(15.0)))
            .id();

        app.world_mut().commands().trigger(TiltControlApplied {
            per_stack: 5.0,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let t = app
            .world()
            .entity(breaker)
            .get::<TiltControlBoost>()
            .unwrap();
        assert!(
            (t.0 - 15.0).abs() < f32::EPSILON,
            "TiltControlBoost should not exceed max_stacks cap, got {}",
            t.0
        );
    }
}
