//! Entropy engine systems — reset counter on node start and tick bump counting.

use bevy::{ecs::message::Messages, prelude::*};
use rand::Rng;

use super::components::EntropyCounter;
use crate::{
    breaker::messages::BumpPerformed,
    effect_v3::{dispatch::fire_dispatch, types::EffectType},
    shared::rng::GameRng,
};

/// Resets all `EntropyCounter` components to zero at the start of each node.
pub fn reset_entropy_counter(mut query: Query<&mut EntropyCounter>) {
    for mut counter in &mut query {
        counter.count = 0;
    }
}

/// Processes `BumpPerformed` messages to increment entropy counters and
/// fire escalating random effects.
///
/// For each bump: increments count (capped at `max_effects`), then fires
/// N effects where N = current count, each selected from the weighted pool.
///
/// Exclusive system — requires `&mut World` for `fire_dispatch`.
pub fn tick_entropy_engine(world: &mut World) {
    // Phase 1: Count bumps this frame.
    let bump_count = world
        .resource::<Messages<BumpPerformed>>()
        .iter_current_update_messages()
        .count();

    if bump_count == 0 {
        return;
    }

    // Phase 2: Collect counter entities and their data.
    let entries: Vec<(Entity, EntropyCounter)> = {
        let mut query = world.query::<(Entity, &EntropyCounter)>();
        query.iter(world).map(|(e, c)| (e, c.clone())).collect()
    };

    // Phase 3: Process bumps sequentially per entity.
    for (entity, mut counter) in entries {
        for _ in 0..bump_count {
            // Increment count (capped at max_effects).
            if counter.count < counter.max_effects {
                counter.count += 1;
            }

            // Fire `count` random effects from pool.
            if counter.pool.is_empty() {
                continue;
            }

            let total_weight: f32 = counter.pool.iter().map(|(w, _)| w.0).sum();
            if total_weight <= 0.0 {
                continue;
            }

            for _ in 0..counter.count {
                let effect = pick_weighted_effect(&counter.pool, total_weight, world);
                fire_dispatch(&effect, entity, "", world);
            }
        }

        // Write back updated counter.
        if let Some(mut c) = world.get_mut::<EntropyCounter>(entity) {
            c.count = counter.count;
        }
    }
}

/// Pick a random effect from a weighted pool using `GameRng`.
fn pick_weighted_effect(
    pool: &[(ordered_float::OrderedFloat<f32>, Box<EffectType>)],
    total_weight: f32,
    world: &mut World,
) -> EffectType {
    let roll: f32 = world.resource_mut::<GameRng>().0.random::<f32>() * total_weight;
    let mut accumulated = 0.0;
    for (weight, effect) in pool {
        accumulated += weight.0;
        if roll < accumulated {
            return (**effect).clone();
        }
    }
    // Fallback to last entry if floating-point imprecision causes no match.
    // Caller guarantees pool is non-empty.
    (*pool[pool.len() - 1].1).clone()
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::{
        breaker::messages::{BumpGrade, BumpPerformed},
        effect_v3::{
            effects::{
                ShockwaveConfig, SpeedBoostConfig, entropy_engine::components::EntropyCounter,
                shockwave::components::ShockwaveSource,
            },
            stacking::EffectStack,
            types::EffectType,
        },
        shared::{
            rng::GameRng,
            test_utils::{TestAppBuilder, tick},
        },
    };

    // -- Helpers ----------------------------------------------------------

    /// Resource to inject `BumpPerformed` messages into the test app.
    #[derive(Resource, Default)]
    struct TestBumpMessages(Vec<BumpPerformed>);

    /// System that writes `BumpPerformed` messages from the test resource.
    fn inject_bumps(messages: Res<TestBumpMessages>, mut writer: MessageWriter<BumpPerformed>) {
        for msg in &messages.0 {
            writer.write(msg.clone());
        }
    }

    fn entropy_app() -> App {
        TestAppBuilder::new()
            .with_message::<BumpPerformed>()
            .with_resource::<TestBumpMessages>()
            .insert_resource(GameRng::from_seed(42))
            .with_system(
                FixedUpdate,
                (
                    inject_bumps.before(tick_entropy_engine),
                    tick_entropy_engine,
                ),
            )
            .build()
    }

    fn make_shockwave_effect() -> (OrderedFloat<f32>, Box<EffectType>) {
        (
            OrderedFloat(1.0),
            Box::new(EffectType::Shockwave(ShockwaveConfig {
                base_range:      OrderedFloat(48.0),
                range_per_level: OrderedFloat(0.0),
                stacks:          1,
                speed:           OrderedFloat(150.0),
            })),
        )
    }

    fn make_speed_boost_effect() -> (OrderedFloat<f32>, Box<EffectType>) {
        (
            OrderedFloat(1.0),
            Box::new(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
        )
    }

    fn spawn_counter(
        app: &mut App,
        count: u32,
        max_effects: u32,
        pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
    ) -> Entity {
        app.world_mut()
            .spawn(EntropyCounter {
                count,
                max_effects,
                pool,
            })
            .id()
    }

    fn queue_bump(app: &mut App) {
        let breaker = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<TestBumpMessages>()
            .0
            .push(BumpPerformed {
                grade: BumpGrade::Perfect,
                bolt: None,
                breaker,
            });
    }

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

    fn reset_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, reset_entropy_counter)
            .build()
    }

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

    // ── Behavior 10: Weighted pool with single entry (deterministic) ──

    #[test]
    fn weighted_pool_selects_from_entries() {
        let mut app = entropy_app();

        // Single-entry pool for deterministic outcome
        spawn_counter(&mut app, 0, 10, vec![make_shockwave_effect()]);
        queue_bump(&mut app);

        tick(&mut app);

        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert_eq!(
            shockwave_count, 1,
            "single-entry pool should always select that entry; expected 1 shockwave, got {shockwave_count}",
        );
    }

    // ── Behavior 11: Pool with all weight on one entry always selects it ──

    #[test]
    fn zero_weight_entry_never_selected() {
        let mut app = entropy_app();

        // Zero-weight SpeedBoost, full-weight Shockwave
        let pool = vec![
            (
                OrderedFloat(0.0),
                Box::new(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                })),
            ),
            make_shockwave_effect(), // weight=1.0
        ];
        let entity = spawn_counter(&mut app, 0, 5, pool);
        queue_bump(&mut app);

        tick(&mut app);

        // Shockwave should be selected (not SpeedBoost)
        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert_eq!(
            shockwave_count, 1,
            "zero-weight entry should not be selected; expected 1 shockwave, got {shockwave_count}",
        );

        // SpeedBoost should NOT be on the entity
        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "zero-weight SpeedBoost should never be selected"
        );
    }

    #[test]
    fn zero_weight_entry_never_selected_reversed_order() {
        let mut app = entropy_app();

        // Full-weight Shockwave first, zero-weight SpeedBoost second
        let pool = vec![
            make_shockwave_effect(), // weight=1.0
            (
                OrderedFloat(0.0),
                Box::new(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                })),
            ),
        ];
        let entity = spawn_counter(&mut app, 0, 5, pool);
        queue_bump(&mut app);

        tick(&mut app);

        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert_eq!(
            shockwave_count, 1,
            "Shockwave with weight=1.0 first should always be selected; got {shockwave_count}",
        );

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "zero-weight SpeedBoost should never be selected (reversed order)"
        );
    }

    // ── Behavior 12: All zero weights fires no effects ──

    #[test]
    fn all_zero_weights_fires_no_effects() {
        let mut app = entropy_app();

        let pool = vec![
            (
                OrderedFloat(0.0),
                Box::new(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                })),
            ),
            (
                OrderedFloat(0.0),
                Box::new(EffectType::Shockwave(
                    crate::effect_v3::effects::ShockwaveConfig {
                        base_range:      OrderedFloat(48.0),
                        range_per_level: OrderedFloat(0.0),
                        stacks:          1,
                        speed:           OrderedFloat(150.0),
                    },
                )),
            ),
        ];
        let entity = spawn_counter(&mut app, 0, 5, pool);
        queue_bump(&mut app);

        tick(&mut app);

        // Count should still increment (the zero-weight guard is after increment)
        let counter = app.world().get::<EntropyCounter>(entity).unwrap();
        assert_eq!(
            counter.count, 1,
            "count should increment to 1 even with zero-weight pool"
        );

        // No shockwave should have been spawned
        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert_eq!(
            shockwave_count, 0,
            "all-zero-weight pool should fire no effects, got {shockwave_count}",
        );
    }

    // ── Behavior 13: Multiple effects fired in one bump use independent draws ──

    #[test]
    fn multiple_fires_use_independent_draws() {
        let mut app = entropy_app();

        // count=2, max=5, all-Shockwave pool — bump increments to 3, fires 3
        spawn_counter(&mut app, 2, 5, vec![make_shockwave_effect()]);
        queue_bump(&mut app);

        tick(&mut app);

        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert_eq!(
            shockwave_count, 3,
            "count=2 incremented to 3, each draw should independently select shockwave; expected 3, got {shockwave_count}",
        );
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

    // ── Behavior 18: Fired effects receive empty source chip ──

    #[test]
    fn fired_effects_receive_empty_source_chip() {
        use crate::effect_v3::components::EffectSourceChip;

        let mut app = entropy_app();

        spawn_counter(&mut app, 0, 3, vec![make_shockwave_effect()]);
        queue_bump(&mut app);

        tick(&mut app);

        // The shockwave should have been spawned with EffectSourceChip(None)
        // because fire_dispatch is called with source: ""
        let chips: Vec<&EffectSourceChip> = app
            .world_mut()
            .query_filtered::<&EffectSourceChip, With<ShockwaveSource>>()
            .iter(app.world())
            .collect();

        assert_eq!(
            chips.len(),
            1,
            "expected 1 shockwave entity with EffectSourceChip, got {}",
            chips.len(),
        );
        assert_eq!(
            chips[0].0, None,
            "fire_dispatch called with source=\"\" should produce EffectSourceChip(None), got {:?}",
            chips[0].0,
        );
    }
}
