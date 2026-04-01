use super::super::*;

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

    // damage = BASE_BOLT_DAMAGE * damage_mult * EDM = 10.0 * 1.5 * 2.0 = 30.0
    let expected_damage = BASE_BOLT_DAMAGE * 1.5 * 2.0;
    assert!(
        (results[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (10.0 * 1.5 * 2.0), got {}",
        expected_damage,
        results[0].damage
    );
}
