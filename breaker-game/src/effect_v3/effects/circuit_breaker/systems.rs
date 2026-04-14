//! Circuit breaker systems — bump counting toward automatic shockwave + bolt spawn.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::components::CircuitBreakerCounter;
use crate::{
    breaker::messages::BumpPerformed,
    effect_v3::{
        commands::FireEffectCommand,
        effects::{ShockwaveConfig, SpawnBoltsConfig},
        types::EffectType,
    },
};

/// Processes `BumpPerformed` messages to decrement circuit breaker counters.
///
/// When a counter reaches zero, queues reward effects (shockwave + spawn bolts)
/// via deferred [`FireEffectCommand`] and resets the counter. Processes bumps
/// sequentially per entity (fire-reset-continue within frame).
pub fn tick_circuit_breaker(
    mut bumps: MessageReader<BumpPerformed>,
    mut counter_query: Query<(Entity, &mut CircuitBreakerCounter)>,
    mut commands: Commands,
) {
    let bump_count = bumps.read().count();
    if bump_count == 0 {
        return;
    }

    for (entity, mut counter) in &mut counter_query {
        for _ in 0..bump_count {
            counter.remaining -= 1;
            if counter.remaining == 0 {
                // Queue reward shockwave.
                commands.queue(FireEffectCommand {
                    entity,
                    effect: EffectType::Shockwave(ShockwaveConfig {
                        base_range:      OrderedFloat(counter.shockwave_range),
                        range_per_level: OrderedFloat(0.0),
                        stacks:          1,
                        speed:           OrderedFloat(counter.shockwave_speed),
                    }),
                    source: String::new(),
                });

                // Queue reward bolt spawn.
                commands.queue(FireEffectCommand {
                    entity,
                    effect: EffectType::SpawnBolts(SpawnBoltsConfig {
                        count:    counter.spawn_count,
                        lifespan: None,
                        inherit:  counter.inherit,
                    }),
                    source: String::new(),
                });

                // Reset counter.
                counter.remaining = counter.bumps_required;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_spatial2d::components::{BaseSpeed, Position2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::components::{Bolt, ExtraBolt, PrimaryBolt},
        breaker::messages::{BumpGrade, BumpPerformed},
        effect_v3::{
            effects::{
                DamageBoostConfig, circuit_breaker::components::CircuitBreakerCounter,
                shockwave::components::ShockwaveSource,
            },
            storage::BoundEffects,
            types::{EffectType, Tree},
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

    fn circuit_breaker_app() -> App {
        TestAppBuilder::new()
            .with_message::<BumpPerformed>()
            .with_resource::<TestBumpMessages>()
            .with_resource::<GameRng>()
            .with_system(
                FixedUpdate,
                (
                    inject_bumps.before(tick_circuit_breaker),
                    tick_circuit_breaker,
                ),
            )
            .build()
    }

    fn spawn_counter(app: &mut App, remaining: u32, bumps_required: u32) -> Entity {
        app.world_mut()
            .spawn(CircuitBreakerCounter {
                remaining,
                bumps_required,
                spawn_count: 2,
                inherit: false,
                shockwave_range: 64.0,
                shockwave_speed: 200.0,
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

    // ── C11-1: Each BumpPerformed decrements the counter ──

    #[test]
    fn bump_decrements_counter() {
        let mut app = circuit_breaker_app();

        let entity = spawn_counter(&mut app, 3, 3);
        queue_bump(&mut app);

        tick(&mut app);

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 2,
            "counter should decrement from 3 to 2 after one bump, got {}",
            counter.remaining,
        );
    }

    #[test]
    fn two_bumps_decrement_by_two() {
        let mut app = circuit_breaker_app();

        let entity = spawn_counter(&mut app, 3, 3);
        queue_bump(&mut app);
        queue_bump(&mut app);

        tick(&mut app);

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 1,
            "counter should decrement from 3 to 1 after two bumps, got {}",
            counter.remaining,
        );
    }

    // ── C11-2: Counter reaching zero fires reward and resets ──

    #[test]
    fn counter_reaching_zero_resets_to_bumps_required() {
        let mut app = circuit_breaker_app();

        let entity = spawn_counter(&mut app, 1, 3);
        queue_bump(&mut app);

        tick(&mut app);

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 3,
            "counter should reset to bumps_required (3) after reaching zero, got {}",
            counter.remaining,
        );
    }

    #[test]
    fn counter_reaching_zero_dispatches_shockwave() {
        let mut app = circuit_breaker_app();

        spawn_counter(&mut app, 1, 3);
        queue_bump(&mut app);

        tick(&mut app);

        // Verify a ShockwaveSource entity was spawned by fire_dispatch
        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert!(
            shockwave_count > 0,
            "shockwave should be spawned when circuit breaker fires, got 0",
        );
    }

    #[test]
    fn bumps_required_one_fires_every_bump() {
        let mut app = circuit_breaker_app();

        let entity = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       1,
                bumps_required:  1,
                spawn_count:     2,
                inherit:         false,
                shockwave_range: 64.0,
                shockwave_speed: 200.0,
            })
            .id();
        queue_bump(&mut app);

        tick(&mut app);

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 1,
            "bumps_required=1: single bump should fire and reset to 1, got {}",
            counter.remaining,
        );
    }

    // ── C11-3: Multiple entities all decrement on same bump ──

    #[test]
    fn multiple_entities_each_decrement_on_same_bump() {
        let mut app = circuit_breaker_app();

        let entity_a = spawn_counter(&mut app, 2, 2);
        let entity_b = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       1,
                bumps_required:  3,
                spawn_count:     1,
                inherit:         false,
                shockwave_range: 50.0,
                shockwave_speed: 150.0,
            })
            .id();

        queue_bump(&mut app);

        tick(&mut app);

        let counter_a = app.world().get::<CircuitBreakerCounter>(entity_a).unwrap();
        assert_eq!(
            counter_a.remaining, 1,
            "entity A should decrement from 2 to 1, got {}",
            counter_a.remaining,
        );

        let counter_b = app.world().get::<CircuitBreakerCounter>(entity_b).unwrap();
        assert_eq!(
            counter_b.remaining, 3,
            "entity B should reset to 3 after reaching zero, got {}",
            counter_b.remaining,
        );
    }

    // ── C11-4: No BumpPerformed — counter unchanged ──

    #[test]
    fn no_bumps_leaves_counter_unchanged() {
        let mut app = circuit_breaker_app();

        let entity = spawn_counter(&mut app, 2, 3);

        tick(&mut app);

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 2,
            "counter should remain 2 with no bumps, got {}",
            counter.remaining,
        );
    }

    #[test]
    fn bump_with_no_counter_entities_does_not_panic() {
        let mut app = circuit_breaker_app();

        queue_bump(&mut app);

        // Should not panic
        tick(&mut app);
    }

    // ── C11-5: Counter wrapping — exactly enough bumps to fire twice ──

    #[test]
    fn three_bumps_fire_twice_with_bumps_required_two_remaining_one() {
        let mut app = circuit_breaker_app();

        let entity = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       1,
                bumps_required:  2,
                spawn_count:     1,
                inherit:         false,
                shockwave_range: 50.0,
                shockwave_speed: 150.0,
            })
            .id();

        queue_bump(&mut app);
        queue_bump(&mut app);
        queue_bump(&mut app);

        tick(&mut app);

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        // 1st bump: remaining 1->0, fire, reset to 2
        // 2nd bump: remaining 2->1
        // 3rd bump: remaining 1->0, fire, reset to 2
        // Final: 2
        assert_eq!(
            counter.remaining, 2,
            "after 3 bumps (fire twice), remaining should be 2, got {}",
            counter.remaining,
        );

        // Should have 2 shockwave entities
        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert_eq!(
            shockwave_count, 2,
            "should have dispatched 2 shockwaves, got {shockwave_count}",
        );
    }

    #[test]
    fn bumps_required_one_with_two_bumps_fires_twice() {
        let mut app = circuit_breaker_app();

        let entity = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       1,
                bumps_required:  1,
                spawn_count:     1,
                inherit:         false,
                shockwave_range: 50.0,
                shockwave_speed: 150.0,
            })
            .id();

        queue_bump(&mut app);
        queue_bump(&mut app);

        tick(&mut app);

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        // Each bump fires and resets: final remaining = 1
        assert_eq!(
            counter.remaining, 1,
            "bumps_required=1 with 2 bumps: fires twice, final remaining should be 1, got {}",
            counter.remaining,
        );
    }

    // ── C11-6: SpawnBolts reward fires alongside shockwave on zero-reach ──

    #[test]
    fn counter_reaching_zero_spawns_spawn_count_extra_bolts() {
        let mut app = circuit_breaker_app();

        app.world_mut().spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  3,
            spawn_count:     2,
            inherit:         false,
            shockwave_range: 64.0,
            shockwave_speed: 200.0,
        });
        queue_bump(&mut app);

        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 2,
            "spawn_count=2 should spawn exactly 2 ExtraBolt entities, got {extra_count}",
        );

        // Regression: shockwave must still fire alongside spawn_bolts.
        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert!(
            shockwave_count >= 1,
            "shockwave should still fire when spawn_bolts fires, got {shockwave_count}",
        );
    }

    #[test]
    fn counter_reaching_zero_spawns_five_extra_bolts_when_spawn_count_is_five() {
        let mut app = circuit_breaker_app();

        app.world_mut().spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  2,
            spawn_count:     5,
            inherit:         false,
            shockwave_range: 64.0,
            shockwave_speed: 200.0,
        });
        queue_bump(&mut app);

        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 5,
            "spawn_count=5 should spawn exactly 5 ExtraBolt entities, got {extra_count}",
        );
    }

    // ── C11-7: spawn_count: 0 fires no ExtraBolt but still fires shockwave ──

    #[test]
    fn spawn_count_zero_fires_no_extra_bolts_but_shockwave_still_fires() {
        let mut app = circuit_breaker_app();

        let entity = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       1,
                bumps_required:  2,
                spawn_count:     0,
                inherit:         false,
                shockwave_range: 64.0,
                shockwave_speed: 200.0,
            })
            .id();
        queue_bump(&mut app);

        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 0,
            "spawn_count=0 should spawn no ExtraBolt entities, got {extra_count}",
        );

        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert!(
            shockwave_count >= 1,
            "shockwave reward should fire regardless of spawn_count, got {shockwave_count}",
        );

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 2,
            "counter should reset to bumps_required (2) even with spawn_count=0, got {}",
            counter.remaining,
        );
    }

    #[test]
    fn spawn_count_zero_spawns_no_bolt_markers_at_all() {
        let mut app = circuit_breaker_app();

        app.world_mut().spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  2,
            spawn_count:     0,
            inherit:         false,
            shockwave_range: 64.0,
            shockwave_speed: 200.0,
        });
        queue_bump(&mut app);

        tick(&mut app);

        // The source entity does NOT carry Bolt, so any non-zero count here
        // proves something was spawned (which should not happen).
        let bolt_count = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            bolt_count, 0,
            "spawn_count=0 should spawn nothing at all, got {bolt_count} Bolt entities",
        );
    }

    // ── C11-8: Counter not reaching zero does NOT spawn ExtraBolt ──

    #[test]
    fn counter_not_reaching_zero_does_not_spawn_extra_bolts() {
        let mut app = circuit_breaker_app();

        let entity = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       3,
                bumps_required:  3,
                spawn_count:     4,
                inherit:         false,
                shockwave_range: 64.0,
                shockwave_speed: 200.0,
            })
            .id();
        queue_bump(&mut app);

        tick(&mut app);

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 2,
            "counter should decrement from 3 to 2 without firing reward, got {}",
            counter.remaining,
        );

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 0,
            "no ExtraBolt should be spawned when counter does not reach zero, got {extra_count}",
        );

        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert_eq!(
            shockwave_count, 0,
            "no shockwave should fire when counter does not reach zero, got {shockwave_count}",
        );
    }

    #[test]
    fn counter_not_reaching_zero_with_two_bumps_does_not_spawn_extra_bolts() {
        let mut app = circuit_breaker_app();

        let entity = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       3,
                bumps_required:  3,
                spawn_count:     10,
                inherit:         false,
                shockwave_range: 64.0,
                shockwave_speed: 200.0,
            })
            .id();
        queue_bump(&mut app);
        queue_bump(&mut app);

        tick(&mut app);

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 1,
            "counter should decrement from 3 to 1 without firing, got {}",
            counter.remaining,
        );

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 0,
            "no ExtraBolt should be spawned when counter does not reach zero, got {extra_count}",
        );
    }

    // ── C11-9: Counter wrapping twice in one frame fires SpawnBolts twice ──

    #[test]
    fn wrapping_twice_in_one_frame_fires_spawn_bolts_twice() {
        let mut app = circuit_breaker_app();

        let entity = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       1,
                bumps_required:  2,
                spawn_count:     1,
                inherit:         false,
                shockwave_range: 64.0,
                shockwave_speed: 200.0,
            })
            .id();
        queue_bump(&mut app);
        queue_bump(&mut app);
        queue_bump(&mut app);

        tick(&mut app);

        // 1st bump: remaining 1->0, fire, reset to 2
        // 2nd bump: remaining 2->1
        // 3rd bump: remaining 1->0, fire, reset to 2
        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 2,
            "2 fires * spawn_count=1 should spawn 2 ExtraBolt entities, got {extra_count}",
        );

        let shockwave_count = app
            .world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count();
        assert_eq!(
            shockwave_count, 2,
            "should have dispatched 2 shockwaves, got {shockwave_count}",
        );

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 2,
            "final counter remaining should be 2 (reset), got {}",
            counter.remaining,
        );
    }

    #[test]
    fn wrapping_twice_in_one_frame_uses_stored_spawn_count_each_time() {
        let mut app = circuit_breaker_app();

        app.world_mut().spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  2,
            spawn_count:     3,
            inherit:         false,
            shockwave_range: 64.0,
            shockwave_speed: 200.0,
        });
        queue_bump(&mut app);
        queue_bump(&mut app);
        queue_bump(&mut app);

        tick(&mut app);

        // 2 fires * spawn_count=3 = 6 ExtraBolt entities
        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 6,
            "2 fires * spawn_count=3 should spawn 6 ExtraBolt entities, got {extra_count}",
        );
    }

    // ── C11-10: bumps_required: 1 with two bumps fires SpawnBolts twice ──

    #[test]
    fn bumps_required_one_with_two_bumps_spawns_four_extra_bolts() {
        let mut app = circuit_breaker_app();

        let entity = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       1,
                bumps_required:  1,
                spawn_count:     2,
                inherit:         false,
                shockwave_range: 64.0,
                shockwave_speed: 200.0,
            })
            .id();
        queue_bump(&mut app);
        queue_bump(&mut app);

        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 4,
            "2 fires * spawn_count=2 should spawn 4 ExtraBolt entities, got {extra_count}",
        );

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 1,
            "final counter remaining should be 1 (reset), got {}",
            counter.remaining,
        );
    }

    #[test]
    fn bumps_required_one_with_two_bumps_and_spawn_count_zero_spawns_none() {
        let mut app = circuit_breaker_app();

        let entity = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       1,
                bumps_required:  1,
                spawn_count:     0,
                inherit:         false,
                shockwave_range: 64.0,
                shockwave_speed: 200.0,
            })
            .id();
        queue_bump(&mut app);
        queue_bump(&mut app);

        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 0,
            "spawn_count=0 should spawn no ExtraBolt entities, got {extra_count}",
        );

        let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
        assert_eq!(
            counter.remaining, 1,
            "final counter remaining should still be 1 after two resets, got {}",
            counter.remaining,
        );
    }

    // ── C11-11: inherit: true attaches BoundEffects to spawned ExtraBolt ──

    #[test]
    fn inherit_true_attaches_bound_effects_to_spawned_extra_bolts() {
        let mut app = circuit_breaker_app();

        let tree_a = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }));
        app.world_mut().spawn((
            PrimaryBolt,
            Bolt,
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
            BaseSpeed(400.0),
            BoundEffects(vec![("chip_a".to_string(), tree_a)]),
        ));

        app.world_mut().spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  2,
            spawn_count:     2,
            inherit:         true,
            shockwave_range: 64.0,
            shockwave_speed: 200.0,
        });
        queue_bump(&mut app);

        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 2,
            "spawn_count=2 should spawn exactly 2 ExtraBolt entities, got {extra_count}",
        );

        let inherited: Vec<&BoundEffects> = app
            .world_mut()
            .query_filtered::<&BoundEffects, With<ExtraBolt>>()
            .iter(app.world())
            .collect();
        assert_eq!(
            inherited.len(),
            2,
            "both ExtraBolt entities should carry BoundEffects, got {}",
            inherited.len(),
        );
        for effects in &inherited {
            assert!(
                effects.0.iter().any(|(name, _)| name == "chip_a"),
                "inherited BoundEffects should contain chip_a entry",
            );
        }
    }

    #[test]
    fn inherit_true_with_no_primary_bolt_still_spawns_extra_bolts_without_bound_effects() {
        let mut app = circuit_breaker_app();

        app.world_mut().spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  2,
            spawn_count:     2,
            inherit:         true,
            shockwave_range: 64.0,
            shockwave_speed: 200.0,
        });
        queue_bump(&mut app);

        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 2,
            "spawn_count=2 should spawn exactly 2 ExtraBolt entities even with no PrimaryBolt, got {extra_count}",
        );

        let inherited_count = app
            .world_mut()
            .query_filtered::<&BoundEffects, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            inherited_count, 0,
            "no PrimaryBolt means no BoundEffects to inherit, got {inherited_count}",
        );
    }

    // ── C11-12: inherit: false does NOT attach BoundEffects even with PrimaryBolt ──

    #[test]
    fn inherit_false_does_not_attach_bound_effects_even_with_primary_bolt() {
        let mut app = circuit_breaker_app();

        let tree_a = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }));
        app.world_mut().spawn((
            PrimaryBolt,
            Bolt,
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
            BaseSpeed(400.0),
            BoundEffects(vec![("chip_a".to_string(), tree_a)]),
        ));

        app.world_mut().spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  2,
            spawn_count:     3,
            inherit:         false,
            shockwave_range: 64.0,
            shockwave_speed: 200.0,
        });
        queue_bump(&mut app);

        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 3,
            "spawn_count=3 should spawn exactly 3 ExtraBolt entities, got {extra_count}",
        );

        let inherited_count = app
            .world_mut()
            .query_filtered::<&BoundEffects, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            inherited_count, 0,
            "inherit=false should NOT attach BoundEffects, got {inherited_count}",
        );
    }

    // ── C11-13: Multiple counter entities each fire their own SpawnBolts ──

    #[test]
    fn multiple_counter_entities_each_fire_own_spawn_bolts_on_same_bump() {
        let mut app = circuit_breaker_app();

        let entity_a = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       1,
                bumps_required:  2,
                spawn_count:     2,
                inherit:         false,
                shockwave_range: 64.0,
                shockwave_speed: 200.0,
            })
            .id();
        let entity_b = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       1,
                bumps_required:  3,
                spawn_count:     5,
                inherit:         false,
                shockwave_range: 50.0,
                shockwave_speed: 150.0,
            })
            .id();
        let entity_c = app
            .world_mut()
            .spawn(CircuitBreakerCounter {
                remaining:       3,
                bumps_required:  3,
                spawn_count:     9,
                inherit:         false,
                shockwave_range: 50.0,
                shockwave_speed: 150.0,
            })
            .id();
        queue_bump(&mut app);

        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 7,
            "2 (A) + 5 (B) + 0 (C) = 7 ExtraBolt entities, got {extra_count}",
        );

        let counter_a = app.world().get::<CircuitBreakerCounter>(entity_a).unwrap();
        assert_eq!(counter_a.remaining, 2, "entity A should reset to 2");

        let counter_b = app.world().get::<CircuitBreakerCounter>(entity_b).unwrap();
        assert_eq!(counter_b.remaining, 3, "entity B should reset to 3");

        let counter_c = app.world().get::<CircuitBreakerCounter>(entity_c).unwrap();
        assert_eq!(
            counter_c.remaining, 2,
            "entity C should decrement from 3 to 2",
        );
    }

    #[test]
    fn multiple_counter_entities_one_with_spawn_count_zero_contributes_nothing() {
        let mut app = circuit_breaker_app();

        app.world_mut().spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  2,
            spawn_count:     0,
            inherit:         false,
            shockwave_range: 64.0,
            shockwave_speed: 200.0,
        });
        app.world_mut().spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  3,
            spawn_count:     4,
            inherit:         false,
            shockwave_range: 50.0,
            shockwave_speed: 150.0,
        });
        queue_bump(&mut app);

        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            extra_count, 4,
            "0 (A) + 4 (B) = 4 ExtraBolt entities, got {extra_count}",
        );
    }
}
