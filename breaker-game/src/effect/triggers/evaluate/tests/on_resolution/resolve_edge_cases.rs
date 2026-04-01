//! Tests for `ResolveOnCommand` no-op edge cases (Behavior 22) and
//! On node consumption behavior (Behavior 24).

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::{Bolt, PrimaryBolt},
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

// -----------------------------------------------------------------------
// Nested On(On()) — inner On is recursively resolved during transfer
// -----------------------------------------------------------------------

#[test]
fn nested_on_resolves_inner_on_to_bolt_immediately() {
    use crate::bolt::components::{Bolt, PrimaryBolt};

    // On(Cell, [On(Bolt, [When(Died, [Do(SpeedBoost)])])])
    // Step 1: outer On(Cell) resolves to cell_b via context
    // Step 2: TransferCommand on cell_b recursively resolves inner On(Bolt)
    // Step 3: inner On(Bolt) resolves to PrimaryBolt via resolve_default
    // Step 4: When(Died) is staged on the bolt (non-permanent inner On)
    let mut world = World::new();

    let cell_b = world
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();
    let _bolt = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    let cmd = ResolveOnCommand {
        target: Target::Cell,
        chip_name: "nested_test".to_string(),
        children: vec![EffectNode::On {
            target: Target::Bolt,
            permanent: false,
            then: vec![EffectNode::When {
                trigger: Trigger::Died,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        permanent: false,
        context_entity: Some(cell_b),
    };
    cmd.apply(&mut world);

    // cell_b should have nothing — inner On was recursively resolved, not stored
    let staged_cell = world.get::<StagedEffects>(cell_b).unwrap();
    assert!(
        staged_cell.0.is_empty(),
        "cell_b should have no staged entries — inner On(Bolt) was resolved immediately"
    );
}

#[test]
fn nested_on_inner_resolves_to_primary_bolt_when_consumed_without_context() {
    // Full chain: outer On(Cell) → inner On(Bolt) deferred to cell's staged.
    // When cell's staged is evaluated (with context=None), inner On(Bolt)
    // should resolve to PrimaryBolt via resolve_default.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let primary_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let secondary_bolt = app
        .world_mut()
        .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
        .id();

    let cell = app
        .world_mut()
        .spawn((
            Cell,
            BoundEffects::default(),
            // Pre-load staged with On(Bolt) — simulates the result of outer On(Cell)
            // having already resolved and transferred this to the cell.
            StagedEffects(vec![(
                "nested_test".into(),
                EffectNode::On {
                    target: Target::Bolt,
                    permanent: false,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                    }],
                },
            )]),
        ))
        .id();

    // Evaluate staged on cell with context=None (simulates a non-collision trigger like Died).
    // The On(Bolt) should be consumed and resolve to PrimaryBolt.
    app.add_systems(Update, sys_evaluate_staged_for_bump);
    app.update();

    // Cell's staged should be empty — On was consumed
    let cell_staged = app.world().get::<StagedEffects>(cell).unwrap();
    assert!(
        cell_staged.0.is_empty(),
        "On(Bolt) should be consumed from cell's StagedEffects"
    );

    // PrimaryBolt should have the When(Died) in its StagedEffects
    let primary_staged = app.world().get::<StagedEffects>(primary_bolt).unwrap();
    assert_eq!(
        primary_staged.0.len(),
        1,
        "PrimaryBolt should have 1 staged entry from resolved inner On(Bolt)"
    );

    // Secondary bolt should have nothing — not PrimaryBolt
    let secondary_staged = app.world().get::<StagedEffects>(secondary_bolt).unwrap();
    assert!(
        secondary_staged.0.is_empty(),
        "Non-primary bolt should NOT receive the inner On(Bolt) transfer"
    );
}

// -----------------------------------------------------------------------
// Nested same-target On nodes must recursively unwrap with context
// -----------------------------------------------------------------------
// On(Cell, [On(Cell, [When(Died, [Do(SpeedBoost)])])]) with context=Some(cell_b)
// should fully unwrap: all On(Cell) layers resolve to cell_b, and the final
// When(Died) lands in cell_b's StagedEffects. No intermediate On node should
// be left stranded in staged without context.

#[test]
fn nested_same_target_on_nodes_unwrap_to_final_when() {
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

    // Three layers of On(Cell) wrapping When(Died, [Do(SpeedBoost)])
    let cmd = ResolveOnCommand {
        target: Target::Cell,
        chip_name: "nested_same".to_string(),
        children: vec![EffectNode::On {
            target: Target::Cell,
            permanent: false,
            then: vec![EffectNode::On {
                target: Target::Cell,
                permanent: false,
                then: vec![EffectNode::When {
                    trigger: Trigger::Died,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            }],
        }],
        permanent: false,
        context_entity: Some(cell_b),
    };
    cmd.apply(&mut world);

    // cell_b should have the final When(Died) in StagedEffects — all On layers unwrapped
    let staged_b = world.get::<StagedEffects>(cell_b).unwrap();
    assert_eq!(
        staged_b.0.len(),
        1,
        "cell_b should have exactly 1 staged entry — the final When(Died) after full unwrap"
    );
    assert!(
        matches!(
            &staged_b.0[0].1,
            EffectNode::When {
                trigger: Trigger::Died,
                ..
            }
        ),
        "cell_b's staged entry should be When(Died, ...), not an intermediate On node"
    );

    // cell_a and cell_c should have nothing
    let staged_a = world.get::<StagedEffects>(cell_a).unwrap();
    let staged_c = world.get::<StagedEffects>(cell_c).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "cell_a should have no staged effects"
    );
    assert!(
        staged_c.0.is_empty(),
        "cell_c should have no staged effects"
    );
}
