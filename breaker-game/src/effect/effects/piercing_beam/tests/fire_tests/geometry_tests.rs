use super::super::*;
use crate::effect::core::EffectSourceChip;

// ── Behavior 16: fire() spawns PiercingBeamRequest with correct beam geometry ──

#[test]
fn fire_spawns_request_with_correct_upward_beam_geometry() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(0.0, 400.0))))
        .id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected one PiercingBeamRequest entity");

    let request = results[0];
    assert!(
        (request.origin.x - 0.0).abs() < f32::EPSILON,
        "origin x should be 0.0, got {}",
        request.origin.x
    );
    assert!(
        (request.origin.y - 0.0).abs() < f32::EPSILON,
        "origin y should be 0.0, got {}",
        request.origin.y
    );
    assert!(
        (request.direction.x - 0.0).abs() < 0.01,
        "direction x should be 0.0, got {}",
        request.direction.x
    );
    assert!(
        (request.direction.y - 1.0).abs() < 0.01,
        "direction y should be 1.0, got {}",
        request.direction.y
    );
    // PlayfieldConfig default: top = 300.0. From (0,0) upward, length = 300.0
    assert!(
        (request.length - 300.0).abs() < 0.01,
        "length should be 300.0 (to top boundary), got {}",
        request.length
    );
    assert!(
        (request.half_width - 10.0).abs() < f32::EPSILON,
        "half_width should be 10.0 (width/2), got {}",
        request.half_width
    );
    let expected_damage = BASE_BOLT_DAMAGE * 1.0;
    assert!(
        (request.damage - expected_damage).abs() < f32::EPSILON,
        "damage should be {}, got {}",
        expected_damage,
        request.damage
    );
}

#[test]
fn fire_entity_near_boundary_produces_short_beam() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Position2D(Vec2::new(0.0, 290.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    let request = results[0];
    // top = 300.0, entity at y=290 -> beam length = 10.0
    assert!(
        (request.length - 10.0).abs() < 0.01,
        "beam near boundary should have short length, got {}",
        request.length
    );
}

// ── Behavior 17: fire() computes beam length in negative direction ──

#[test]
fn fire_computes_beam_length_to_bottom_boundary() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Position2D(Vec2::new(0.0, 200.0)),
            Velocity2D(Vec2::new(0.0, -400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    let request = results[0];
    assert!(
        (request.direction.x - 0.0).abs() < 0.01,
        "direction x should be 0.0"
    );
    assert!(
        (request.direction.y - (-1.0)).abs() < 0.01,
        "direction y should be -1.0, got {}",
        request.direction.y
    );
    // bottom = -300.0, entity at y=200 -> distance = 500.0
    assert!(
        (request.length - 500.0).abs() < 0.01,
        "beam length should be 500.0, got {}",
        request.length
    );
    assert!(
        (request.origin.y - 200.0).abs() < f32::EPSILON,
        "origin y should be 200.0"
    );
}

// ── Behavior 18: fire() handles diagonal velocity ──

#[test]
fn fire_handles_diagonal_velocity_direction() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(300.0, 300.0))))
        .id();

    fire(entity, 1.0, 30.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    let request = results[0];
    // Normalized (300, 300) -> approximately (0.707, 0.707)
    let expected_dir = Vec2::new(300.0, 300.0).normalize();
    assert!(
        (request.direction.x - expected_dir.x).abs() < 0.01,
        "direction x should be ~0.707, got {}",
        request.direction.x
    );
    assert!(
        (request.direction.y - expected_dir.y).abs() < 0.01,
        "direction y should be ~0.707, got {}",
        request.direction.y
    );
    assert!(
        (request.half_width - 15.0).abs() < f32::EPSILON,
        "half_width should be 15.0, got {}",
        request.half_width
    );
    // Beam should extend to whichever boundary is hit first along diagonal
    // From (0,0) at 45 degrees: right=400 -> t_x = 400/0.707 ~ 565.7
    //                             top=300 -> t_y = 300/0.707 ~ 424.3
    // min(565.7, 424.3) ~ 424.26
    assert!(
        (request.length - 424.26).abs() < 1.0,
        "beam length should be ~424.26 (top boundary hit first at 45 degrees), got {}",
        request.length
    );
}

// ── Behavior 20: fire() with missing Velocity2D defaults to Vec2::Y ──

#[test]
fn fire_with_missing_velocity_defaults_direction_to_y() {
    let mut world = piercing_beam_fire_world();

    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "request should be spawned even without Velocity2D"
    );

    let request = results[0];
    assert!(
        (request.direction.x - 0.0).abs() < 0.01,
        "missing velocity should default direction x to 0.0"
    );
    assert!(
        (request.direction.y - 1.0).abs() < 0.01,
        "missing velocity should default direction y to 1.0 (Vec2::Y)"
    );
    // Beam extends from (0,0) upward to top=300
    assert!(
        (request.length - 300.0).abs() < 0.01,
        "beam should extend to top boundary, got {}",
        request.length
    );
}

#[test]
fn fire_with_no_transform_and_no_velocity_defaults_both() {
    let mut world = piercing_beam_fire_world();

    let entity = world.spawn_empty().id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "request should be spawned");

    let request = results[0];
    assert!(
        (request.origin.x).abs() < f32::EPSILON,
        "origin should default to 0.0 x"
    );
    assert!(
        (request.origin.y).abs() < f32::EPSILON,
        "origin should default to 0.0 y"
    );
    assert!(
        (request.direction.y - 1.0).abs() < 0.01,
        "direction should default to Vec2::Y"
    );
}

// ── Behavior 21: fire() with zero velocity defaults to Vec2::Y ──

#[test]
fn fire_with_zero_velocity_defaults_direction_to_y() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO)))
        .id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "request should be spawned even with zero velocity"
    );

    let request = results[0];
    assert!(
        (request.direction.y - 1.0).abs() < 0.01,
        "zero velocity should default direction to Vec2::Y, got direction ({}, {})",
        request.direction.x,
        request.direction.y
    );
}

// ── Behavior 22: fire() with no Transform defaults origin to Vec2::ZERO ──

#[test]
fn fire_with_no_transform_defaults_origin_to_zero() {
    let mut world = piercing_beam_fire_world();

    let entity = world.spawn(Velocity2D(Vec2::new(0.0, 400.0))).id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    let request = results[0];
    assert!(
        (request.origin.x).abs() < f32::EPSILON,
        "origin x should default to 0.0"
    );
    assert!(
        (request.origin.y).abs() < f32::EPSILON,
        "origin y should default to 0.0"
    );
    assert!(
        (request.direction.y - 1.0).abs() < 0.01,
        "direction should be Vec2::Y"
    );
    // From (0,0) upward to top=300
    assert!(
        (request.length - 300.0).abs() < 0.01,
        "length should be 300.0"
    );
}

// ── Behavior 16 extra: request entity has CleanupOnNodeExit ──

#[test]
fn fire_request_entity_has_cleanup_on_node_exit() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(0.0, 400.0))))
        .id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<PiercingBeamRequest>>();
    let request_entity = query.iter(&world).next().expect("request should exist");

    assert!(
        world.get::<CleanupOnNodeExit>(request_entity).is_some(),
        "PiercingBeamRequest entity should have CleanupOnNodeExit"
    );
}

// ── Behavior 23: reverse() is a no-op ──

#[test]
fn reverse_is_noop() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(10.0, 20.0, 0.0)).id();

    reverse(entity, "", &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "entity should still exist after no-op reverse"
    );
}

#[test]
fn reverse_on_empty_entity_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    reverse(entity, "", &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "empty entity should still exist after no-op reverse"
    );
}

// ── Regression: fire() reads Position2D for origin, NOT Transform ──

#[test]
fn fire_reads_position2d_not_transform_for_beam_origin() {
    let mut world = piercing_beam_fire_world();

    // Position2D and Transform deliberately diverge — fire() must use Position2D.
    let entity = world
        .spawn((
            Position2D(Vec2::new(100.0, 150.0)),
            Transform::from_xyz(-50.0, -75.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected one PiercingBeamRequest entity");

    let request = results[0];
    assert!(
        (request.origin.x - 100.0).abs() < f32::EPSILON,
        "origin x should be 100.0 (from Position2D), got {}",
        request.origin.x
    );
    assert!(
        (request.origin.y - 150.0).abs() < f32::EPSILON,
        "origin y should be 150.0 (from Position2D), got {}",
        request.origin.y
    );
    // top=300, Position2D.y=150 -> beam length = 300 - 150 = 150
    assert!(
        (request.length - 150.0).abs() < 0.01,
        "beam length should be 150.0 (from Position2D.y=150 to top=300), got {}",
        request.length
    );
}

#[test]
fn fire_reads_position2d_zero_not_transform_for_beam_origin() {
    let mut world = piercing_beam_fire_world();

    // Edge case: Position2D at zero with divergent Transform — must use Position2D.
    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            Transform::from_xyz(200.0, 200.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected one PiercingBeamRequest entity");

    let request = results[0];
    assert!(
        (request.origin.x).abs() < f32::EPSILON,
        "origin x should be 0.0 (from Position2D), NOT 200.0 (from Transform), got {}",
        request.origin.x
    );
    assert!(
        (request.origin.y).abs() < f32::EPSILON,
        "origin y should be 0.0 (from Position2D), NOT 200.0 (from Transform), got {}",
        request.origin.y
    );
    // top=300, Position2D.y=0 -> beam length = 300
    assert!(
        (request.length - 300.0).abs() < 0.01,
        "beam length should be 300.0 (from y=0 to top=300), got {}",
        request.length
    );
}

// ── Regression: fire() falls back to Vec2::ZERO when Position2D absent (NOT to Transform) ──

#[test]
fn fire_without_position2d_falls_back_to_zero_not_transform() {
    let mut world = piercing_beam_fire_world();

    // Entity has Transform but NO Position2D — fire() must use Vec2::ZERO, NOT Transform.
    let entity = world
        .spawn((
            Transform::from_xyz(100.0, 200.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected one PiercingBeamRequest entity");

    let request = results[0];
    assert!(
        (request.origin.x).abs() < f32::EPSILON,
        "origin x should be 0.0 (Vec2::ZERO fallback), NOT 100.0 (Transform), got {}",
        request.origin.x
    );
    assert!(
        (request.origin.y).abs() < f32::EPSILON,
        "origin y should be 0.0 (Vec2::ZERO fallback), NOT 200.0 (Transform), got {}",
        request.origin.y
    );
    // top=300, origin at y=0 -> beam length = 300
    assert!(
        (request.length - 300.0).abs() < 0.01,
        "beam length should be 300.0 (from y=0 to top=300), got {}",
        request.length
    );
}

#[test]
fn fire_with_only_transform_no_position2d_no_velocity_falls_back_to_zero() {
    let mut world = piercing_beam_fire_world();

    // Edge case: ONLY Transform (no Position2D, no Velocity2D) — origin must be Vec2::ZERO.
    let entity = world.spawn(Transform::from_xyz(100.0, 200.0, 0.0)).id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected one PiercingBeamRequest entity");

    let request = results[0];
    assert!(
        (request.origin.x).abs() < f32::EPSILON,
        "origin x should be 0.0 (Vec2::ZERO fallback), NOT 100.0 (Transform), got {}",
        request.origin.x
    );
    assert!(
        (request.origin.y).abs() < f32::EPSILON,
        "origin y should be 0.0 (Vec2::ZERO fallback), NOT 200.0 (Transform), got {}",
        request.origin.y
    );
    // No velocity -> direction defaults to Vec2::Y, beam from y=0 to top=300 -> length = 300
    assert!(
        (request.direction.y - 1.0).abs() < 0.01,
        "missing velocity should default direction to Vec2::Y"
    );
    assert!(
        (request.length - 300.0).abs() < 0.01,
        "beam length should be 300.0 (from y=0 to top=300), got {}",
        request.length
    );
}

// ── Regression: fire() with Position2D but no Transform works correctly ──

#[test]
fn fire_with_position2d_but_no_transform_uses_position2d() {
    let mut world = piercing_beam_fire_world();

    // Entity has Position2D and Velocity2D but NO Transform.
    let entity = world
        .spawn((
            Position2D(Vec2::new(50.0, -100.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected one PiercingBeamRequest entity");

    let request = results[0];
    assert!(
        (request.origin.x - 50.0).abs() < f32::EPSILON,
        "origin x should be 50.0 (from Position2D), got {}",
        request.origin.x
    );
    assert!(
        (request.origin.y - (-100.0)).abs() < f32::EPSILON,
        "origin y should be -100.0 (from Position2D), got {}",
        request.origin.y
    );
    assert!(
        (request.direction.x - 0.0).abs() < 0.01,
        "direction x should be 0.0, got {}",
        request.direction.x
    );
    assert!(
        (request.direction.y - 1.0).abs() < 0.01,
        "direction y should be 1.0, got {}",
        request.direction.y
    );
    // top=300, Position2D.y=-100 -> beam length = 300 - (-100) = 400
    assert!(
        (request.length - 400.0).abs() < 0.01,
        "beam length should be 400.0 (from y=-100 to top=300), got {}",
        request.length
    );
}

#[test]
fn fire_with_position2d_near_top_boundary_produces_short_beam() {
    let mut world = piercing_beam_fire_world();

    // Edge case: Position2D very close to top boundary.
    let entity = world
        .spawn((
            Position2D(Vec2::new(0.0, 299.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected one PiercingBeamRequest entity");

    let request = results[0];
    assert!(
        (request.origin.y - 299.0).abs() < f32::EPSILON,
        "origin y should be 299.0 (from Position2D), got {}",
        request.origin.y
    );
    // top=300, Position2D.y=299 -> beam length = 300 - 299 = 1.0
    assert!(
        (request.length - 1.0).abs() < 0.01,
        "beam length should be 1.0 (from y=299 to top=300), got {}",
        request.length
    );
}

// -- Section G: EffectSourceChip attribution tests ───────────────────

#[test]
fn fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(0.0, 400.0))))
        .id();

    fire(entity, 2.0, 10.0, "laser", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0,
        Some("laser".to_string()),
        "spawned PiercingBeamRequest should have EffectSourceChip(Some(\"laser\"))"
    );
}

#[test]
fn fire_stores_effect_source_chip_none_with_empty_chip_name() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(0.0, 400.0))))
        .id();

    fire(entity, 2.0, 10.0, "", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0, None,
        "empty source_chip should produce EffectSourceChip(None)"
    );
}
