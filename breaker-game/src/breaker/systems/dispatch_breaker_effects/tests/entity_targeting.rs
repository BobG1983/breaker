use bevy::prelude::*;

use super::helpers::{TEST_BREAKER_NAME, make_test_definition, test_app_with_dispatch};
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::components::Cell,
    effect::{
        BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, StagedEffects, Target,
        Trigger,
    },
    wall::components::Wall,
};

// ---- Behavior 9: Bolt-targeted effects pushed to all Bolt entities ----

#[test]
fn dispatch_pushes_bolt_targeted_effects_to_all_bolt_entities() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, None);
    def.effects = vec![RootEffect::On {
        target: Target::Bolt,
        then: vec![EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
    }];
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    let bolt1 = app.world_mut().spawn(Bolt).id();
    let bolt2 = app.world_mut().spawn(Bolt).id();
    app.update();

    for (label, bolt) in [("bolt1", bolt1), ("bolt2", bolt2)] {
        let bound = app
            .world()
            .get::<BoundEffects>(bolt)
            .unwrap_or_else(|| panic!("{label} should have BoundEffects inserted"));
        assert_eq!(
            bound.0.len(),
            1,
            "{label} should have 1 entry in BoundEffects"
        );
        assert_eq!(
            &bound.0[0].0, "",
            "{label} chip name should be empty string"
        );
        assert!(
            app.world().get::<StagedEffects>(bolt).is_some(),
            "{label} should have StagedEffects inserted"
        );
    }
}

#[test]
fn dispatch_bolt_targeted_with_zero_bolts_no_panic() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, None);
    def.effects = vec![RootEffect::On {
        target: Target::Bolt,
        then: vec![EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
    }];
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    // No bolt entities spawned
    app.update();
    // Should not panic
}

// ---- Behavior 10: AllBolts-targeted effects pushed to all Bolt entities ----

#[test]
fn dispatch_pushes_all_bolts_targeted_effects_to_all_bolt_entities() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, None);
    def.effects = vec![RootEffect::On {
        target: Target::AllBolts,
        then: vec![EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
    }];
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    let bolt1 = app.world_mut().spawn(Bolt).id();
    let bolt2 = app.world_mut().spawn(Bolt).id();
    let bolt3 = app.world_mut().spawn(Bolt).id();
    app.update();

    for (label, bolt) in [("bolt1", bolt1), ("bolt2", bolt2), ("bolt3", bolt3)] {
        let bound = app
            .world()
            .get::<BoundEffects>(bolt)
            .unwrap_or_else(|| panic!("{label} should have BoundEffects"));
        assert_eq!(
            bound.0.len(),
            1,
            "{label} should have 1 entry in BoundEffects"
        );
    }
}

// ---- Behavior 11: Cell-targeted effects pushed to all Cell entities ----

#[test]
fn dispatch_pushes_cell_targeted_effects_to_all_cell_entities() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, None);
    def.effects = vec![RootEffect::On {
        target: Target::Cell,
        then: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shockwave {
                base_range: 32.0,
                range_per_level: 8.0,
                stacks: 1,
                speed: 400.0,
            })],
        }],
    }];
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    let cell1 = app.world_mut().spawn(Cell).id();
    let cell2 = app.world_mut().spawn(Cell).id();
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
        assert!(
            app.world().get::<StagedEffects>(cell).is_some(),
            "{label} should have StagedEffects inserted"
        );
    }
}

// ---- Behavior 12: AllCells-targeted effects pushed to all Cell entities ----

#[test]
fn dispatch_pushes_all_cells_targeted_effects_to_all_cell_entities() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, None);
    def.effects = vec![RootEffect::On {
        target: Target::AllCells,
        then: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shockwave {
                base_range: 32.0,
                range_per_level: 8.0,
                stacks: 1,
                speed: 400.0,
            })],
        }],
    }];
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    let cell1 = app.world_mut().spawn(Cell).id();
    let cell2 = app.world_mut().spawn(Cell).id();
    let cell3 = app.world_mut().spawn(Cell).id();
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

// ---- Behavior 13: Wall-targeted effects pushed to all Wall entities ----

#[test]
fn dispatch_pushes_wall_targeted_effects_to_all_wall_entities() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, None);
    def.effects = vec![RootEffect::On {
        target: Target::Wall,
        then: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shockwave {
                base_range: 32.0,
                range_per_level: 8.0,
                stacks: 1,
                speed: 400.0,
            })],
        }],
    }];
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    let wall1 = app.world_mut().spawn(Wall).id();
    let wall2 = app.world_mut().spawn(Wall).id();
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
        assert!(
            app.world().get::<StagedEffects>(wall).is_some(),
            "{label} should have StagedEffects inserted"
        );
    }
}

// ---- Behavior 14: AllWalls-targeted effects pushed to all Wall entities ----

#[test]
fn dispatch_pushes_all_walls_targeted_effects_to_all_wall_entities() {
    let mut def = make_test_definition(TEST_BREAKER_NAME, None);
    def.effects = vec![RootEffect::On {
        target: Target::AllWalls,
        then: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shockwave {
                base_range: 32.0,
                range_per_level: 8.0,
                stacks: 1,
                speed: 400.0,
            })],
        }],
    }];
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    let wall1 = app.world_mut().spawn(Wall).id();
    let wall2 = app.world_mut().spawn(Wall).id();
    let wall3 = app.world_mut().spawn(Wall).id();
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
