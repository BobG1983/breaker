//! Tests for On-node handling in `walk_bound_node`, `walk_staged_node`,
//! and `ResolveOnCommand` resolution (behaviors 14-24).

use bevy::prelude::*;

use super::super::system::*;
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::components::Cell,
    effect::{commands::ResolveOnCommand, core::*, effects::speed_boost::ActiveSpeedBoosts},
    wall::components::Wall,
};

// -----------------------------------------------------------------------
// Test helper systems
// -----------------------------------------------------------------------

fn sys_evaluate_bound_for_node_start(
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::NodeStart;
    for (entity, bound, mut staged) in &mut query {
        evaluate_bound_effects(&trigger, entity, bound, &mut staged, &mut commands);
    }
}

fn sys_evaluate_staged_for_node_start(
    mut query: Query<(Entity, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::NodeStart;
    for (entity, mut staged) in &mut query {
        evaluate_staged_effects(&trigger, entity, &mut staged, &mut commands);
    }
}

fn sys_evaluate_staged_for_bump(
    mut query: Query<(Entity, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::Bump;
    for (entity, mut staged) in &mut query {
        evaluate_staged_effects(&trigger, entity, &mut staged, &mut commands);
    }
}

// -----------------------------------------------------------------------
// Section K: walk_bound_node pushes On children to StagedEffects
// -----------------------------------------------------------------------

// ── Behavior 14: walk_bound_node pushes On children to StagedEffects ──

#[test]
fn walk_bound_node_pushes_on_child_to_staged_effects_when_trigger_matches() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let bound_node = EffectNode::When {
        trigger: Trigger::NodeStart,
        then: vec![EffectNode::On {
            target: Target::AllCells,
            permanent: true,
            then: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
            }],
        }],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![("cell_fortify".into(), bound_node)]),
            StagedEffects::default(),
        ))
        .id();

    app.add_systems(Update, sys_evaluate_bound_for_node_start);
    app.update();

    // After evaluation, StagedEffects should have 1 entry (the On node)
    let staged = app.world().get::<StagedEffects>(entity).unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "StagedEffects should have 1 entry (the On node pushed from walk_bound_node)"
    );
    assert_eq!(staged.0[0].0, "cell_fortify", "chip_name preserved");
    assert!(
        matches!(
            &staged.0[0].1,
            EffectNode::On {
                target: Target::AllCells,
                permanent: true,
                then: inner,
            } if inner.len() == 1
        ),
        "Pushed entry should be the On(AllCells, permanent: true, ...) node, got {:?}",
        staged.0[0].1
    );

    // BoundEffects should be unchanged (entries are never consumed)
    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 1, "BoundEffects entry must not be consumed");
}

// ── Behavior 14 edge case: On node with multiple children ──

#[test]
fn walk_bound_node_pushes_on_with_multiple_children_as_single_entry() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let bound_node = EffectNode::When {
        trigger: Trigger::NodeStart,
        then: vec![EffectNode::On {
            target: Target::AllBolts,
            permanent: true,
            then: vec![
                EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
                },
                EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 500.0,
                    })],
                },
            ],
        }],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![("bolt_buff".into(), bound_node)]),
            StagedEffects::default(),
        ))
        .id();

    app.add_systems(Update, sys_evaluate_bound_for_node_start);
    app.update();

    let staged = app.world().get::<StagedEffects>(entity).unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "Entire On node (with both children) should be pushed as a single entry"
    );

    if let EffectNode::On { then: inner, .. } = &staged.0[0].1 {
        assert_eq!(
            inner.len(),
            2,
            "On node should have 2 children (both When nodes)"
        );
    } else {
        panic!("Expected On(...) in StagedEffects, got {:?}", staged.0[0].1);
    }
}

// -----------------------------------------------------------------------
// Section L: walk_staged_node handles On nodes via ResolveOnCommand
// -----------------------------------------------------------------------

// ── Behavior 15: On in StagedEffects consumed and children transferred to targets ──

#[test]
fn on_node_in_staged_effects_consumed_and_resolved_to_target_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Source entity with the On node in StagedEffects
    let source = app
        .world_mut()
        .spawn(StagedEffects(vec![(
            "cell_fortify".into(),
            EffectNode::On {
                target: Target::AllCells,
                permanent: true,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                }],
            },
        )]))
        .id();

    // Target Cell entities
    let cell_a = app
        .world_mut()
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();
    let cell_b = app
        .world_mut()
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    app.add_systems(Update, sys_evaluate_staged_for_node_start);
    // First update: evaluate_staged_effects runs, queues ResolveOnCommand
    app.update();

    // The On entry should be consumed from source's StagedEffects
    let staged = app.world().get::<StagedEffects>(source).unwrap();
    assert_eq!(
        staged.0.len(),
        0,
        "On node should be consumed from StagedEffects after evaluation"
    );

    // After command application, each Cell should have BoundEffects updated
    for (label, cell) in [("cell_a", cell_a), ("cell_b", cell_b)] {
        let bound = app.world().get::<BoundEffects>(cell).unwrap();
        assert_eq!(
            bound.0.len(),
            1,
            "{label} should have 1 BoundEffects entry after ResolveOnCommand"
        );
        assert_eq!(bound.0[0].0, "cell_fortify", "{label} chip_name preserved");
        assert!(
            matches!(
                &bound.0[0].1,
                EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    ..
                }
            ),
            "{label} should have When(Impacted(Bolt), ...) in BoundEffects, got {:?}",
            bound.0[0].1
        );
    }
}

// ── Behavior 15 edge case: permanent: false sends to StagedEffects ──

#[test]
fn on_node_with_permanent_false_sends_children_to_staged_effects_on_targets() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let source = app
        .world_mut()
        .spawn(StagedEffects(vec![(
            "cell_fortify".into(),
            EffectNode::On {
                target: Target::AllCells,
                permanent: false,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                }],
            },
        )]))
        .id();

    let cell = app
        .world_mut()
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    app.add_systems(Update, sys_evaluate_staged_for_node_start);
    app.update();

    // On node consumed from source
    let staged = app.world().get::<StagedEffects>(source).unwrap();
    assert_eq!(staged.0.len(), 0, "On node should be consumed");

    // With permanent: false, children go to StagedEffects (not BoundEffects)
    let cell_staged = app.world().get::<StagedEffects>(cell).unwrap();
    assert_eq!(
        cell_staged.0.len(),
        1,
        "Cell should have 1 StagedEffects entry (permanent: false)"
    );

    let cell_bound = app.world().get::<BoundEffects>(cell).unwrap();
    assert!(
        cell_bound.0.is_empty(),
        "Cell BoundEffects should remain empty when permanent: false"
    );
}

// -----------------------------------------------------------------------
// ResolveOnCommand unit tests (behaviors 16-23)
// -----------------------------------------------------------------------

// ── Behavior 16: ResolveOnCommand resolves AllCells ──

#[test]
fn resolve_on_command_resolves_all_cells_to_cell_entities() {
    let mut world = World::new();
    let cell_a = world
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();
    let cell_b = world
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();
    let cell_c = world
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    // Also spawn non-target entities to ensure they are not affected
    let breaker = world
        .spawn((Breaker, BoundEffects::default(), StagedEffects::default()))
        .id();
    let bolt = world
        .spawn((
            Bolt,
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveSpeedBoosts::default(),
        ))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::AllCells,
        chip_name: "cell_fortify".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    // Each Cell should have 1 BoundEffects entry
    for (label, entity) in [("cell_a", cell_a), ("cell_b", cell_b), ("cell_c", cell_c)] {
        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
        assert_eq!(bound.0[0].0, "cell_fortify", "{label} chip_name");
        assert!(
            matches!(
                &bound.0[0].1,
                EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    ..
                }
            ),
            "{label} node should be When(Impacted(Bolt), ...)"
        );
    }

    // Non-target entities should be unaffected
    let breaker_bound = world.get::<BoundEffects>(breaker).unwrap();
    assert!(
        breaker_bound.0.is_empty(),
        "Breaker BoundEffects should be unchanged"
    );
    let bolt_bound = world.get::<BoundEffects>(bolt).unwrap();
    assert!(
        bolt_bound.0.is_empty(),
        "Bolt BoundEffects should be unchanged"
    );
}

// ── Behavior 16 edge case: AllCells with Do children fires immediately ──

#[test]
fn resolve_on_command_all_cells_with_do_children_fires_immediately() {
    use crate::effect::effects::damage_boost::ActiveDamageBoosts;

    let mut world = World::new();
    let cell = world
        .spawn((
            Cell,
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveDamageBoosts::default(),
        ))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::AllCells,
        chip_name: "cell_damage".to_string(),
        children: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        permanent: true,
    };
    cmd.apply(&mut world);

    // Do child should fire immediately
    let boosts = world.get::<ActiveDamageBoosts>(cell).unwrap();
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Do child should fire immediately on Cell entity"
    );

    // BoundEffects should remain empty (Do fires, not pushed)
    let bound = world.get::<BoundEffects>(cell).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects should remain empty when only Do children"
    );
}

// ── Behavior 17: ResolveOnCommand resolves AllBolts ──

#[test]
fn resolve_on_command_resolves_all_bolts_to_bolt_entities() {
    let mut world = World::new();
    let bolt_a = world
        .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
        .id();
    let bolt_b = world
        .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::AllBolts,
        chip_name: "bolt_chain".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(EffectKind::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 500.0,
            })],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    for (label, entity) in [("bolt_a", bolt_a), ("bolt_b", bolt_b)] {
        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
        assert_eq!(bound.0[0].0, "bolt_chain", "{label} chip_name");
    }
}

// ── Behavior 17 edge case: AllBolts with multiple children ──

#[test]
fn resolve_on_command_all_bolts_with_multiple_children() {
    let mut world = World::new();
    let bolt = world
        .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::AllBolts,
        chip_name: "bolt_chain".to_string(),
        children: vec![
            EffectNode::When {
                trigger: Trigger::Bumped,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
            },
            EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 2.0 })],
            },
        ],
        permanent: true,
    };
    cmd.apply(&mut world);

    let bound = world.get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Bolt should have 2 BoundEffects entries (one per child)"
    );
    assert_eq!(bound.0[0].0, "bolt_chain");
    assert_eq!(bound.0[1].0, "bolt_chain");
}

// ── Behavior 18: ResolveOnCommand resolves AllWalls ──

#[test]
fn resolve_on_command_resolves_all_walls_to_wall_entities() {
    let mut world = World::new();
    let wall_a = world
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id();
    let wall_b = world
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::AllWalls,
        chip_name: "wall_boost".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    for (label, entity) in [("wall_a", wall_a), ("wall_b", wall_b)] {
        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
        assert_eq!(bound.0[0].0, "wall_boost", "{label} chip_name");
    }
}

// ── Behavior 18 edge case: Single Wall entity ──

#[test]
fn resolve_on_command_all_walls_with_single_wall() {
    let mut world = World::new();
    let wall = world
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::AllWalls,
        chip_name: "wall_boost".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    let bound = world.get::<BoundEffects>(wall).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Single Wall should have 1 BoundEffects entry"
    );
}

// ── Behavior 19: Bolt target resolves to all Bolt entities ──

#[test]
fn resolve_on_command_bolt_target_fires_do_on_all_bolt_entities() {
    let mut world = World::new();
    let bolt_a = world
        .spawn((
            Bolt,
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveSpeedBoosts::default(),
        ))
        .id();
    let bolt_b = world
        .spawn((
            Bolt,
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveSpeedBoosts::default(),
        ))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Bolt,
        chip_name: "bolt_speed".to_string(),
        children: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
        permanent: true,
    };
    cmd.apply(&mut world);

    for (label, entity) in [("bolt_a", bolt_a), ("bolt_b", bolt_b)] {
        let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(
            speed.0,
            vec![1.2],
            "{label} should have ActiveSpeedBoosts [1.2] from fired Do"
        );
    }
}

// ── Behavior 19 edge case: Single Bolt entity ──

#[test]
fn resolve_on_command_bolt_target_with_single_bolt() {
    let mut world = World::new();
    let bolt = world
        .spawn((
            Bolt,
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveSpeedBoosts::default(),
        ))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Bolt,
        chip_name: "bolt_speed".to_string(),
        children: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
        permanent: true,
    };
    cmd.apply(&mut world);

    let speed = world.get::<ActiveSpeedBoosts>(bolt).unwrap();
    assert_eq!(
        speed.0,
        vec![1.2],
        "Single Bolt should have ActiveSpeedBoosts [1.2]"
    );
}

// ── Behavior 20: Cell target resolves to all Cell entities ──

#[test]
fn resolve_on_command_cell_target_resolves_to_all_cell_entities() {
    let mut world = World::new();
    let cell_a = world
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();
    let cell_b = world
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Cell,
        chip_name: "cell_armor".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 2 })],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    for (label, entity) in [("cell_a", cell_a), ("cell_b", cell_b)] {
        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
        assert_eq!(bound.0[0].0, "cell_armor", "{label} chip_name");
    }
}

// ── Behavior 21: Wall target resolves to all Wall entities ──

#[test]
fn resolve_on_command_wall_target_resolves_to_all_wall_entities() {
    let mut world = World::new();
    let wall = world
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Wall,
        chip_name: "wall_reflect".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    let bound = world.get::<BoundEffects>(wall).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "wall_reflect");
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                ..
            }
        ),
        "Wall entry should be When(Impacted(Bolt), ...)"
    );
}

// ── Behavior 21 edge case: Three Wall entities ──

#[test]
fn resolve_on_command_wall_target_with_three_walls() {
    let mut world = World::new();
    let wall_a = world
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id();
    let wall_b = world
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id();
    let wall_c = world
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Wall,
        chip_name: "wall_reflect".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    for (label, entity) in [("wall_a", wall_a), ("wall_b", wall_b), ("wall_c", wall_c)] {
        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
    }
}

// ── Behavior 22: No matching entities -- no-op ──

#[test]
fn resolve_on_command_with_no_matching_entities_is_noop() {
    let mut world = World::new();

    // Spawn a Breaker but target AllCells -- no Cell entities exist
    let breaker = world
        .spawn((Breaker, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::AllCells,
        chip_name: "cell_fortify".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        }],
        permanent: true,
    };
    // Should not panic
    cmd.apply(&mut world);

    // Breaker's BoundEffects should remain empty
    let breaker_bound = world.get::<BoundEffects>(breaker).unwrap();
    assert!(
        breaker_bound.0.is_empty(),
        "Breaker BoundEffects should remain empty (not an AllCells target)"
    );
}

// ── Behavior 22 edge case: AllBolts with no bolts ──

#[test]
fn resolve_on_command_all_bolts_with_no_bolts_is_noop() {
    let mut world = World::new();

    let cmd = ResolveOnCommand {
        target: Target::AllBolts,
        chip_name: "bolt_chain".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(EffectKind::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 500.0,
            })],
        }],
        permanent: true,
    };
    // Should not panic
    cmd.apply(&mut world);
}

// ── Behavior 22 edge case: AllWalls with no walls ──

#[test]
fn resolve_on_command_all_walls_with_no_walls_is_noop() {
    let mut world = World::new();

    let cmd = ResolveOnCommand {
        target: Target::AllWalls,
        chip_name: "wall_boost".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: true,
    };
    // Should not panic
    cmd.apply(&mut world);
}

// ── Behavior 23: Breaker target resolves to Breaker entity ──

#[test]
fn resolve_on_command_breaker_target_resolves_to_breaker_entity() {
    let mut world = World::new();
    let breaker = world
        .spawn((Breaker, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Breaker,
        chip_name: "breaker_buff".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    let bound = world.get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1, "Breaker should have 1 BoundEffects entry");
    assert_eq!(bound.0[0].0, "breaker_buff");
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                ..
            }
        ),
        "Breaker entry should be When(PerfectBump, ...)"
    );
}

// ── Behavior 23 edge case: Breaker target with no Breaker entity ──

#[test]
fn resolve_on_command_breaker_target_with_no_breaker_is_noop() {
    let mut world = World::new();

    let cmd = ResolveOnCommand {
        target: Target::Breaker,
        chip_name: "breaker_buff".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        }],
        permanent: true,
    };
    // Should not panic
    cmd.apply(&mut world);
}

// ── Behavior 24: On node in StagedEffects consumed regardless of trigger ──

#[test]
fn on_node_in_staged_effects_consumed_regardless_of_trigger() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let source = app
        .world_mut()
        .spawn(StagedEffects(vec![(
            "cell_fortify".into(),
            EffectNode::On {
                target: Target::AllCells,
                permanent: true,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                }],
            },
        )]))
        .id();

    let cell = app
        .world_mut()
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    // Evaluate for Bump (NOT NodeStart) -- On node should still be consumed
    app.add_systems(Update, sys_evaluate_staged_for_bump);
    app.update();

    let staged = app.world().get::<StagedEffects>(source).unwrap();
    assert_eq!(
        staged.0.len(),
        0,
        "On node should be consumed regardless of which trigger is being evaluated"
    );

    // Cell should still get the resolved entry
    let cell_bound = app.world().get::<BoundEffects>(cell).unwrap();
    assert_eq!(
        cell_bound.0.len(),
        1,
        "Cell should have 1 BoundEffects entry from the resolved On node"
    );
}

// ── Behavior 24 edge case: Mixed On and When in StagedEffects ──

#[test]
fn mixed_on_and_when_in_staged_effects_both_consumed_when_trigger_matches() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let source = app
        .world_mut()
        .spawn(StagedEffects(vec![
            (
                "chip_a".into(),
                EffectNode::On {
                    target: Target::AllCells,
                    permanent: true,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                    }],
                },
            ),
            (
                "chip_b".into(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Death,
                        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
                    }],
                },
            ),
        ]))
        .id();

    let _cell = app
        .world_mut()
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    // Evaluate for Bump: On consumed (trigger-independent), When(Bump) also consumed
    app.add_systems(Update, sys_evaluate_staged_for_bump);
    app.update();

    let staged = app.world().get::<StagedEffects>(source).unwrap();
    // The On node is consumed, the When(Bump) is consumed (its non-Do child When(Death) is added),
    // so we expect 1 addition from the When(Bump) match
    assert_eq!(
        staged.0.len(),
        1,
        "After evaluation: On consumed, When(Bump) consumed, When(Death) added as addition. Net: 1"
    );
    assert!(
        matches!(
            &staged.0[0].1,
            EffectNode::When {
                trigger: Trigger::Death,
                ..
            }
        ),
        "Remaining entry should be the When(Death, ...) addition from the consumed When(Bump)"
    );
}
