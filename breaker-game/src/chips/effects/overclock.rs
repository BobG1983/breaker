//! Overclock chip effect observer — pushes overclock trigger chains into
//! `ActiveChains`.

use bevy::prelude::*;

use crate::{
    behaviors::ActiveChains,
    chips::definition::{ChipEffect, ChipEffectApplied, ImpactTarget},
};

/// Observer: adds overclock trigger chains to `ActiveChains` when a
/// `ChipEffectApplied` with an `Overclock` effect is observed.
///
/// Self-selects via pattern matching — ignores `Amp` and `Augment` effects.
pub(crate) fn handle_overclock(trigger: On<ChipEffectApplied>, mut active: ResMut<ActiveChains>) {
    let event = trigger.event();
    if let ChipEffect::Overclock(chain) = &event.effect {
        active
            .0
            .push((Some(event.chip_name.clone()), chain.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::definition::{AmpEffect, ChipEffect, TriggerChain};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ActiveChains>()
            .add_observer(handle_overclock);
        app
    }

    #[test]
    fn handle_overclock_pushes_chain_to_active() {
        let mut app = test_app();
        let chain = TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
            ImpactTarget::Cell,
            vec![TriggerChain::test_shockwave(64.0)],
        )]);

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Overclock(chain.clone()),
            max_stacks: 1,
            chip_name: "test_chip".to_owned(),
        });
        app.world_mut().flush();

        let active = app.world().resource::<ActiveChains>();
        assert_eq!(
            active.0.len(),
            1,
            "handle_overclock should push the chain into ActiveChains"
        );
        assert_eq!(active.0[0].1, chain);
    }

    #[test]
    fn handle_overclock_ignores_amp() {
        let mut app = test_app();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::Piercing(1)),
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let active = app.world().resource::<ActiveChains>();
        assert!(
            active.0.is_empty(),
            "handle_overclock should ignore Amp effects"
        );
    }
}
