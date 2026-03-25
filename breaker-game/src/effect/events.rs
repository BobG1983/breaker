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
    /// The chip name that originated this chain, or `None` for archetype chains.
    pub source_chip: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::Effect;

    #[test]
    fn effect_fired_with_some_bolt() {
        let event = EffectFired {
            effect: TriggerChain::test_shockwave(64.0),
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert_eq!(event.bolt, Some(Entity::PLACEHOLDER));
        assert_eq!(
            event.effect,
            TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            }
        );
    }

    #[test]
    fn effect_fired_with_none_bolt() {
        let event = EffectFired {
            effect: TriggerChain::test_lose_life(),
            bolt: None,
            source_chip: None,
        };
        assert_eq!(event.bolt, None);
        assert_eq!(event.effect, TriggerChain::LoseLife);
    }

    // =========================================================================
    // B12b: EffectFired should carry Effect (not TriggerChain) (behavior 17)
    // These tests verify the Effect type matches what EffectFired will carry
    // after migration. They exercise evaluate_node which fails with todo!().
    // =========================================================================

    #[test]
    fn effect_type_matches_shockwave_for_effect_fired() {
        use super::super::{
            definition::{EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        // Verify Effect::Shockwave has the same shape as TriggerChain::Shockwave
        // so it can replace TriggerChain in EffectFired.effect after migration.
        let effect = Effect::Shockwave {
            base_range: 64.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
        };
        // After migration: EffectFired { effect, bolt, source_chip }
        // For now, verify Effect equality works (used by handlers for matching)
        assert_eq!(
            effect,
            Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            }
        );
        // This assertion will fail — evaluate_node fires Effect, not TriggerChain
        let node = EffectNode::Trigger(Trigger::OnBoltLost, vec![EffectNode::Leaf(effect.clone())]);
        let result = evaluate_node(TriggerKind::BoltLost, &node);
        assert_eq!(result, vec![NodeEvalResult::Fire(effect)]);
    }

    #[test]
    fn effect_type_lose_life_for_effect_fired_with_source_chip() {
        use super::super::{
            definition::{EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        // Verify Effect::LoseLife exists and works with source_chip pattern
        let effect = Effect::LoseLife;
        let source = Some("Aegis".to_owned());
        // After migration: EffectFired { effect: Effect::LoseLife, bolt: None, source_chip }
        assert_eq!(effect, Effect::LoseLife);
        assert_eq!(source, Some("Aegis".to_owned()));
        // This assertion will fail — evaluate_node not implemented
        let node = EffectNode::trigger_leaf(Trigger::OnBoltLost, effect.clone());
        let result = evaluate_node(TriggerKind::BoltLost, &node);
        assert_eq!(result, vec![NodeEvalResult::Fire(effect)]);
    }
}
