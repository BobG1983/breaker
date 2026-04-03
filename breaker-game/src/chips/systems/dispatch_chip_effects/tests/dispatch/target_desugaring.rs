//! Non-Breaker target desugaring tests — behaviors 7-12.
//!
//! Non-Breaker targets (`AllBolts`, Bolt, `AllCells`, Cell, `AllWalls`, Wall) are
//! wrapped in `When(NodeStart, On(...))` and pushed to the Breaker's
//! `BoundEffects`.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{BoundEffects, EffectKind, EffectNode, Target, Trigger},
};

// ── Behavior 7: Target `AllBolts` desugars to Breaker's BoundEffects ──

#[test]
fn all_bolts_target_desugars_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Parry Shockwave",
        Target::AllBolts,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 500.0,
            })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Parry Shockwave");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared BoundEffects entry for AllBolts"
    );
    assert_eq!(bound.0[0].0, "Parry Shockwave");
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                ..
            }
        ),
        "AllBolts should desugar to When(NodeStart, [On(AllBolts, ...)])"
    );
}

// ── Behavior 7 edge case: Zero Breaker entities — no panic ──

#[test]
fn all_bolts_target_with_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Empty Target",
        Target::AllBolts,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        },
        5,
    );
    insert_chip(&mut app, def);

    // No breaker entities spawned — desugaring has nowhere to push
    select_chip(&mut app, "Empty Target");

    // Should not panic
    app.update();
}

// ── Behavior 8: Target `Bolt` desugars to Breaker (same as AllBolts) ──

#[test]
fn bolt_target_desugars_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Slight Bolt Speed",
        EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 }),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Slight Bolt Speed");

    app.update();

    // Bolt target desugars — wrapped in When(NodeStart, On(Bolt, ...)) on Breaker
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared BoundEffects entry for Bolt target"
    );
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                ..
            }
        ),
        "Bolt target should desugar to When(NodeStart, [On(Bolt, ...)])"
    );
}

// ── Behavior 9: Target `AllCells` desugars to Breaker's BoundEffects ──

#[test]
fn all_cells_target_desugars_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Shield",
        Target::AllCells,
        EffectNode::When {
            trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield {
                duration: 5.0,
                reflection_cost: 0.0,
            })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Shield");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared BoundEffects entry for AllCells"
    );
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                ..
            }
        ),
        "AllCells should desugar to When(NodeStart, [On(AllCells, ...)])"
    );
}

// ── Behavior 10: Target `Cell` desugars same as `AllCells` ──

#[test]
fn cell_target_desugars_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Shield Single",
        Target::Cell,
        EffectNode::When {
            trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield {
                duration: 5.0,
                reflection_cost: 0.0,
            })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Shield Single");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared BoundEffects entry (Cell target desugars like AllCells)"
    );
}

// ── Behavior 10 edge case: Zero Breaker entities for Cell target — no panic ──

#[test]
fn cell_target_with_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Empty Cell",
        Target::Cell,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        },
        5,
    );
    insert_chip(&mut app, def);

    // No breaker entities spawned
    select_chip(&mut app, "Empty Cell");
    app.update();
}

// ── Behavior 11: Target `AllWalls` desugars to Breaker's BoundEffects ──

#[test]
fn all_walls_target_desugars_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Effect",
        Target::AllWalls,
        EffectNode::When {
            trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Wall Effect");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared BoundEffects entry for AllWalls"
    );
}

// ── Behavior 12: Target `Wall` desugars same as `AllWalls` ──

#[test]
fn wall_target_desugars_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Single",
        Target::Wall,
        EffectNode::When {
            trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Wall Single");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared BoundEffects entry (Wall target desugars like AllWalls)"
    );
}

// ── Behavior 12 edge case: Zero Breaker entities for Wall target — no panic ──

#[test]
fn wall_target_with_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Empty Wall",
        Target::Wall,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        },
        5,
    );
    insert_chip(&mut app, def);

    // No breaker entities spawned
    select_chip(&mut app, "Empty Wall");
    app.update();
}
