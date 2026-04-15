use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{super::system::on_bumped, helpers::*};
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::{BoundEffects, StagedEffects},
        types::{EffectType, Tree, Trigger},
    },
};

// ================================================================
// Wave C — Staged effect dispatch on on_bumped bridge
// ================================================================

// ----------------------------------------------------------------
// Behavior 18: staged entry whose inner trigger matches fires and
//              is consumed on the same tick
// ----------------------------------------------------------------
#[test]
fn on_bumped_staged_entry_fires_and_is_consumed_on_match() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let breaker = app.world_mut().spawn_empty().id();
    let bolt = app
        .world_mut()
        .spawn((
            BoundEffects(vec![]),
            StagedEffects(vec![(
                "chip_a".to_string(),
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            )]),
        ))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: Some(bolt),
        breaker,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("staged When should have fired on matching trigger");
    assert_eq!(stack.len(), 1);

    let staged = app
        .world()
        .get::<StagedEffects>(bolt)
        .expect("StagedEffects should still exist (empty)");
    assert!(
        staged.0.is_empty(),
        "staged entry must be consumed via commands.remove_effect"
    );

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("BoundEffects should still exist");
    assert!(
        bound.0.is_empty(),
        "remove_effect only removed the staged copy"
    );
}

// ----------------------------------------------------------------
// Behavior 19: staged entry whose inner trigger does NOT match the
//              active trigger remains staged
// ----------------------------------------------------------------
#[test]
fn on_bumped_staged_entry_non_matching_trigger_remains_staged() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let breaker = app.world_mut().spawn_empty().id();
    let bolt = app
        .world_mut()
        .spawn((
            BoundEffects(vec![]),
            StagedEffects(vec![(
                "chip_a".to_string(),
                Tree::When(
                    Trigger::BoltLostOccurred,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            )]),
        ))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: Some(bolt),
        breaker,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none(),
        "inner must not fire when its gate trigger does not match"
    );

    let staged = app.world().get::<StagedEffects>(bolt).unwrap();
    assert_eq!(staged.0.len(), 1);
    assert_eq!(staged.0[0].0, "chip_a");
}

// ----------------------------------------------------------------
// Behavior 20: staged entries are walked BEFORE bound entries
//              (snapshot semantics — no same-tick arming fire)
// ----------------------------------------------------------------
#[test]
fn on_bumped_staged_walks_before_bound_no_same_tick_fire() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let breaker = app.world_mut().spawn_empty().id();
    let bolt = app
        .world_mut()
        .spawn(BoundEffects(vec![(
            "chip_outer".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                )),
            ),
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: Some(bolt),
        breaker,
    }]));

    tick(&mut app);

    let staged = app
        .world()
        .get::<StagedEffects>(bolt)
        .expect("outer bound When should have armed the inner");
    assert_eq!(staged.0.len(), 1);
    assert_eq!(
        staged.0[0],
        (
            "chip_outer".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )
    );

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none(),
        "freshly armed staged entry MUST NOT fire in the same tick it was armed"
    );
}

// ----------------------------------------------------------------
// Behavior 21: second bump fires the staged inner When (end-to-end
//              "two bumps → one fire")
// ----------------------------------------------------------------
#[test]
fn on_bumped_second_bump_fires_staged_inner_when() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let breaker = app.world_mut().spawn_empty().id();
    let bolt = app
        .world_mut()
        .spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                )),
            ),
        )]))
        .id();

    // Tick 1: inject one BumpPerformed → outer arms inner into StagedEffects
    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: Some(bolt),
        breaker,
    }]));
    tick(&mut app);

    {
        let staged = app
            .world()
            .get::<StagedEffects>(bolt)
            .expect("tick 1 should have armed the inner");
        assert_eq!(staged.0.len(), 1);
    }
    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none(),
        "no fire after tick 1"
    );

    // Tick 2: inject another BumpPerformed → staged inner fires + outer
    // re-arms. Entry-specific consume leaves the freshly staged re-arm
    // intact.
    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: Some(bolt),
        breaker,
    }]));
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("staged inner When should have fired on tick 2");
    assert_eq!(stack.len(), 1, "exactly one fire after two bumps");

    // The outer bound When is UNTOUCHED — RemoveStagedEffectCommand
    // never sweeps BoundEffects.
    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("outer bound When should still be present");
    assert_eq!(bound.0.len(), 1, "outer bound When persists");

    // StagedEffects has exactly one entry: the re-armed inner
    // `When(Bumped, Fire(SpeedBoost))` installed by the bound walk
    // after the entry-specific remove swept the just-consumed entry.
    let staged = app
        .world()
        .get::<StagedEffects>(bolt)
        .expect("bound walk should have re-armed inner into StagedEffects");
    assert_eq!(staged.0.len(), 1, "single re-armed inner entry");
    assert_eq!(
        staged.0[0],
        (
            "chip_a".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        ),
        "re-armed inner is the one-level-peeled inner When"
    );
}

// ----------------------------------------------------------------
// Behavior 22: third bump fires again (repeating pattern)
// ----------------------------------------------------------------
#[test]
fn on_bumped_third_bump_fires_again_repeating_pattern() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let breaker = app.world_mut().spawn_empty().id();
    let bolt = app
        .world_mut()
        .spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                )),
            ),
        )]))
        .id();

    // Tick 1: arm inner (staged.len == 1, no fire)
    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: Some(bolt),
        breaker,
    }]));
    tick(&mut app);

    // Tick 2: staged inner fires, outer bound re-arms (stack.len == 1)
    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: Some(bolt),
        breaker,
    }]));
    tick(&mut app);

    // Tick 3: inject third bump → staged inner fires AGAIN and
    // outer bound re-arms a fresh inner.
    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: Some(bolt),
        breaker,
    }]));
    tick(&mut app);

    {
        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .expect("Fire should have run on tick 2 and tick 3");
        assert_eq!(
            stack.len(),
            2,
            "repeating pattern: each matching bump after priming adds one fire"
        );

        let staged = app
            .world()
            .get::<StagedEffects>(bolt)
            .expect("bound walk re-armed inner on tick 3");
        assert_eq!(staged.0.len(), 1, "single re-armed inner after tick 3");
        assert_eq!(
            staged.0[0],
            (
                "chip_a".to_string(),
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            ),
            "re-armed inner is the one-level-peeled inner When"
        );

        let bound = app
            .world()
            .get::<BoundEffects>(bolt)
            .expect("outer bound When should still be present");
        assert_eq!(bound.0.len(), 1, "outer bound When persists across ticks");
    }

    // Tick 4: inject a fourth bump — one more fire, capping the
    // assertion at len == 3 to lock in "+1 per bump" without an
    // unbounded loop.
    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: Some(bolt),
        breaker,
    }]));
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Fire should have run again on tick 4");
    assert_eq!(
        stack.len(),
        3,
        "repeating invariant: every matching bump adds exactly one fire"
    );
}

// ================================================================
// Wave C Part D — same-source triple-nested `When` chain primes
//                 across 3 ticks then fires once per trigger
// ================================================================

// ----------------------------------------------------------------
// Behavior 26: Triple-nested When with same source — primes across
//              3 ticks, then fires once per matching trigger
//              (repeating)
// ----------------------------------------------------------------
#[test]
fn on_bumped_triple_nested_when_same_source_primes_then_repeats() {
    fn inject_and_tick(app: &mut App, bolt: Entity, breaker: Entity) {
        app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
            breaker,
        }]));
        tick(app);
    }
    fn stack_len(app: &App, bolt: Entity) -> Option<usize> {
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .map(EffectStack::len)
    }
    fn assert_bound_len_1(app: &App, bolt: Entity) {
        assert_eq!(app.world().get::<BoundEffects>(bolt).unwrap().0.len(), 1);
    }
    fn assert_staged_pair(app: &App, bolt: Entity, a: &Tree, b: &Tree) {
        let staged = app.world().get::<StagedEffects>(bolt).unwrap();
        assert_eq!(staged.0.len(), 2);
        assert_eq!(staged.0[0], ("chip_a".to_string(), a.clone()));
        assert_eq!(staged.0[1], ("chip_a".to_string(), b.clone()));
    }

    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));
    let breaker = app.world_mut().spawn_empty().id();

    let fire_tree = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    }));
    let inner2 = Tree::When(Trigger::Bumped, Box::new(fire_tree));
    let inner1 = Tree::When(Trigger::Bumped, Box::new(inner2.clone()));
    let outer = Tree::When(Trigger::Bumped, Box::new(inner1.clone()));

    let bolt = app
        .world_mut()
        .spawn(BoundEffects(vec![("chip_a".to_string(), outer)]))
        .id();

    // Tick 1: bound walk arms inner1 → staged=[inner1], no fire.
    inject_and_tick(&mut app, bolt, breaker);
    assert_bound_len_1(&app, bolt);
    let staged = app.world().get::<StagedEffects>(bolt).unwrap();
    assert_eq!(staged.0.len(), 1);
    assert_eq!(staged.0[0], ("chip_a".to_string(), inner1.clone()));
    assert_eq!(stack_len(&app, bolt), None, "no fire after tick 1");

    // Tick 2: staged walk arms inner2, bound re-arms inner1 → staged=[inner2, inner1], no fire.
    inject_and_tick(&mut app, bolt, breaker);
    assert_bound_len_1(&app, bolt);
    assert_staged_pair(&app, bolt, &inner2, &inner1);
    assert_eq!(stack_len(&app, bolt), None, "no fire after tick 2");

    // Ticks 3, 4, 5: first/second/third fires. Staged layout stable at [inner2, inner1].
    for expected_fires in 1..=3 {
        inject_and_tick(&mut app, bolt, breaker);
        assert_bound_len_1(&app, bolt);
        assert_staged_pair(&app, bolt, &inner2, &inner1);
        assert_eq!(
            stack_len(&app, bolt),
            Some(expected_fires),
            "fire count should equal tick count past priming"
        );
    }
}
