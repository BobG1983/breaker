//! Pure evaluation function — maps a runtime `Trigger` + `EffectNode` to a `NodeEvalResult`.
//!
//! Unified version covering all trigger kinds including bump grades
//! (`EarlyBump`, `LateBump`, `BumpWhiff`) and `Death`.

use super::definition::{Effect, EffectNode, ImpactTarget, Trigger};

/// Result of evaluating a trigger against an `EffectNode`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NodeEvalResult {
    /// The trigger does not match the node's trigger.
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
/// or if the trigger doesn't match the node's trigger.
///
/// Each child produces `Fire(effect)` if it is a `Do`, or `Arm(node)`
/// if it is another `When` (needs further resolution).
///
/// [`Trigger::Selected`] has no runtime trigger mapping and
/// always returns `vec![NoMatch]`.
///
/// [`Trigger::TimeExpires`] has no runtime trigger mapping
/// (it is timer-based removal, not a trigger event) and always returns
/// `vec![NoMatch]`.
pub(crate) fn evaluate_node(trigger: Trigger, node: &EffectNode) -> Vec<NodeEvalResult> {
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
        EffectNode::Do(_)
        | EffectNode::Until { .. }
        | EffectNode::Once(_)
        | EffectNode::On { .. } => {
            vec![NodeEvalResult::NoMatch]
        }
    }
}

/// Returns `true` if the runtime trigger matches the declared trigger on the node.
///
/// `Selected`, `TimeExpires`, and `NodeTimerThreshold` have no runtime trigger mapping
/// and always return `false`.
fn trigger_matches(runtime: Trigger, declared: Trigger) -> bool {
    match (runtime, declared) {
        (Trigger::Impact(a), Trigger::Impact(b)) => a == b,
        (Trigger::PerfectBump, Trigger::PerfectBump)
        | (Trigger::Bump, Trigger::Bump)
        | (Trigger::EarlyBump, Trigger::EarlyBump)
        | (Trigger::LateBump, Trigger::LateBump)
        | (Trigger::BumpWhiff, Trigger::BumpWhiff)
        | (Trigger::CellDestroyed, Trigger::CellDestroyed)
        | (Trigger::BoltLost, Trigger::BoltLost)
        | (Trigger::Death, Trigger::Death)
        | (Trigger::NoBump, Trigger::NoBump)
        | (Trigger::PerfectBumped, Trigger::PerfectBumped)
        | (Trigger::Bumped, Trigger::Bumped)
        | (Trigger::EarlyBumped, Trigger::EarlyBumped)
        | (Trigger::LateBumped, Trigger::LateBumped) => true,
        _ => false,
    }
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
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
            "PerfectBump should match When(PerfectBump) and fire Do leaf"
        );
    }

    #[test]
    fn evaluate_node_when_non_matching_trigger_returns_no_match() {
        let node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::BoltLost, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "BoltLost should not match When(PerfectBump)"
        );
    }

    #[test]
    fn evaluate_node_when_matching_arms_nested_when() {
        let inner = EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Cell),
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        let node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![inner.clone()],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Arm(inner)],
            "PerfectBump with inner When child should Arm"
        );
    }

    #[test]
    fn evaluate_node_bare_do_returns_no_match() {
        let node = EffectNode::Do(Effect::LoseLife);
        let result = evaluate_node(Trigger::PerfectBump, &node);
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
        let result = evaluate_node(Trigger::PerfectBump, &node);
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
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "Once node should return NoMatch (consumed externally)"
        );
    }

    #[test]
    fn evaluate_node_when_selected_always_returns_no_match() {
        let node = EffectNode::When {
            trigger: Trigger::Selected,
            then: vec![EffectNode::Do(Effect::Piercing(1))],
        };
        let triggers = [
            Trigger::PerfectBump,
            Trigger::Bump,
            Trigger::EarlyBump,
            Trigger::LateBump,
            Trigger::BumpWhiff,
            Trigger::Impact(ImpactTarget::Cell),
            Trigger::Impact(ImpactTarget::Breaker),
            Trigger::Impact(ImpactTarget::Wall),
            Trigger::CellDestroyed,
            Trigger::BoltLost,
            Trigger::Death,
        ];
        for trigger in triggers {
            let result = evaluate_node(trigger, &node);
            assert_eq!(
                result,
                vec![NodeEvalResult::NoMatch],
                "Selected should NEVER match {trigger:?}"
            );
        }
    }

    #[test]
    fn evaluate_node_when_time_expires_always_returns_no_match() {
        let node = EffectNode::When {
            trigger: Trigger::TimeExpires(3.0),
            then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
        };
        let triggers = [
            Trigger::PerfectBump,
            Trigger::Bump,
            Trigger::EarlyBump,
            Trigger::LateBump,
            Trigger::BumpWhiff,
            Trigger::Impact(ImpactTarget::Cell),
            Trigger::Impact(ImpactTarget::Breaker),
            Trigger::Impact(ImpactTarget::Wall),
            Trigger::CellDestroyed,
            Trigger::BoltLost,
            Trigger::Death,
        ];
        for trigger in triggers {
            let result = evaluate_node(trigger, &node);
            assert_eq!(
                result,
                vec![NodeEvalResult::NoMatch],
                "TimeExpires has no runtime trigger mapping — should return NoMatch for {trigger:?}"
            );
        }
    }

    #[test]
    fn evaluate_node_all_triggers_match_correctly() {
        use crate::effect::definition::ImpactTarget as IT;
        let pairs: Vec<(Trigger, Trigger)> = vec![
            (Trigger::PerfectBump, Trigger::PerfectBump),
            (Trigger::Bump, Trigger::Bump),
            (Trigger::EarlyBump, Trigger::EarlyBump),
            (Trigger::LateBump, Trigger::LateBump),
            (Trigger::BumpWhiff, Trigger::BumpWhiff),
            (Trigger::Impact(IT::Cell), Trigger::Impact(IT::Cell)),
            (Trigger::Impact(IT::Breaker), Trigger::Impact(IT::Breaker)),
            (Trigger::Impact(IT::Wall), Trigger::Impact(IT::Wall)),
            (Trigger::CellDestroyed, Trigger::CellDestroyed),
            (Trigger::BoltLost, Trigger::BoltLost),
            (Trigger::Death, Trigger::Death),
        ];
        let leaf = Effect::test_shockwave(64.0);
        for (runtime, declared) in pairs {
            let node = EffectNode::When {
                trigger: declared,
                then: vec![EffectNode::Do(leaf.clone())],
            };
            let result = evaluate_node(runtime, &node);
            assert_eq!(
                result,
                vec![NodeEvalResult::Fire(leaf.clone())],
                "Trigger::{runtime:?} should match Trigger::{declared:?}"
            );
        }
    }

    #[test]
    fn evaluate_node_when_multiple_do_children_returns_multiple_fire_results() {
        let node = EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![
                EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                }),
                EffectNode::Do(Effect::SpeedBoost {
                    multiplier: 1.2,
                }),
            ],
        };
        let result = evaluate_node(Trigger::Bump, &node);
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
            trigger: Trigger::Impact(ImpactTarget::Breaker),
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::Impact(ImpactTarget::Cell), &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "Impact(Cell) must NOT match When(Impact(Breaker))"
        );
    }

    // =========================================================================
    // Trigger variant tests (formerly TriggerKind extension tests)
    // =========================================================================

    #[test]
    fn trigger_death_exists() {
        let trigger = Trigger::Death;
        assert_ne!(trigger, Trigger::BoltLost);
    }

    #[test]
    fn trigger_has_runtime_variants() {
        let triggers = [
            Trigger::PerfectBump,
            Trigger::Bump,
            Trigger::EarlyBump,
            Trigger::LateBump,
            Trigger::BumpWhiff,
            Trigger::Impact(ImpactTarget::Cell),
            Trigger::Impact(ImpactTarget::Breaker),
            Trigger::Impact(ImpactTarget::Wall),
            Trigger::CellDestroyed,
            Trigger::BoltLost,
            Trigger::Death,
        ];
        assert_eq!(
            triggers.len(),
            11,
            "Trigger should have 11 runtime variants"
        );
    }

    // =========================================================================
    // NoBump + Bumped trigger variant tests
    // =========================================================================

    #[test]
    fn evaluate_node_no_bump_trigger_fires_do_leaf() {
        let node = EffectNode::When {
            trigger: Trigger::NoBump,
            then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
        };
        let result = evaluate_node(Trigger::NoBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::DamageBoost(2.0))],
            "NoBump should match When(NoBump) and fire Do leaf"
        );
    }

    #[test]
    fn evaluate_node_no_bump_does_not_match_perfect_bump() {
        let node = EffectNode::When {
            trigger: Trigger::NoBump,
            then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "PerfectBump should not match When(NoBump)"
        );
    }

    #[test]
    fn evaluate_node_perfect_bumped_fires_do_leaf() {
        let node = EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::PerfectBumped, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::test_shockwave(64.0))],
            "PerfectBumped should match When(PerfectBumped) and fire Do leaf"
        );
    }

    #[test]
    fn evaluate_node_perfect_bumped_does_not_match_perfect_bump() {
        let node = EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "PerfectBump should not match When(PerfectBumped) — Bump != Bumped"
        );
    }

    #[test]
    fn evaluate_node_bumped_fires_do_leaf() {
        let node = EffectNode::When {
            trigger: Trigger::Bumped,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::Bumped, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::test_shockwave(64.0))],
            "Bumped should match When(Bumped) and fire Do leaf"
        );
    }

    #[test]
    fn evaluate_node_bumped_does_not_match_bump() {
        let node = EffectNode::When {
            trigger: Trigger::Bumped,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::Bump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "Bump should not match When(Bumped) — Bump != Bumped"
        );
    }

    #[test]
    fn evaluate_node_early_bumped_fires_do_leaf() {
        let node = EffectNode::When {
            trigger: Trigger::EarlyBumped,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::EarlyBumped, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::test_shockwave(64.0))],
            "EarlyBumped should match When(EarlyBumped) and fire Do leaf"
        );
    }

    #[test]
    fn evaluate_node_early_bumped_does_not_match_early_bump() {
        let node = EffectNode::When {
            trigger: Trigger::EarlyBumped,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::EarlyBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "EarlyBump should not match When(EarlyBumped) — Bump != Bumped"
        );
    }

    #[test]
    fn evaluate_node_late_bumped_fires_do_leaf() {
        let node = EffectNode::When {
            trigger: Trigger::LateBumped,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::LateBumped, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::test_shockwave(64.0))],
            "LateBumped should match When(LateBumped) and fire Do leaf"
        );
    }

    #[test]
    fn evaluate_node_late_bumped_does_not_match_late_bump() {
        let node = EffectNode::When {
            trigger: Trigger::LateBumped,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::LateBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "LateBump should not match When(LateBumped) — Bump != Bumped"
        );
    }

    #[test]
    fn evaluate_node_no_bump_does_not_match_any_bump_variant() {
        let bump_triggers = [Trigger::PerfectBump, Trigger::Bump, Trigger::BumpWhiff];
        for bump_trigger in bump_triggers {
            let node = EffectNode::When {
                trigger: bump_trigger,
                then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
            };
            let result = evaluate_node(Trigger::NoBump, &node);
            assert_eq!(
                result,
                vec![NodeEvalResult::NoMatch],
                "NoBump should not match When({bump_trigger:?})"
            );
        }
    }

    // =========================================================================
    // EffectNode::On — evaluate_node returns NoMatch (stub behavior)
    // =========================================================================

    #[test]
    fn evaluate_node_returns_no_match_for_on() {
        use crate::effect::definition::Target;

        let node = EffectNode::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(Effect::LoseLife)],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "On nodes are not evaluated by trigger matching — should return NoMatch"
        );
    }
}
