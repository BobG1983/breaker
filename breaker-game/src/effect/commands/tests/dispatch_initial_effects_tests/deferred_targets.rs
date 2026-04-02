//! Tests for `AllBolts`, `AllCells`, `AllWalls` deferred target dispatch (behaviors 6-8).

use super::helpers::*;

// ── Behavior 6: AllBolts target is deferred -- wrapped and pushed to first breaker ──

#[test]
fn all_bolts_target_deferred_wrapped_on_first_breaker() {
    let mut world = World::new();
    let def = BreakerDefinition::default();
    let breaker = world
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    world
        .entity_mut(breaker)
        .insert((BoundEffects::default(), StagedEffects::default()));
    let _bolt_a = world.spawn(Bolt).id();
    let _bolt_b = world.spawn(Bolt).id();

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::AllBolts,
            then: vec![EffectNode::When {
                trigger: Trigger::Bumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry for the deferred AllBolts wrapper"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "", "Chip name should be empty string for None");

    // Expected shape: When(NodeStart, [On(AllBolts, permanent: true, [original children])])
    let expected = EffectNode::When {
        trigger: Trigger::NodeStart,
        then: vec![EffectNode::On {
            target: Target::AllBolts,
            permanent: true,
            then: vec![EffectNode::When {
                trigger: Trigger::Bumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    };
    assert_eq!(
        *node, expected,
        "Deferred AllBolts wrapper should be When(NodeStart, [On(AllBolts, permanent: true, [original])])"
    );
}

#[test]
fn all_bolts_do_children_still_deferred_not_fired() {
    let mut world = World::new();
    let def = BreakerDefinition::default();
    let breaker = world
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    world
        .entity_mut(breaker)
        .insert((BoundEffects::default(), StagedEffects::default()));
    let bolt = world.spawn((Bolt, ActiveDamageBoosts(vec![]))).id();

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::AllBolts,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    // The Do children should NOT fire immediately -- they are wrapped in the deferred node
    let boosts = world
        .get::<ActiveDamageBoosts>(bolt)
        .expect("Bolt should have ActiveDamageBoosts");
    assert!(
        boosts.0.is_empty(),
        "AllBolts Do children should be deferred, not fired immediately on bolt"
    );

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 deferred wrapper entry"
    );
}

// ── Behavior 7: AllCells target is deferred ──────────────────────────────

#[test]
fn all_cells_target_deferred_wrapped_on_first_breaker() {
    let mut world = World::new();
    let def = BreakerDefinition::default();
    let breaker = world
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    world
        .entity_mut(breaker)
        .insert((BoundEffects::default(), StagedEffects::default()));
    let cell_a = world.spawn(Cell).id();
    let cell_b = world.spawn(Cell).id();

    let original_children = vec![EffectNode::When {
        trigger: Trigger::Impacted(ImpactTarget::Bolt),
        then: vec![EffectNode::Do(EffectKind::Shockwave {
            base_range: 32.0,
            range_per_level: 8.0,
            stacks: 1,
            speed: 400.0,
        })],
    }];

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::AllCells,
            then: original_children.clone(),
        }],
        source_chip: Some("cascade".to_string()),
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry for the deferred AllCells wrapper"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(
        chip_name, "cascade",
        "Chip name should be 'cascade' for source_chip Some('cascade')"
    );

    let expected = EffectNode::When {
        trigger: Trigger::NodeStart,
        then: vec![EffectNode::On {
            target: Target::AllCells,
            permanent: true,
            then: original_children,
        }],
    };
    assert_eq!(
        *node, expected,
        "Deferred AllCells wrapper should be When(NodeStart, [On(AllCells, permanent: true, [original])])"
    );

    // Cell entities should have no new BoundEffects
    for (cell, label) in [(cell_a, "cell_a"), (cell_b, "cell_b")] {
        assert!(
            world.get::<BoundEffects>(cell).is_none()
                || world.get::<BoundEffects>(cell).unwrap().0.is_empty(),
            "{label} should have no BoundEffects entries (effects deferred to breaker)"
        );
    }
}

#[test]
fn all_cells_multiple_children_wrapped_in_single_on() {
    let mut world = World::new();
    let def = BreakerDefinition::default();
    let breaker = world
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    world
        .entity_mut(breaker)
        .insert((BoundEffects::default(), StagedEffects::default()));

    let children = vec![
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        },
        EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        },
    ];

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::AllCells,
            then: children.clone(),
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "Multiple children should be wrapped in a single deferred entry"
    );

    let expected = EffectNode::When {
        trigger: Trigger::NodeStart,
        then: vec![EffectNode::On {
            target: Target::AllCells,
            permanent: true,
            then: children,
        }],
    };
    assert_eq!(
        bound.0[0].1, expected,
        "All original children should be inside a single On(AllCells, permanent: true, ...)"
    );
}

// ── Behavior 8: AllWalls target is deferred ──────────────────────────────

#[test]
fn all_walls_target_deferred_wrapped_on_first_breaker() {
    let mut world = World::new();
    let def = BreakerDefinition::default();
    let breaker = world
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    world
        .entity_mut(breaker)
        .insert((BoundEffects::default(), StagedEffects::default()));
    let wall_a = world.spawn(Wall).id();
    let wall_b = world.spawn(Wall).id();

    let original_children = vec![EffectNode::When {
        trigger: Trigger::Impacted(ImpactTarget::Bolt),
        then: vec![EffectNode::Do(EffectKind::Shockwave {
            base_range: 32.0,
            range_per_level: 8.0,
            stacks: 1,
            speed: 400.0,
        })],
    }];

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::AllWalls,
            then: original_children.clone(),
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry for the deferred AllWalls wrapper"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "", "Chip name should be empty string for None");

    let expected = EffectNode::When {
        trigger: Trigger::NodeStart,
        then: vec![EffectNode::On {
            target: Target::AllWalls,
            permanent: true,
            then: original_children,
        }],
    };
    assert_eq!(
        *node, expected,
        "Deferred AllWalls wrapper should be When(NodeStart, [On(AllWalls, permanent: true, [original])])"
    );

    // Wall entities should have no new BoundEffects
    for (wall, label) in [(wall_a, "wall_a"), (wall_b, "wall_b")] {
        assert!(
            world.get::<BoundEffects>(wall).is_none()
                || world.get::<BoundEffects>(wall).unwrap().0.is_empty(),
            "{label} should have no BoundEffects entries (effects deferred to breaker)"
        );
    }
}

#[test]
fn all_walls_empty_then_still_creates_wrapper() {
    let mut world = World::new();
    let def = BreakerDefinition::default();
    let breaker = world
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    world
        .entity_mut(breaker)
        .insert((BoundEffects::default(), StagedEffects::default()));

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::AllWalls,
            then: vec![],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "AllWalls with empty then should still create the wrapper entry"
    );

    let expected = EffectNode::When {
        trigger: Trigger::NodeStart,
        then: vec![EffectNode::On {
            target: Target::AllWalls,
            permanent: true,
            then: vec![],
        }],
    };
    assert_eq!(
        bound.0[0].1, expected,
        "Wrapper should contain empty inner then"
    );
}
