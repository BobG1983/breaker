use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    bolt::components::ExtraBolt,
    effect_v3::effects::circuit_breaker::components::CircuitBreakerCounter,
    shared::test_utils::tick,
};

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
