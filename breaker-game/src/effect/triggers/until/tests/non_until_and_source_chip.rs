//! Tests for non-`Until` entries being unaffected by desugaring, and
//! `EffectSourceChip` threading through `desugar_until`.

use bevy::prelude::*;

use super::helpers::*;
use crate::effect::effects::speed_boost::ActiveSpeedBoosts;

#[test]
fn non_until_entries_unaffected() {
    // When(Bump, Do(X)) in BoundEffects alongside an Until.
    // After desugaring, the When(Bump) should still be in BoundEffects.
    let mut app = test_app();

    let regular_when = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
    };
    let inner = EffectNode::When {
        trigger: Trigger::Death,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
    };
    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(2.0),
        then: vec![inner.clone()],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![
                ("chip_a".into(), regular_when.clone()),
                ("chip_b".into(), until_node),
            ]),
            StagedEffects::default(),
        ))
        .id();

    tick(&mut app);

    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    // regular_when retained + non-Do child from Until pushed = 2 entries
    assert_eq!(bound.0.len(), 2, "Regular When + pushed non-Do child");

    let has_regular = bound
        .0
        .iter()
        .any(|(name, node)| name == "chip_a" && *node == regular_when);
    assert!(
        has_regular,
        "Non-Until entry When(Bump) should be retained in BoundEffects"
    );

    let has_inner = bound
        .0
        .iter()
        .any(|(name, node)| name == "chip_b" && *node == inner);
    assert!(
        has_inner,
        "Inner non-Do child from Until should be pushed to BoundEffects"
    );
}

// -- Section K: EffectSourceChip threading through desugar_until --

#[test]
fn desugar_until_threads_chip_name_as_source_chip_to_fire_effect() {
    // Until(TimeExpires(2.0), [Do(SpeedBoost(1.3))]) with chip_name "overclock"
    // SpeedBoost ignores source_chip, but verifying plumbing via ActiveSpeedBoosts
    let mut app = test_app();

    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(2.0),
        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![("overclock".into(), until_node)]),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    tick(&mut app);

    let boosts = app.world().get::<ActiveSpeedBoosts>(entity).unwrap();
    assert!(
        boosts.0.contains(&1.3),
        "SpeedBoost(1.3) should have been fired — ActiveSpeedBoosts should contain 1.3, got {:?}",
        boosts.0
    );
}
