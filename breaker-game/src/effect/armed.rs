//! `ArmedEffects` component for the unified effect system.

use bevy::prelude::*;

use crate::effect::definition::EffectNode;

/// Partially-resolved trigger chains attached to a specific bolt entity.
///
/// When a trigger chain matches but the inner chain is not a leaf,
/// the inner chain is pushed onto this component's list. Subsequent
/// triggers evaluate against these armed chains.
/// Each entry is `(chip_name, chain)` where `chip_name` is `None` for
/// breaker-originating chains and `Some(name)` for chip/evolution chains.
#[derive(Component, Debug, Default)]
pub(crate) struct ArmedEffects(pub Vec<(Option<String>, EffectNode)>);

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
    // after migration. They exercise evaluate_node.
    // =========================================================================

    #[test]
    fn effect_node_for_armed_effects_impact_trigger() {
        use super::super::evaluate::evaluate_node;

        // Verify the shape of what ArmedEffects stores:
        // (None, EffectNode::When { trigger: OnImpact(Cell), then: [Do(Shockwave)] })
        let node = EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Cell),
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        assert!(matches!(
            &node,
            EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                ..
            }
        ));
        // Verify evaluate_node resolves this armed trigger
        let result = evaluate_node(Trigger::Impact(ImpactTarget::Cell), &node);
        assert_eq!(
            result,
            Some(
                vec![EffectNode::Do(Effect::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                })]
                .as_slice()
            )
        );
    }
}
