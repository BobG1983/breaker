//! Behaviors 1-4, 14-16: basic bolt effect dispatch (migrated to `effect_v3`).

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::helpers::{TEST_BOLT_NAME, test_app_with_dispatch};
use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef},
        definition::BoltDefinition,
    },
    effect_v3::{
        effects::SpeedBoostConfig,
        storage::BoundEffects,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
};

/// Helper: creates a minimal `BoltDefinition` with the given effects.
fn make_bolt_def(name: &str, effects: Vec<RootNode>) -> BoltDefinition {
    BoltDefinition {
        name: name.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects,
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

fn speed_boost_tree(multiplier: f32) -> Tree {
    Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(multiplier),
    }))
}

fn when_perfect_bumped(inner: Tree) -> Tree {
    Tree::When(Trigger::PerfectBumped, Box::new(inner))
}

fn when_early_bumped(inner: Tree) -> Tree {
    Tree::When(Trigger::EarlyBumped, Box::new(inner))
}

fn when_late_bumped(inner: Tree) -> Tree {
    Tree::When(Trigger::LateBumped, Box::new(inner))
}

// ---- Behavior 1: Bolt-targeted Stamp pushed to bolt BoundEffects on spawn ----

#[test]
fn dispatch_pushes_bolt_targeted_stamp_to_bolt_bound_effects() {
    let tree = when_perfect_bumped(speed_boost_tree(1.5));
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootNode::Stamp(StampTarget::Bolt, tree.clone())],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects inserted");
    assert_eq!(bound.0.len(), 1, "expected 1 effect in BoundEffects");
    assert_eq!(
        &bound.0[0].0, "",
        "chip name should be empty string for bolt-definition-sourced effects"
    );
    assert_eq!(bound.0[0].1, tree, "stamped tree should match");
}

#[test]
fn dispatch_preserves_pre_existing_bound_effects_on_bolt() {
    let tree = when_perfect_bumped(speed_boost_tree(1.5));
    let prior_tree = speed_boost_tree(2.0);
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootNode::Stamp(StampTarget::Bolt, tree)],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BoundEffects(vec![("prior_chip".to_owned(), prior_tree)]),
        ))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "expected 2 entries (1 pre-existing + 1 dispatched)"
    );
    assert_eq!(
        &bound.0[0].0, "prior_chip",
        "pre-existing entry should be preserved at index 0"
    );
}

// ---- Behavior 2: Empty effects definition produces no BoundEffects entries ----

#[test]
fn dispatch_empty_effects_does_not_add_entries() {
    let def = make_bolt_def("PlainBolt", vec![]);
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef("PlainBolt".to_owned()),
            BoundEffects::default(),
        ))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        0,
        "empty effects definition should leave BoundEffects empty"
    );
}

#[test]
fn dispatch_empty_effects_preserves_pre_existing_entries() {
    let prior_tree = speed_boost_tree(2.0);
    let def = make_bolt_def("PlainBolt", vec![]);
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef("PlainBolt".to_owned()),
            BoundEffects(vec![("existing_chip".to_owned(), prior_tree)]),
        ))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "pre-existing entry should remain when effects are empty"
    );
    assert_eq!(&bound.0[0].0, "existing_chip");
}

// ---- Behavior 3: Stamp tree is stored in BoundEffects ----

#[test]
fn dispatch_stamps_tree_into_bound_effects() {
    let tree = speed_boost_tree(1.5);
    let def = make_bolt_def(
        "EffectBolt",
        vec![RootNode::Stamp(StampTarget::Bolt, tree.clone())],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("EffectBolt".to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects inserted");
    assert_eq!(bound.0.len(), 1, "stamped tree should be stored");
    assert_eq!(bound.0[0].1, tree, "stored tree should match stamped tree");
}

// ---- Behavior 4: Multiple root effects on same bolt all dispatched ----

#[test]
fn dispatch_pushes_multiple_root_effects_on_same_bolt() {
    let def = make_bolt_def(
        "MultiBolt",
        vec![
            RootNode::Stamp(
                StampTarget::Bolt,
                when_perfect_bumped(speed_boost_tree(1.5)),
            ),
            RootNode::Stamp(StampTarget::Bolt, when_early_bumped(speed_boost_tree(1.1))),
            RootNode::Stamp(StampTarget::Bolt, when_late_bumped(speed_boost_tree(1.1))),
        ],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("MultiBolt".to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects");
    assert_eq!(bound.0.len(), 3, "expected 3 effects in BoundEffects");
}

#[test]
fn dispatch_zero_root_effects_pushes_nothing() {
    let def = make_bolt_def("EmptyBolt", vec![]);
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("EmptyBolt".to_owned())))
        .id();
    app.update();

    // No BoundEffects should be added, or if present, 0 entries
    if let Some(bound) = app.world().get::<BoundEffects>(bolt) {
        assert_eq!(
            bound.0.len(),
            0,
            "zero root effects should result in zero entries"
        );
    }
}

// ---- Behavior 14: Empty string used as chip name for all dispatched effects ----

#[test]
fn dispatch_uses_empty_string_as_chip_name() {
    let tree = when_perfect_bumped(speed_boost_tree(1.5));
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootNode::Stamp(StampTarget::Bolt, tree)],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects");
    for (i, (chip_name, _)) in bound.0.iter().enumerate() {
        assert_eq!(
            chip_name, "",
            "entry {i} chip name should be empty string for bolt-definition-sourced effects"
        );
    }
}

// ---- Behavior 15: Multiple Stamp roots all dispatched ----

#[test]
fn dispatch_pushes_all_stamps() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![
            RootNode::Stamp(
                StampTarget::Bolt,
                when_perfect_bumped(speed_boost_tree(1.5)),
            ),
            RootNode::Stamp(StampTarget::Bolt, when_early_bumped(speed_boost_tree(1.1))),
        ],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects");
    assert_eq!(bound.0.len(), 2, "both Stamp roots should be pushed");
}

// ---- Behavior 16: System only triggers on Added<BoltDefinitionRef>, not on every frame ----

#[test]
fn dispatch_only_triggers_on_added_bolt_definition_ref() {
    let tree = when_perfect_bumped(speed_boost_tree(1.5));
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootNode::Stamp(StampTarget::Bolt, tree)],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();

    // First update: Added<BoltDefinitionRef> triggers dispatch
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects after first update");
    let count_after_first = bound.0.len();

    // Second update: no new Added<BoltDefinitionRef> -- should not add more entries
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        count_after_first,
        "no additional entries should be added on subsequent frames"
    );
}

#[test]
fn dispatch_second_bolt_on_later_frame_triggers_new_dispatch() {
    let tree = when_perfect_bumped(speed_boost_tree(1.5));
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootNode::Stamp(StampTarget::Bolt, tree)],
    );
    let mut app = test_app_with_dispatch(def);
    let _bolt1 = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();

    // First update: dispatch for bolt1 (Added triggers)
    app.update();

    // Spawn bolt2 on a later frame
    let bolt2 = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();

    app.update();

    // bolt2 should get effects from its own dispatch
    let bound2 = app
        .world()
        .get::<BoundEffects>(bolt2)
        .expect("bolt2 should have BoundEffects after its spawn frame");
    assert!(
        !bound2.0.is_empty(),
        "bolt2 should have at least 1 entry from its dispatch"
    );

    // After two more frames with no new Added<BoltDefinitionRef>,
    // no further entries should be added
    let bolt2_count = bound2.0.len();
    app.update();

    let bound2_after = app.world().get::<BoundEffects>(bolt2).unwrap();
    assert_eq!(
        bound2_after.0.len(),
        bolt2_count,
        "no additional entries should be added on frames without new Added<BoltDefinitionRef>"
    );
}
