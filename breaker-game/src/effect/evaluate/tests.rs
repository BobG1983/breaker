use super::*;
use crate::effect::definition::{Effect, EffectNode, ImpactTarget, Trigger};

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
        Some(
            vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })]
            .as_slice()
        ),
        "PerfectBump should match When(PerfectBump) and return children"
    );
}

#[test]
fn evaluate_node_when_non_matching_trigger_returns_none() {
    let node = EffectNode::When {
        trigger: Trigger::PerfectBump,
        then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
    };
    let result = evaluate_node(Trigger::BoltLost, &node);
    assert_eq!(result, None, "BoltLost should not match When(PerfectBump)");
}

#[test]
fn evaluate_node_when_matching_returns_nested_when_children() {
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
        Some(vec![inner].as_slice()),
        "PerfectBump with inner When child should return children slice"
    );
}

#[test]
fn evaluate_node_bare_do_returns_none() {
    let node = EffectNode::Do(Effect::LoseLife);
    let result = evaluate_node(Trigger::PerfectBump, &node);
    assert_eq!(result, None, "bare Do node should return None");
}

#[test]
fn evaluate_node_until_returns_none() {
    let node = EffectNode::Until {
        until: Trigger::TimeExpires(3.0),
        then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
    };
    let result = evaluate_node(Trigger::PerfectBump, &node);
    assert_eq!(
        result, None,
        "Until node should return None (not trigger-gated)"
    );
}

#[test]
fn evaluate_node_once_returns_none() {
    let node = EffectNode::Once(vec![EffectNode::Do(Effect::SecondWind {
        invuln_secs: 3.0,
    })]);
    let result = evaluate_node(Trigger::PerfectBump, &node);
    assert_eq!(
        result, None,
        "Once node should return None (consumed externally)"
    );
}

#[test]
fn evaluate_node_when_time_expires_always_returns_none() {
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
        Trigger::Impacted(ImpactTarget::Cell),
        Trigger::Impacted(ImpactTarget::Wall),
        Trigger::Impacted(ImpactTarget::Breaker),
        Trigger::Died,
        Trigger::DestroyedCell,
    ];
    for trigger in triggers {
        let result = evaluate_node(trigger, &node);
        assert_eq!(
            result, None,
            "TimeExpires has no runtime trigger mapping — should return None for {trigger:?}"
        );
    }
}

#[test]
fn evaluate_node_when_node_timer_threshold_always_returns_none() {
    let node = EffectNode::When {
        trigger: Trigger::NodeTimerThreshold(0.5),
        then: vec![EffectNode::Do(Effect::LoseLife)],
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
        Trigger::NoBump,
        Trigger::PerfectBumped,
        Trigger::Bumped,
        Trigger::EarlyBumped,
        Trigger::LateBumped,
        Trigger::Impacted(ImpactTarget::Cell),
        Trigger::Impacted(ImpactTarget::Wall),
        Trigger::Impacted(ImpactTarget::Breaker),
        Trigger::Died,
        Trigger::DestroyedCell,
    ];
    for trigger in triggers {
        let result = evaluate_node(trigger, &node);
        assert_eq!(
            result, None,
            "NodeTimerThreshold has no runtime trigger mapping — should return None for {trigger:?}"
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
        (Trigger::Impacted(IT::Cell), Trigger::Impacted(IT::Cell)),
        (
            Trigger::Impacted(IT::Breaker),
            Trigger::Impacted(IT::Breaker),
        ),
        (Trigger::Impacted(IT::Wall), Trigger::Impacted(IT::Wall)),
        (Trigger::Died, Trigger::Died),
        (Trigger::DestroyedCell, Trigger::DestroyedCell),
    ];
    let leaf = Effect::test_shockwave(64.0);
    for (runtime, declared) in &pairs {
        let node = EffectNode::When {
            trigger: *declared,
            then: vec![EffectNode::Do(leaf.clone())],
        };
        let result = evaluate_node(*runtime, &node);
        assert!(
            result.is_some(),
            "Trigger::{runtime:?} should match Trigger::{declared:?}"
        );
        let children = result.unwrap();
        assert_eq!(
            children,
            &[EffectNode::Do(leaf.clone())],
            "Trigger::{runtime:?} should return Do leaf children"
        );
    }
    assert_eq!(pairs.len(), 16, "should test 16 runtime trigger pairs");
}

#[test]
fn evaluate_node_when_multiple_do_children_returns_all_children() {
    let node = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![
            EffectNode::Do(Effect::SpawnBolts {
                count: 1,
                lifespan: None,
                inherit: false,
            }),
            EffectNode::Do(Effect::SpeedBoost { multiplier: 1.2 }),
        ],
    };
    let result = evaluate_node(Trigger::Bump, &node);
    let children = result.expect("should match");
    assert_eq!(
        children.len(),
        2,
        "should return 2 children for 2 Do children"
    );
    assert!(matches!(
        children[0],
        EffectNode::Do(Effect::SpawnBolts { .. })
    ));
    assert!(matches!(
        children[1],
        EffectNode::Do(Effect::SpeedBoost { .. })
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
        result, None,
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
        Trigger::NoBump,
        Trigger::PerfectBumped,
        Trigger::Bumped,
        Trigger::EarlyBumped,
        Trigger::LateBumped,
        Trigger::Impacted(ImpactTarget::Cell),
        Trigger::Impacted(ImpactTarget::Wall),
        Trigger::Impacted(ImpactTarget::Breaker),
        Trigger::Died,
        Trigger::DestroyedCell,
    ];
    assert_eq!(
        triggers.len(),
        21,
        "Trigger should have 21 runtime variants (Selected removed, 5 new targeted added)"
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
        Some(vec![EffectNode::Do(Effect::DamageBoost(2.0))].as_slice()),
        "NoBump should match When(NoBump) and return children"
    );
}

#[test]
fn evaluate_node_no_bump_does_not_match_perfect_bump() {
    let node = EffectNode::When {
        trigger: Trigger::NoBump,
        then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
    };
    let result = evaluate_node(Trigger::PerfectBump, &node);
    assert_eq!(result, None, "PerfectBump should not match When(NoBump)");
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
        Some(vec![EffectNode::Do(Effect::test_shockwave(64.0))].as_slice()),
        "PerfectBumped should match When(PerfectBumped) and return children"
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
        result, None,
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
        Some(vec![EffectNode::Do(Effect::test_shockwave(64.0))].as_slice()),
        "Bumped should match When(Bumped) and return children"
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
        result, None,
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
        Some(vec![EffectNode::Do(Effect::test_shockwave(64.0))].as_slice()),
        "EarlyBumped should match When(EarlyBumped) and return children"
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
        result, None,
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
        Some(vec![EffectNode::Do(Effect::test_shockwave(64.0))].as_slice()),
        "LateBumped should match When(LateBumped) and return children"
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
        result, None,
        "LateBump should not match When(LateBumped) — Bump != Bumped"
    );
}

#[test]
fn evaluate_node_no_bump_does_not_match_any_bump_variant() {
    let bump_triggers = [
        Trigger::PerfectBump,
        Trigger::Bump,
        Trigger::BumpWhiff,
        Trigger::Bumped,
        Trigger::PerfectBumped,
        Trigger::EarlyBumped,
        Trigger::LateBumped,
    ];
    for bump_trigger in bump_triggers {
        let node = EffectNode::When {
            trigger: bump_trigger,
            then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
        };
        let result = evaluate_node(Trigger::NoBump, &node);
        assert_eq!(
            result, None,
            "NoBump should not match When({bump_trigger:?})"
        );
    }
}

// =========================================================================
// EffectNode::On — evaluate_node returns None (stub behavior)
// =========================================================================

#[test]
fn evaluate_node_returns_none_for_on() {
    use crate::effect::definition::Target;

    let node = EffectNode::On {
        target: Target::Bolt,
        then: vec![EffectNode::Do(Effect::LoseLife)],
    };
    let result = evaluate_node(Trigger::PerfectBump, &node);
    assert_eq!(
        result, None,
        "On nodes are not evaluated by trigger matching — should return None"
    );
}

// =========================================================================
// New targeted triggers: Impacted, Died, DestroyedCell
// =========================================================================

#[test]
fn evaluate_node_impacted_cell_matches_when_impacted_cell() {
    use crate::effect::definition::ImpactTarget as IT;
    let node = EffectNode::When {
        trigger: Trigger::Impacted(IT::Cell),
        then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
    };
    let result = evaluate_node(Trigger::Impacted(IT::Cell), &node);
    assert_eq!(
        result,
        Some(vec![EffectNode::Do(Effect::test_shockwave(64.0))].as_slice()),
        "Impacted(Cell) should match When(Impacted(Cell)) and return children"
    );
}

#[test]
fn evaluate_node_impacted_does_not_match_impact() {
    use crate::effect::definition::ImpactTarget as IT;
    let node = EffectNode::When {
        trigger: Trigger::Impact(IT::Cell),
        then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
    };
    let result = evaluate_node(Trigger::Impacted(IT::Cell), &node);
    assert_eq!(
        result, None,
        "Impacted(Cell) must NOT match When(Impact(Cell)) — different trigger kinds"
    );
}

#[test]
fn evaluate_node_impact_does_not_match_impacted() {
    use crate::effect::definition::ImpactTarget as IT;
    let node = EffectNode::When {
        trigger: Trigger::Impacted(IT::Cell),
        then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
    };
    let result = evaluate_node(Trigger::Impact(IT::Cell), &node);
    assert_eq!(
        result, None,
        "Impact(Cell) must NOT match When(Impacted(Cell)) — different trigger kinds"
    );
}

#[test]
fn evaluate_node_died_matches_when_died() {
    let node = EffectNode::When {
        trigger: Trigger::Died,
        then: vec![EffectNode::Do(Effect::LoseLife)],
    };
    let result = evaluate_node(Trigger::Died, &node);
    assert_eq!(
        result,
        Some(vec![EffectNode::Do(Effect::LoseLife)].as_slice()),
        "Died should match When(Died) and return children"
    );
}

#[test]
fn evaluate_node_died_does_not_match_death() {
    let node = EffectNode::When {
        trigger: Trigger::Death,
        then: vec![EffectNode::Do(Effect::LoseLife)],
    };
    let result = evaluate_node(Trigger::Died, &node);
    assert_eq!(
        result, None,
        "Died must NOT match When(Death) — different trigger kinds"
    );
}

#[test]
fn evaluate_node_death_does_not_match_died() {
    let node = EffectNode::When {
        trigger: Trigger::Died,
        then: vec![EffectNode::Do(Effect::LoseLife)],
    };
    let result = evaluate_node(Trigger::Death, &node);
    assert_eq!(
        result, None,
        "Death must NOT match When(Died) — different trigger kinds"
    );
}

#[test]
fn evaluate_node_destroyed_cell_matches() {
    let node = EffectNode::When {
        trigger: Trigger::DestroyedCell,
        then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
    };
    let result = evaluate_node(Trigger::DestroyedCell, &node);
    assert_eq!(
        result,
        Some(vec![EffectNode::Do(Effect::DamageBoost(2.0))].as_slice()),
        "DestroyedCell should match When(DestroyedCell) and return children"
    );
}

#[test]
fn evaluate_node_destroyed_cell_does_not_match_cell_destroyed() {
    let node = EffectNode::When {
        trigger: Trigger::CellDestroyed,
        then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
    };
    let result = evaluate_node(Trigger::DestroyedCell, &node);
    assert_eq!(
        result, None,
        "DestroyedCell must NOT match When(CellDestroyed) — different trigger kinds"
    );
}

#[test]
fn evaluate_node_cell_destroyed_does_not_match_destroyed_cell() {
    let node = EffectNode::When {
        trigger: Trigger::DestroyedCell,
        then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
    };
    let result = evaluate_node(Trigger::CellDestroyed, &node);
    assert_eq!(
        result, None,
        "CellDestroyed must NOT match When(DestroyedCell) — different trigger kinds"
    );
}
