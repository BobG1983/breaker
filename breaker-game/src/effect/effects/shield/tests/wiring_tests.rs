use super::helpers::*;

// ── Section D: register() wires tick system ─────────────────────────

// Behavior 14: register() adds tick_shield_wall_timer to FixedUpdate

#[test]
fn register_adds_tick_system_to_fixed_update() {
    let mut app = test_app();
    register(&mut app);

    // Spawn a wall with nearly-expired timer
    let wall = app
        .world_mut()
        .spawn((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(0.001, TimerMode::Once)),
        ))
        .id();

    tick(&mut app);

    // If register() wired the system, the wall should be despawned
    assert!(
        app.world().get_entity(wall).is_err(),
        "register should add tick system that despawns expired ShieldWall entities"
    );
}

// ── Section E: EffectKind::Shield enum change ────────────────────────

// Behavior 15: EffectKind::Shield has duration field (f32), not stacks

#[test]
fn effect_kind_shield_has_duration_field() {
    use crate::effect::EffectKind;
    // Compile-time verification: if Shield still uses stacks: u32, this won't compile
    let shield = EffectKind::Shield { duration: 5.0 };
    drop(shield);
}

#[test]
fn effect_kind_shield_zero_duration_compiles() {
    use crate::effect::EffectKind;
    // Edge case: zero duration is valid at the enum level
    let shield = EffectKind::Shield { duration: 0.0 };
    drop(shield);
}

// Behavior 16: EffectKind::Shield fire() dispatches end-to-end

#[test]
fn effect_kind_shield_fire_dispatches_end_to_end() {
    use crate::effect::EffectKind;

    let mut world = test_world();
    let entity = world.spawn_empty().id();

    EffectKind::Shield { duration: 5.0 }.fire(entity, "parry", &mut world);

    let count = world
        .query_filtered::<Entity, With<ShieldWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        count, 1,
        "EffectKind::Shield dispatch should spawn a ShieldWall entity"
    );

    let timers: Vec<&ShieldWallTimer> = world
        .query_filtered::<&ShieldWallTimer, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(timers.len(), 1, "ShieldWall should have ShieldWallTimer");
    assert!(
        (timers[0].0.remaining_secs() - 5.0).abs() < 0.01,
        "timer should have ~5.0 remaining after dispatch, got {:.4}",
        timers[0].0.remaining_secs()
    );
}

#[test]
fn effect_kind_shield_fire_dispatches_short_duration() {
    // Edge case: duration 0.5
    use crate::effect::EffectKind;

    let mut world = test_world();
    let entity = world.spawn_empty().id();

    EffectKind::Shield { duration: 0.5 }.fire(entity, "parry", &mut world);

    let timers: Vec<&ShieldWallTimer> = world
        .query_filtered::<&ShieldWallTimer, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(timers.len(), 1);
    assert!(
        (timers[0].0.remaining_secs() - 0.5).abs() < 0.01,
        "timer should have ~0.5 remaining, got {:.4}",
        timers[0].0.remaining_secs()
    );
}
