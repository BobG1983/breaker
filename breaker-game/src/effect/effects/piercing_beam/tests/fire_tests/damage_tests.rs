use super::super::helpers::*;

// ── Behavior 19: fire() applies damage_mult ──

#[test]
fn fire_applies_damage_mult_to_base_bolt_damage() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(0.0, 400.0))))
        .id();

    fire(entity, 3.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 3.0;
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
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(0.0, 400.0))))
        .id();

    fire(entity, 0.0, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    assert!(
        (results[0].damage - 0.0).abs() < f32::EPSILON,
        "damage_mult=0.0 should produce damage 0.0, got {}",
        results[0].damage
    );
}

// ── Damage scaling: fire() includes ActiveDamageBoosts in pre-computed damage ──

#[test]
fn fire_scales_damage_by_effective_damage_multiplier() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
            crate::effect::effects::damage_boost::ActiveDamageBoosts(vec![2.0]),
        ))
        .id();

    fire(entity, 1.5, 20.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected one PiercingBeamRequest entity");

    // damage = DEFAULT_BOLT_BASE_DAMAGE * damage_mult * EDM = 10.0 * 1.5 * 2.0 = 30.0
    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 1.5 * 2.0;
    assert!(
        (results[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (10.0 * 1.5 * 2.0), got {}",
        expected_damage,
        results[0].damage
    );
}

// ── Behavior 23: fire() snapshots BoltBaseDamage from source bolt ──

#[test]
fn fire_snapshots_bolt_base_damage_from_source_bolt() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Position2D(Vec2::new(50.0, 50.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
            crate::bolt::components::BoltBaseDamage(20.0),
        ))
        .id();

    fire(entity, 2.0, 10.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    // damage = BoltBaseDamage(20.0) * damage_mult(2.0) * EDM(1.0) = 40.0
    let expected_damage = 20.0 * 2.0;
    assert!(
        (results[0].damage - expected_damage).abs() < f32::EPSILON,
        "damage should be {} (20.0 * 2.0), got {}",
        expected_damage,
        results[0].damage
    );
}

// ── Behavior 23 edge case: source has no BoltBaseDamage -- falls back to DEFAULT ──

#[test]
fn fire_falls_back_to_default_bolt_base_damage_when_source_has_none() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Position2D(Vec2::new(50.0, 50.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    fire(entity, 2.0, 10.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    // damage = DEFAULT_BOLT_BASE_DAMAGE(10.0) * 2.0 = 20.0
    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 2.0;
    assert!(
        (results[0].damage - expected_damage).abs() < f32::EPSILON,
        "damage should be {} (10.0 * 2.0), got {}",
        expected_damage,
        results[0].damage
    );
}

// ── Behavior 24: fire() with BoltBaseDamage and ActiveDamageBoosts stacks ──

#[test]
fn fire_with_bolt_base_damage_and_active_damage_boosts_stacks() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
            crate::bolt::components::BoltBaseDamage(15.0),
            crate::effect::effects::damage_boost::ActiveDamageBoosts(vec![2.0]),
        ))
        .id();

    fire(entity, 1.5, 10.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    // damage = BoltBaseDamage(15.0) * damage_mult(1.5) * EDM(2.0) = 45.0
    let expected_damage = 15.0 * 1.5 * 2.0;
    assert!(
        (results[0].damage - expected_damage).abs() < f32::EPSILON,
        "damage should be {} (15.0 * 1.5 * 2.0), got {}",
        expected_damage,
        results[0].damage
    );
}

// ── Behavior 24 edge case: BoltBaseDamage(10.0) and no ActiveDamageBoosts -- identical to old ──

#[test]
fn fire_with_bolt_base_damage_10_and_no_boosts_identical_to_old() {
    let mut world = piercing_beam_fire_world();

    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
            crate::bolt::components::BoltBaseDamage(10.0),
        ))
        .id();

    fire(entity, 1.5, 10.0, "", &mut world);

    let mut query = world.query::<&PiercingBeamRequest>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1);

    // damage = 10.0 * 1.5 * 1.0 = 15.0
    let expected_damage = 10.0 * 1.5;
    assert!(
        (results[0].damage - expected_damage).abs() < f32::EPSILON,
        "damage should be {} (10.0 * 1.5 * 1.0), got {}",
        expected_damage,
        results[0].damage
    );
}
