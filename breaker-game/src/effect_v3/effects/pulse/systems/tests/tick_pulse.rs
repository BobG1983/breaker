use std::{collections::HashSet, time::Duration};

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::*;
use crate::{
    cells::components::Cell,
    effect_v3::{components::EffectSourceChip, effects::pulse::components::*},
    shared::{
        death_pipeline::DamageDealt,
        test_utils::{MessageCollector, tick},
    },
};

// ── A. apply_pulse_damage — damage emission ────────────────────────────

// #1
#[test]
fn pulse_ring_damages_cell_strictly_inside_radius() {
    let mut app = damage_test_app();

    let cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
    let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "expected exactly 1 DamageDealt<Cell>");
    assert_eq!(msgs.0[0].target, cell);
    assert_eq!(msgs.0[0].dealer, Some(ring));
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "expected amount == 10.0, got {}",
        msgs.0[0].amount,
    );
    assert_eq!(msgs.0[0].source_chip, None);

    let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
    assert!(
        damaged.0.contains(&cell),
        "PulseRingDamaged should track the damaged cell",
    );
}

// #2
#[test]
fn pulse_ring_does_not_damage_cell_outside_radius() {
    let mut app = damage_test_app();

    let _cell = spawn_cell(&mut app, Vec2::new(100.0, 0.0));
    let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        0,
        "expected 0 DamageDealt<Cell> for out-of-range cell",
    );

    let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
    assert_eq!(
        damaged.0.len(),
        0,
        "PulseRingDamaged must stay empty when no cells are in range",
    );
}

// #3
#[test]
fn pulse_ring_includes_cell_exactly_on_boundary() {
    let mut app = damage_test_app();

    // 3-4-5 right triangle: sqrt(30^2 + 40^2) == 50.0 exactly.
    let cell = spawn_cell(&mut app, Vec2::new(30.0, 40.0));
    let _ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        1,
        "boundary distance (== radius) must damage the cell",
    );
    assert_eq!(msgs.0[0].target, cell);
}

// #4
#[test]
fn pulse_ring_does_not_redamage_previously_damaged_cell() {
    let mut app = damage_test_app();

    let cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
    let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

    tick(&mut app);
    {
        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "first tick should emit one DamageDealt<Cell>",
        );
    }

    // The `First`-schedule auto-clear empties the collector at the start
    // of the second update, so dedup produces zero messages, not stale ones.
    tick(&mut app);
    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        0,
        "second tick must not re-damage a cell already in PulseRingDamaged",
    );

    let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
    assert!(
        damaged.0.contains(&cell),
        "PulseRingDamaged must still contain the cell after the second tick",
    );
}

// #5
#[test]
fn pulse_ring_does_not_damage_dead_cell() {
    let mut app = damage_test_app();

    let _dead_cell = spawn_dead_cell(&mut app, Vec2::new(20.0, 0.0));
    let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        0,
        "Dead cells must be filtered out by the alive-cell query",
    );

    let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
    assert_eq!(
        damaged.0.len(),
        0,
        "PulseRingDamaged must remain empty — dead cells must not be inserted",
    );
}

// #6
#[test]
fn pulse_ring_damage_multiplies_base_damage_by_multiplier() {
    let mut app = damage_test_app();

    let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
    let _ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 2.5);

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1);
    assert!(
        (msgs.0[0].amount - 25.0).abs() < f32::EPSILON,
        "expected amount == 10.0 * 2.5 == 25.0, got {}",
        msgs.0[0].amount,
    );
}

// #7
#[test]
fn pulse_ring_with_zero_cells_emits_no_damage_and_does_not_panic() {
    let mut app = damage_test_app();

    // NO cell spawns.
    let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        0,
        "empty-world pulse ring must emit no damage",
    );

    let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
    assert_eq!(damaged.0.len(), 0);
}

// #8
#[test]
fn pulse_ring_damages_all_cells_within_radius_in_one_tick() {
    let mut app = damage_test_app();

    let cell_a = spawn_cell(&mut app, Vec2::new(10.0, 0.0));
    let cell_b = spawn_cell(&mut app, Vec2::new(50.0, 0.0));
    let cell_c = spawn_cell(&mut app, Vec2::new(90.0, 0.0));
    let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 100.0, 5.0, 1.0);

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 3, "expected three damage messages");

    for msg in &msgs.0 {
        assert!(
            (msg.amount - 5.0).abs() < f32::EPSILON,
            "every message should carry amount == 5.0, got {}",
            msg.amount,
        );
    }

    let targets: HashSet<Entity> = msgs.0.iter().map(|m| m.target).collect();
    assert_eq!(targets, HashSet::from([cell_a, cell_b, cell_c]));

    let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
    assert!(damaged.0.contains(&cell_a));
    assert!(damaged.0.contains(&cell_b));
    assert!(damaged.0.contains(&cell_c));
}

// #9
#[test]
fn two_independent_pulse_rings_have_independent_damaged_sets() {
    let mut app = damage_test_app();

    let cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
    let ring_a = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);
    let ring_b = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        2,
        "each independent pulse ring must emit its own DamageDealt<Cell>",
    );

    for msg in &msgs.0 {
        assert_eq!(msg.target, cell);
    }

    let dealers: HashSet<Option<Entity>> = msgs.0.iter().map(|m| m.dealer).collect();
    assert_eq!(dealers, HashSet::from([Some(ring_a), Some(ring_b)]));

    assert!(
        app.world()
            .get::<PulseRingDamaged>(ring_a)
            .unwrap()
            .0
            .contains(&cell),
    );
    assert!(
        app.world()
            .get::<PulseRingDamaged>(ring_b)
            .unwrap()
            .0
            .contains(&cell),
    );
}

// ── B. apply_pulse_damage — source_chip propagation ────────────────────

// #10
#[test]
fn pulse_ring_propagates_some_source_chip_in_damage_dealt() {
    let mut app = damage_test_app();

    let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
    app.world_mut().spawn((
        Position2D(Vec2::ZERO),
        PulseRingRadius(50.0),
        PulseRingBaseDamage(10.0),
        PulseRingDamageMultiplier(1.0),
        PulseRingDamaged(HashSet::new()),
        EffectSourceChip(Some("storm_chip".to_string())),
    ));

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
    assert_eq!(
        msgs.0[0].source_chip,
        Some("storm_chip".to_string()),
        "DamageDealt should carry source_chip from EffectSourceChip, got {:?}",
        msgs.0[0].source_chip,
    );
}

// #11
#[test]
fn pulse_ring_propagates_none_source_chip() {
    let mut app = damage_test_app();

    let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
    app.world_mut().spawn((
        Position2D(Vec2::ZERO),
        PulseRingRadius(50.0),
        PulseRingBaseDamage(10.0),
        PulseRingDamageMultiplier(1.0),
        PulseRingDamaged(HashSet::new()),
        EffectSourceChip(None),
    ));

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
    assert_eq!(
        msgs.0[0].source_chip, None,
        "EffectSourceChip(None) must survive unchanged, got {:?}",
        msgs.0[0].source_chip,
    );
}

// #12
#[test]
fn pulse_ring_without_effect_source_chip_component_writes_none() {
    let mut app = damage_test_app();

    let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
    // Deliberately NO EffectSourceChip component.
    let _ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        1,
        "missing-component pulse ring must still match the query (Option<&EffectSourceChip>)",
    );
    assert_eq!(msgs.0[0].source_chip, None);
}

// ── C. tick_pulse_ring — radius expansion ──────────────────────────────

// #13
#[test]
fn tick_pulse_ring_expands_radius_by_speed_times_dt() {
    let mut app = tick_test_app();

    let ring = app
        .world_mut()
        .spawn((PulseRingRadius(0.0), PulseRingSpeed(200.0)))
        .id();

    tick_with_dt(&mut app, Duration::from_millis(250));

    let radius = app.world().get::<PulseRingRadius>(ring).unwrap().0;
    assert!(
        (radius - 50.0).abs() < f32::EPSILON,
        "expected radius == 200.0 * 0.25 == 50.0, got {radius}",
    );
}

// #14
#[test]
fn tick_pulse_ring_accumulates_across_two_ticks() {
    let mut app = tick_test_app();

    let ring = app
        .world_mut()
        .spawn((PulseRingRadius(10.0), PulseRingSpeed(200.0)))
        .id();

    tick_with_dt(&mut app, Duration::from_millis(250));
    tick_with_dt(&mut app, Duration::from_millis(250));

    let radius = app.world().get::<PulseRingRadius>(ring).unwrap().0;
    // 10.0 + 2 * (200.0 * 0.25) == 10.0 + 100.0 == 110.0
    assert!(
        (radius - 110.0).abs() < f32::EPSILON,
        "expected radius == 110.0 (10.0 + 2 * 50.0), got {radius}",
    );
}

// #15
#[test]
fn tick_pulse_ring_with_zero_speed_leaves_radius_unchanged() {
    let mut app = tick_test_app();

    let ring = app
        .world_mut()
        .spawn((PulseRingRadius(42.0), PulseRingSpeed(0.0)))
        .id();

    tick_with_dt(&mut app, Duration::from_millis(250));

    let radius = app.world().get::<PulseRingRadius>(ring).unwrap().0;
    assert!(
        (radius - 42.0).abs() < f32::EPSILON,
        "zero-speed pulse ring must not expand, got {radius}",
    );
}

// ── D. despawn_finished_pulse_ring — termination ───────────────────────

// #16
#[test]
fn despawn_finished_pulse_ring_despawns_when_radius_equals_max() {
    let mut app = despawn_test_app();

    app.world_mut()
        .spawn((PulseRingRadius(100.0), PulseRingMaxRadius(100.0)));

    tick(&mut app);

    let count = app
        .world_mut()
        .query::<&PulseRingRadius>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "radius == max_radius must despawn (boundary of >=)",
    );
}

// #17
#[test]
fn despawn_finished_pulse_ring_despawns_when_radius_greater_than_max() {
    let mut app = despawn_test_app();

    app.world_mut()
        .spawn((PulseRingRadius(150.0), PulseRingMaxRadius(100.0)));

    tick(&mut app);

    let count = app
        .world_mut()
        .query::<&PulseRingRadius>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0, "radius > max_radius must despawn");
}

// #18
#[test]
fn despawn_finished_pulse_ring_does_not_despawn_when_radius_less_than_max() {
    let mut app = despawn_test_app();

    app.world_mut()
        .spawn((PulseRingRadius(50.0), PulseRingMaxRadius(100.0)));

    tick(&mut app);

    let surviving: Vec<(f32, f32)> = app
        .world_mut()
        .query::<(&PulseRingRadius, &PulseRingMaxRadius)>()
        .iter(app.world())
        .map(|(r, m)| (r.0, m.0))
        .collect();
    assert_eq!(
        surviving.len(),
        1,
        "expanding pulse ring (radius < max) must not despawn",
    );
    assert!((surviving[0].0 - 50.0).abs() < f32::EPSILON);
    assert!((surviving[0].1 - 100.0).abs() < f32::EPSILON);
}
