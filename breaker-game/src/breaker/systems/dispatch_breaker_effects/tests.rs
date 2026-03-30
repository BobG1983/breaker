use bevy::prelude::*;

use super::*;
use crate::{
    bolt::components::Bolt,
    breaker::{
        SelectedBreaker,
        components::Breaker,
        definition::{BreakerDefinition, BreakerStatOverrides},
        registry::BreakerRegistry,
    },
    cells::components::Cell,
    effect::{
        BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, StagedEffects, Target,
        Trigger,
    },
    wall::components::Wall,
};

const TEST_BREAKER_NAME: &str = "TestBreaker";

fn test_app_with_dispatch(def: BreakerDefinition) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = BreakerRegistry::default();
    registry.insert(def.name.clone(), def);
    app.insert_resource(registry)
        .insert_resource(SelectedBreaker(TEST_BREAKER_NAME.to_owned()))
        .add_systems(Update, dispatch_breaker_effects);
    app
}

// ---- Behavior 6: Breaker-targeted When children pushed to breaker BoundEffects ----

#[test]
fn dispatch_pushes_breaker_targeted_when_to_breaker_bound_effects() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::LoseLife)],
            }],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1, "expected 1 effect in BoundEffects");
    assert_eq!(
        &bound.0[0].0, "",
        "chip name should be empty string for breaker-defined effects"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::When { trigger: Trigger::BoltLost, then } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::LoseLife))
    ));
}

#[test]
fn dispatch_empty_effects_leaves_bound_effects_empty() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        0,
        "empty effects definition should leave BoundEffects empty"
    );
}

// ---- Behavior 7: Bare Do children fired immediately, not stored in BoundEffects ----

#[test]
fn dispatch_fires_bare_do_immediately_not_stored_in_bound_effects() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
                EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                },
            ],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "only the When child should be stored; the bare Do should be fired immediately"
    );
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::BoltLost,
                ..
            }
        ),
        "the stored entry should be the When node, not the Do"
    );
}

#[test]
fn dispatch_mixed_do_and_when_stores_only_when() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
                EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                },
            ],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "only the When child should be stored; Do should be fired immediately"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::BoltLost,
            ..
        }
    ));
}

// ---- Behavior 8: Multiple Breaker-targeted effects ----

#[test]
fn dispatch_pushes_multiple_breaker_targeted_effects() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::EarlyBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::LateBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
        ],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 4, "expected 4 effects in BoundEffects");
}

// ---- Behavior 9: Bolt-targeted effects pushed to all Bolt entities ----

#[test]
fn dispatch_pushes_bolt_targeted_effects_to_all_bolt_entities() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
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
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    // No bolt entities spawned
    app.update();
    // Should not panic
}

// ---- Behavior 10: AllBolts-targeted effects pushed to all Bolt entities ----

#[test]
fn dispatch_pushes_all_bolts_targeted_effects_to_all_bolt_entities() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::AllBolts,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
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
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
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
        }],
    };
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
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
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
        }],
    };
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
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
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
        }],
    };
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
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
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
        }],
    };
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

// ---- Behavior 15: Mixed targets (Aegis-style) ----

#[test]
fn dispatch_handles_mixed_targets_aegis_style() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: Some(3),
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::EarlyBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::LateBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
        ],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    let bolt = app.world_mut().spawn(Bolt).id();
    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "breaker should have exactly 1 effect (BoltLost -> LoseLife)"
    );

    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects inserted");
    assert_eq!(
        bolt_bound.0.len(),
        3,
        "bolt should have exactly 3 effects (PerfectBumped, EarlyBumped, LateBumped)"
    );

    assert!(
        app.world().get::<StagedEffects>(bolt).is_some(),
        "bolt should have StagedEffects inserted"
    );
}

// ---- Behavior 16: Preserves existing BoundEffects on target entities ----

#[test]
fn dispatch_preserves_existing_bound_effects_on_target_entities() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoundEffects(vec![(
                "existing_chip".to_owned(),
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
            )]),
            StagedEffects(vec![(
                "staged_chip".to_owned(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                },
            )]),
        ))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "bolt should have 2 entries (1 existing + 1 dispatched)"
    );
    assert_eq!(
        &bound.0[0].0, "existing_chip",
        "existing entry should be preserved at index 0"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::Do(EffectKind::DamageBoost(v)) if (*v - 2.0).abs() < f32::EPSILON
    ));

    let staged = app.world().get::<StagedEffects>(bolt).unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "existing StagedEffects should be preserved, not replaced"
    );
    assert_eq!(
        &staged.0[0].0, "staged_chip",
        "existing staged entry should be preserved"
    );
}

// ---- Behavior 17: Inserts BoundEffects and StagedEffects on entities that lack them ----

#[test]
fn dispatch_inserts_bound_effects_and_staged_effects_when_absent() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    // Spawn bolt with no BoundEffects and no StagedEffects
    let bolt = app.world_mut().spawn(Bolt).id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("BoundEffects should be inserted on bolt");
    assert_eq!(bound.0.len(), 1, "bolt should have 1 dispatched entry");

    let staged = app
        .world()
        .get::<StagedEffects>(bolt)
        .expect("StagedEffects should be inserted on bolt");
    assert_eq!(
        staged.0.len(),
        0,
        "newly inserted StagedEffects should be empty"
    );
}

#[test]
fn dispatch_inserts_staged_effects_when_bound_effects_present_but_staged_absent() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    app.world_mut().spawn((Breaker, BoundEffects::default()));
    // Spawn bolt WITH BoundEffects (containing a prior entry) but WITHOUT StagedEffects
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoundEffects(vec![(
                "prior_chip".to_owned(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
                },
            )]),
        ))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("BoundEffects should still be present on bolt");
    assert_eq!(
        bound.0.len(),
        2,
        "bolt should have 2 entries (1 prior + 1 dispatched)"
    );
    assert_eq!(
        &bound.0[0].0, "prior_chip",
        "prior entry should be preserved at index 0"
    );

    assert!(
        app.world().get::<StagedEffects>(bolt).is_some(),
        "StagedEffects should be inserted even though only BoundEffects was present initially"
    );
}

// ---- Behavior 18: Missing breaker in registry is a no-op ----

#[test]
fn dispatch_with_missing_breaker_in_registry_is_noop() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(BreakerRegistry::default())
        .insert_resource(SelectedBreaker("NonExistent".to_owned()))
        .add_systems(Update, dispatch_breaker_effects);

    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    let bolt = app.world_mut().spawn((Bolt, BoundEffects::default())).id();
    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        0,
        "no effects should be dispatched when breaker not in registry"
    );
    let bolt_bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bolt_bound.0.len(),
        0,
        "no effects should be dispatched when breaker not in registry"
    );
}

// ---- Behavior 19: No breaker entity is a no-op ----

#[test]
fn dispatch_with_no_breaker_entity_is_noop() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    // No breaker entity spawned
    let bolt = app.world_mut().spawn((Bolt, BoundEffects::default())).id();
    app.update();

    let bolt_bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bolt_bound.0.len(),
        0,
        "no effects should be dispatched when no breaker entity exists"
    );
}

// ---- Behavior 20: All children of an On node pushed, not just the first ----

#[test]
fn dispatch_pushes_all_children_of_on_node() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                },
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            ],
        }],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "both children of the On node should be pushed"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::BoltLost,
            ..
        }
    ));
    assert!(matches!(
        &bound.0[1].1,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            ..
        }
    ));
}

// ---- Behavior 21: Empty string used as chip name for all pushed effects ----

#[test]
fn dispatch_uses_empty_string_as_chip_name_for_all_targets() {
    let def = BreakerDefinition {
        name: TEST_BREAKER_NAME.to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
        ],
    };
    let mut app = test_app_with_dispatch(def);
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    let bolt = app.world_mut().spawn(Bolt).id();
    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        &breaker_bound.0[0].0, "",
        "breaker chip name should be empty string"
    );

    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects");
    assert_eq!(
        &bolt_bound.0[0].0, "",
        "bolt chip name should be empty string"
    );
}
