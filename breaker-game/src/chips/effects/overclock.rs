//! Overclock chip effect observer — pushes overclock trigger chains into
//! `ActiveOverclocks`.

use bevy::prelude::*;

use crate::{
    bolt::behaviors::ActiveOverclocks,
    chips::definition::{ChipEffect, ChipEffectApplied},
};

/// Observer: adds overclock trigger chains to `ActiveOverclocks` when a
/// `ChipEffectApplied` with an `Overclock` effect is observed.
///
/// Self-selects via pattern matching — ignores `Amp` and `Augment` effects.
pub(crate) fn handle_overclock(
    trigger: On<ChipEffectApplied>,
    mut active: ResMut<ActiveOverclocks>,
) {
    if let ChipEffect::Overclock(chain) = &trigger.event().effect {
        active.0.push(chain.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::definition::{AmpEffect, ChipEffect, TriggerChain};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ActiveOverclocks>()
            .add_observer(handle_overclock);
        app
    }

    #[test]
    fn handle_overclock_pushes_chain_to_active() {
        let mut app = test_app();
        let chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::OnImpact(Box::new(
            TriggerChain::Shockwave { range: 64.0 },
        ))));

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Overclock(chain.clone()),
            max_stacks: 1,
        });
        app.world_mut().flush();

        let active = app.world().resource::<ActiveOverclocks>();
        assert_eq!(
            active.0.len(),
            1,
            "handle_overclock should push the chain into ActiveOverclocks"
        );
        assert_eq!(active.0[0], chain);
    }

    #[test]
    fn handle_overclock_ignores_amp() {
        let mut app = test_app();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::Piercing(1)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let active = app.world().resource::<ActiveOverclocks>();
        assert!(
            active.0.is_empty(),
            "handle_overclock should ignore Amp effects"
        );
    }
}
