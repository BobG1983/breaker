use bevy::prelude::*;

use super::definitions::*;

#[cfg(test)]
impl EffectKind {
    /// Build a test shockwave effect with the given base range.
    #[must_use]
    pub fn test_shockwave(base_range: f32) -> Self {
        Self::Shockwave {
            base_range,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
        }
    }
}

// -- Behavior 1: EffectKind::Pulse carries an interval field (serde round-trip) --

#[test]
fn pulse_serde_round_trip_with_explicit_interval() {
    let ron_str =
        "Pulse(base_range: 32.0, range_per_level: 8.0, stacks: 1, speed: 50.0, interval: 0.25)";
    let effect: EffectKind =
        ron::from_str(ron_str).expect("should deserialize Pulse with explicit interval");

    match &effect {
        EffectKind::Pulse {
            base_range,
            range_per_level,
            stacks,
            speed,
            interval,
        } => {
            assert!(
                (*base_range - 32.0).abs() < f32::EPSILON,
                "expected base_range 32.0, got {base_range}"
            );
            assert!(
                (*range_per_level - 8.0).abs() < f32::EPSILON,
                "expected range_per_level 8.0, got {range_per_level}"
            );
            assert_eq!(*stacks, 1, "expected stacks 1");
            assert!(
                (*speed - 50.0).abs() < f32::EPSILON,
                "expected speed 50.0, got {speed}"
            );
            assert!(
                (*interval - 0.25).abs() < f32::EPSILON,
                "expected interval 0.25, got {interval}"
            );
        }
        other => panic!("expected Pulse variant, got {other:?}"),
    }
}

#[test]
fn pulse_serde_default_interval_when_omitted() {
    let ron_str = "Pulse(base_range: 32.0, range_per_level: 8.0, stacks: 1, speed: 50.0)";
    let effect: EffectKind = ron::from_str(ron_str)
        .expect("should deserialize Pulse with omitted interval using serde default");

    match &effect {
        EffectKind::Pulse { interval, .. } => {
            assert!(
                (*interval - 0.5).abs() < f32::EPSILON,
                "expected default interval 0.5, got {interval}"
            );
        }
        other => panic!("expected Pulse variant, got {other:?}"),
    }
}

// -- Behavior 6: EffectKind::Attraction carries a max_force field (serde round-trip) --

#[test]
fn attraction_serde_round_trip_with_explicit_max_force() {
    let ron_str = "Attraction(attraction_type: Cell, force: 500.0, max_force: Some(300.0))";
    let effect: EffectKind =
        ron::from_str(ron_str).expect("should deserialize Attraction with explicit max_force");

    match &effect {
        EffectKind::Attraction {
            attraction_type,
            force,
            max_force,
        } => {
            assert_eq!(
                *attraction_type,
                AttractionType::Cell,
                "expected attraction_type Cell"
            );
            assert!(
                (*force - 500.0).abs() < f32::EPSILON,
                "expected force 500.0, got {force}"
            );
            assert_eq!(
                *max_force,
                Some(300.0),
                "expected max_force Some(300.0), got {max_force:?}"
            );
        }
        other => panic!("expected Attraction variant, got {other:?}"),
    }
}

#[test]
fn attraction_serde_default_max_force_when_omitted() {
    let ron_str = "Attraction(attraction_type: Cell, force: 500.0)";
    let effect: EffectKind = ron::from_str(ron_str)
        .expect("should deserialize Attraction with omitted max_force using serde default");

    match &effect {
        EffectKind::Attraction { max_force, .. } => {
            assert_eq!(
                *max_force, None,
                "expected default max_force None, got {max_force:?}"
            );
        }
        other => panic!("expected Attraction variant, got {other:?}"),
    }
}

// -- Section A: chip_attribution helper --

#[test]
fn chip_attribution_converts_empty_string_to_none() {
    let result = chip_attribution("");
    assert!(
        result.is_none(),
        "empty string should map to None, got {result:?}"
    );
}

#[test]
fn chip_attribution_converts_non_empty_string_to_some() {
    let result = chip_attribution("shockwave_chip");
    assert_eq!(
        result,
        Some("shockwave_chip".to_string()),
        "non-empty string should map to Some(...)"
    );
}

#[test]
fn chip_attribution_single_space_returns_some() {
    let result = chip_attribution(" ");
    assert_eq!(
        result,
        Some(" ".to_string()),
        "single space should map to Some, not None"
    );
}

#[test]
fn chip_attribution_single_char_returns_some() {
    let result = chip_attribution("a");
    assert_eq!(
        result,
        Some("a".to_string()),
        "single char should map to Some"
    );
}

// -- Section B: EffectKind::fire()/reverse() dispatch threading --

#[test]
fn effect_kind_fire_passes_source_chip_to_shockwave_spawned_entity_has_effect_source_chip() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    EffectKind::Shockwave {
        base_range: 24.0,
        range_per_level: 8.0,
        stacks: 1,
        speed: 50.0,
    }
    .fire(entity, "test_chip", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0,
        Some("test_chip".to_string()),
        "spawned shockwave should have EffectSourceChip(Some(\"test_chip\"))"
    );
}

#[test]
fn effect_kind_fire_passes_empty_source_chip_to_shockwave_spawned_entity_has_none() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    EffectKind::Shockwave {
        base_range: 24.0,
        range_per_level: 8.0,
        stacks: 1,
        speed: 50.0,
    }
    .fire(entity, "", &mut world);

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

#[test]
fn effect_kind_fire_passes_source_chip_to_explode_spawned_request_has_effect_source_chip() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(50.0, 75.0, 0.0)).id();

    EffectKind::Explode {
        range: 60.0,
        damage: 2.0,
    }
    .fire(entity, "explode_chip", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0,
        Some("explode_chip".to_string()),
        "spawned ExplodeRequest should have EffectSourceChip(Some(\"explode_chip\"))"
    );
}

#[test]
fn effect_kind_fire_passes_empty_source_chip_to_explode_spawned_request_has_none() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(50.0, 75.0, 0.0)).id();

    EffectKind::Explode {
        range: 60.0,
        damage: 2.0,
    }
    .fire(entity, "", &mut world);

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

#[test]
fn effect_kind_reverse_accepts_source_chip_without_panic_for_non_damage_effect() {
    use crate::effect::effects::speed_boost::ActiveSpeedBoosts;

    let mut world = World::new();
    let entity = world.spawn(ActiveSpeedBoosts(vec![1.5])).id();

    EffectKind::SpeedBoost { multiplier: 1.5 }.reverse(entity, "", &mut world);

    let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
    assert!(
        active.0.is_empty(),
        "reverse should remove the boost entry, got {:?}",
        active.0
    );
}

#[test]
fn effect_kind_reverse_with_non_empty_source_chip_same_behavior_for_non_damage_effect() {
    use crate::effect::effects::speed_boost::ActiveSpeedBoosts;

    let mut world = World::new();
    let entity = world.spawn(ActiveSpeedBoosts(vec![1.5])).id();

    EffectKind::SpeedBoost { multiplier: 1.5 }.reverse(entity, "any_chip", &mut world);

    let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
    assert!(
        active.0.is_empty(),
        "source_chip should be ignored for non-damage reverse, got {:?}",
        active.0
    );
}

// -- MirrorProtocol serde round-trip --

#[test]
fn mirror_protocol_serde_round_trip_inherit_true() {
    let ron_str = "MirrorProtocol(inherit: true)";
    let effect: EffectKind =
        ron::from_str(ron_str).expect("should deserialize MirrorProtocol(inherit: true)");

    assert_eq!(
        effect,
        EffectKind::MirrorProtocol { inherit: true },
        "deserialized MirrorProtocol should match expected variant"
    );
}

#[test]
fn mirror_protocol_serde_round_trip_inherit_false() {
    let ron_str = "MirrorProtocol(inherit: false)";
    let effect: EffectKind =
        ron::from_str(ron_str).expect("should deserialize MirrorProtocol(inherit: false)");

    assert_eq!(
        effect,
        EffectKind::MirrorProtocol { inherit: false },
        "deserialized MirrorProtocol edge case should match"
    );
}

// -- MirrorProtocol EffectKind dispatch --

#[test]
fn effect_kind_fire_dispatches_mirror_protocol_spawns_mirrored_bolts() {
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    use crate::{
        bolt::{
            components::{Bolt, ExtraBolt, ImpactSide, LastImpact},
            definition::BoltDefinition,
            registry::BoltRegistry,
        },
        shared::rng::GameRng,
    };

    let mut world = World::new();
    world.insert_resource(GameRng::default());
    let mut bolt_registry = BoltRegistry::default();
    bolt_registry.insert(
        "Bolt".to_string(),
        BoltDefinition {
            name: "Bolt".to_string(),
            base_speed: 720.0,
            min_speed: 360.0,
            max_speed: 1440.0,
            radius: 14.0,
            base_damage: 10.0,
            effects: vec![],
            color_rgb: [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
            min_radius: None,
            max_radius: None,
        },
    );
    world.insert_resource(bolt_registry);

    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(60.0, 250.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
            LastImpact {
                position: Vec2::new(50.0, 200.0),
                side: ImpactSide::Top,
            },
        ))
        .id();

    EffectKind::MirrorProtocol { inherit: true }.fire(bolt_entity, "mirror_protocol", &mut world);

    let mut query =
        world.query_filtered::<(&Position2D, &Velocity2D), (With<Bolt>, With<ExtraBolt>)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "EffectKind::fire dispatch should spawn 1 mirrored bolt"
    );

    let (pos, vel) = results[0];
    assert_eq!(
        pos.0,
        Vec2::new(40.0, 250.0),
        "mirrored bolt position via dispatch"
    );
    assert_eq!(
        vel.0,
        Vec2::new(-100.0, 400.0),
        "mirrored bolt velocity via dispatch"
    );
}

#[test]
fn effect_kind_reverse_dispatches_mirror_protocol_noop() {
    use crate::{bolt::components::Bolt, shared::rng::GameRng};

    let mut world = World::new();
    world.insert_resource(GameRng::default());

    let bolt_entity = world.spawn(Bolt).id();

    // Should not panic -- noop behavior
    EffectKind::MirrorProtocol { inherit: true }.reverse(
        bolt_entity,
        "mirror_protocol",
        &mut world,
    );

    // Verify no entities were despawned (bolt_entity still exists)
    assert!(
        world.get_entity(bolt_entity).is_ok(),
        "reverse should not despawn any entities"
    );
}
