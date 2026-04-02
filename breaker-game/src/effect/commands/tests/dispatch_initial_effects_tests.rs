//! Tests for `DispatchInitialEffects` command (behaviors 1-15).

use bevy::prelude::*;

use super::super::ext::*;
use crate::{
    bolt::components::{Bolt, ExtraBolt, PrimaryBolt},
    breaker::{components::Breaker, definition::BreakerDefinition},
    cells::components::Cell,
    effect::{core::*, effects::damage_boost::ActiveDamageBoosts},
    wall::components::Wall,
};

// ── Behavior 1: Breaker target with Do effect fires immediately ──────────

#[test]
fn breaker_target_do_effect_fires_immediately() {
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
    world.entity_mut(breaker).insert(ActiveDamageBoosts(vec![]));

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("ActiveDamageBoosts should be present");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Do(DamageBoost(2.0)) should fire immediately on Breaker"
    );
}

#[test]
fn breaker_target_multiple_bare_do_children_all_fire() {
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
    world.entity_mut(breaker).insert(ActiveDamageBoosts(vec![]));

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
                EffectNode::Do(EffectKind::DamageBoost(3.0)),
            ],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("ActiveDamageBoosts should be present");
    assert_eq!(
        boosts.0.len(),
        2,
        "Both Do children should fire immediately on Breaker"
    );
}

// ── Behavior 2: Breaker target with When effect pushes to BoundEffects ───

#[test]
fn breaker_target_when_effect_pushes_to_bound_effects() {
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
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have exactly 1 entry for the When node"
    );
    assert_eq!(
        bound.0[0].0, "",
        "Chip name should be empty string when source_chip is None"
    );
    assert_eq!(
        bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        "Stored effect node should match the When node"
    );
}

#[test]
fn breaker_target_mixed_do_and_when_fires_do_stores_when() {
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
    world.entity_mut(breaker).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts(vec![]),
    ));

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            ],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("ActiveDamageBoosts should be present");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Do(DamageBoost(2.0)) should fire immediately"
    );

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have exactly 1 entry (the When node)"
    );
}

// ── Behavior 3: Bolt target dispatches to PrimaryBolt only ───────────────

#[test]
fn bolt_target_dispatches_to_primary_bolt_only() {
    let mut world = World::new();
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let extra = world
        .spawn((
            Bolt,
            ExtraBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "PrimaryBolt should have 1 BoundEffects entry"
    );
    assert_eq!(
        primary_bound.0[0].0, "",
        "Chip name should be empty string for source_chip None"
    );

    let extra_bound = world
        .get::<BoundEffects>(extra)
        .expect("ExtraBolt should have BoundEffects");
    assert!(
        extra_bound.0.is_empty(),
        "ExtraBolt should have 0 BoundEffects entries (Bolt target -> PrimaryBolt only)"
    );
}

#[test]
fn bolt_target_no_primary_bolt_but_breaker_still_processed() {
    // Bolt target with no PrimaryBolt is a no-op for bolt, but a co-dispatched
    // Breaker target must still work. Tests both no-op bolt + positive breaker.
    let mut world = World::new();
    let _extra = world
        .spawn((
            Bolt,
            ExtraBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
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
        effects: vec![
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // Breaker must still be processed even though Bolt target was a no-op
    let breaker_bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry even though Bolt target had no PrimaryBolt"
    );
}

// ── Behavior 4: Cell target is a no-op (no effects dispatched) ───────────

#[test]
fn cell_target_is_noop_but_breaker_target_processes() {
    // Cell target should be skipped, while a Breaker target in the same call processes.
    let mut world = World::new();
    let cell_a = world.spawn((Cell, BoundEffects::default())).id();
    let cell_b = world.spawn((Cell, BoundEffects::default())).id();
    let breaker = world
        .spawn({
            let def = BreakerDefinition::default();
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build()
        })
        .id();
    world.entity_mut(breaker).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts(vec![]),
    ));

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::Cell,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.1))],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // Cells must remain untouched
    let bound_a = world.get::<BoundEffects>(cell_a).unwrap();
    let bound_b = world.get::<BoundEffects>(cell_b).unwrap();
    assert!(
        bound_a.0.is_empty(),
        "Cell target should be noop -- cell_a BoundEffects should be empty"
    );
    assert!(
        bound_b.0.is_empty(),
        "Cell target should be noop -- cell_b BoundEffects should be empty"
    );

    // Breaker must be processed (fails with stub)
    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("Breaker should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Breaker Do(DamageBoost(2.0)) should fire even though Cell target was noop"
    );
}

#[test]
fn cell_target_when_children_noop_with_breaker_processed() {
    let mut world = World::new();
    let cell = world.spawn((Cell, BoundEffects::default())).id();
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
        effects: vec![
            RootEffect::On {
                target: Target::Cell,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.1))],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    let cell_bound = world.get::<BoundEffects>(cell).unwrap();
    assert!(
        cell_bound.0.is_empty(),
        "Cell target with When children should still be noop"
    );

    let breaker_bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker When effect should be stored even though Cell target was noop"
    );
}

// ── Behavior 5: Wall target is a no-op (no effects dispatched) ───────────

#[test]
fn wall_target_is_noop_but_breaker_target_processes() {
    let mut world = World::new();
    let wall_a = world.spawn((Wall, BoundEffects::default())).id();
    let wall_b = world.spawn((Wall, BoundEffects::default())).id();
    let breaker = world
        .spawn({
            let def = BreakerDefinition::default();
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build()
        })
        .id();
    world.entity_mut(breaker).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts(vec![]),
    ));

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
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
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    let bound_a = world.get::<BoundEffects>(wall_a).unwrap();
    let bound_b = world.get::<BoundEffects>(wall_b).unwrap();
    assert!(
        bound_a.0.is_empty(),
        "Wall target should be noop -- wall_a BoundEffects should be empty"
    );
    assert!(
        bound_b.0.is_empty(),
        "Wall target should be noop -- wall_b BoundEffects should be empty"
    );

    // Breaker must be processed (fails with stub)
    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("Breaker should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Breaker Do should fire even though Wall target was noop"
    );
}

#[test]
fn wall_target_do_children_noop_with_breaker_processed() {
    let mut world = World::new();
    let wall = world.spawn((Wall, BoundEffects::default())).id();
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
        effects: vec![
            RootEffect::On {
                target: Target::Wall,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    let wall_bound = world.get::<BoundEffects>(wall).unwrap();
    assert!(
        wall_bound.0.is_empty(),
        "Wall target with bare Do children should still be noop"
    );

    let breaker_bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker When effect should be stored even though Wall target was noop"
    );
}

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

// ── Behavior 9: No breaker exists -- AllBolts/AllCells/AllWalls graceful no-op ──
// These are true no-ops: no crash, no effects dispatched. To ensure the test
// fails with the stub, we also dispatch a Bolt-targeted effect in the same call
// and verify the PrimaryBolt was processed.

#[test]
fn no_breaker_all_bolts_graceful_noop_bolt_still_processed() {
    let mut world = World::new();
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::AllBolts,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // PrimaryBolt should get the Bolt-targeted When (fails with stub)
    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "No breaker -> AllBolts deferred is silently dropped, but Bolt target should still work"
    );
}

#[test]
fn no_breaker_all_cells_graceful_noop_breaker_absent_no_panic() {
    // Edge case: no breaker, AllCells target. Must not panic.
    // Combined with a Bolt target for a positive assertion.
    let mut world = World::new();
    let _cell = world.spawn((Cell, BoundEffects::default())).id();
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::AllCells,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "No breaker -> AllCells deferred dropped, but Bolt target should still work"
    );
}

#[test]
fn no_breaker_all_walls_graceful_noop_bolt_still_processed() {
    let mut world = World::new();
    let _wall = world.spawn((Wall, BoundEffects::default())).id();
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::AllWalls,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "No breaker -> AllWalls deferred dropped, but Bolt target should still work"
    );
}

// ── Behavior 10: Source chip Some passthrough to BoundEffects entries ─────

#[test]
fn source_chip_some_passes_through_to_bound_effects() {
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
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        source_chip: Some("overclock".to_string()),
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(bound.0.len(), 1, "BoundEffects should have exactly 1 entry");
    assert_eq!(
        bound.0[0].0, "overclock",
        "Chip name should be 'overclock' from source_chip Some('overclock')"
    );
}

#[test]
fn source_chip_special_characters_stored_as_is() {
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
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
        }],
        source_chip: Some("flux-v2.1".to_string()),
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(bound.0.len(), 1, "BoundEffects should have exactly 1 entry");
    assert_eq!(
        bound.0[0].0, "flux-v2.1",
        "Special characters in chip name should be stored as-is"
    );
}

// ── Behavior 11: Source chip None maps to empty string ───────────────────

#[test]
fn source_chip_none_maps_to_empty_string() {
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
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::LoseLife)],
            }],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(bound.0.len(), 1, "BoundEffects should have exactly 1 entry");
    assert_eq!(
        bound.0[0].0, "",
        "Source chip None should produce empty string chip name"
    );
}

#[test]
fn source_chip_some_empty_string_same_as_none() {
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
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::LoseLife)],
            }],
        }],
        source_chip: Some(String::new()),
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(bound.0.len(), 1, "BoundEffects should have exactly 1 entry");
    assert_eq!(
        bound.0[0].0, "",
        "Source chip Some('') should produce empty string chip name (same as None)"
    );
}

// ── Behavior 12: Empty effects list is a no-op ──────────────────────────
// An empty effects list means nothing to dispatch -- the stub already does
// nothing. To guarantee RED failure, we dispatch an empty list alongside a
// non-empty list and assert the non-empty one was processed.

#[test]
fn empty_effects_list_alongside_real_effect() {
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

    // First call: empty list (should be a no-op)
    DispatchInitialEffects {
        effects: vec![],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert!(
        bound.0.is_empty(),
        "Empty effects list should not add any BoundEffects entries"
    );

    // Second call: real effect (must be processed -- fails with stub)
    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(
        bound.0.len(),
        1,
        "Non-empty effects list should add 1 BoundEffects entry"
    );
}

#[test]
fn on_with_empty_then_alongside_real_effect() {
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
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(
        bound.0.len(),
        1,
        "On(Breaker) with empty then should add 0 entries, but the second On should add 1"
    );
}

// ── Behavior 13: Multiple RootEffects with different targets all processed ──

#[test]
fn multiple_root_effects_different_targets_all_processed() {
    let mut world = World::new();
    let breaker = world
        .spawn({
            let def = BreakerDefinition::default();
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build()
        })
        .id();
    world.entity_mut(breaker).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts(vec![]),
    ));
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let extra = world
        .spawn((
            Bolt,
            ExtraBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![
                    EffectNode::Do(EffectKind::DamageBoost(2.0)),
                    EffectNode::When {
                        trigger: Trigger::BoltLost,
                        then: vec![EffectNode::Do(EffectKind::LoseLife)],
                    },
                ],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // Breaker: Do fired, When stored
    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("Breaker should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Breaker's DamageBoost(2.0) should fire immediately"
    );
    let breaker_bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry (When(BoltLost))"
    );

    // PrimaryBolt: When stored
    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "PrimaryBolt should have 1 BoundEffects entry (When(PerfectBumped))"
    );

    // ExtraBolt: nothing
    let extra_bound = world
        .get::<BoundEffects>(extra)
        .expect("ExtraBolt should have BoundEffects");
    assert!(
        extra_bound.0.is_empty(),
        "ExtraBolt should have 0 BoundEffects entries"
    );
}

#[test]
fn three_root_effects_breaker_bolt_all_bolts() {
    let mut world = World::new();
    let breaker = world
        .spawn({
            let def = BreakerDefinition::default();
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build()
        })
        .id();
    world.entity_mut(breaker).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts(vec![]),
    ));
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::AllBolts,
                then: vec![EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // Breaker: Do fired + AllBolts deferred wrapper
    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("Breaker should have ActiveDamageBoosts");
    assert_eq!(boosts.0, vec![2.0], "Breaker's Do should fire immediately");

    let breaker_bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry (AllBolts deferred wrapper)"
    );

    // PrimaryBolt: Bolt-targeted When only, NOT the AllBolts deferred one
    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "PrimaryBolt should have 1 BoundEffects entry (Bolt target only, not AllBolts deferred)"
    );
}

// ── Behavior 14: BoundEffects and StagedEffects inserted if absent ───────

#[test]
fn bound_effects_and_staged_effects_inserted_if_absent() {
    let mut world = World::new();
    // Spawn breaker with ONLY the marker -- no BoundEffects/StagedEffects
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

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be inserted when absent");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry after dispatch"
    );

    let staged = world
        .get::<StagedEffects>(breaker)
        .expect("StagedEffects should be inserted when absent");
    assert!(
        staged.0.is_empty(),
        "StagedEffects should be inserted empty"
    );
}

#[test]
fn prior_bound_effects_preserved_new_entry_appended() {
    let mut world = World::new();
    let prior_entry = (
        "prior_chip".to_string(),
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(0.5))],
        },
    );
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
        .insert(BoundEffects(vec![prior_entry]));

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 entries: 1 prior + 1 new"
    );
    assert_eq!(
        bound.0[0].0, "prior_chip",
        "Prior entry should be preserved at index 0"
    );
    assert_eq!(
        bound.0[1].0, "",
        "New entry should be appended at index 1 with empty chip name"
    );

    // StagedEffects should be inserted (was absent)
    let staged = world
        .get::<StagedEffects>(breaker)
        .expect("StagedEffects should be inserted when absent");
    assert!(
        staged.0.is_empty(),
        "StagedEffects should be inserted empty"
    );
}

// ── Behavior 15: Bolt target with bare Do fires on PrimaryBolt immediately ──

#[test]
fn bolt_target_bare_do_fires_on_primary_bolt() {
    let mut world = World::new();
    let primary = world
        .spawn((Bolt, PrimaryBolt, ActiveDamageBoosts(vec![])))
        .id();
    let extra = world
        .spawn((Bolt, ExtraBolt, ActiveDamageBoosts(vec![])))
        .id();

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let primary_boosts = world
        .get::<ActiveDamageBoosts>(primary)
        .expect("PrimaryBolt should have ActiveDamageBoosts");
    assert_eq!(
        primary_boosts.0,
        vec![1.5],
        "Do(DamageBoost(1.5)) should fire on PrimaryBolt"
    );

    let extra_boosts = world
        .get::<ActiveDamageBoosts>(extra)
        .expect("ExtraBolt should have ActiveDamageBoosts");
    assert!(
        extra_boosts.0.is_empty(),
        "ExtraBolt should NOT have DamageBoost fired (Bolt target -> PrimaryBolt only)"
    );
}

#[test]
fn bolt_target_do_fires_on_only_bolt_when_it_is_primary() {
    let mut world = World::new();
    let primary = world
        .spawn((Bolt, PrimaryBolt, ActiveDamageBoosts(vec![])))
        .id();

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let boosts = world
        .get::<ActiveDamageBoosts>(primary)
        .expect("PrimaryBolt should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![1.5],
        "Single PrimaryBolt should receive the Do effect"
    );
}

#[test]
fn bolt_target_do_no_bolts_alongside_breaker() {
    // Zero bolts -> Bolt target is no-op. Breaker target must still work.
    let mut world = World::new();
    let breaker = world
        .spawn({
            let def = BreakerDefinition::default();
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build()
        })
        .id();
    world.entity_mut(breaker).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts(vec![]),
    ));

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // Breaker must still be processed (fails with stub)
    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("Breaker should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Breaker Do should fire even with zero bolts for Bolt target"
    );
}
