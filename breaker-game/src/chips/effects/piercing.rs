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
    let ChipEffect::Amp(AmpEffect::Piercing(per_stack)) = trigger.event().effect.clone() else {
        return;
    };
    let max_stacks = trigger.event().max_stacks;
    for (entity, mut existing) in &mut query {
        let had_piercing = existing.is_some();
        let piercing_before = existing.as_deref().map(|p| p.0);
        stack_u32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            Piercing,
        );
        if had_piercing {
            let piercing_after = existing.as_deref().map(|p| p.0);
            if piercing_after != piercing_before
                && let Some(stacked_count) = piercing_after
            {
                commands
                    .entity(entity)
                    .insert(PiercingRemaining(stacked_count));
            }
        } else {
            // stack_u32 inserted Piercing(per_stack) via deferred commands; mirror it
            commands.entity(entity).insert(PiercingRemaining(per_stack));
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
    fn stacking_with_partial_remaining_refreshes_remaining() {
        // Bolt has 1 pierce remaining out of 1 max. Stacking adds another stack.
        // PiercingRemaining should refresh to the new Piercing total (2).
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((Bolt, Piercing(1), PiercingRemaining(1)))
            .id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::Piercing(1)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 2, "Piercing should stack from 1 to 2");

        let pr = app.world().entity(bolt).get::<PiercingRemaining>().unwrap();
        assert_eq!(
            pr.0, 2,
            "PiercingRemaining should refresh to new Piercing total (2)"
        );
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
