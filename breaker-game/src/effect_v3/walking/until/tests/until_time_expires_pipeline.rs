//! Group F — End-to-end pipeline: arm via `When(NodeStartOccurred, Until(TimeExpires(0.5), Fire(SpeedBoost(2.0))))`,
//! tick `tick_effect_timers` until expiry, watch `on_time_expires` dispatch
//! `TimeExpires(0.5)` filtered to this Until, observe reversal.

use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::{BoundEffects, StagedEffects},
        triggers::time::{
            bridges::system::on_time_expires, components::EffectTimers,
            messages::EffectTimerExpired, tick_timers::tick_effect_timers,
        },
        types::{ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
        walking::{UntilApplied, walk_effects::walk_bound_effects},
    },
    prelude::*,
};

fn test_source(name: &str) -> SourceId {
    SourceId::from(name.to_owned())
}

fn surge_when_until_speed_tree(duration: f32, multiplier: f32) -> Tree {
    Tree::When(
        Trigger::NodeStartOccurred,
        Box::new(Tree::Until(
            Trigger::TimeExpires(OrderedFloat(duration)),
            Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                },
            ))),
        )),
    )
}

fn pipeline_test_app() -> App {
    // Production ordering: bridge before tick (mirroring the existing
    // end_to_end_timer_expires_and_bridge_fires_across_two_ticks test).
    TestAppBuilder::new()
        .with_message::<EffectTimerExpired>()
        .with_system(
            FixedUpdate,
            (
                on_time_expires.before(tick_effect_timers),
                tick_effect_timers,
            ),
        )
        .build()
}

fn drive_node_start_walk(app: &mut App, entity: Entity) {
    // TODO: replace with real node-bridge dispatch once the bridge is
    // accessible in test scaffolding without booting full state hierarchy.
    let trees = app
        .world()
        .get::<BoundEffects>(entity)
        .map(|b| b.0.clone())
        .unwrap_or_default();
    let world = app.world_mut();
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, world);
        walk_bound_effects(
            entity,
            &Trigger::NodeStartOccurred,
            &TriggerContext::None,
            &trees,
            &mut commands,
        );
    }
    queue.apply(world);
}

// ----------------------------------------------------------------
// F1: End-to-end. When(NodeStartOccurred, Until(TimeExpires(0.5),
//     Fire(SpeedBoost(2.0)))) reverses naturally after 0.5s of ticking.
// ----------------------------------------------------------------

#[test]
fn until_time_expires_full_pipeline_reverses_after_natural_expiry() {
    let mut app = pipeline_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![(
            "surge".to_string(),
            surge_when_until_speed_tree(0.5, 2.0),
        )]))
        .id();

    // Step 1: drive NodeStartOccurred to arm
    drive_node_start_walk(&mut app, entity);

    // Step 2: post-arm assertions — stack 1, timer 1, source "surge", remaining 0.5
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after arming");
    assert_eq!(stack.len(), 1);
    {
        let entries: Vec<&(SourceId, SpeedBoostConfig)> = stack.iter().collect();
        assert_eq!(entries[0].1.multiplier, OrderedFloat(2.0));
    }

    let timers = app
        .world()
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should be armed after the When -> Until pipeline");
    assert_eq!(timers.timers.len(), 1);
    assert_eq!(timers.timers[0].2, test_source("surge"));
    assert_eq!(timers.timers[0].1, OrderedFloat(0.5));
    assert_eq!(timers.timers[0].0, OrderedFloat(0.5));

    // Step 3: tick 35 times — covers 32 expiry steps + 1 bridge dispatch + 2 spare
    for _ in 0..35 {
        tick(&mut app);
    }

    // Step 4: post-tick assertions — boost reversed, timer cleaned, UntilApplied empty,
    //          BoundEffects has only the original When entry
    let stack_empty = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(
        stack_empty,
        "EffectStack should be empty after natural expiry"
    );

    let timers_clean = match app.world().get::<EffectTimers>(entity) {
        None => true,
        Some(t) => !t
            .timers
            .iter()
            .any(|(_, _, src)| *src == test_source("surge")),
    };
    assert!(
        timers_clean,
        "EffectTimers entry under surge should be gone after natural expiry"
    );

    let until_applied_chip = app
        .world()
        .get::<UntilApplied>(entity)
        .is_some_and(|ua| ua.0.contains("surge"));
    assert!(
        !until_applied_chip,
        "UntilApplied should not contain surge after natural expiry"
    );

    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    let surge_entries: Vec<&Tree> = bound
        .0
        .iter()
        .filter(|(name, _)| name == "surge")
        .map(|(_, tree)| tree)
        .collect();
    assert_eq!(
        surge_entries.len(),
        1,
        "BoundEffects should contain exactly one entry under surge after natural expiry"
    );
    assert_eq!(
        surge_entries[0],
        &surge_when_until_speed_tree(0.5, 2.0),
        "the remaining surge entry must be the original When wrapper, not the inner Until"
    );

    let staged_clean = app
        .world()
        .get::<StagedEffects>(entity)
        .is_none_or(|s| s.0.iter().all(|(name, _)| name != "surge"));
    assert!(
        staged_clean,
        "StagedEffects must NOT contain a surge entry — the Until is bound, not staged"
    );
}

#[test]
fn until_time_expires_full_pipeline_does_not_reverse_before_expiry() {
    let mut app = pipeline_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![(
            "surge".to_string(),
            surge_when_until_speed_tree(0.5, 2.0),
        )]))
        .id();

    drive_node_start_walk(&mut app, entity);

    // Tick only 5 times — well short of 32 + 1 needed for expiry
    for _ in 0..5 {
        tick(&mut app);
    }

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should still be present");
    assert_eq!(
        stack.len(),
        1,
        "boost should NOT yet have reversed after only 5 ticks"
    );
}

#[test]
fn until_time_expires_full_pipeline_when_outer_re_arms_after_expiry() {
    // Proves the When outer survives the Until's lifecycle and can re-arm
    // a fresh Until on a second NodeStartOccurred.
    let mut app = pipeline_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![(
            "surge".to_string(),
            surge_when_until_speed_tree(0.5, 2.0),
        )]))
        .id();

    drive_node_start_walk(&mut app, entity);
    for _ in 0..35 {
        tick(&mut app);
    }

    // Sanity: Until is gone, boost reversed
    let stack_empty = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(stack_empty, "boost should have reversed after first cycle");

    // Re-arm via second NodeStartOccurred
    drive_node_start_walk(&mut app, entity);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should reappear after second arm");
    assert_eq!(stack.len(), 1, "boost should be re-applied");

    let timers = app
        .world()
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should be re-armed");
    assert_eq!(timers.timers.len(), 1);
    assert_eq!(
        timers.timers[0],
        (OrderedFloat(0.5), OrderedFloat(0.5), test_source("surge"),)
    );

    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    let surge_count = bound.0.iter().filter(|(name, _)| name == "surge").count();
    assert_eq!(
        surge_count, 2,
        "after re-arming, BoundEffects should contain TWO entries under surge: the When outer and the freshly bound Until"
    );
}
