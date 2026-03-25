//! `ActiveEffects` resource — runtime list of active trigger chains.

use bevy::prelude::*;

use crate::chips::definition::TriggerChain;

/// All trigger chains currently active for the run.
///
/// Populated by `init_breaker` from the breaker definition and by
/// `dispatch_chip_effects` when a chip with a triggered chain is selected.
/// Each entry is `(chip_name, chain)` where `chip_name` is `None` for
/// breaker-originating chains and `Some(name)` for chip/evolution chains.
#[derive(Resource, Debug, Default)]
pub struct ActiveEffects(pub Vec<(Option<String>, TriggerChain)>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::{Effect, EffectNode, Trigger};

    #[test]
    fn active_effects_default_is_empty() {
        let active = ActiveEffects::default();
        assert!(
            active.0.is_empty(),
            "ActiveEffects::default() should produce an empty vec"
        );
    }

    // =========================================================================
    // B12b: ActiveEffects should store (Option<String>, EffectNode) (behavior 15)
    // These tests verify the EffectNode types that ActiveEffects will hold
    // after migration. They exercise evaluate_node which fails with todo!().
    // =========================================================================

    #[test]
    fn effect_node_for_active_effects_with_chip_name() {
        use super::super::evaluate::{NodeEvalResult, TriggerKind, evaluate_node};

        // Verify the shape of what ActiveEffects will store after migration:
        // (Some("Surge"), EffectNode::Trigger(OnPerfectBump, [Leaf(Shockwave {...})]))
        let chip_name: Option<String> = Some("Surge".to_owned());
        let node = EffectNode::Trigger(
            Trigger::OnPerfectBump,
            vec![EffectNode::Leaf(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        );
        assert_eq!(chip_name, Some("Surge".to_owned()));
        assert!(matches!(
            &node,
            EffectNode::Trigger(Trigger::OnPerfectBump, _)
        ));
        // Verify evaluate_node works with this node (fails with todo!)
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn effect_node_for_active_effects_breaker_chain_none_chip_name() {
        use super::super::evaluate::{NodeEvalResult, TriggerKind, evaluate_node};

        // Breaker chains have None chip name
        let chip_name: Option<String> = None;
        let node = EffectNode::trigger_leaf(Trigger::OnBoltLost, Effect::LoseLife);
        assert!(chip_name.is_none());
        // Verify evaluate_node works (fails with todo!)
        let result = evaluate_node(TriggerKind::BoltLost, &node);
        assert_eq!(result, vec![NodeEvalResult::Fire(Effect::LoseLife)]);
    }
}
