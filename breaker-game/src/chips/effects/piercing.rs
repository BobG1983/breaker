//! Piercing chip effect observer — bolt passes through N cells.

use bevy::prelude::*;

use super::stack_u32;
use crate::{
    bolt::components::Bolt,
    chips::{
        components::{Piercing, PiercingRemaining},
        definition::{AmpEffect, ChipEffect, ChipEffectApplied},
    },
};

/// Observer: applies piercing stacking to all bolt entities.
pub(crate) fn handle_piercing(
    trigger: On<ChipEffectApplied>,
    mut query: Query<(Entity, Option<&mut Piercing>), With<Bolt>>,
    mut commands: Commands,
) {
    let ChipEffect::Amp(AmpEffect::Piercing(per_stack)) = trigger.event().effect else {
        return;
    };
    let max_stacks = trigger.event().max_stacks;
    for (entity, mut existing) in &mut query {
        let is_new = existing.is_none();
        let old_val = existing.as_deref().map(|p| p.0);
        stack_u32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            Piercing,
        );
        if is_new {
            // stack_u32 inserted Piercing(per_stack) via deferred commands; mirror it
            commands.entity(entity).insert(PiercingRemaining(per_stack));
        } else {
            let new_val = existing.as_deref().map(|p| p.0);
            if new_val != old_val
                && let Some(val) = new_val
            {
                commands.entity(entity).insert(PiercingRemaining(val));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::{components::PiercingRemaining, definition::AmpEffect};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_piercing);
        app
    }

    #[test]
    fn inserts_piercing_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::Piercing(1)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 1);
    }

    #[test]
    fn stacks_piercing() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, Piercing(1))).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::Piercing(1)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 2);
    }

    #[test]
    fn respects_max_stacks() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, Piercing(3))).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::Piercing(1)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 3);
    }

    #[test]
    fn ignores_non_piercing_effects() {
        let mut app = test_app();
        app.world_mut().spawn(Bolt);

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::DamageBoost(1.5)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        assert!(
            app.world_mut()
                .query::<&Piercing>()
                .iter(app.world())
                .next()
                .is_none()
        );
    }

    // --- PiercingRemaining tests ---

    #[test]
    fn inserts_piercing_remaining_on_first_application() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::Piercing(2)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 2);
        let pr = app.world().entity(bolt).get::<PiercingRemaining>().unwrap();
        assert_eq!(pr.0, 2);
    }

    #[test]
    fn stacking_updates_piercing_remaining_to_new_value() {
        let mut app = test_app();
        // Bolt has used all its pierces: PiercingRemaining is 0 but Piercing is 1
        let bolt = app
            .world_mut()
            .spawn((Bolt, Piercing(1), PiercingRemaining(0)))
            .id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::Piercing(1)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 2);
        let pr = app.world().entity(bolt).get::<PiercingRemaining>().unwrap();
        assert_eq!(pr.0, 2);
    }

    #[test]
    fn max_stacks_leaves_piercing_remaining_unchanged() {
        let mut app = test_app();
        // At cap: Piercing(3) with per_stack=1 and max_stacks=3
        let bolt = app
            .world_mut()
            .spawn((Bolt, Piercing(3), PiercingRemaining(1)))
            .id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::Piercing(1)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 3);
        let pr = app.world().entity(bolt).get::<PiercingRemaining>().unwrap();
        assert_eq!(pr.0, 1);
    }

    #[test]
    fn non_piercing_effect_does_not_insert_piercing_remaining() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::DamageBoost(1.5)),
            max_stacks: 2,
        });
        app.world_mut().flush();

        assert!(app.world().entity(bolt).get::<Piercing>().is_none());
        assert!(
            app.world()
                .entity(bolt)
                .get::<PiercingRemaining>()
                .is_none()
        );
    }
}
