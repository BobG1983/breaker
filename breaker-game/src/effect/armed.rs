//! `ArmedEffects` component for the unified effect system.

use bevy::prelude::*;

use crate::chips::definition::TriggerChain;

/// Partially-resolved trigger chains attached to a specific bolt entity.
///
/// When a trigger chain matches but the inner chain is not a leaf,
/// the inner chain is pushed onto this component's list. Subsequent
/// triggers evaluate against these armed chains.
/// Each entry is `(chip_name, chain)` where `chip_name` is `None` for
/// breaker-originating chains and `Some(name)` for chip/evolution chains.
#[derive(Component, Debug, Default)]
pub(crate) struct ArmedEffects(pub Vec<(Option<String>, TriggerChain)>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::{Effect, EffectNode, ImpactTarget, Trigger};

    #[test]
    fn armed_effects_default_is_empty() {
        let armed = ArmedEffects::default();
        assert!(
            armed.0.is_empty(),
            "ArmedEffects::default() should produce an empty vec"
        );
    }

    // =========================================================================
    // B12b: ArmedEffects should store (Option<String>, EffectNode) (behavior 16)
    // These tests verify the EffectNode types that ArmedEffects will hold
    // after migration. They exercise evaluate_node which fails with todo!().
    // =========================================================================

    #[test]
    fn effect_node_for_armed_effects_impact_trigger() {
        use super::super::evaluate::{NodeEvalResult, TriggerKind, evaluate_node};

        // Verify the shape of what ArmedEffects will store after migration:
        // (None, EffectNode::Trigger(OnImpact(Cell), [Leaf(Shockwave {...})]))
        let node = EffectNode::Trigger(
            Trigger::OnImpact(ImpactTarget::Cell),
            vec![EffectNode::Leaf(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        );
        assert!(matches!(
            &node,
            EffectNode::Trigger(Trigger::OnImpact(ImpactTarget::Cell), _)
        ));
        // Verify evaluate_node resolves this armed trigger (fails with todo!)
        let result = evaluate_node(TriggerKind::CellImpact, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })]
        );
    }
}
