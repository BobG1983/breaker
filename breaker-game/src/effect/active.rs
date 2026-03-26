//! `ActiveEffects` resource — runtime list of active trigger chains.

use bevy::prelude::*;

use crate::effect::definition::EffectNode;

/// All trigger chains currently active for the run.
///
/// Populated by `init_breaker` from the breaker definition and by
/// `dispatch_chip_effects` when a chip with a triggered chain is selected.
/// Each entry is `(chip_name, chain)` where `chip_name` is `None` for
/// breaker-originating chains and `Some(name)` for chip/evolution chains.
#[derive(Resource, Debug, Default)]
pub struct ActiveEffects(pub Vec<(Option<String>, EffectNode)>);

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
    // after migration. They exercise evaluate_node.
    // =========================================================================

    #[test]
    fn effect_node_for_active_effects_with_chip_name() {
        use super::super::evaluate::{NodeEvalResult, evaluate_node};

        // Verify the shape of what ActiveEffects stores:
        // (Some("Surge"), EffectNode::When { trigger: PerfectBump, then: [Do(Shockwave)] })
        let chip_name: Option<String> = Some("Surge".to_owned());
        let node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        assert_eq!(chip_name, Some("Surge".to_owned()));
        assert!(matches!(
            &node,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                ..
            }
        ));
        // Verify evaluate_node works with this node
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn effect_node_for_active_effects_breaker_chain_none_chip_name() {
        use super::super::evaluate::{NodeEvalResult, evaluate_node};

        // Breaker chains have None chip name
        let chip_name: Option<String> = None;
        let node = EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife);
        assert!(chip_name.is_none());
        // Verify evaluate_node works
        let result = evaluate_node(Trigger::BoltLost, &node);
        assert_eq!(result, vec![NodeEvalResult::Fire(Effect::LoseLife)]);
    }
}
