//! Chain hit chip effect observer — bolt chains to N additional cells on hit.

use bevy::prelude::*;

use super::stack_u32;
use crate::{
    bolt::components::Bolt,
    chips::{
        components::ChainHit,
        definition::{AmpEffect, ChipEffect, ChipEffectApplied},
    },
};

/// Observer: applies chain hit stacking to all bolt entities.
pub(crate) fn handle_chain_hit(
    trigger: On<ChipEffectApplied>,
    mut query: Query<(Entity, Option<&mut ChainHit>), With<Bolt>>,
    mut commands: Commands,
) {
    let ChipEffect::Amp(AmpEffect::ChainHit(per_stack)) = trigger.event().effect.clone() else {
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
            ChainHit,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_chain_hit);
        app
    }

    #[test]
    fn inserts_chain_hit_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::ChainHit(2)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let c = app.world().entity(bolt).get::<ChainHit>().unwrap();
        assert_eq!(c.0, 2);
    }

    #[test]
    fn stacks_chain_hit() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, ChainHit(2))).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::ChainHit(2)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let c = app.world().entity(bolt).get::<ChainHit>().unwrap();
        assert_eq!(c.0, 4, "ChainHit should stack from 2 to 4");
    }

    #[test]
    fn respects_max_stacks_chain_hit() {
        let mut app = test_app();
        // 3 stacks of 2 = 6 (at cap)
        let bolt = app.world_mut().spawn((Bolt, ChainHit(6))).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::ChainHit(2)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let c = app.world().entity(bolt).get::<ChainHit>().unwrap();
        assert_eq!(
            c.0, 6,
            "ChainHit should not exceed max_stacks cap, got {}",
            c.0
        );
    }
}
