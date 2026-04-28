//! Coverage-fill: Shape 2 (`Until(TimeExpires(_), Sequence([...]))`) reversal
//! under bridge-dispatched `TimeExpires` expiry.
//!
//! Audit gap (Contract 4 from the `Until(TimeExpires(_), _)` wiring audit at
//! commit 945254fd):
//!
//! `until_with_sequence_reverses_all_effects_on_gate_trigger_match` (in
//! `basic_until/sequence.rs`) covers Sequence reversal under the synchronous
//! `Trigger::Bumped` gate. `until_time_expires_full_pipeline_reverses_after_natural_expiry`
//! (in `until_time_expires_pipeline.rs`) covers `Fire(_)` reversal under the
//! bridge-dispatched `TimeExpires` natural expiry. Neither covers the
//! intersection: a multi-effect Sequence reversed by the bridge after the
//! `tick_effect_timers` natural-expiry path.
//!
//! A future refactor in `apply_until_fire_branch` or `reverse_scoped_tree`
//! could break the multi-entry Sequence iteration on the bridge path while
//! leaving the synchronous Bumped path intact. This test pins the
//! intersection.

use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::{
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
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

fn until_time_expires_sequence_tree(
    duration: f32,
    speed_multiplier: f32,
    damage_multiplier: f32,
) -> Tree {
    Tree::Until(
        Trigger::TimeExpires(OrderedFloat(duration)),
        Box::new(ScopedTree::Sequence(vec![
            ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(speed_multiplier),
            }),
            ReversibleEffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(damage_multiplier),
            }),
        ])),
    )
}

/// Same `FixedUpdate` ordering as `until_time_expires_full_pipeline_reverses_after_natural_expiry`:
/// `on_time_expires` runs before `tick_effect_timers` so a timer that expires
/// during a tick is dispatched the next frame.
fn pipeline_test_app() -> App {
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
// Contract 4: Until(TimeExpires(1.0), Sequence([SpeedBoost(1.5),
// DamageBoost(1.5)])) — both effects must reverse on natural bridge-dispatched
// expiry. Concrete values: duration 1.0s, both multipliers 1.5, tick at 64 Hz
// for 70 ticks (well past 64 expiry steps + 1 bridge dispatch).
// ----------------------------------------------------------------

#[test]
fn until_time_expires_with_sequence_reverses_all_effects_on_natural_expiry() {
    let mut app = pipeline_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![(
            "chip_seq".to_string(),
            until_time_expires_sequence_tree(1.0, 1.5, 1.5),
        )]))
        .id();

    drive_node_start_walk(&mut app, entity);

    // Step 2: post-arm sanity — both effects fired, timer armed for 1.0s
    let speed_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("SpeedBoost EffectStack must exist after Sequence arm");
    {
        let entries: Vec<&(SourceId, SpeedBoostConfig)> = speed_stack.iter().collect();
        assert_eq!(entries.len(), 1, "SpeedBoost must be applied once on arm");
        assert_eq!(entries[0].0, test_source("chip_seq"));
        assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));
    }

    let dmg_stack = app
        .world()
        .get::<DamageBoostStack>(entity)
        .expect("DamageBoostStack must exist after Sequence arm");
    assert!(
        !dmg_stack.is_empty(),
        "DamageBoost must be applied once on arm"
    );
    assert!(
        (dmg_stack.aggregate_persistent(None) - 1.5).abs() < 1e-5,
        "DamageBoost aggregate must be 1.5 after arm — got {}",
        dmg_stack.aggregate_persistent(None)
    );

    let timers = app
        .world()
        .get::<EffectTimers>(entity)
        .expect("EffectTimers must be armed after Sequence arm");
    assert_eq!(timers.timers.len(), 1);
    assert_eq!(
        timers.timers[0],
        (
            OrderedFloat(1.0),
            OrderedFloat(1.0),
            test_source("chip_seq"),
        ),
        "(remaining, original, source) must be (1.0, 1.0, chip_seq) after arm"
    );

    let until_applied = app
        .world()
        .get::<UntilApplied>(entity)
        .expect("UntilApplied must contain chip_seq after arm");
    assert!(until_applied.0.contains("chip_seq"));

    // Tick at 64 Hz for 70 ticks: 64 expiry decrements + 1 bridge dispatch +
    // 1 message-consume tick + 4 spare. Matches the slack convention in
    // `until_time_expires_full_pipeline_reverses_after_natural_expiry`.
    for _ in 0..70 {
        tick(&mut app);
    }

    // Step 4: assert BOTH effects reversed naturally via bridge
    let speed_empty = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(
        speed_empty,
        "SpeedBoost EffectStack must be empty after natural expiry — \
         Sequence iteration must reverse the SpeedBoost arm"
    );

    let dmg_empty = app
        .world()
        .get::<DamageBoostStack>(entity)
        .is_none_or(DamageBoostStack::is_empty);
    assert!(
        dmg_empty,
        "DamageBoostStack must be empty after natural expiry — \
         Sequence iteration must reverse the DamageBoost arm"
    );

    // Timer entry under chip_seq must be cleared
    let timers_clean = app.world().get::<EffectTimers>(entity).is_none_or(|t| {
        !t.timers
            .iter()
            .any(|(_, _, src)| *src == test_source("chip_seq"))
    });
    assert!(
        timers_clean,
        "EffectTimers entry under chip_seq must be cleared after natural expiry"
    );

    // UntilApplied must no longer contain chip_seq
    let until_applied_after = app
        .world()
        .get::<UntilApplied>(entity)
        .is_some_and(|ua| ua.0.contains("chip_seq"));
    assert!(
        !until_applied_after,
        "UntilApplied must not contain chip_seq after natural expiry"
    );

    // BoundEffects: chip_seq Until entry must be removed. teardown_until_entry
    // may also remove the BoundEffects component entirely if the vec becomes
    // empty — both outcomes (component absent, or component present with no
    // chip_seq entry) satisfy the contract. Same is_none_or pattern as the
    // EffectStack / DamageBoostStack / EffectTimers / UntilApplied checks above.
    let bound_clean = app
        .world()
        .get::<BoundEffects>(entity)
        .is_none_or(|b| b.0.iter().all(|(name, _)| name != "chip_seq"));
    assert!(
        bound_clean,
        "BoundEffects entry chip_seq must be removed on natural expiry"
    );
}
