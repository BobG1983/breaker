use std::time::Duration;

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{mutators::hazards::definition::HazardKind, prelude::*};

// ══════════════════════════════════════════════════════════════════════
// Group A — volatility_grow_cells interval / heal semantics
// ══════════════════════════════════════════════════════════════════════

// Behavior 1
#[test]
fn emits_heal_dealt_after_one_interval_stack_1() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1, "expected exactly one HealDealt<Cell>");
    let msg = &heals[0];
    assert_eq!(msg.target, cell);
    assert!(
        (msg.amount - 1.0).abs() < f32::EPSILON,
        "amount should be 1.0, got {}",
        msg.amount
    );
    assert!(matches!(msg.cap, HealCap::Max));
    assert_eq!(msg.healer, None);
    assert_eq!(
        msg.source,
        Some(SourceId::hazard(HazardKind::Volatility).build())
    );

    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        timer.elapsed.abs() < 1e-5,
        "timer.elapsed should roll over to ~0.0, got {}",
        timer.elapsed
    );
}

// Behavior 1 edge — 4.999s tick
#[test]
fn no_heal_before_interval_crossed() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(4.999));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        (timer.elapsed - 4.999).abs() < 1e-3,
        "timer.elapsed should be ~4.999, got {}",
        timer.elapsed
    );
}

// Behavior 1 edge — five 1.0s ticks accumulate to one heal
#[test]
fn five_1s_ticks_accumulate_to_one_heal() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    let mut emitted = 0usize;
    for _ in 0..5 {
        tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
        emitted += heals_for(&app, cell).len();
    }

    assert_eq!(
        emitted, 1,
        "exactly one HealDealt<Cell> should be emitted across five 1s ticks"
    );
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        timer.elapsed.abs() < 1e-5,
        "timer.elapsed should roll over to ~0.0 after tick 5, got {}",
        timer.elapsed
    );
}

// Behavior 2 — HealCap::Max variant
#[test]
fn heal_cap_variant_is_max() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1);
    assert_eq!(heals[0].cap, HealCap::Max);
}

// Behavior 2 edge — source string exact match
#[test]
fn heal_source_is_hazard_volatility() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1);
    assert_eq!(
        heals[0].source,
        Some(SourceId::hazard(HazardKind::Volatility).build())
    );
}

// Behavior 3 — no heal at cap; timer still rolls over
#[test]
fn no_heal_emitted_at_max_cap_but_timer_rolls_over() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        timer.elapsed.abs() < 1e-5,
        "timer.elapsed should roll over to ~0.0 even at cap, got {}",
        timer.elapsed
    );
}

// Behavior 3 edge — slightly above cap, no message
#[test]
fn no_heal_emitted_slightly_above_cap() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    spawn_cell_with_timer_max(&mut app, 20.000_001, 10.0, Some(20.0), 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    assert_eq!(heal_collector_len(&app), 0);
}

// Behavior 3 edge — just under cap, exactly one heal
#[test]
fn heal_emitted_just_under_cap() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer_max(&mut app, 19.999, 10.0, Some(20.0), 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1);
    assert!((heals[0].amount - 1.0).abs() < f32::EPSILON);
    assert_eq!(heals[0].cap, HealCap::Max);
}

// Behavior 3 edge — two sequential ticks at cap, no emissions either tick
#[test]
fn at_cap_two_ticks_emit_zero_heals() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));
    assert_eq!(heal_collector_len(&app), 0, "after tick 1 at cap");

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));
    assert_eq!(heal_collector_len(&app), 0, "after tick 2 at cap");
}

// Behavior 3 edge — ten sequential 5s ticks baseline, ten emissions
#[test]
fn ten_sequential_ticks_under_cap_emit_ten_heals() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    let mut total = 0usize;
    for _ in 0..10 {
        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));
        let heals = heals_for(&app, cell);
        for msg in &heals {
            assert!((msg.amount - 1.0).abs() < f32::EPSILON);
            assert_eq!(msg.cap, HealCap::Max);
        }
        total += heals.len();
    }
    assert_eq!(
        total, 10,
        "expected exactly 10 HealDealt<Cell> across 10 ticks"
    );
}

// Behavior 4 — two cells, two distinct heals
#[test]
fn heal_target_matches_cell_entity_two_cells() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let a = spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9);
    let b = spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.2));

    let collector = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(collector.len(), 2, "expected exactly two HealDealt<Cell>");
    for msg in collector {
        assert!((msg.amount - 1.0).abs() < f32::EPSILON);
        assert_eq!(msg.cap, HealCap::Max);
    }
    let targets: std::collections::HashSet<Entity> = collector.iter().map(|m| m.target).collect();
    let expected: std::collections::HashSet<Entity> = [a, b].into_iter().collect();
    assert_eq!(targets, expected);
}

// Behavior 4 edge — four cells
#[test]
fn heal_target_matches_cell_entity_four_cells() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cells = [
        spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9),
        spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9),
        spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9),
        spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9),
    ];

    tick_with_dt(&mut app, Duration::from_secs_f32(0.2));

    let collector = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(collector.len(), 4);
    let targets: std::collections::HashSet<Entity> = collector.iter().map(|m| m.target).collect();
    assert_eq!(targets.len(), 4, "each target should be distinct");
    for c in cells {
        assert!(targets.contains(&c));
    }
}

// Behavior 5 — dead cell (current == 0)
#[test]
fn dead_cell_receives_no_heal() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 0.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    assert!(heals_for(&app, cell).is_empty());
}

// Behavior 5 edge — negative current (post-overkill)
#[test]
fn dead_cell_negative_current_receives_no_heal() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, -1.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    assert!(heals_for(&app, cell).is_empty());
}

// Behavior 5 edge — mixed batch (dead + living)
#[test]
fn mixed_dead_and_living_only_living_gets_heal() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let dead = spawn_cell_with_timer(&mut app, 0.0, 10.0, 0.0);
    let living = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    assert!(heals_for(&app, dead).is_empty());
    let living_heals = heals_for(&app, living);
    assert_eq!(living_heals.len(), 1);
    assert_eq!(living_heals[0].target, living);
}

// ── B36a: source matches builder-produced hazard:volatility ──

#[test]
fn volatility_heal_source_equals_builder() {
    use crate::{mutators::hazards::definition::HazardKind, prelude::SourceIdExt};

    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let heals = heals_for(&app, cell);
    assert!(!heals.is_empty());
    let expected = SourceId::hazard(HazardKind::Volatility).build();
    for h in &heals {
        assert_eq!(h.source.as_ref(), Some(&expected));
    }
}
