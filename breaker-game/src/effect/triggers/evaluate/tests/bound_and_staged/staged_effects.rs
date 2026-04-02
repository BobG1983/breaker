//! Tests for `evaluate_staged_effects`.

use bevy::prelude::*;

use super::helpers::*;

#[test]
fn staged_entry_consumed_on_matching_trigger() {
    // When(Bump, Do(X)) in StagedEffects -- after evaluating for Bump,
    // the entry should be consumed (removed). We use a non-Do child
    // to avoid command panics, but the When itself is still consumed.
    let mut app = test_app();
    app.add_systems(Update, (sys_evaluate_staged_for_bump, sys_snapshot).chain());

    let inner = when_do(Trigger::Death, EffectKind::DamageBoost(2.0));
    let node = when_child(Trigger::Bump, inner);

    let entity = app
        .world_mut()
        .spawn(StagedEffects(vec![("chip_a".into(), node)]))
        .id();
    // Also need BoundEffects for the query in other systems, but
    // sys_evaluate_staged_for_bump only queries StagedEffects.

    app.update();

    let snap = app.world().resource::<Snapshot>();
    assert_eq!(
        snap.staged_len, 1,
        "When consumed, its non-Do child should be added to StagedEffects (net 1 entry)"
    );

    // The original When(Bump) entry should be gone; the remaining entry
    // should be the inner When(Death, Do(DamageBoost)).
    let staged = app.world().get::<StagedEffects>(entity).unwrap();
    assert!(
        matches!(
            &staged.0[0].1,
            EffectNode::When {
                trigger: Trigger::Death,
                ..
            }
        ),
        "Remaining entry should be the inner When(Death) pushed as addition"
    );
}

#[test]
fn staged_non_matching_trigger_retained() {
    // When(Bump, Do(X)) in StagedEffects -- evaluate for BumpWhiff.
    // Entry should be retained because the trigger does not match.
    let mut app = test_app();
    app.add_systems(
        Update,
        (sys_evaluate_staged_for_bump_whiff, sys_snapshot).chain(),
    );

    let node = when_do(Trigger::Bump, EffectKind::DamageBoost(2.0));
    app.world_mut()
        .spawn(StagedEffects(vec![("chip_a".into(), node)]));

    app.update();

    let snap = app.world().resource::<Snapshot>();
    assert_eq!(
        snap.staged_len, 1,
        "Non-matching staged entry must be retained"
    );
}

#[test]
fn once_consumed_when_child_trigger_matches() {
    // Once([When(Bump, Do(X))]) in StagedEffects -- evaluate for Bump.
    // The Once should be consumed because a child matched.
    // Use non-Do inner child to avoid fire_effect panics.
    let mut app = test_app();
    app.add_systems(Update, (sys_evaluate_staged_for_bump, sys_snapshot).chain());

    let inner_when = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![when_do(Trigger::Death, EffectKind::DamageBoost(2.0))],
    };
    let once_node = EffectNode::Once(vec![inner_when]);
    app.world_mut()
        .spawn(StagedEffects(vec![("chip_a".into(), once_node)]));

    app.update();

    let snap = app.world().resource::<Snapshot>();
    // Once is consumed; its non-Do children are pushed as additions.
    // So net staged should have 1 entry (the inner When(Death, Do(X))).
    assert_eq!(
        snap.staged_len, 1,
        "Once consumed, addition from inner When should remain"
    );
    assert!(
        matches!(
            &snap.staged_entries[0].1,
            EffectNode::When {
                trigger: Trigger::Death,
                ..
            }
        ),
        "The addition should be the inner non-Do node"
    );
}

#[test]
fn once_retained_when_no_child_matches() {
    // Once([When(Bump, Do(X))]) in StagedEffects -- evaluate for Death.
    // No child matches, so Once should be retained.
    let mut app = test_app();
    app.add_systems(
        Update,
        (sys_evaluate_staged_for_death, sys_snapshot).chain(),
    );

    let inner_when = when_do(Trigger::Bump, EffectKind::DamageBoost(2.0));
    let once_node = EffectNode::Once(vec![inner_when]);
    app.world_mut()
        .spawn(StagedEffects(vec![("chip_a".into(), once_node)]));

    app.update();

    let snap = app.world().resource::<Snapshot>();
    assert_eq!(
        snap.staged_len, 1,
        "Once must be retained when no child matches"
    );
    assert!(
        matches!(&snap.staged_entries[0].1, EffectNode::Once(_)),
        "Retained entry should still be an Once node"
    );
}

#[test]
fn bare_do_in_staged_not_consumed() {
    // A bare Do(X) in StagedEffects should not be consumed by any trigger
    // evaluation -- walk_staged_node returns false for non-When/non-Once.
    let mut app = test_app();
    app.add_systems(Update, (sys_evaluate_staged_for_bump, sys_snapshot).chain());

    let do_node = EffectNode::Do(EffectKind::DamageBoost(2.0));
    app.world_mut()
        .spawn(StagedEffects(vec![("chip_a".into(), do_node)]));

    app.update();

    let snap = app.world().resource::<Snapshot>();
    assert_eq!(
        snap.staged_len, 1,
        "Bare Do in StagedEffects must not be consumed"
    );
    assert!(
        matches!(&snap.staged_entries[0].1, EffectNode::Do(_)),
        "Retained entry should still be a Do node"
    );
}

// -- A11: Reverse node fires reverse_effect and queues RemoveChainsCommand ─

use crate::effect::effects::damage_boost::ActiveDamageBoosts;

#[test]
fn reverse_node_fires_reverse_effect_and_removes_bound_chain() {
    let mut app = test_app();
    app.add_systems(Update, sys_evaluate_staged_for_bump);

    let matching_chain = EffectNode::When {
        trigger: Trigger::PerfectBump,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
    };

    let reverse_node = EffectNode::Reverse {
        effects: vec![EffectKind::DamageBoost(2.0)],
        chains: vec![matching_chain.clone()],
    };

    let staged_entry = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![reverse_node],
    };

    let entity = app
        .world_mut()
        .spawn((
            StagedEffects(vec![("aegis".into(), staged_entry)]),
            BoundEffects(vec![("aegis".into(), matching_chain)]),
            ActiveDamageBoosts(vec![2.0]),
        ))
        .id();

    app.update();

    // The When(Bump) entry should be consumed from StagedEffects
    let staged = app.world().get::<StagedEffects>(entity).unwrap();
    assert!(
        staged.0.is_empty(),
        "When(Bump) entry should be consumed from StagedEffects, got {} entries",
        staged.0.len()
    );

    // reverse_effect for DamageBoost(2.0) should have removed it from ActiveDamageBoosts
    let boosts = app.world().get::<ActiveDamageBoosts>(entity).unwrap();
    assert!(
        boosts.0.is_empty(),
        "ActiveDamageBoosts should be empty after reverse_effect, got {:?}",
        boosts.0
    );

    // RemoveChainsCommand should have removed the matching chain from BoundEffects
    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects should be empty after RemoveChainsCommand removed the chain, got {} entries",
        bound.0.len()
    );
}

#[test]
fn reverse_node_with_empty_chains_fires_reverse_effect_only() {
    let mut app = test_app();
    app.add_systems(Update, sys_evaluate_staged_for_bump);

    let reverse_node = EffectNode::Reverse {
        effects: vec![EffectKind::DamageBoost(2.0)],
        chains: vec![],
    };

    let staged_entry = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![reverse_node],
    };

    let bound_chain = EffectNode::When {
        trigger: Trigger::PerfectBump,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
    };

    let entity = app
        .world_mut()
        .spawn((
            StagedEffects(vec![("aegis".into(), staged_entry)]),
            BoundEffects(vec![("aegis".into(), bound_chain.clone())]),
            ActiveDamageBoosts(vec![2.0]),
        ))
        .id();

    app.update();

    // reverse_effect should still fire
    let boosts = app.world().get::<ActiveDamageBoosts>(entity).unwrap();
    assert!(
        boosts.0.is_empty(),
        "ActiveDamageBoosts should be empty after reverse_effect, got {:?}",
        boosts.0
    );

    // But RemoveChainsCommand should NOT have been queued (chains is empty)
    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should be unchanged when reverse has empty chains"
    );
    assert_eq!(
        bound.0[0].1, bound_chain,
        "Original bound chain should still be present"
    );
}
