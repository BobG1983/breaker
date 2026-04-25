use std::time::Duration;

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    cells::behaviors::survival::salvo::components::Salvo,
    mutators::hazards::{
        definition::HazardKind, renewal::system::RenewalConfig, resources::ActiveHazards,
    },
    prelude::*,
};

// ══════════════════════════════════════════════════════════════════════
// Group F — cross-cutting
// ══════════════════════════════════════════════════════════════════════

// Behavior 23 — heal amount == hp_per_interval, not scaled by dt or stacks
#[test]
fn heal_amount_is_hp_per_interval_0_25() {
    let mut app = test_app_playing();
    app.world_mut().insert_resource(VolatilityConfig {
        hp_per_interval: 0.25,
        interval_secs:   5.0,
        max_multiplier:  2.0,
    });
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1);
    assert!((heals[0].amount - 0.25).abs() < f32::EPSILON);
    assert_eq!(heals[0].cap, HealCap::Max);
}

// Behavior 23 edge — hp_per_interval = 3.0
#[test]
fn heal_amount_is_hp_per_interval_3_0() {
    let mut app = test_app_playing();
    app.world_mut().insert_resource(VolatilityConfig {
        hp_per_interval: 3.0,
        interval_secs:   5.0,
        max_multiplier:  2.0,
    });
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1);
    assert!((heals[0].amount - 3.0).abs() < f32::EPSILON);
    assert_eq!(heals[0].cap, HealCap::Max);
}

// Behavior 23 edge — stacks change interval, not amount
#[test]
fn stacks_change_interval_not_amount() {
    let mut app = test_app_playing();
    app.world_mut().insert_resource(VolatilityConfig {
        hp_per_interval: 3.0,
        interval_secs:   5.0,
        max_multiplier:  2.0,
    });
    add_volatility_stacks(&mut app, 5);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    // Effective interval at stack=5: 5.0 / (1 + 0.25*4) = 5.0 / 2.0 = 2.5s.
    tick_with_dt(&mut app, Duration::from_secs_f32(2.5));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1);
    assert!((heals[0].amount - 3.0).abs() < f32::EPSILON);
    assert_eq!(heals[0].cap, HealCap::Max);
}

// Behavior 24 — healer == None
#[test]
fn heal_healer_is_none() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1);
    assert_eq!(heals[0].healer, None);
}

// Behavior 25 — message lands in HealDealt<Cell>, not other T's
#[test]
fn heal_lands_in_cell_queue_not_other_monomorphizations() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .with_message_capture::<HealDealt<Cell>>()
        .with_message_capture::<HealDealt<Bolt>>()
        .with_message_capture::<HealDealt<Breaker>>()
        .with_message_capture::<HealDealt<Wall>>()
        .with_message_capture::<HealDealt<Salvo>>()
        .build();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    assert_eq!(
        app.world()
            .resource::<MessageCollector<HealDealt<Cell>>>()
            .0
            .len(),
        1
    );
    assert!(
        app.world()
            .resource::<MessageCollector<HealDealt<Bolt>>>()
            .0
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<MessageCollector<HealDealt<Breaker>>>()
            .0
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<MessageCollector<HealDealt<Wall>>>()
            .0
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<MessageCollector<HealDealt<Salvo>>>()
            .0
            .is_empty()
    );
}

// Behavior 26 — Volatility + Renewal active; Volatility's source tag is preserved
#[test]
fn two_hazards_active_volatility_source_tag_preserved() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    app.world_mut().insert_resource(RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 0.2,
    });
    add_volatility_stacks(&mut app, 1);
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Renewal);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let heals: Vec<_> = heals_for(&app, cell)
        .into_iter()
        .filter(|m| m.source == Some(SourceId::hazard(HazardKind::Volatility).build()))
        .collect();
    assert_eq!(heals.len(), 1);
    assert_eq!(heals[0].cap, HealCap::Max);
    assert_eq!(
        heals[0].source,
        Some(SourceId::hazard(HazardKind::Volatility).build())
    );
}

// Behavior 27 — timer advances when at cap, pre-send gate blocks only emit
#[test]
fn timer_still_advances_when_at_cap() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(2.5));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        (timer.elapsed - 2.5).abs() < 1e-5,
        "timer.elapsed should be ~2.5, got {}",
        timer.elapsed
    );
}

// Behavior 27 edge — second 2.5s tick from elapsed≈2.5 rolls over, still 0 messages
#[test]
fn at_cap_second_tick_crosses_interval_still_zero_emissions() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(2.5));
    tick_with_dt(&mut app, Duration::from_secs_f32(2.5));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        timer.elapsed.abs() < 1e-5,
        "timer.elapsed should roll over to ~0.0, got {}",
        timer.elapsed
    );
}

// Behavior 27 edge — at cap, 10.0s (two intervals) still emits zero
#[test]
fn at_cap_two_intervals_emit_zero_and_roll_over() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(10.0));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(timer.elapsed.abs() < 1e-5);
}

// Behavior 28 — Hp.max NOT restored when hazard deactivates
#[test]
fn hp_max_not_restored_on_deactivation() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 10.0, 10.0, None);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    assert_eq!(app.world().get::<Hp>(cell).unwrap().max, Some(20.0));

    // Deactivate via backdoor.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .force_insert_entry(HazardKind::Volatility, 0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "Hp.max must remain lifted after deactivation (no restore)"
    );
    assert_eq!(heal_collector_len(&app), 0);
}

// Behavior 28 edge — pre-existing game max (Some(15.0)) lifted to Some(20.0), not restored
#[test]
fn hp_max_not_restored_even_when_original_was_some() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 10.0, 10.0, Some(15.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    assert_eq!(
        app.world().get::<Hp>(cell).unwrap().max,
        Some(20.0),
        "Hp.max should be lifted from 15.0 to 20.0 on first tick"
    );

    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .force_insert_entry(HazardKind::Volatility, 0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "Hp.max must NOT be restored to Some(15.0) after deactivation"
    );
}
