use bevy::prelude::*;

use super::helpers::*;
use crate::{
    effect_v3::{
        effects::{
            SpeedBoostConfig, entropy_engine::components::EntropyCounter,
            shockwave::components::ShockwaveSource,
        },
        stacking::EffectStack,
    },
    shared::test_utils::tick,
};

// ── C12-1: Each BumpPerformed increments the counter ──

#[test]
fn bump_increments_counter() {
    let mut app = entropy_app();

    let entity = spawn_counter(&mut app, 0, 3, vec![make_shockwave_effect()]);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(
        counter.count, 1,
        "count should increment from 0 to 1, got {}",
        counter.count,
    );
}

#[test]
fn counter_caps_at_max_effects() {
    let mut app = entropy_app();

    let entity = spawn_counter(&mut app, 3, 3, vec![make_shockwave_effect()]);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(
        counter.count, 3,
        "count should cap at max_effects (3) when already at max, got {}",
        counter.count,
    );
}

// ── C12-2: Activation fires N random effects where N = count ──

#[test]
fn bump_fires_count_effects_from_pool() {
    let mut app = entropy_app();

    // count=2 means: bump increments to 3, fires 3 effects
    spawn_counter(&mut app, 2, 5, vec![make_shockwave_effect()]);
    queue_bump(&mut app);

    tick(&mut app);

    // With a single shockwave in the pool, all 3 fires should spawn shockwaves
    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        shockwave_count, 3,
        "should fire 3 shockwave effects (count incremented to 3), got {shockwave_count}",
    );
}

#[test]
fn single_pool_entry_always_selected() {
    let mut app = entropy_app();

    // count=0, max=5, one entry: bump to 1, fire 1 effect
    spawn_counter(&mut app, 0, 5, vec![make_shockwave_effect()]);
    queue_bump(&mut app);

    tick(&mut app);

    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        shockwave_count, 1,
        "single pool entry: 1 shockwave should fire, got {shockwave_count}",
    );
}

// ── C12-3: No BumpPerformed — counter unchanged ──

#[test]
fn no_bumps_leaves_counter_unchanged() {
    let mut app = entropy_app();

    let entity = spawn_counter(&mut app, 1, 3, vec![make_shockwave_effect()]);

    tick(&mut app);

    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(
        counter.count, 1,
        "count should remain 1 with no bumps, got {}",
        counter.count,
    );
}

// ── C12-4: Multiple bumps each trigger escalation ──

#[test]
fn two_bumps_fire_escalating_effects() {
    let mut app = entropy_app();

    // count=0, max=5, one shockwave entry
    spawn_counter(&mut app, 0, 5, vec![make_shockwave_effect()]);
    queue_bump(&mut app);
    queue_bump(&mut app);

    tick(&mut app);

    // First bump: count 0->1, fire 1 effect
    // Second bump: count 1->2, fire 2 effects
    // Total: 3 effects
    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        shockwave_count, 3,
        "two bumps should fire 1+2=3 effects total, got {shockwave_count}",
    );
}

#[test]
fn counter_after_two_bumps_equals_two() {
    let mut app = entropy_app();

    let entity = spawn_counter(&mut app, 0, 5, vec![make_shockwave_effect()]);
    queue_bump(&mut app);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(
        counter.count, 2,
        "count should be 2 after two bumps, got {}",
        counter.count,
    );
}

#[test]
fn three_bumps_cap_at_max_effects_for_second_and_third() {
    let mut app = entropy_app();

    // count starts at max_effects-1 (2), max_effects=3
    let entity = spawn_counter(&mut app, 2, 3, vec![make_shockwave_effect()]);
    queue_bump(&mut app);
    queue_bump(&mut app);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(
        counter.count, 3,
        "count should cap at max_effects (3), got {}",
        counter.count,
    );

    // First bump: count 2->3, fire 3 effects
    // Second bump: count 3 (capped), fire 3 effects
    // Third bump: count 3 (capped), fire 3 effects
    // Total: 9 effects
    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        shockwave_count, 9,
        "three bumps near max should fire 3+3+3=9 effects, got {shockwave_count}",
    );
}

// ── C12-5: Empty pool results in no effects ──

#[test]
fn empty_pool_fires_no_effects() {
    let mut app = entropy_app();

    let entity = spawn_counter(&mut app, 0, 3, vec![]);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(
        counter.count, 1,
        "count should increment even with empty pool, got {}",
        counter.count,
    );

    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        shockwave_count, 0,
        "empty pool should fire no effects, got {shockwave_count}",
    );
}

// ── C12-6: Multiple entities process bumps independently ──

#[test]
fn multiple_entities_process_bumps_independently() {
    let mut app = entropy_app();

    // Entity A: count=1, max=3, shockwave pool
    let entity_a = spawn_counter(&mut app, 1, 3, vec![make_shockwave_effect()]);

    // Entity B: count=0, max=2, speed boost pool
    let entity_b = spawn_counter(&mut app, 0, 2, vec![make_speed_boost_effect()]);

    queue_bump(&mut app);

    tick(&mut app);

    let counter_a = app.world().get::<EntropyCounter>(entity_a).unwrap();
    assert_eq!(
        counter_a.count, 2,
        "entity A count should be 2, got {}",
        counter_a.count,
    );

    let counter_b = app.world().get::<EntropyCounter>(entity_b).unwrap();
    assert_eq!(
        counter_b.count, 1,
        "entity B count should be 1, got {}",
        counter_b.count,
    );

    // Entity A: fires 2 shockwaves (count=2)
    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        shockwave_count, 2,
        "entity A should fire 2 shockwaves, got {shockwave_count}",
    );

    // Entity B: fires 1 speed boost (count=1) — creates EffectStack on entity B
    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity_b);
    assert!(
        stack.is_some(),
        "entity B should have SpeedBoost effect stack after fire",
    );
}

// ── Behavior 8: reset_entropy_counter sets count to 0 for all entities ──

#[test]
fn reset_sets_all_counts_to_zero() {
    let mut app = reset_app();

    let entity_a = app
        .world_mut()
        .spawn(EntropyCounter {
            count:       3,
            max_effects: 5,
            pool:        vec![make_shockwave_effect()],
        })
        .id();

    let entity_b = app
        .world_mut()
        .spawn(EntropyCounter {
            count:       1,
            max_effects: 2,
            pool:        vec![make_speed_boost_effect()],
        })
        .id();

    tick(&mut app);

    let counter_a = app.world().get::<EntropyCounter>(entity_a).unwrap();
    assert_eq!(counter_a.count, 0, "entity_a count should reset to 0");
    assert_eq!(
        counter_a.max_effects, 5,
        "entity_a max_effects should be unchanged"
    );
    assert_eq!(counter_a.pool.len(), 1, "entity_a pool should be unchanged");

    let counter_b = app.world().get::<EntropyCounter>(entity_b).unwrap();
    assert_eq!(counter_b.count, 0, "entity_b count should reset to 0");
    assert_eq!(
        counter_b.max_effects, 2,
        "entity_b max_effects should be unchanged"
    );
    assert_eq!(counter_b.pool.len(), 1, "entity_b pool should be unchanged");
}

#[test]
fn reset_leaves_zero_count_unchanged() {
    let mut app = reset_app();

    let entity = app
        .world_mut()
        .spawn(EntropyCounter {
            count:       0,
            max_effects: 5,
            pool:        vec![make_shockwave_effect()],
        })
        .id();

    tick(&mut app);

    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(
        counter.count, 0,
        "count already at 0 should remain 0 after reset"
    );
}

// ── Behavior 9: reset with no entities does nothing ──

#[test]
fn reset_with_no_entities_does_not_panic() {
    let mut app = reset_app();
    // No EntropyCounter entities spawned
    tick(&mut app);
    // If we reach here, no panic occurred.
}

// ── Behavior 15: Bump with no counter entities does not panic ──

#[test]
fn bump_with_no_counter_entities_does_not_panic() {
    let mut app = entropy_app();
    // No EntropyCounter entities, just a bump
    queue_bump(&mut app);

    tick(&mut app);
    // If we reach here, no panic occurred.
}

// ── Behavior 16: max_effects=1 caps at 1 and fires exactly 1 ──

#[test]
fn max_effects_one_fires_exactly_one() {
    let mut app = entropy_app();

    let entity = spawn_counter(&mut app, 0, 1, vec![make_shockwave_effect()]);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(counter.count, 1, "count should cap at 1 with max_effects=1");

    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        shockwave_count, 1,
        "max_effects=1 should fire exactly 1 effect, got {shockwave_count}",
    );
}

#[test]
fn max_effects_one_two_bumps_fires_two_total() {
    let mut app = entropy_app();

    let entity = spawn_counter(&mut app, 0, 1, vec![make_shockwave_effect()]);
    queue_bump(&mut app);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(counter.count, 1, "count should cap at 1 even with 2 bumps");

    // First bump: count 0->1, fires 1. Second bump: count stays at 1, fires 1.
    // Total: 2 shockwaves
    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        shockwave_count, 2,
        "two bumps with max_effects=1: first fires 1, second fires 1 = 2 total; got {shockwave_count}",
    );
}

// ── Behavior 17: max_effects=0 means no increment and no effects ──

#[test]
fn max_effects_zero_fires_nothing() {
    let mut app = entropy_app();

    let entity = spawn_counter(&mut app, 0, 0, vec![make_shockwave_effect()]);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(
        counter.count, 0,
        "count should stay at 0 when max_effects=0"
    );

    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        shockwave_count, 0,
        "max_effects=0 should fire no effects, got {shockwave_count}",
    );
}
