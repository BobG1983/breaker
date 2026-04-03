use super::helpers::*;

// ── Section C: tick_shield_wall_timer system ─────────────────────────

// Behavior 10: Timer decrements by delta time each tick

#[test]
fn timer_decrements_by_delta_time() {
    let mut app = test_app();
    register(&mut app);
    let wall = app
        .world_mut()
        .spawn((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(5.0, TimerMode::Once)),
        ))
        .id();

    tick(&mut app);

    let timer = app.world().get::<ShieldWallTimer>(wall).unwrap();
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected_remaining = 5.0 - dt;
    assert!(
        (timer.0.remaining_secs() - expected_remaining).abs() < 0.001,
        "timer should have ~{expected_remaining:.6} remaining after one tick, got {:.6}",
        timer.0.remaining_secs()
    );
    assert!(
        !timer.0.is_finished(),
        "timer should not be finished after one tick"
    );
}

#[test]
fn timer_near_zero_does_not_finish_prematurely() {
    // Edge case: timer at ~0.016 remaining should not finish after one ~0.015625s tick
    let mut app = test_app();
    register(&mut app);
    let wall = app
        .world_mut()
        .spawn((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(0.016, TimerMode::Once)),
        ))
        .id();

    tick(&mut app);

    let timer = app.world().get::<ShieldWallTimer>(wall).unwrap();
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    // 0.016 - 0.015625 = ~0.000375 remaining, not finished
    let expected_remaining = 0.016 - dt;
    assert!(
        expected_remaining > 0.0,
        "sanity check: expected remaining should be positive"
    );
    assert!(
        !timer.0.is_finished(),
        "timer with ~{expected_remaining:.6} remaining should not be finished"
    );
    // Wall should still exist
    assert!(
        app.world().get_entity(wall).is_ok(),
        "wall should not be despawned when timer is not finished"
    );
}

// Behavior 11: Wall despawned when timer reaches zero

#[test]
fn wall_despawned_when_timer_expires() {
    let mut app = test_app();
    register(&mut app);
    let wall = app
        .world_mut()
        .spawn((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(0.001, TimerMode::Once)),
        ))
        .id();

    tick(&mut app);

    // Timer 0.001 << dt ~0.015625, so timer finishes and wall should be despawned
    assert!(
        app.world().get_entity(wall).is_err(),
        "ShieldWall entity should be despawned when timer reaches zero"
    );
}

#[test]
fn wall_despawned_when_timer_already_finished() {
    // Edge case: timer already at 0.0 remaining
    let mut app = test_app();
    register(&mut app);
    let mut timer = Timer::from_seconds(0.001, TimerMode::Once);
    timer.tick(std::time::Duration::from_secs_f32(1.0)); // force finish
    let wall = app
        .world_mut()
        .spawn((ShieldWall, ShieldWallTimer(timer)))
        .id();

    tick(&mut app);

    assert!(
        app.world().get_entity(wall).is_err(),
        "wall should be despawned when timer was already finished"
    );
}

// Behavior 12: Timer only ticks ShieldWall entities

#[test]
fn timer_only_ticks_shield_wall_entities() {
    let mut app = test_app();
    register(&mut app);

    // ShieldWall entity with timer
    app.world_mut().spawn((
        ShieldWall,
        ShieldWallTimer(Timer::from_seconds(5.0, TimerMode::Once)),
    ));

    // Non-ShieldWall entity with a ShieldWallTimer (should NOT be ticked by the system
    // since the system queries for (ShieldWall, ShieldWallTimer))
    let non_wall = app
        .world_mut()
        .spawn(ShieldWallTimer(Timer::from_seconds(5.0, TimerMode::Once)))
        .id();

    tick(&mut app);

    // Non-ShieldWall entity's timer should be unchanged (still 5.0 remaining)
    let timer = app.world().get::<ShieldWallTimer>(non_wall).unwrap();
    assert!(
        (timer.0.remaining_secs() - 5.0).abs() < 0.001,
        "non-ShieldWall entity timer should not be ticked, remaining: {:.6}",
        timer.0.remaining_secs()
    );
}

// Behavior 13: Multiple ShieldWall entities tick independently

#[test]
fn multiple_shield_walls_tick_independently() {
    let mut app = test_app();
    register(&mut app);

    let wall_a = app
        .world_mut()
        .spawn((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(5.0, TimerMode::Once)),
        ))
        .id();
    let wall_b = app
        .world_mut()
        .spawn((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(0.001, TimerMode::Once)),
        ))
        .id();

    tick(&mut app);

    // Wall A should survive (timer not finished)
    assert!(
        app.world().get_entity(wall_a).is_ok(),
        "wall_a (5.0s timer) should survive after one tick"
    );
    let timer_a = app.world().get::<ShieldWallTimer>(wall_a).unwrap();
    assert!(
        !timer_a.0.is_finished(),
        "wall_a timer should not be finished"
    );

    // Wall B should be despawned (timer expired)
    assert!(
        app.world().get_entity(wall_b).is_err(),
        "wall_b (0.001s timer) should be despawned after one tick"
    );
}
