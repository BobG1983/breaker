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
}
