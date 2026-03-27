//! Bolt speed boost chip effect observer — adds flat speed to bolt.

use bevy::prelude::*;

use super::stack_f32;
use crate::{bolt::components::Bolt, chips::components::BoltSpeedBoost};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a speed boost passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpeedBoostApplied {
    /// Speed multiplier per stack.
    pub multiplier: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    // FUTURE: may be used for upcoming phases
    // /// Name of the chip that applied this effect.
    // pub chip_name: String,
}

/// Observer: applies bolt speed boost stacking to all bolt entities.
pub(crate) fn handle_bolt_speed_boost(
    trigger: On<SpeedBoostApplied>,
    mut query: Query<(Entity, Option<&mut BoltSpeedBoost>), With<Bolt>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let per_stack = event.multiplier;
    let max_stacks = event.max_stacks;
    for (entity, mut existing) in &mut query {
        stack_f32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            BoltSpeedBoost,
        );
    }
}

/// Registers all observers and systems for the bolt speed boost effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_bolt_speed_boost);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_bolt_speed_boost);
        app
    }

    #[test]
    fn inserts_bolt_speed_boost_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(SpeedBoostApplied {
            multiplier: 1.1,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let s = app.world().entity(bolt).get::<BoltSpeedBoost>().unwrap();
        assert!((s.0 - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_bolt_speed_boost() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, BoltSpeedBoost(1.1))).id();

        app.world_mut().commands().trigger(SpeedBoostApplied {
            multiplier: 1.1,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let s = app.world().entity(bolt).get::<BoltSpeedBoost>().unwrap();
        assert!(
            (s.0 - 2.2).abs() < f32::EPSILON,
            "BoltSpeedBoost should stack from 1.1 to 2.2, got {}",
            s.0
        );
    }

    #[test]
    fn applies_to_all_bolt_entities() {
        let mut app = test_app();
        let bolt_a = app.world_mut().spawn(Bolt).id();
        let bolt_b = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(SpeedBoostApplied {
            multiplier: 1.1,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let sa = app.world().entity(bolt_a).get::<BoltSpeedBoost>().unwrap();
        let sb = app.world().entity(bolt_b).get::<BoltSpeedBoost>().unwrap();
        assert!((sa.0 - 1.1).abs() < f32::EPSILON);
        assert!((sb.0 - 1.1).abs() < f32::EPSILON);
    }
}
