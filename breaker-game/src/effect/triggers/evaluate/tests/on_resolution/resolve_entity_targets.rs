//! Tests for `ResolveOnCommand` resolving Bolt, Cell, Wall, Breaker entity targets
//! (Behaviors 19-21, 23).

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::components::Cell,
    effect::{commands::ResolveOnCommand, core::*, effects::speed_boost::ActiveSpeedBoosts},
    wall::components::Wall,
};

// -----------------------------------------------------------------------
// Behavior 19: Bolt target resolves to all Bolt entities
// -----------------------------------------------------------------------

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

// -----------------------------------------------------------------------
// Behavior 20: Cell target resolves to all Cell entities
// -----------------------------------------------------------------------

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

// -----------------------------------------------------------------------
// Behavior 21: Wall target resolves to all Wall entities
// -----------------------------------------------------------------------

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

// -----------------------------------------------------------------------
// Behavior 23: Breaker target resolves to Breaker entity
// -----------------------------------------------------------------------

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
