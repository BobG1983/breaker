//! End-to-end desugaring tests for `AllWalls` target resolution.

use bevy::prelude::*;

use super::{super::super::helpers::*, helpers::sys_evaluate_node_start};
use crate::{
    chips::definition::ChipDefinition,
    effect::{
        BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, StagedEffects, Target,
        Trigger,
    },
};

// ── Behavior L2: AllWalls target distributes to all wall entities end-to-end ──

/// Setup helper for the `AllWalls` E2E desugaring test.
///
/// Builds a test app with dispatch (but NOT `NodeStart` evaluation), inserts a
/// "Wall Fortify" chip definition targeting `AllWalls`, spawns a Breaker and
/// two Walls, selects the chip, and runs one update (Phase 1: dispatch only).
///
/// Returns `(app, breaker, wall_a, wall_b)`.
fn setup_e2e_all_walls_app() -> (App, Entity, Entity, Entity) {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Wall Fortify".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllWalls,
            then: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    let wall_a = spawn_wall(&mut app);
    let wall_b = spawn_wall(&mut app);

    select_chip(&mut app, "Wall Fortify");
    app.update();

    // Precondition: walls have no BoundEffects before NodeStart fires.
    for wall in [wall_a, wall_b] {
        let bound = app.world().get::<BoundEffects>(wall).unwrap();
        assert!(
            bound.0.is_empty(),
            "Wall should have 0 BoundEffects entries before NodeStart"
        );
    }

    (app, breaker, wall_a, wall_b)
}

/// Asserts that the given wall entity has exactly one `BoundEffects` entry
/// matching `When(Impacted(Bolt), [Do(DamageBoost(1.5))])` with chip name "Wall Fortify".
fn assert_wall_has_damage_boost_bound_effect(app: &App, wall: Entity, label: &str) {
    let bound = app.world().get::<BoundEffects>(wall).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "After NodeStart, {label} should have 1 BoundEffects entry, got {}",
        bound.0.len()
    );

    let (chip, node) = &bound.0[0];
    assert_eq!(
        chip, "Wall Fortify",
        "{label}'s BoundEffects chip_name should be 'Wall Fortify'"
    );
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: do_children,
            } if do_children.len() == 1 && matches!(
                &do_children[0],
                EffectNode::Do(EffectKind::DamageBoost(m)) if (*m - 1.5).abs() < f32::EPSILON
            )
        ),
        "{label} should have When(Impacted(Bolt), [Do(DamageBoost(1.5))]), got {node:?}"
    );
}

/// End-to-end integration test: chip selection -> desugaring -> `NodeStart` trigger
/// -> walls get permanent `BoundEffects`.
///
/// Verifies the full chain: `dispatch_chip_effects` desugars `AllWalls` target
/// to `When(NodeStart, On(AllWalls, permanent: true, ...))` on the Breaker,
/// then when `NodeStart` fires the `On(AllWalls)` node resolves to each Wall
/// entity and installs `When(Impacted(Bolt), Do(DamageBoost(1.5)))` in
/// their `BoundEffects` (permanent, not `StagedEffects`).
#[test]
fn chip_all_walls_target_desugars_and_resolves_to_wall_bound_effects_on_node_start() {
    let (mut app, breaker, wall_a, wall_b) = setup_e2e_all_walls_app();

    // ── Phase 2 assertions: Breaker has desugared When(NodeStart) entry ──

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "After dispatch, Breaker should have exactly 1 BoundEffects entry (desugared AllWalls)"
    );

    let (chip_name, node) = &breaker_bound.0[0];
    assert_eq!(
        chip_name, "Wall Fortify",
        "chip_name must be 'Wall Fortify'"
    );
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllWalls,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: do_children,
                    } if do_children.len() == 1 && matches!(
                        &do_children[0],
                        EffectNode::Do(EffectKind::DamageBoost(m)) if (*m - 1.5).abs() < f32::EPSILON
                    )
                )
            )
        ),
        "Breaker's entry should be When(NodeStart, [On(AllWalls, permanent: true, \
         [When(Impacted(Bolt), [Do(DamageBoost(1.5))])])]), got {node:?}"
    );

    // ── Phase 3: Register evaluate system, fire NodeStart trigger ──

    app.add_systems(
        Update,
        sys_evaluate_node_start
            .after(crate::chips::systems::dispatch_chip_effects::dispatch_chip_effects),
    );

    app.world_mut()
        .resource_mut::<PendingChipSelections>()
        .0
        .clear();

    app.update();

    // ── Phase 3 assertions: Both Wall entities have permanent BoundEffects ──

    assert_wall_has_damage_boost_bound_effect(&app, wall_a, "Wall A");
    assert_wall_has_damage_boost_bound_effect(&app, wall_b, "Wall B");

    // ── Phase 3 assertions: Both Wall entities have 0 StagedEffects ──
    // permanent: true means children go to BoundEffects, not StagedEffects.

    for (wall, label) in [(wall_a, "Wall A"), (wall_b, "Wall B")] {
        let staged = app.world().get::<StagedEffects>(wall).unwrap();
        assert!(
            staged.0.is_empty(),
            "{label} should have 0 StagedEffects (permanent routing), got {}",
            staged.0.len()
        );
    }

    // ── Phase 3 assertions: Breaker's On(AllWalls) consumed from StagedEffects ──

    let breaker_staged = app.world().get::<StagedEffects>(breaker).unwrap();
    assert!(
        breaker_staged.0.is_empty(),
        "Breaker's StagedEffects should be empty after On(AllWalls) was consumed, got {} entries",
        breaker_staged.0.len()
    );

    // ── Phase 3 edge case: Breaker itself must NOT have the inner When(Impacted(Bolt)) ──

    let breaker_bound_after = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound_after.0.len(),
        1,
        "Breaker should still have exactly 1 BoundEffects entry (the When(NodeStart) wrapper)"
    );
    assert!(
        matches!(
            &breaker_bound_after.0[0].1,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                ..
            }
        ),
        "Breaker's only BoundEffects entry should be When(NodeStart, ...), not an inner effect"
    );
}
