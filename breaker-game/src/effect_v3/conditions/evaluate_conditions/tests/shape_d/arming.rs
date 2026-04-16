use bevy::prelude::*;

use super::super::{super::system::*, helpers::*};
use crate::{
    effect_v3::{
        storage::BoundEffects,
        types::{BumpTarget, EffectType, ParticipantTarget, Terminal, Tree},
    },
    state::types::NodeState,
};

// ================================================================
// Shape D: During(Cond, On(Participant, Fire(reversible)))
// ================================================================

// ----------------------------------------------------------------
// Behavior 13: Cond entering true installs a Tree::On armed entry
//              into BoundEffects with #armed[0] key
// ----------------------------------------------------------------

#[test]
fn shape_d_cond_entering_true_installs_armed_on_entry() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain 2 entries: original During + armed On"
    );

    // Original During entry preserved
    assert_eq!(bound.0[0].0, "chip_redirect");
    assert!(
        matches!(bound.0[0].1, Tree::During(..)),
        "First entry should still be the original During tree"
    );

    // Armed On entry installed
    let armed = bound
        .0
        .iter()
        .find(|(name, _)| name == "chip_redirect#armed[0]");
    assert!(
        armed.is_some(),
        "Should find armed On with key 'chip_redirect#armed[0]'"
    );
    let (_, armed_tree) = armed.unwrap();
    // Verify it is Tree::On with widened Terminal::Fire(EffectType::...), not ScopedTerminal
    assert!(
        matches!(
            armed_tree,
            Tree::On(
                ParticipantTarget::Bump(BumpTarget::Bolt),
                Terminal::Fire(EffectType::SpeedBoost(..))
            )
        ),
        "Armed entry should be Tree::On(Bump(Bolt), Terminal::Fire(EffectType::SpeedBoost(...)))"
    );

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist");
    assert!(
        da.0.contains("chip_redirect"),
        "DuringActive should contain 'chip_redirect'"
    );
}

// ----------------------------------------------------------------
// Behavior 14: Cond staying true does not re-install the armed On entry
// ----------------------------------------------------------------

#[test]
fn shape_d_cond_staying_true_does_not_reinstall_armed_on_entry() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);
    evaluate_conditions(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain exactly 2 entries after two evaluations"
    );
}
