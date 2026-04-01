//! Tests for `ResolveOnCommand` resolving `AllCells`, `AllBolts`, `AllWalls` targets
//! (Behaviors 16-18).

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::components::Cell,
    effect::{commands::ResolveOnCommand, core::*, effects::speed_boost::ActiveSpeedBoosts},
    wall::components::Wall,
};

// -----------------------------------------------------------------------
// Behavior 16: ResolveOnCommand resolves AllCells
// -----------------------------------------------------------------------

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
        context_entity: None,
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
        context_entity: None,
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

// -----------------------------------------------------------------------
// Behavior 17: ResolveOnCommand resolves AllBolts
// -----------------------------------------------------------------------

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
        context_entity: None,
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
        context_entity: None,
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

// -----------------------------------------------------------------------
// Behavior 18: ResolveOnCommand resolves AllWalls
// -----------------------------------------------------------------------

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
        context_entity: None,
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
        context_entity: None,
    };
    cmd.apply(&mut world);

    let bound = world.get::<BoundEffects>(wall).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Single Wall should have 1 BoundEffects entry"
    );
}

// -----------------------------------------------------------------------
// Context entity must NOT narrow AllCells/AllBolts/AllWalls
// -----------------------------------------------------------------------

#[test]
fn all_cells_with_context_entity_still_resolves_to_all_cells() {
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

    // context_entity points to cell_b, but AllCells should still hit all 3
    let cmd = ResolveOnCommand {
        target: Target::AllCells,
        chip_name: "all_test".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context_entity: Some(cell_b),
    };
    cmd.apply(&mut world);

    for (label, entity) in [("cell_a", cell_a), ("cell_b", cell_b), ("cell_c", cell_c)] {
        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "{label} should have 1 StagedEffects entry — AllCells must not be narrowed by context_entity"
        );
    }
}

#[test]
fn all_bolts_with_context_entity_still_resolves_to_all_bolts() {
    let mut world = World::new();
    let bolt_a = world
        .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
        .id();
    let bolt_b = world
        .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
        .id();
    let bolt_c = world
        .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::AllBolts,
        chip_name: "all_test".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context_entity: Some(bolt_b),
    };
    cmd.apply(&mut world);

    for (label, entity) in [("bolt_a", bolt_a), ("bolt_b", bolt_b), ("bolt_c", bolt_c)] {
        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "{label} should have 1 StagedEffects entry — AllBolts must not be narrowed by context_entity"
        );
    }
}

#[test]
fn all_walls_with_context_entity_still_resolves_to_all_walls() {
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
        target: Target::AllWalls,
        chip_name: "all_test".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context_entity: Some(wall_b),
    };
    cmd.apply(&mut world);

    for (label, entity) in [("wall_a", wall_a), ("wall_b", wall_b), ("wall_c", wall_c)] {
        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "{label} should have 1 StagedEffects entry — AllWalls must not be narrowed by context_entity"
        );
    }
}
