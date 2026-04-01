//! Tests for `ResolveOnCommand` resolving singular entity targets (Bolt, Cell, Wall, Breaker).
//!
//! Singular target resolution rules:
//! - `Bolt` without context → entities with `Bolt` + `PrimaryBolt`
//! - `Breaker` without context → entities with `Breaker` + `PrimaryBreaker`
//! - `Cell` without context → no-op (empty)
//! - `Wall` without context → no-op (empty)
//! - Any singular target with `context_entity` → that specific entity (if marker matches)

use bevy::prelude::*;

use crate::{
    bolt::components::{Bolt, PrimaryBolt},
    breaker::components::{Breaker, PrimaryBreaker},
    cells::components::Cell,
    effect::{commands::ResolveOnCommand, core::*, effects::speed_boost::ActiveSpeedBoosts},
    wall::components::Wall,
};

// -----------------------------------------------------------------------
// Bolt without context → PrimaryBolt only
// -----------------------------------------------------------------------

#[test]
fn bolt_without_context_resolves_to_primary_bolt_only() {
    let mut world = World::new();
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveSpeedBoosts::default(),
        ))
        .id();
    let secondary = world
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
        context_entity: None,
    };
    cmd.apply(&mut world);

    let primary_speed = world.get::<ActiveSpeedBoosts>(primary).unwrap();
    assert_eq!(
        primary_speed.0,
        vec![1.2],
        "PrimaryBolt should receive the effect"
    );
    let secondary_speed = world.get::<ActiveSpeedBoosts>(secondary).unwrap();
    assert!(
        secondary_speed.0.is_empty(),
        "Non-primary bolt should NOT receive the effect"
    );
}

#[test]
fn bolt_without_context_and_no_primary_bolt_is_noop() {
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
        context_entity: None,
    };
    cmd.apply(&mut world);

    let speed = world.get::<ActiveSpeedBoosts>(bolt).unwrap();
    assert!(
        speed.0.is_empty(),
        "Bolt without PrimaryBolt should not receive the effect"
    );
}

// -----------------------------------------------------------------------
// Breaker without context → PrimaryBreaker only
// -----------------------------------------------------------------------

#[test]
fn breaker_without_context_resolves_to_primary_breaker_only() {
    let mut world = World::new();
    let primary = world
        .spawn((
            Breaker,
            PrimaryBreaker,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let secondary = world
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
        context_entity: None,
    };
    cmd.apply(&mut world);

    let primary_bound = world.get::<BoundEffects>(primary).unwrap();
    assert_eq!(
        primary_bound.0.len(),
        1,
        "PrimaryBreaker should have 1 BoundEffects entry"
    );
    let secondary_bound = world.get::<BoundEffects>(secondary).unwrap();
    assert!(
        secondary_bound.0.is_empty(),
        "Non-primary breaker should NOT receive the effect"
    );
}

#[test]
fn breaker_without_context_and_no_primary_breaker_is_noop() {
    let mut world = World::new();

    let cmd = ResolveOnCommand {
        target: Target::Breaker,
        chip_name: "breaker_buff".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        }],
        permanent: true,
        context_entity: None,
    };
    // Should not panic
    cmd.apply(&mut world);
}

// -----------------------------------------------------------------------
// Cell without context → no-op
// -----------------------------------------------------------------------

#[test]
fn cell_without_context_is_noop() {
    let mut world = World::new();
    let cell = world
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
        context_entity: None,
    };
    cmd.apply(&mut world);

    let bound = world.get::<BoundEffects>(cell).unwrap();
    assert!(
        bound.0.is_empty(),
        "Cell without context_entity should not receive effects"
    );
}

// -----------------------------------------------------------------------
// Wall without context → no-op
// -----------------------------------------------------------------------

#[test]
fn wall_without_context_is_noop() {
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
        context_entity: None,
    };
    cmd.apply(&mut world);

    let bound = world.get::<BoundEffects>(wall).unwrap();
    assert!(
        bound.0.is_empty(),
        "Wall without context_entity should not receive effects"
    );
}

// -----------------------------------------------------------------------
// Singular targets WITH context_entity → specific entity
// -----------------------------------------------------------------------

#[test]
fn bolt_with_context_resolves_to_specific_bolt() {
    let mut world = World::new();
    let bolt_a = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let bolt_b = world
        .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Bolt,
        chip_name: "test".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context_entity: Some(bolt_b),
    };
    cmd.apply(&mut world);

    let staged_a = world.get::<StagedEffects>(bolt_a).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "bolt_a (primary) should NOT get the effect — context targets bolt_b"
    );
    let staged_b = world.get::<StagedEffects>(bolt_b).unwrap();
    assert_eq!(
        staged_b.0.len(),
        1,
        "bolt_b should get the effect via context_entity"
    );
}

#[test]
fn cell_with_context_resolves_to_specific_cell() {
    let mut world = World::new();
    let cell_a = world
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();
    let cell_b = world
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Cell,
        chip_name: "test".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context_entity: Some(cell_b),
    };
    cmd.apply(&mut world);

    let staged_a = world.get::<StagedEffects>(cell_a).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "cell_a should NOT get the effect — context targets cell_b"
    );
    let staged_b = world.get::<StagedEffects>(cell_b).unwrap();
    assert_eq!(
        staged_b.0.len(),
        1,
        "cell_b should get the effect via context_entity"
    );
}

#[test]
fn wall_with_context_resolves_to_specific_wall() {
    let mut world = World::new();
    let wall_a = world
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id();
    let wall_b = world
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Wall,
        chip_name: "test".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context_entity: Some(wall_b),
    };
    cmd.apply(&mut world);

    let staged_a = world.get::<StagedEffects>(wall_a).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "wall_a should NOT get the effect — context targets wall_b"
    );
    let staged_b = world.get::<StagedEffects>(wall_b).unwrap();
    assert_eq!(
        staged_b.0.len(),
        1,
        "wall_b should get the effect via context_entity"
    );
}

#[test]
fn breaker_with_context_resolves_to_specific_breaker() {
    let mut world = World::new();
    let breaker_a = world
        .spawn((
            Breaker,
            PrimaryBreaker,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let breaker_b = world
        .spawn((Breaker, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Breaker,
        chip_name: "test".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context_entity: Some(breaker_b),
    };
    cmd.apply(&mut world);

    let staged_a = world.get::<StagedEffects>(breaker_a).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "breaker_a (primary) should NOT get the effect — context targets breaker_b"
    );
    let staged_b = world.get::<StagedEffects>(breaker_b).unwrap();
    assert_eq!(
        staged_b.0.len(),
        1,
        "breaker_b should get the effect via context_entity"
    );
}

// -----------------------------------------------------------------------
// Context entity with wrong marker → falls through to default
// -----------------------------------------------------------------------

#[test]
fn cell_context_on_bolt_target_is_noop() {
    let mut world = World::new();
    let primary_bolt = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let cell = world
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    // Context is a Cell, target is Bolt — mismatch, should be no-op
    let cmd = ResolveOnCommand {
        target: Target::Bolt,
        chip_name: "test".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context_entity: Some(cell),
    };
    cmd.apply(&mut world);

    let bolt_staged = world.get::<StagedEffects>(primary_bolt).unwrap();
    assert!(
        bolt_staged.0.is_empty(),
        "PrimaryBolt should NOT get the effect — context was a Cell (wrong marker), no-op"
    );
}

#[test]
fn bolt_context_on_cell_target_is_noop() {
    let mut world = World::new();
    let bolt = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let cell = world
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    // Context is a Bolt, target is Cell — mismatch, Cell default is no-op
    let cmd = ResolveOnCommand {
        target: Target::Cell,
        chip_name: "test".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context_entity: Some(bolt),
    };
    cmd.apply(&mut world);

    let cell_staged = world.get::<StagedEffects>(cell).unwrap();
    assert!(
        cell_staged.0.is_empty(),
        "Cell should get nothing — context was a Bolt (wrong marker), Cell default is no-op"
    );
}

#[test]
fn bolt_context_on_wall_target_is_noop() {
    let mut world = World::new();
    let bolt = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let wall = world
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id();

    // Context is a Bolt, target is Wall — mismatch, Wall default is no-op
    let cmd = ResolveOnCommand {
        target: Target::Wall,
        chip_name: "test".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Died,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context_entity: Some(bolt),
    };
    cmd.apply(&mut world);

    let wall_staged = world.get::<StagedEffects>(wall).unwrap();
    assert!(
        wall_staged.0.is_empty(),
        "Wall should get nothing — context was a Bolt (wrong marker), Wall default is no-op"
    );
}
