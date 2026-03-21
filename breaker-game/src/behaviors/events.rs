//! `EffectFired` event — fired when a unified trigger chain resolves to a leaf.

use bevy::prelude::*;

use crate::chips::definition::TriggerChain;

/// Fired when a unified trigger chain fully resolves to a leaf effect.
///
/// Consumed by per-effect observers (`shockwave`, `life_lost`, `time_penalty`, etc.).
#[derive(Event, Clone, Debug)]
pub(crate) struct EffectFired {
    /// The leaf effect to execute.
    pub effect: TriggerChain,
    /// The bolt entity that triggered the effect, or `None` for global triggers
    /// (cell destroyed, bolt lost) that have no specific bolt.
    pub bolt: Option<Entity>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_fired_with_some_bolt() {
        let event = EffectFired {
            effect: TriggerChain::test_shockwave(64.0),
            bolt: Some(Entity::PLACEHOLDER),
        };
        assert_eq!(event.bolt, Some(Entity::PLACEHOLDER));
        assert_eq!(
            event.effect,
            TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
            }
        );
    }

    #[test]
    fn effect_fired_with_none_bolt() {
        let event = EffectFired {
            effect: TriggerChain::test_lose_life(),
            bolt: None,
        };
        assert_eq!(event.bolt, None);
        assert_eq!(event.effect, TriggerChain::LoseLife);
    }
}
