//! End-to-end desugaring tests for `AllBolts` target resolution.

use bevy::prelude::*;

use super::helpers::sys_evaluate_node_start;
use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger},
};

// ── Behavior L1: AllBolts target distributes to all bolt entities end-to-end ──

/// Setup helper for the `AllBolts` E2E desugaring test.
///
/// Builds a test app with dispatch (but NOT `NodeStart` evaluation), inserts a
/// "Bolt Enhance" chip definition targeting `AllBolts`, spawns a Breaker and
/// three Bolts, selects the chip, and runs one update (Phase 1: dispatch only).
///
/// Returns `(app, breaker, bolt_a, bolt_b, bolt_c)`.
fn setup_e2e_all_bolts_app() -> (App, Entity, Entity, Entity, Entity) {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Bolt Enhance".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllBolts,
            then: vec![EffectNode::When {
                trigger: Trigger::Bumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    let bolt_c = spawn_bolt(&mut app);

    select_chip(&mut app, "Bolt Enhance");
    app.update();

    // Precondition: bolts have no BoundEffects before NodeStart fires.
    for bolt in [bolt_a, bolt_b, bolt_c] {
        let bound = app.world().get::<BoundEffects>(bolt).unwrap();
        assert!(
            bound.0.is_empty(),
            "Bolt should have 0 BoundEffects entries before NodeStart"
        );
    }

    (app, breaker, bolt_a, bolt_b, bolt_c)
}

/// Asserts that the given bolt entity has exactly one `BoundEffects` entry
/// matching `When(Bumped, [Do(SpeedBoost { multiplier: 1.5 })])` with chip name "Bolt Enhance".
fn assert_bolt_has_speed_boost_bound_effect(app: &App, bolt: Entity, label: &str) {
    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "After NodeStart, {label} should have 1 BoundEffects entry, got {}",
        bound.0.len()
    );

    let (chip, node) = &bound.0[0];
    assert_eq!(
        chip, "Bolt Enhance",
        "{label}'s BoundEffects chip_name should be 'Bolt Enhance'"
    );
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::Bumped,
                then: do_children,
            } if do_children.len() == 1 && matches!(
                &do_children[0],
                EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (*multiplier - 1.5).abs() < f32::EPSILON
            )
        ),
        "{label} should have When(Bumped, [Do(SpeedBoost {{ multiplier: 1.5 }})]), got {node:?}"
    );
}

/// End-to-end integration test: chip selection -> desugaring -> `NodeStart` trigger
/// -> bolts get permanent `BoundEffects`.
///
/// Verifies the full chain: `dispatch_chip_effects` desugars `AllBolts` target
/// to `When(NodeStart, On(AllBolts, permanent: true, ...))` on the Breaker,
/// then when `NodeStart` fires the `On(AllBolts)` node resolves to each Bolt
/// entity and installs `When(Bumped, Do(SpeedBoost { multiplier: 1.5 }))` in
/// their `BoundEffects` (permanent, not `StagedEffects`).
#[test]
fn chip_all_bolts_target_desugars_and_resolves_to_bolt_bound_effects_on_node_start() {
    let (mut app, breaker, bolt_a, bolt_b, bolt_c) = setup_e2e_all_bolts_app();

    // ── Phase 2 assertions: Breaker has desugared When(NodeStart) entry ──

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "After dispatch, Breaker should have exactly 1 BoundEffects entry (desugared AllBolts)"
    );

    let (chip_name, node) = &breaker_bound.0[0];
    assert_eq!(
        chip_name, "Bolt Enhance",
        "chip_name must be 'Bolt Enhance'"
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
                    target: Target::AllBolts,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: do_children,
                    } if do_children.len() == 1 && matches!(
                        &do_children[0],
                        EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (*multiplier - 1.5).abs() < f32::EPSILON
                    )
                )
            )
        ),
        "Breaker's entry should be When(NodeStart, [On(AllBolts, permanent: true, \
         [When(Bumped, [Do(SpeedBoost {{ multiplier: 1.5 }})])])]), got {node:?}"
    );

    // ── Phase 3: Register evaluate system, fire NodeStart trigger ──

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

    app.update();

    // ── Phase 3 assertions: All 3 Bolt entities have permanent BoundEffects ──

    assert_bolt_has_speed_boost_bound_effect(&app, bolt_a, "Bolt A");
    assert_bolt_has_speed_boost_bound_effect(&app, bolt_b, "Bolt B");
    assert_bolt_has_speed_boost_bound_effect(&app, bolt_c, "Bolt C");

    // ── Phase 3 assertions: All 3 Bolt entities have 0 StagedEffects ──
    // permanent: true means children go to BoundEffects, not StagedEffects.

    for (bolt, label) in [(bolt_a, "Bolt A"), (bolt_b, "Bolt B"), (bolt_c, "Bolt C")] {
        let staged = app.world().get::<StagedEffects>(bolt).unwrap();
        assert!(
            staged.0.is_empty(),
            "{label} should have 0 StagedEffects (permanent routing), got {}",
            staged.0.len()
        );
    }

    // ── Phase 3 assertions: Breaker's On(AllBolts) consumed from StagedEffects ──

    let breaker_staged = app.world().get::<StagedEffects>(breaker).unwrap();
    assert!(
        breaker_staged.0.is_empty(),
        "Breaker's StagedEffects should be empty after On(AllBolts) was consumed, got {} entries",
        breaker_staged.0.len()
    );

    // ── Phase 3 edge case: Breaker itself must NOT have the inner When(Bumped) ──
    // The Breaker should only have the original When(NodeStart, ...) wrapper,
    // not the inner When(Bumped, ...) that was distributed to bolts.

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
