//! Pure evaluation function — maps a trigger kind + chain to an `EvalResult`.
//!
//! Unified version covering all trigger kinds including bump grades
//! (`EarlyBump`, `LateBump`, `BumpWhiff`).

use super::definition::{Effect, EffectNode, ImpactTarget, Trigger};
use crate::chips::definition::TriggerChain;

/// The kind of trigger event that occurred at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TriggerKind {
    /// A perfect bump was performed.
    PerfectBump,
    /// Any non-whiff bump (Early, Late, or Perfect).
    BumpSuccess,
    /// Bump pressed before the perfect zone.
    EarlyBump,
    /// Bump pressed after the bolt hit.
    LateBump,
    /// Forward bump window expired without bolt contact.
    BumpWhiff,
    /// A bolt hit a cell.
    CellImpact,
    /// A bolt bounced off the breaker.
    BreakerImpact,
    /// A bolt bounced off a wall.
    WallImpact,
    /// A cell was destroyed.
    CellDestroyed,
    /// A bolt was lost.
    BoltLost,
}

/// Result of evaluating a trigger kind against a trigger chain.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EvalResult {
    /// The trigger kind does not match the chain's outermost trigger.
    NoMatch,
    /// The trigger matched, and the inner chain should be armed on the bolt.
    Arm(TriggerChain),
    /// The trigger matched and the inner chain is a leaf — fire immediately.
    Fire(TriggerChain),
}

/// Evaluates whether a runtime trigger event matches the outermost trigger
/// of a `TriggerChain`.
///
/// Returns a `Vec<EvalResult>` with one entry per inner effect in the
/// matched trigger variant's `Vec<TriggerChain>`. Returns `vec![NoMatch]`
/// if the chain is a leaf (no trigger wrapper) or if the trigger kind
/// doesn't match the chain's outermost trigger wrapper.
///
/// Each inner effect produces `Fire(inner)` if it is a leaf, or
/// `Arm(inner)` if it is another trigger wrapper (needs further resolution).
pub(crate) fn evaluate(trigger: TriggerKind, chain: &TriggerChain) -> Vec<EvalResult> {
    let ((TriggerKind::PerfectBump, TriggerChain::OnPerfectBump(effects))
    | (TriggerKind::CellImpact, TriggerChain::OnImpact(ImpactTarget::Cell, effects))
    | (TriggerKind::BreakerImpact, TriggerChain::OnImpact(ImpactTarget::Breaker, effects))
    | (TriggerKind::WallImpact, TriggerChain::OnImpact(ImpactTarget::Wall, effects))
    | (TriggerKind::BumpSuccess, TriggerChain::OnBump(effects))
    | (TriggerKind::CellDestroyed, TriggerChain::OnCellDestroyed(effects))
    | (TriggerKind::BoltLost, TriggerChain::OnBoltLost(effects))
    | (TriggerKind::EarlyBump, TriggerChain::OnEarlyBump(effects))
    | (TriggerKind::LateBump, TriggerChain::OnLateBump(effects))
    | (TriggerKind::BumpWhiff, TriggerChain::OnBumpWhiff(effects))) = (trigger, chain)
    else {
        return vec![EvalResult::NoMatch];
    };
    effects
        .iter()
        .map(|e| {
            if e.is_leaf() {
                EvalResult::Fire(e.clone())
            } else {
                EvalResult::Arm(e.clone())
            }
        })
        .collect()
}

/// Result of evaluating a trigger kind against an `EffectNode`.
///
/// Mirrors `EvalResult` but uses the split `Effect` / `EffectNode` types.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NodeEvalResult {
    /// The trigger kind does not match the node's trigger.
    NoMatch,
    /// The trigger matched and a child node should be armed on the bolt.
    Arm(EffectNode),
    /// The trigger matched and the child is a leaf — fire immediately.
    Fire(Effect),
}

/// Evaluates whether a runtime trigger event matches an [`EffectNode`].
///
/// Returns a `Vec<NodeEvalResult>` with one entry per child in the
/// matched trigger variant's children. Returns `vec![NoMatch]` if
/// the node is a bare `Leaf` (no trigger wrapper) or if the trigger
/// kind doesn't match the node's trigger.
///
/// Each child produces `Fire(effect)` if it is a `Leaf`, or `Arm(node)`
/// if it is another `Trigger` (needs further resolution).
///
/// [`Trigger::OnSelected`] has no runtime [`TriggerKind`] mapping and
/// always returns `vec![NoMatch]`.
pub(crate) fn evaluate_node(trigger: TriggerKind, node: &EffectNode) -> Vec<NodeEvalResult> {
    let EffectNode::Trigger(trigger_variant, children) = node else {
        // Bare leaf — no trigger wrapper to match against.
        return vec![NodeEvalResult::NoMatch];
    };

    // OnSelected is a passive trigger — never matches any runtime TriggerKind.
    if !matches!(
        (trigger, trigger_variant),
        (TriggerKind::PerfectBump, Trigger::OnPerfectBump)
            | (TriggerKind::BumpSuccess, Trigger::OnBump)
            | (TriggerKind::EarlyBump, Trigger::OnEarlyBump)
            | (TriggerKind::LateBump, Trigger::OnLateBump)
            | (TriggerKind::BumpWhiff, Trigger::OnBumpWhiff)
            | (
                TriggerKind::CellImpact,
                Trigger::OnImpact(ImpactTarget::Cell)
            )
            | (
                TriggerKind::BreakerImpact,
                Trigger::OnImpact(ImpactTarget::Breaker)
            )
            | (
                TriggerKind::WallImpact,
                Trigger::OnImpact(ImpactTarget::Wall)
            )
            | (TriggerKind::CellDestroyed, Trigger::OnCellDestroyed)
            | (TriggerKind::BoltLost, Trigger::OnBoltLost)
    ) {
        return vec![NodeEvalResult::NoMatch];
    }

    children
        .iter()
        .map(|child| match child {
            EffectNode::Leaf(effect) => NodeEvalResult::Fire(effect.clone()),
            EffectNode::Trigger(..) => NodeEvalResult::Arm(child.clone()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::ImpactTarget;

    // --- Fire tests: trigger matches and inner chain is a leaf ---

    #[test]
    fn perfect_bump_with_on_perfect_bump_leaf_fires() {
        let chain = TriggerChain::OnPerfectBump(vec![TriggerChain::test_shockwave(64.0)]);
        let result = evaluate(TriggerKind::PerfectBump, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Fire(TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
            "PerfectBump should match OnPerfectBump(leaf) and fire"
        );
    }

    #[test]
    fn early_bump_with_on_early_bump_lose_life_fires() {
        let chain = TriggerChain::OnEarlyBump(vec![TriggerChain::test_lose_life()]);
        let result = evaluate(TriggerKind::EarlyBump, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Fire(TriggerChain::LoseLife)],
            "EarlyBump should match OnEarlyBump(LoseLife) and fire"
        );
    }

    #[test]
    fn late_bump_with_on_late_bump_time_penalty_fires() {
        let chain = TriggerChain::OnLateBump(vec![TriggerChain::test_time_penalty(3.0)]);
        let result = evaluate(TriggerKind::LateBump, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Fire(TriggerChain::TimePenalty { seconds: 3.0 })],
            "LateBump should match OnLateBump(TimePenalty) and fire"
        );
    }

    #[test]
    fn bump_whiff_with_on_bump_whiff_lose_life_fires() {
        let chain = TriggerChain::OnBumpWhiff(vec![TriggerChain::test_lose_life()]);
        let result = evaluate(TriggerKind::BumpWhiff, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Fire(TriggerChain::LoseLife)],
            "BumpWhiff should match OnBumpWhiff(LoseLife) and fire"
        );
    }

    #[test]
    fn bump_success_with_on_bump_leaf_fires() {
        let chain = TriggerChain::OnBump(vec![TriggerChain::test_shield(3.0)]);
        let result = evaluate(TriggerKind::BumpSuccess, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Fire(TriggerChain::Shield {
                base_duration: 3.0,
                duration_per_level: 0.0,
                stacks: 1,
            })],
            "BumpSuccess should match OnBump(leaf) and fire"
        );
    }

    #[test]
    fn cell_impact_with_on_impact_cell_leaf_fires() {
        let chain =
            TriggerChain::OnImpact(ImpactTarget::Cell, vec![TriggerChain::test_shockwave(64.0)]);
        let result = evaluate(TriggerKind::CellImpact, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Fire(TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
            "CellImpact should match OnImpact(Cell, leaf) and fire"
        );
    }

    #[test]
    fn breaker_impact_with_on_impact_breaker_leaf_fires() {
        let chain = TriggerChain::OnImpact(
            ImpactTarget::Breaker,
            vec![TriggerChain::test_multi_bolt(2)],
        );
        let result = evaluate(TriggerKind::BreakerImpact, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Fire(TriggerChain::MultiBolt {
                base_count: 2,
                count_per_level: 0,
                stacks: 1,
            })],
            "BreakerImpact should match OnImpact(Breaker, leaf) and fire"
        );
    }

    #[test]
    fn wall_impact_with_on_impact_wall_leaf_fires() {
        let chain =
            TriggerChain::OnImpact(ImpactTarget::Wall, vec![TriggerChain::test_shield(5.0)]);
        let result = evaluate(TriggerKind::WallImpact, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Fire(TriggerChain::Shield {
                base_duration: 5.0,
                duration_per_level: 0.0,
                stacks: 1,
            })],
            "WallImpact should match OnImpact(Wall, leaf) and fire"
        );
    }

    #[test]
    fn cell_destroyed_with_on_cell_destroyed_leaf_fires() {
        let chain = TriggerChain::OnCellDestroyed(vec![TriggerChain::test_shield(5.0)]);
        let result = evaluate(TriggerKind::CellDestroyed, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Fire(TriggerChain::Shield {
                base_duration: 5.0,
                duration_per_level: 0.0,
                stacks: 1,
            })],
            "CellDestroyed should match OnCellDestroyed(leaf) and fire"
        );
    }

    #[test]
    fn bolt_lost_with_on_bolt_lost_leaf_fires() {
        let chain = TriggerChain::OnBoltLost(vec![TriggerChain::test_shockwave(32.0)]);
        let result = evaluate(TriggerKind::BoltLost, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Fire(TriggerChain::Shockwave {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
            "BoltLost should match OnBoltLost(leaf) and fire"
        );
    }

    // --- Arm tests: trigger matches but inner chain is not a leaf ---

    #[test]
    fn perfect_bump_with_on_perfect_bump_non_leaf_arms() {
        let inner_chain =
            TriggerChain::OnImpact(ImpactTarget::Cell, vec![TriggerChain::test_shockwave(64.0)]);
        let chain = TriggerChain::OnPerfectBump(vec![inner_chain.clone()]);
        let result = evaluate(TriggerKind::PerfectBump, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Arm(inner_chain)],
            "PerfectBump with non-leaf inner should return Arm"
        );
    }

    #[test]
    fn cell_impact_with_on_impact_cell_non_leaf_arms() {
        let chain = TriggerChain::OnImpact(
            ImpactTarget::Cell,
            vec![TriggerChain::OnCellDestroyed(vec![
                TriggerChain::test_shockwave(32.0),
            ])],
        );
        let result = evaluate(TriggerKind::CellImpact, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Arm(TriggerChain::OnCellDestroyed(vec![
                TriggerChain::test_shockwave(32.0)
            ]))],
            "CellImpact with non-leaf inner should return Arm with inner chain"
        );
    }

    #[test]
    fn early_bump_with_on_early_bump_non_leaf_arms() {
        let inner_chain =
            TriggerChain::OnImpact(ImpactTarget::Cell, vec![TriggerChain::test_shockwave(64.0)]);
        let chain = TriggerChain::OnEarlyBump(vec![inner_chain.clone()]);
        let result = evaluate(TriggerKind::EarlyBump, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Arm(inner_chain)],
            "EarlyBump with non-leaf inner should return Arm"
        );
    }

    #[test]
    fn three_deep_chain_returns_arm_with_two_deep_remaining() {
        let chain = TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
            ImpactTarget::Cell,
            vec![TriggerChain::OnCellDestroyed(vec![
                TriggerChain::test_shockwave(64.0),
            ])],
        )]);
        let result = evaluate(TriggerKind::PerfectBump, &chain);
        assert_eq!(
            result,
            vec![EvalResult::Arm(TriggerChain::OnImpact(
                ImpactTarget::Cell,
                vec![TriggerChain::OnCellDestroyed(vec![
                    TriggerChain::test_shockwave(64.0),
                ])],
            ))],
            "3-deep chain should peel off outermost trigger and return Arm with 2-deep remaining"
        );
    }

    // --- NoMatch tests: trigger kind does not match chain's outermost trigger ---

    #[test]
    fn perfect_bump_does_not_match_on_impact() {
        let chain =
            TriggerChain::OnImpact(ImpactTarget::Cell, vec![TriggerChain::test_shockwave(64.0)]);
        let result = evaluate(TriggerKind::PerfectBump, &chain);
        assert_eq!(
            result,
            vec![EvalResult::NoMatch],
            "PerfectBump should not match OnImpact -- wrong trigger kind"
        );
    }

    #[test]
    fn early_bump_does_not_match_on_perfect_bump() {
        let chain = TriggerChain::OnPerfectBump(vec![TriggerChain::test_shockwave(64.0)]);
        let result = evaluate(TriggerKind::EarlyBump, &chain);
        assert_eq!(
            result,
            vec![EvalResult::NoMatch],
            "EarlyBump should not match OnPerfectBump -- distinct trigger kinds"
        );
    }

    #[test]
    fn cell_impact_does_not_match_on_impact_breaker() {
        let chain = TriggerChain::OnImpact(
            ImpactTarget::Breaker,
            vec![TriggerChain::test_shockwave(64.0)],
        );
        let result = evaluate(TriggerKind::CellImpact, &chain);
        assert_eq!(
            result,
            vec![EvalResult::NoMatch],
            "CellImpact must NOT match OnImpact(Breaker, ...) -- ImpactTarget discrimination required"
        );
    }

    #[test]
    fn bump_success_does_not_match_on_perfect_bump() {
        let chain = TriggerChain::OnPerfectBump(vec![TriggerChain::test_shockwave(64.0)]);
        let result = evaluate(TriggerKind::BumpSuccess, &chain);
        assert_eq!(
            result,
            vec![EvalResult::NoMatch],
            "BumpSuccess must NOT match OnPerfectBump -- distinct trigger kinds"
        );
    }

    #[test]
    fn perfect_bump_does_not_match_on_bump() {
        let chain = TriggerChain::OnBump(vec![TriggerChain::test_shield(3.0)]);
        let result = evaluate(TriggerKind::PerfectBump, &chain);
        assert_eq!(
            result,
            vec![EvalResult::NoMatch],
            "PerfectBump should not match OnBump -- distinct trigger kinds"
        );
    }

    #[test]
    fn bare_leaf_returns_no_match() {
        let chain = TriggerChain::test_shockwave(64.0);
        let result = evaluate(TriggerKind::PerfectBump, &chain);
        assert_eq!(
            result,
            vec![EvalResult::NoMatch],
            "bare leaf (not wrapped in a trigger) should return NoMatch"
        );
    }

    // =========================================================================
    // B12b: evaluate_node with EffectNode (behaviors 10-14)
    // =========================================================================

    #[test]
    fn evaluate_node_perfect_bump_matching_leaf_fires() {
        let node = EffectNode::Trigger(
            Trigger::OnPerfectBump,
            vec![EffectNode::Leaf(Effect::test_shockwave(64.0))],
        );
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
            "PerfectBump should match OnPerfectBump(Leaf) and fire"
        );
    }

    #[test]
    fn evaluate_node_non_matching_trigger_returns_no_match() {
        let node = EffectNode::Trigger(
            Trigger::OnPerfectBump,
            vec![EffectNode::Leaf(Effect::test_shockwave(64.0))],
        );
        let result = evaluate_node(TriggerKind::BoltLost, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "BoltLost should not match OnPerfectBump"
        );
    }

    #[test]
    fn evaluate_node_trigger_wrapping_trigger_returns_arm() {
        let inner = EffectNode::Trigger(
            Trigger::OnImpact(super::super::definition::ImpactTarget::Cell),
            vec![EffectNode::Leaf(Effect::test_shockwave(64.0))],
        );
        let node = EffectNode::Trigger(Trigger::OnPerfectBump, vec![inner.clone()]);
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Arm(inner)],
            "PerfectBump with inner Trigger child should Arm"
        );
    }

    #[test]
    fn evaluate_node_bare_leaf_returns_no_match() {
        let node = EffectNode::Leaf(Effect::test_shockwave(64.0));
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "bare Leaf node should return NoMatch"
        );
    }

    #[test]
    fn evaluate_node_all_trigger_kinds_match_correctly() {
        use super::super::definition::ImpactTarget as IT;
        let pairs: Vec<(TriggerKind, Trigger)> = vec![
            (TriggerKind::PerfectBump, Trigger::OnPerfectBump),
            (TriggerKind::BumpSuccess, Trigger::OnBump),
            (TriggerKind::EarlyBump, Trigger::OnEarlyBump),
            (TriggerKind::LateBump, Trigger::OnLateBump),
            (TriggerKind::BumpWhiff, Trigger::OnBumpWhiff),
            (TriggerKind::CellImpact, Trigger::OnImpact(IT::Cell)),
            (TriggerKind::BreakerImpact, Trigger::OnImpact(IT::Breaker)),
            (TriggerKind::WallImpact, Trigger::OnImpact(IT::Wall)),
            (TriggerKind::CellDestroyed, Trigger::OnCellDestroyed),
            (TriggerKind::BoltLost, Trigger::OnBoltLost),
        ];
        let leaf = Effect::test_shockwave(64.0);
        for (kind, trigger) in &pairs {
            let node = EffectNode::Trigger(*trigger, vec![EffectNode::Leaf(leaf.clone())]);
            let result = evaluate_node(*kind, &node);
            assert_eq!(
                result,
                vec![NodeEvalResult::Fire(leaf.clone())],
                "TriggerKind::{kind:?} should match Trigger::{trigger:?}"
            );
        }
    }

    #[test]
    fn evaluate_node_cell_impact_does_not_match_breaker_impact_target() {
        let node = EffectNode::Trigger(
            Trigger::OnImpact(super::super::definition::ImpactTarget::Breaker),
            vec![EffectNode::Leaf(Effect::test_shockwave(64.0))],
        );
        let result = evaluate_node(TriggerKind::CellImpact, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "CellImpact must NOT match OnImpact(Breaker)"
        );
    }

    #[test]
    fn evaluate_node_on_selected_always_returns_no_match() {
        let node = EffectNode::Trigger(
            Trigger::OnSelected,
            vec![EffectNode::Leaf(Effect::Piercing(1))],
        );
        // Test against every TriggerKind — OnSelected should never match
        let kinds = [
            TriggerKind::PerfectBump,
            TriggerKind::BumpSuccess,
            TriggerKind::EarlyBump,
            TriggerKind::LateBump,
            TriggerKind::BumpWhiff,
            TriggerKind::CellImpact,
            TriggerKind::BreakerImpact,
            TriggerKind::WallImpact,
            TriggerKind::CellDestroyed,
            TriggerKind::BoltLost,
        ];
        for kind in &kinds {
            let result = evaluate_node(*kind, &node);
            assert_eq!(
                result,
                vec![NodeEvalResult::NoMatch],
                "OnSelected should NEVER match {kind:?}"
            );
        }
    }

    #[test]
    fn evaluate_node_multiple_children_returns_multiple_results() {
        let node = EffectNode::Trigger(
            Trigger::OnBump,
            vec![
                EffectNode::Leaf(Effect::SpawnBolt),
                EffectNode::Leaf(Effect::SpeedBoost {
                    target: super::super::definition::Target::Bolt,
                    multiplier: 1.2,
                }),
            ],
        );
        let result = evaluate_node(TriggerKind::BumpSuccess, &node);
        assert_eq!(result.len(), 2, "should return 2 Fire results for 2 leaves");
        assert!(matches!(result[0], NodeEvalResult::Fire(Effect::SpawnBolt)));
        assert!(matches!(
            result[1],
            NodeEvalResult::Fire(Effect::SpeedBoost { .. })
        ));
    }
}
