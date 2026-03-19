//! Piercing chip effect observer — bolt passes through N cells.

use bevy::prelude::*;

use super::stack_u32;
use crate::{
    bolt::components::Bolt,
    chips::{
        components::Piercing,
        definition::{AmpEffect, ChipEffect},
        messages::ChipEffectApplied,
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
        stack_u32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            Piercing,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::definition::AmpEffect;

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
}
