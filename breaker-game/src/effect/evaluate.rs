//! Pure evaluation function — maps a trigger kind + `EffectNode` to a `NodeEvalResult`.
//!
//! Unified version covering all trigger kinds including bump grades
//! (`EarlyBump`, `LateBump`, `BumpWhiff`) and the new `Death` kind.

use super::definition::{Effect, EffectNode, ImpactTarget, Trigger};

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
    /// The breaker died (all lives lost or timer expired).
    Death,
}

/// Result of evaluating a trigger kind against an `EffectNode`.
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
/// the node is a bare `Do` (no trigger wrapper), `Until`, `Once`,
/// or if the trigger kind doesn't match the node's trigger.
///
/// Each child produces `Fire(effect)` if it is a `Do`, or `Arm(node)`
/// if it is another `When` (needs further resolution).
///
/// [`Trigger::OnSelected`] has no runtime [`TriggerKind`] mapping and
/// always returns `vec![NoMatch]`.
///
/// [`Trigger::TimeExpires`] has no runtime [`TriggerKind`] mapping
/// (it is timer-based removal, not a trigger event) and always returns
/// `vec![NoMatch]`.
pub(crate) fn evaluate_node(trigger: TriggerKind, node: &EffectNode) -> Vec<NodeEvalResult> {
    match node {
        EffectNode::When {
            trigger: node_trigger,
            then,
        } => {
            if trigger_matches(trigger, *node_trigger) {
                then.iter()
                    .map(|child| match child {
                        EffectNode::Do(effect) => NodeEvalResult::Fire(effect.clone()),
                        _ => NodeEvalResult::Arm(child.clone()),
                    })
                    .collect()
            } else {
                vec![NodeEvalResult::NoMatch]
            }
        }
        EffectNode::Do(_) | EffectNode::Until { .. } | EffectNode::Once(_) => {
            vec![NodeEvalResult::NoMatch]
        }
    }
}

/// Returns `true` if the runtime `TriggerKind` matches the declared `Trigger`.
///
/// `OnSelected` and `TimeExpires` have no runtime trigger mapping and always return `false`.
fn trigger_matches(kind: TriggerKind, trigger: Trigger) -> bool {
    matches!(
        (kind, trigger),
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
            | (TriggerKind::Death, Trigger::OnDeath)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::ImpactTarget;

    // =========================================================================
    // C7 Wave 1 Part F: evaluate_node for new EffectNode shape (behaviors 31-38)
    // =========================================================================

    #[test]
    fn evaluate_node_when_matching_trigger_fires_do_leaf() {
        let node = EffectNode::When {
            trigger: Trigger::OnPerfectBump,
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
            "PerfectBump should match When(OnPerfectBump) and fire Do leaf"
        );
    }

    #[test]
    fn evaluate_node_when_non_matching_trigger_returns_no_match() {
        let node = EffectNode::When {
            trigger: Trigger::OnPerfectBump,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(TriggerKind::BoltLost, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "BoltLost should not match When(OnPerfectBump)"
        );
    }

    #[test]
    fn evaluate_node_when_matching_arms_nested_when() {
        let inner = EffectNode::When {
            trigger: Trigger::OnImpact(ImpactTarget::Cell),
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        let node = EffectNode::When {
            trigger: Trigger::OnPerfectBump,
            then: vec![inner.clone()],
        };
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Arm(inner)],
            "PerfectBump with inner When child should Arm"
        );
    }

    #[test]
    fn evaluate_node_bare_do_returns_no_match() {
        let node = EffectNode::Do(Effect::LoseLife);
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "bare Do node should return NoMatch"
        );
    }

    #[test]
    fn evaluate_node_until_returns_no_match() {
        let node = EffectNode::Until {
            until: Trigger::TimeExpires(3.0),
            then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
        };
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "Until node should return NoMatch (not trigger-gated)"
        );
    }

    #[test]
    fn evaluate_node_once_returns_no_match() {
        let node = EffectNode::Once(vec![EffectNode::Do(Effect::SecondWind {
            invuln_secs: 3.0,
        })]);
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "Once node should return NoMatch (consumed externally)"
        );
    }

    #[test]
    fn evaluate_node_when_on_selected_always_returns_no_match() {
        let node = EffectNode::When {
            trigger: Trigger::OnSelected,
            then: vec![EffectNode::Do(Effect::Piercing(1))],
        };
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
            TriggerKind::Death,
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
    fn evaluate_node_when_time_expires_always_returns_no_match() {
        let node = EffectNode::When {
            trigger: Trigger::TimeExpires(3.0),
            then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
        };
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
            TriggerKind::Death,
        ];
        for kind in &kinds {
            let result = evaluate_node(*kind, &node);
            assert_eq!(
                result,
                vec![NodeEvalResult::NoMatch],
                "TimeExpires has no TriggerKind — should return NoMatch for {kind:?}"
            );
        }
    }

    #[test]
    fn evaluate_node_all_trigger_kinds_match_correctly() {
        use crate::effect::definition::ImpactTarget as IT;
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
            (TriggerKind::Death, Trigger::OnDeath),
        ];
        let leaf = Effect::test_shockwave(64.0);
        for (kind, trigger) in &pairs {
            let node = EffectNode::When {
                trigger: *trigger,
                then: vec![EffectNode::Do(leaf.clone())],
            };
            let result = evaluate_node(*kind, &node);
            assert_eq!(
                result,
                vec![NodeEvalResult::Fire(leaf.clone())],
                "TriggerKind::{kind:?} should match Trigger::{trigger:?}"
            );
        }
    }

    #[test]
    fn evaluate_node_when_multiple_do_children_returns_multiple_fire_results() {
        let node = EffectNode::When {
            trigger: Trigger::OnBump,
            then: vec![
                EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                }),
                EffectNode::Do(Effect::SpeedBoost {
                    target: super::super::definition::Target::Bolt,
                    multiplier: 1.2,
                }),
            ],
        };
        let result = evaluate_node(TriggerKind::BumpSuccess, &node);
        assert_eq!(
            result.len(),
            2,
            "should return 2 Fire results for 2 Do children"
        );
        assert!(matches!(
            result[0],
            NodeEvalResult::Fire(Effect::SpawnBolts { .. })
        ));
        assert!(matches!(
            result[1],
            NodeEvalResult::Fire(Effect::SpeedBoost { .. })
        ));
    }

    #[test]
    fn evaluate_node_cell_impact_does_not_match_breaker_impact_target() {
        let node = EffectNode::When {
            trigger: Trigger::OnImpact(ImpactTarget::Breaker),
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(TriggerKind::CellImpact, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "CellImpact must NOT match When(OnImpact(Breaker))"
        );
    }

    // =========================================================================
    // C7 Wave 1 Part K: TriggerKind extension (behaviors 49-50)
    // =========================================================================

    #[test]
    fn trigger_kind_death_exists() {
        let kind = TriggerKind::Death;
        assert_ne!(kind, TriggerKind::BoltLost);
    }

    #[test]
    fn trigger_kind_has_eleven_variants() {
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
            TriggerKind::Death,
        ];
        assert_eq!(kinds.len(), 11, "TriggerKind should have 11 variants");
    }
}
