use super::*;

#[test]
fn tick_tether_beam_damages_every_tick_no_cooldown() {
    let mut app = damage_test_app();

    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);
    spawn_test_cell(&mut app, 50.0, 0.0);

    // First tick
    tick(&mut app);

    // Second tick
    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "beam should damage cell on each tick (no cooldown), got {} messages",
        collector.0.len()
    );
}

#[test]
fn tick_tether_beam_despawns_beam_when_bolt_a_despawned() {
    let mut app = damage_test_app();

    let (bolt_a, _bolt_b, beam) =
        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Despawn bolt_a
    app.world_mut().despawn(bolt_a);

    tick(&mut app);

    assert!(
        app.world().get_entity(beam).is_err(),
        "beam entity should be despawned when bolt_a is gone"
    );
}

#[test]
fn tick_tether_beam_despawns_beam_when_bolt_b_despawned() {
    let mut app = damage_test_app();

    let (_bolt_a, bolt_b, beam) =
        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Despawn bolt_b
    app.world_mut().despawn(bolt_b);

    tick(&mut app);

    assert!(
        app.world().get_entity(beam).is_err(),
        "beam entity should be despawned when bolt_b is gone"
    );
}

#[test]
fn tick_tether_beam_despawns_beam_when_both_bolts_despawned() {
    let mut app = damage_test_app();

    let (bolt_a, bolt_b, beam) =
        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Despawn both
    app.world_mut().despawn(bolt_a);
    app.world_mut().despawn(bolt_b);

    tick(&mut app);

    assert!(
        app.world().get_entity(beam).is_err(),
        "beam entity should be despawned when both bolts are gone"
    );
}

#[test]
fn tick_tether_beam_bolt_a_survives_beam_cleanup() {
    let mut app = damage_test_app();

    let (bolt_a, bolt_b, _beam) =
        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Despawn bolt_b, keep bolt_a alive
    app.world_mut().despawn(bolt_b);

    tick(&mut app);

    assert!(
        app.world().get_entity(bolt_a).is_ok(),
        "bolt_a should still exist after beam cleanup"
    );
}

#[test]
fn multiple_tether_beams_operate_independently() {
    let mut app = damage_test_app();

    // Beam 1: (0, 0) to (100, 0), damage_mult=1.0
    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Beam 2: (200, 0) to (300, 0), damage_mult=2.0
    spawn_tether_beam(&mut app, Vec2::new(200.0, 0.0), Vec2::new(300.0, 0.0), 2.0);

    // Cell 1 near beam 1
    let cell1 = spawn_test_cell(&mut app, 50.0, 0.0);
    // Cell 2 near beam 2
    let cell2 = spawn_test_cell(&mut app, 250.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected 2 DamageCell messages (one per beam), got {}",
        collector.0.len()
    );

    // Find damage for each cell
    let cell1_damage = collector.0.iter().find(|m| m.cell == cell1);
    let cell2_damage = collector.0.iter().find(|m| m.cell == cell2);

    assert!(cell1_damage.is_some(), "cell1 should be damaged by beam 1");
    assert!(cell2_damage.is_some(), "cell2 should be damaged by beam 2");

    assert!(
        (cell1_damage.unwrap().damage - BASE_BOLT_DAMAGE * 1.0).abs() < f32::EPSILON,
        "cell1 damage should be BASE_BOLT_DAMAGE * 1.0 = 10.0"
    );
    assert!(
        (cell2_damage.unwrap().damage - BASE_BOLT_DAMAGE * 2.0).abs() < f32::EPSILON,
        "cell2 damage should be BASE_BOLT_DAMAGE * 2.0 = 20.0"
    );
}

#[test]
fn cell_midway_between_two_beams_not_reached_by_either() {
    let mut app = damage_test_app();

    // Beam 1: (0, 0) to (100, 0)
    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);
    // Beam 2: (200, 0) to (300, 0)
    spawn_tether_beam(&mut app, Vec2::new(200.0, 0.0), Vec2::new(300.0, 0.0), 2.0);

    // Cell at (150, 0) — between the two beams, not reached by either
    spawn_test_cell(&mut app, 150.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "cell at (150, 0) should not be reached by either beam"
    );
}
