//! End-to-end desugaring tests for `AllCells` target resolution.

use bevy::prelude::*;

use super::{super::super::helpers::*, helpers::sys_evaluate_node_start};
use crate::{
    chips::definition::ChipDefinition,
    effect::{
        BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, StagedEffects, Target,
        Trigger,
    },
};

// ── Section K: End-to-end desugaring -> NodeStart -> Cell resolution ──

/// Asserts that the given cell entity has exactly one `BoundEffects` entry
/// matching `When(Impacted(Bolt), [Do(Shield(1))])` with chip name "Cell Shield".
fn assert_cell_has_shield_bound_effect(app: &App, cell: Entity, label: &str) {
    let bound = app.world().get::<BoundEffects>(cell).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "After NodeStart, {label} should have 1 BoundEffects entry, got {}",
        bound.0.len()
    );

    let (chip, node) = &bound.0[0];
    assert_eq!(
        chip, "Cell Shield",
        "{label}'s BoundEffects chip_name should be 'Cell Shield'"
    );
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: do_children,
            } if do_children.len() == 1 && matches!(
                &do_children[0],
                EffectNode::Do(EffectKind::Shield { stacks: 1 })
            )
        ),
        "{label} should have When(Impacted(Bolt), [Do(Shield(1))]), got {node:?}"
    );
}

/// Setup helper for the E2E desugaring test.
///
/// Builds a test app with dispatch (but NOT `NodeStart` evaluation), inserts a
/// "Cell Shield" chip definition targeting `AllCells`, spawns a Breaker and
/// two Cells, selects the chip, and runs one update (Phase 1: dispatch only).
///
/// The caller is responsible for registering `sys_evaluate_node_start` after
/// verifying Phase 1 assertions (desugared entry on Breaker, 0 `BoundEffects` on
/// Cells).
///
/// Returns `(app, breaker, cell_a, cell_b)`.
fn setup_e2e_desugaring_app() -> (App, Entity, Entity, Entity) {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Cell Shield".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllCells,
            then: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    let cell_a = spawn_cell(&mut app);
    let cell_b = spawn_cell(&mut app);

    select_chip(&mut app, "Cell Shield");
    app.update();

    // Precondition: cells have no BoundEffects before NodeStart fires.
    for cell in [cell_a, cell_b] {
        let bound = app.world().get::<BoundEffects>(cell).unwrap();
        assert!(
            bound.0.is_empty(),
            "Cell should have 0 BoundEffects entries before NodeStart"
        );
    }

    (app, breaker, cell_a, cell_b)
}

/// End-to-end integration test: chip selection -> desugaring -> `NodeStart` trigger
/// -> cells get permanent `BoundEffects`.
///
/// Verifies the full chain: `dispatch_chip_effects` desugars `AllCells` target
/// to `When(NodeStart, On(AllCells, permanent: true, ...))` on the Breaker,
/// then when `NodeStart` fires the `On(AllCells)` node resolves to each Cell
/// entity and installs `When(Impacted(Bolt), Do(Shield { stacks: 1 }))` in
/// their `BoundEffects` (permanent, not `StagedEffects`).
#[test]
fn chip_all_cells_target_desugars_and_resolves_to_cell_bound_effects_on_node_start() {
    let (mut app, breaker, cell_a, cell_b) = setup_e2e_desugaring_app();

    // ── Phase 2 assertions: Breaker has desugared When(NodeStart) entry ──

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "After dispatch, Breaker should have exactly 1 BoundEffects entry (desugared AllCells)"
    );

    let (chip_name, node) = &breaker_bound.0[0];
    assert_eq!(chip_name, "Cell Shield", "chip_name must be 'Cell Shield'");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllCells,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: do_children,
                    } if do_children.len() == 1 && matches!(
                        &do_children[0],
                        EffectNode::Do(EffectKind::Shield { stacks: 1 })
                    )
                )
            )
        ),
        "Breaker's entry should be When(NodeStart, [On(AllCells, permanent: true, \
         [When(Impacted(Bolt), [Do(Shield(1))])])]), got {node:?}"
    );

    // ── Phase 3: Register evaluate system, fire NodeStart trigger ──

    // Now that Phase 2 is verified, add the evaluate system so NodeStart
    // processing happens on the next update (not in the same frame as
    // dispatch).
    app.add_systems(
        Update,
        sys_evaluate_node_start
            .after(crate::chips::systems::dispatch_chip_effects::dispatch_chip_effects),
    );

    // Clear pending selections so dispatch_chip_effects does not re-process
    // on the second update.
    app.world_mut()
        .resource_mut::<PendingChipSelections>()
        .0
        .clear();

    // Run another update — sys_evaluate_node_start evaluates NodeStart on
    // all entities with BoundEffects.  The Breaker's When(NodeStart) fires,
    // pushing On(AllCells) to StagedEffects, which then resolves to Cell
    // entities via ResolveOnCommand.
    app.update();

    // ── Phase 3 assertions: Both Cell entities have permanent BoundEffects ──

    assert_cell_has_shield_bound_effect(&app, cell_a, "Cell A");
    assert_cell_has_shield_bound_effect(&app, cell_b, "Cell B");

    // ── Phase 3 assertions: Both Cell entities have 0 StagedEffects ──
    // permanent: true means children go to BoundEffects, not StagedEffects.

    let first_staged = app.world().get::<StagedEffects>(cell_a).unwrap();
    assert!(
        first_staged.0.is_empty(),
        "Cell A should have 0 StagedEffects (permanent routing), got {}",
        first_staged.0.len()
    );

    let second_staged = app.world().get::<StagedEffects>(cell_b).unwrap();
    assert!(
        second_staged.0.is_empty(),
        "Cell B should have 0 StagedEffects (permanent routing), got {}",
        second_staged.0.len()
    );

    // ── Phase 3 assertions: Breaker's When(NodeStart) processed ──
    // After NodeStart evaluation, the On(AllCells) child was pushed to
    // StagedEffects and consumed by ResolveOnCommand.  The Breaker's
    // StagedEffects should be empty (On node consumed).

    let breaker_staged = app.world().get::<StagedEffects>(breaker).unwrap();
    assert!(
        breaker_staged.0.is_empty(),
        "Breaker's StagedEffects should be empty after On(AllCells) was consumed, got {} entries",
        breaker_staged.0.len()
    );
}

// ── Behavior L3: AllCells target with DamageBoost distributes end-to-end ──

/// Setup helper for the `AllCells` `DamageBoost` E2E desugaring test.
///
/// Builds a test app with dispatch (but NOT `NodeStart` evaluation), inserts a
/// "Cell Burn" chip definition targeting `AllCells` with `DamageBoost`(2.0),
/// spawns a Breaker and three Cells, selects the chip, and runs one update
/// (Phase 1: dispatch only).
///
/// Returns `(app, breaker, cell_a, cell_b, cell_c)`.
fn setup_e2e_all_cells_damage_boost_app() -> (App, Entity, Entity, Entity, Entity) {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Cell Burn".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllCells,
            then: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    let cell_a = spawn_cell(&mut app);
    let cell_b = spawn_cell(&mut app);
    let cell_c = spawn_cell(&mut app);

    select_chip(&mut app, "Cell Burn");
    app.update();

    // Precondition: cells have no BoundEffects before NodeStart fires.
    for cell in [cell_a, cell_b, cell_c] {
        let bound = app.world().get::<BoundEffects>(cell).unwrap();
        assert!(
            bound.0.is_empty(),
            "Cell should have 0 BoundEffects entries before NodeStart"
        );
    }

    (app, breaker, cell_a, cell_b, cell_c)
}

/// Asserts that the given cell entity has exactly one `BoundEffects` entry
/// matching `When(Impacted(Bolt), [Do(DamageBoost(2.0))])` with chip name "Cell Burn".
fn assert_cell_has_damage_boost_bound_effect(app: &App, cell: Entity, label: &str) {
    let bound = app.world().get::<BoundEffects>(cell).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "After NodeStart, {label} should have 1 BoundEffects entry, got {}",
        bound.0.len()
    );

    let (chip, node) = &bound.0[0];
    assert_eq!(
        chip, "Cell Burn",
        "{label}'s BoundEffects chip_name should be 'Cell Burn'"
    );
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: do_children,
            } if do_children.len() == 1 && matches!(
                &do_children[0],
                EffectNode::Do(EffectKind::DamageBoost(m)) if (*m - 2.0).abs() < f32::EPSILON
            )
        ),
        "{label} should have When(Impacted(Bolt), [Do(DamageBoost(2.0))]), got {node:?}"
    );
}

/// End-to-end integration test: chip selection -> desugaring -> `NodeStart` trigger
/// -> cells get permanent `BoundEffects` (`DamageBoost` variant).
///
/// This mirrors the existing Section K test but uses `DamageBoost(2.0)` instead
/// of `Shield { stacks: 1 }`, and targets three cells instead of two, to verify
/// the pipeline works with different effect kinds.
#[test]
fn chip_all_cells_damage_boost_target_desugars_and_resolves_to_cell_bound_effects_on_node_start() {
    let (mut app, breaker, cell_a, cell_b, cell_c) = setup_e2e_all_cells_damage_boost_app();

    // ── Phase 2 assertions: Breaker has desugared When(NodeStart) entry ──

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "After dispatch, Breaker should have exactly 1 BoundEffects entry (desugared AllCells)"
    );

    let (chip_name, node) = &breaker_bound.0[0];
    assert_eq!(chip_name, "Cell Burn", "chip_name must be 'Cell Burn'");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllCells,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: do_children,
                    } if do_children.len() == 1 && matches!(
                        &do_children[0],
                        EffectNode::Do(EffectKind::DamageBoost(m)) if (*m - 2.0).abs() < f32::EPSILON
                    )
                )
            )
        ),
        "Breaker's entry should be When(NodeStart, [On(AllCells, permanent: true, \
         [When(Impacted(Bolt), [Do(DamageBoost(2.0))])])]), got {node:?}"
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

    // ── Phase 3 assertions: All 3 Cell entities have permanent BoundEffects ──

    assert_cell_has_damage_boost_bound_effect(&app, cell_a, "Cell A");
    assert_cell_has_damage_boost_bound_effect(&app, cell_b, "Cell B");
    assert_cell_has_damage_boost_bound_effect(&app, cell_c, "Cell C");

    // ── Phase 3 assertions: All 3 Cell entities have 0 StagedEffects ──
    // permanent: true means children go to BoundEffects, not StagedEffects.

    for (cell, label) in [(cell_a, "Cell A"), (cell_b, "Cell B"), (cell_c, "Cell C")] {
        let staged = app.world().get::<StagedEffects>(cell).unwrap();
        assert!(
            staged.0.is_empty(),
            "{label} should have 0 StagedEffects (permanent routing), got {}",
            staged.0.len()
        );
    }

    // ── Phase 3 assertions: Breaker's On(AllCells) consumed from StagedEffects ──

    let breaker_staged = app.world().get::<StagedEffects>(breaker).unwrap();
    assert!(
        breaker_staged.0.is_empty(),
        "Breaker's StagedEffects should be empty after On(AllCells) was consumed, got {} entries",
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
