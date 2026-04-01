//! Tests for `ResolveOnCommand` no-op edge cases (Behavior 22) and
//! On node consumption behavior (Behavior 24).

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    breaker::components::Breaker,
    cells::components::Cell,
    effect::{commands::ResolveOnCommand, core::*},
};

// -----------------------------------------------------------------------
// Behavior 22: No matching entities -- no-op
// -----------------------------------------------------------------------

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
        context_entity: None,
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
        context_entity: None,
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
        context_entity: None,
    };
    // Should not panic
    cmd.apply(&mut world);
}

// -----------------------------------------------------------------------
// Behavior 24: On node in StagedEffects consumed regardless of trigger
// -----------------------------------------------------------------------

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
