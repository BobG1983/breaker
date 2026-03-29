use super::*;

// ── Behavior 16: fire() spawns PiercingBeamRequest with correct beam geometry ──

#[test]
fn fire_spawns_request_with_correct_upward_beam_geometry() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, &mut world);

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
            Transform::from_xyz(0.0, 290.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, &mut world);

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
            Transform::from_xyz(0.0, 200.0, 0.0),
            Velocity2D(Vec2::new(0.0, -400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, &mut world);

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
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity2D(Vec2::new(300.0, 300.0)),
        ))
        .id();

    fire(entity, 1.0, 30.0, &mut world);

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

// ── Behavior 19: fire() applies damage_mult ──

#[test]
fn fire_applies_damage_mult_to_base_bolt_damage() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 3.0, 20.0, &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    let expected_damage = BASE_BOLT_DAMAGE * 3.0;
    assert!(
        (results[0].damage - expected_damage).abs() < f32::EPSILON,
        "damage should be {}, got {}",
        expected_damage,
        results[0].damage
    );
}

#[test]
fn fire_with_zero_damage_mult_produces_zero_damage() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 0.0, 20.0, &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    assert!(
        (results[0].damage - 0.0).abs() < f32::EPSILON,
        "damage_mult=0.0 should produce damage 0.0, got {}",
        results[0].damage
    );
}

// ── Behavior 20: fire() with missing Velocity2D defaults to Vec2::Y ──

#[test]
fn fire_with_missing_velocity_defaults_direction_to_y() {
    let mut world = piercing_beam_fire_world();

    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    fire(entity, 1.0, 20.0, &mut world);

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

    fire(entity, 1.0, 20.0, &mut world);

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
        .spawn((Transform::from_xyz(0.0, 0.0, 0.0), Velocity2D(Vec2::ZERO)))
        .id();

    fire(entity, 1.0, 20.0, &mut world);

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

    fire(entity, 1.0, 20.0, &mut world);

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
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 1.0, 20.0, &mut world);

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

    reverse(entity, &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "entity should still exist after no-op reverse"
    );
}

#[test]
fn reverse_on_empty_entity_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    reverse(entity, &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "empty entity should still exist after no-op reverse"
    );
}
