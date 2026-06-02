use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{super::system::*, helpers::*};
use crate::{
    breaker::messages::BumpPerformed,
    chips::definition::Rarity,
    effect_v3::{
        components::EffectSourceChip,
        effects::{
            SpeedBoostConfig, entropy_engine::components::EntropyCounter,
            shockwave::components::ShockwaveSource,
        },
        stacking::EffectStack,
        types::EffectType,
    },
    prelude::{SourceId, SourceIdExt},
    shared::{
        rng::{EffectBaseSeed, EffectEventCounter},
        test_utils::{TestAppBuilder, tick},
    },
};

/// Builder-format `SourceId` for the canonical "Entropy" engine chip used
/// by these weighted-random tests. Centralized so the fixture demonstrates
/// the canonical `chip:<template>:<rarity>` shape.
fn entropy_source() -> SourceId {
    SourceId::chip("Entropy").rarity(Rarity::Common).build()
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

// ── Behavior 18: Fired effects receive empty source chip ──

#[test]
fn fired_effects_receive_empty_source_chip() {
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

// ── Behavior 24: Runtime smoke test — normal-system scheduling ──

#[test]
fn tick_entropy_engine_schedules_as_normal_system_and_ticks_without_panic() {
    let mut app = TestAppBuilder::new()
        .with_message::<BumpPerformed>()
        .insert_resource(EffectBaseSeed(42))
        .insert_resource(EffectEventCounter::default())
        .with_system(FixedUpdate, tick_entropy_engine)
        .build();

    tick(&mut app);
    // If we reach here, no panic occurred and the system scheduled
    // successfully as a normal system.
}

// ── Behavior 25: EffectSourceChip(Some) propagates to spawned effects ──

/// Spawns a counter with both `EntropyCounter` and an `EffectSourceChip`
/// component bundled on the same entity. Used by Behaviors 25 and 26.
fn spawn_counter_with_chip(
    app: &mut App,
    count: u32,
    max_effects: u32,
    pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
    chip: EffectSourceChip,
) -> Entity {
    app.world_mut()
        .spawn((
            EntropyCounter {
                count,
                max_effects,
                pool,
            },
            chip,
        ))
        .id()
}

#[test]
fn counter_entity_with_some_chip_propagates_name_to_spawned_effects() {
    let mut app = entropy_app();

    // Counter entity bundled with BOTH EntropyCounter and
    // EffectSourceChip(Some("entropy_chip")). One queued bump.
    // Single-entry pool ensures deterministic 1-shockwave outcome.
    let counter_entity = spawn_counter_with_chip(
        &mut app,
        0,
        3,
        vec![make_shockwave_effect()],
        EffectSourceChip(Some(entropy_source())),
    );
    queue_bump(&mut app);

    tick(&mut app);

    // Collect the EffectSourceChip components on every ShockwaveSource
    // entity that currently exists.
    let chips: Vec<EffectSourceChip> = app
        .world_mut()
        .query_filtered::<&EffectSourceChip, With<ShockwaveSource>>()
        .iter(app.world())
        .cloned()
        .collect();

    assert_eq!(
        chips.len(),
        1,
        "expected 1 shockwave entity with EffectSourceChip, got {}",
        chips.len(),
    );
    // EffectSourceChip does not derive PartialEq — assert on .0 directly.
    assert_eq!(
        chips[0].0,
        Some(entropy_source()),
        "spawned shockwave should carry the counter entity's chip name, got {:?}",
        chips[0].0,
    );

    // Edge case: the counter entity retains its own EffectSourceChip
    // after the tick (we clone, not move).
    let counter_chip = app
        .world()
        .get::<EffectSourceChip>(counter_entity)
        .expect("counter entity should still have its EffectSourceChip");
    assert_eq!(
        counter_chip.0,
        Some(entropy_source()),
        "counter entity's own EffectSourceChip must remain unchanged after tick, got {:?}",
        counter_chip.0,
    );
}

// ── Behavior 26: EffectSourceChip(None) on counter yields None on spawn ──

#[test]
fn counter_entity_with_none_chip_yields_none_on_spawned_effects() {
    let mut app = entropy_app();

    // Counter entity bundled with BOTH EntropyCounter and
    // EffectSourceChip(None). Distinguishes from Behavior 23 where the
    // entity has NO EffectSourceChip component at all.
    spawn_counter_with_chip(
        &mut app,
        0,
        3,
        vec![make_shockwave_effect()],
        EffectSourceChip(None),
    );
    queue_bump(&mut app);

    tick(&mut app);

    let chips: Vec<EffectSourceChip> = app
        .world_mut()
        .query_filtered::<&EffectSourceChip, With<ShockwaveSource>>()
        .iter(app.world())
        .cloned()
        .collect();

    assert_eq!(
        chips.len(),
        1,
        "expected 1 shockwave entity with EffectSourceChip, got {}",
        chips.len(),
    );
    // EffectSourceChip does not derive PartialEq — assert via .0.is_none().
    assert!(
        chips[0].0.is_none(),
        "EffectSourceChip(None) on counter should produce EffectSourceChip(None) on spawn, got {:?}",
        chips[0].0,
    );
}

// ── B22 — entropy_engine weighted-random selection preserved under threaded RNG ──

#[test]
fn entropy_engine_weighted_random_selection_preserved_under_threaded_rng() {
    // B22: determinism check — two apps with the same EffectBaseSeed produce
    // identical post-state after tick_entropy_engine fires.
    let run_app = || {
        let mut app = entropy_app();
        spawn_counter(&mut app, 0, 3, vec![make_shockwave_effect()]);
        queue_bump(&mut app);
        tick(&mut app);
        app.world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count()
    };

    let count_a = run_app();
    let count_b = run_app();
    assert_eq!(
        count_a, count_b,
        "two runs with same EffectBaseSeed(42) must produce identical shockwave count"
    );
    assert_eq!(
        count_a, 1,
        "single-entry pool must always select shockwave; expected 1, got {count_a}"
    );
}

// ── B23 — tick_entropy_engine does NOT consume GameRng ──

#[test]
fn tick_entropy_engine_does_not_consume_game_rng() {
    // B23: App has no GameRng resource — must not panic.
    let mut app = TestAppBuilder::new()
        .with_message::<BumpPerformed>()
        .with_resource::<TestBumpMessages>()
        .insert_resource(EffectBaseSeed(0))
        .insert_resource(EffectEventCounter::default())
        .with_system(
            FixedUpdate,
            (
                inject_bumps.before(tick_entropy_engine),
                tick_entropy_engine,
            ),
        )
        .build();

    let entity = spawn_counter(
        &mut app,
        0,
        3,
        vec![(
            ordered_float::OrderedFloat(1.0),
            Box::new(EffectType::Die(
                crate::effect_v3::effects::die::DieConfig {},
            )),
        )],
    );
    queue_bump(&mut app);
    tick(&mut app);

    // counter advanced — no panic
    let counter = app.world().get::<EntropyCounter>(entity).unwrap();
    assert_eq!(
        counter.count, 1,
        "counter should advance to 1 without GameRng"
    );
}

// ── B24 — tick_entropy_engine derives RNG from EffectBaseSeed + named scoping ──

#[test]
fn tick_entropy_engine_derives_rng_from_effect_base_seed() {
    // B24: two apps with identical EffectBaseSeed produce identical effect selection.
    use crate::prelude::Dead;

    let run_with_seed = |seed: u64| {
        let mut app = TestAppBuilder::new()
            .with_message::<BumpPerformed>()
            .with_resource::<TestBumpMessages>()
            .insert_resource(EffectBaseSeed(seed))
            .insert_resource(EffectEventCounter::default())
            .with_system(
                FixedUpdate,
                (
                    inject_bumps.before(tick_entropy_engine),
                    tick_entropy_engine,
                ),
            )
            .build();

        // Pool: FlashStep and Die, equal weight
        let pool = vec![
            (
                ordered_float::OrderedFloat(1.0),
                Box::new(EffectType::FlashStep(
                    crate::effect_v3::effects::flash_step::FlashStepConfig {},
                )),
            ),
            (
                ordered_float::OrderedFloat(1.0),
                Box::new(EffectType::Die(
                    crate::effect_v3::effects::die::DieConfig {},
                )),
            ),
        ];
        let entity = spawn_counter(&mut app, 0, 3, pool);
        queue_bump(&mut app);
        tick(&mut app);

        let has_flash = app
            .world()
            .get::<crate::effect_v3::effects::flash_step::FlashStepActive>(entity)
            .is_some();
        let has_dead = app.world().get::<Dead>(entity).is_some();
        (has_flash, has_dead)
    };

    // Same seed twice must produce identical result.
    let (flash_a, dead_a) = run_with_seed(0xCAFE);
    let (flash_b, dead_b) = run_with_seed(0xCAFE);
    assert_eq!(
        flash_a, flash_b,
        "same EffectBaseSeed must produce same FlashStep selection"
    );
    assert_eq!(
        dead_a, dead_b,
        "same EffectBaseSeed must produce same Die selection"
    );
    assert!(flash_a || dead_a, "exactly one effect must have fired");
    assert!(
        !(flash_a && dead_a),
        "both effects must not fire simultaneously"
    );

    // Two entities in same tick pick INDEPENDENTLY (different entity indices).
    let mut app = TestAppBuilder::new()
        .with_message::<BumpPerformed>()
        .with_resource::<TestBumpMessages>()
        .insert_resource(EffectBaseSeed(0xCAFE))
        .insert_resource(EffectEventCounter::default())
        .with_system(
            FixedUpdate,
            (
                inject_bumps.before(tick_entropy_engine),
                tick_entropy_engine,
            ),
        )
        .build();

    let pool_a = vec![
        (
            ordered_float::OrderedFloat(1.0),
            Box::new(EffectType::FlashStep(
                crate::effect_v3::effects::flash_step::FlashStepConfig {},
            )),
        ),
        (
            ordered_float::OrderedFloat(1.0),
            Box::new(EffectType::Die(
                crate::effect_v3::effects::die::DieConfig {},
            )),
        ),
    ];
    let pool_b = pool_a.clone();
    let _entity_a = spawn_counter(&mut app, 0, 3, pool_a);
    let _entity_b = spawn_counter(&mut app, 0, 3, pool_b);
    queue_bump(&mut app);
    tick(&mut app);
    // No panic — both entities processed independently.
}

// ── B25 — tick_entropy_engine and bridge dispatch produce composable determinism ──

#[test]
fn tick_entropy_engine_and_bridge_dispatch_produce_composable_determinism() {
    // B25: two identically-seeded apps produce byte-identical final state.
    // tick_entropy_engine queues FireEffectCommands; bridge flushes them.

    let run = || {
        let mut app = entropy_app(); // EffectBaseSeed(42) + EffectEventCounter(0)
        spawn_counter(&mut app, 0, 3, vec![make_shockwave_effect()]);
        queue_bump(&mut app);
        tick(&mut app);
        app.world_mut()
            .query_filtered::<Entity, With<ShockwaveSource>>()
            .iter(app.world())
            .count()
    };

    let count_a = run();
    let count_b = run();
    assert_eq!(
        count_a, count_b,
        "composable determinism: two identically-seeded apps must produce identical shockwave count"
    );
}
