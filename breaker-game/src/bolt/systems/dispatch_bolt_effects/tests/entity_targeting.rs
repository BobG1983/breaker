//! Behaviors 5-10: target entity resolution for bolt effects (migrated to `effect_v3`).

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::helpers::{TEST_BOLT_NAME, test_app_with_dispatch};
use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef},
        definition::BoltDefinition,
    },
    cells::components::Cell,
    effect_v3::{
        effects::{LoseLifeConfig, ShockwaveConfig, SpeedBoostConfig},
        storage::BoundEffects,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    walls::components::Wall,
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

fn shockwave_tree() -> Tree {
    Tree::Fire(EffectType::Shockwave(ShockwaveConfig {
        base_range:      OrderedFloat(32.0),
        range_per_level: OrderedFloat(8.0),
        stacks:          1,
        speed:           OrderedFloat(400.0),
    }))
}

// ---- Behavior 5: Breaker-targeted effects dispatched to breaker entity, not bolt ----

#[test]
fn dispatch_pushes_breaker_targeted_effects_to_breaker_entity() {
    let def = make_bolt_def(
        "CrossBolt",
        vec![RootNode::Stamp(
            StampTarget::Breaker,
            Tree::When(
                Trigger::BoltLostOccurred,
                Box::new(Tree::Fire(EffectType::LoseLife(LoseLifeConfig {}))),
            ),
        )],
    );
    let mut app = test_app_with_dispatch(def);
    let breaker = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut()
        .entity_mut(breaker)
        .insert(BoundEffects::default());
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("CrossBolt".to_owned())))
        .id();
    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "breaker should have 1 effect from bolt definition"
    );

    // Bolt should NOT have effects from Breaker-targeted root
    if let Some(bolt_bound) = app.world().get::<BoundEffects>(bolt) {
        let def_entry_count = bolt_bound.0.iter().filter(|(n, _)| n.is_empty()).count();
        assert_eq!(
            def_entry_count, 0,
            "bolt should have 0 definition-sourced entries from Breaker-targeted effect"
        );
    }
}

#[test]
fn dispatch_breaker_targeted_with_no_breaker_entity_skips_silently() {
    let def = make_bolt_def(
        "CrossBolt",
        vec![RootNode::Stamp(
            StampTarget::Breaker,
            Tree::When(
                Trigger::BoltLostOccurred,
                Box::new(Tree::Fire(EffectType::LoseLife(LoseLifeConfig {}))),
            ),
        )],
    );
    let mut app = test_app_with_dispatch(def);
    // No breaker entity spawned
    app.world_mut()
        .spawn((Bolt, BoltDefinitionRef("CrossBolt".to_owned())));
    // Should not panic
    app.update();
}

// ---- Behavior 6: ActiveBolts-targeted effects dispatched to all bolt entities ----

#[test]
fn dispatch_pushes_active_bolts_targeted_effects_to_all_bolt_entities() {
    let def = make_bolt_def(
        "GroupBolt",
        vec![RootNode::Stamp(
            StampTarget::ActiveBolts,
            Tree::When(Trigger::PerfectBumped, Box::new(speed_boost_tree(1.5))),
        )],
    );
    let mut app = test_app_with_dispatch(def);
    // bolt_b already exists WITHOUT BoltDefinitionRef (it's a pre-existing bolt)
    let bolt_b = app.world_mut().spawn(Bolt).id();
    // bolt_a is the newly spawned bolt with a definition ref (triggers Added)
    let bolt_a = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("GroupBolt".to_owned())))
        .id();
    app.update();

    for (label, bolt) in [("bolt_a", bolt_a), ("bolt_b", bolt_b)] {
        let bound = app
            .world()
            .get::<BoundEffects>(bolt)
            .unwrap_or_else(|| panic!("{label} should have BoundEffects inserted"));
        assert_eq!(
            bound.0.len(),
            1,
            "{label} should have 1 entry in BoundEffects"
        );
    }
}

#[test]
fn dispatch_active_bolts_targeted_with_single_bolt() {
    let def = make_bolt_def(
        "GroupBolt",
        vec![RootNode::Stamp(
            StampTarget::ActiveBolts,
            Tree::When(Trigger::PerfectBumped, Box::new(speed_boost_tree(1.5))),
        )],
    );
    let mut app = test_app_with_dispatch(def);
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("GroupBolt".to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects");
    assert_eq!(bound.0.len(), 1, "single bolt should get the entry");
}

// ---- Behavior 7: Cell-targeted effects dispatched to all cell entities ----

#[test]
fn dispatch_pushes_cell_targeted_effects_to_all_cell_entities() {
    let def = make_bolt_def(
        "CellBolt",
        vec![RootNode::Stamp(StampTarget::ActiveCells, shockwave_tree())],
    );
    let mut app = test_app_with_dispatch(def);
    let cell1 = app.world_mut().spawn(Cell).id();
    let cell2 = app.world_mut().spawn(Cell).id();
    app.world_mut()
        .spawn((Bolt, BoltDefinitionRef("CellBolt".to_owned())));
    app.update();

    for (label, cell) in [("cell1", cell1), ("cell2", cell2)] {
        let bound = app
            .world()
            .get::<BoundEffects>(cell)
            .unwrap_or_else(|| panic!("{label} should have BoundEffects inserted"));
        assert_eq!(
            bound.0.len(),
            1,
            "{label} should have 1 entry in BoundEffects"
        );
    }
}

#[test]
fn dispatch_cell_targeted_with_zero_cells_no_panic() {
    let def = make_bolt_def(
        "CellBolt",
        vec![RootNode::Stamp(StampTarget::ActiveCells, shockwave_tree())],
    );
    let mut app = test_app_with_dispatch(def);
    app.world_mut()
        .spawn((Bolt, BoltDefinitionRef("CellBolt".to_owned())));
    // No cell entities spawned
    app.update();
    // Should not panic
}

// ---- Behavior 8: EveryCell-targeted effects dispatched to all cell entities ----

#[test]
fn dispatch_pushes_every_cell_targeted_effects_to_all_cell_entities() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootNode::Stamp(StampTarget::EveryCell, shockwave_tree())],
    );
    let mut app = test_app_with_dispatch(def);
    let cell1 = app.world_mut().spawn(Cell).id();
    let cell2 = app.world_mut().spawn(Cell).id();
    let cell3 = app.world_mut().spawn(Cell).id();
    app.world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())));
    app.update();

    for (label, cell) in [("cell1", cell1), ("cell2", cell2), ("cell3", cell3)] {
        let bound = app
            .world()
            .get::<BoundEffects>(cell)
            .unwrap_or_else(|| panic!("{label} should have BoundEffects"));
        assert_eq!(
            bound.0.len(),
            1,
            "{label} should have 1 entry in BoundEffects"
        );
    }
}

// ---- Behavior 9: Wall-targeted effects dispatched to all wall entities ----

#[test]
fn dispatch_pushes_wall_targeted_effects_to_all_wall_entities() {
    let def = make_bolt_def(
        "WallBolt",
        vec![RootNode::Stamp(StampTarget::ActiveWalls, shockwave_tree())],
    );
    let mut app = test_app_with_dispatch(def);
    let wall1 = app.world_mut().spawn(Wall).id();
    let wall2 = app.world_mut().spawn(Wall).id();
    app.world_mut()
        .spawn((Bolt, BoltDefinitionRef("WallBolt".to_owned())));
    app.update();

    for (label, wall) in [("wall1", wall1), ("wall2", wall2)] {
        let bound = app
            .world()
            .get::<BoundEffects>(wall)
            .unwrap_or_else(|| panic!("{label} should have BoundEffects inserted"));
        assert_eq!(
            bound.0.len(),
            1,
            "{label} should have 1 entry in BoundEffects"
        );
    }
}

#[test]
fn dispatch_wall_targeted_with_zero_walls_no_panic() {
    let def = make_bolt_def(
        "WallBolt",
        vec![RootNode::Stamp(StampTarget::ActiveWalls, shockwave_tree())],
    );
    let mut app = test_app_with_dispatch(def);
    app.world_mut()
        .spawn((Bolt, BoltDefinitionRef("WallBolt".to_owned())));
    // No wall entities spawned
    app.update();
    // Should not panic
}

// ---- Behavior 10: EveryWall-targeted effects dispatched to all wall entities ----

#[test]
fn dispatch_pushes_every_wall_targeted_effects_to_all_wall_entities() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootNode::Stamp(StampTarget::EveryWall, shockwave_tree())],
    );

    let mut app = test_app_with_dispatch(def);
    let wall1 = app.world_mut().spawn(Wall).id();
    let wall2 = app.world_mut().spawn(Wall).id();
    let wall3 = app.world_mut().spawn(Wall).id();
    app.world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())));
    app.update();

    for (label, wall) in [("wall1", wall1), ("wall2", wall2), ("wall3", wall3)] {
        let bound = app
            .world()
            .get::<BoundEffects>(wall)
            .unwrap_or_else(|| panic!("{label} should have BoundEffects"));
        assert_eq!(
            bound.0.len(),
            1,
            "{label} should have 1 entry in BoundEffects"
        );
    }
}
