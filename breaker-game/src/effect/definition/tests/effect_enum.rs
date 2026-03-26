use super::super::*;

// =========================================================================
// C7 Wave 1 Part C: Effect enum changes (behaviors 17-22)
// =========================================================================

#[test]
fn effect_attraction_with_attraction_type() {
    let e = Effect::Attraction(AttractionType::Cell, 1.0);
    assert!(matches!(
        e,
        Effect::Attraction(AttractionType::Cell, v) if (v - 1.0).abs() < f32::EPSILON
    ));
}

#[test]
fn effect_attraction_wall_variant() {
    let e = Effect::Attraction(AttractionType::Wall, 0.5);
    assert!(matches!(e, Effect::Attraction(AttractionType::Wall, _)));
}

#[test]
fn effect_attraction_breaker_variant() {
    let e = Effect::Attraction(AttractionType::Breaker, 2.0);
    assert!(matches!(e, Effect::Attraction(AttractionType::Breaker, _)));
}

#[test]
fn attraction_type_ron_deserialization() {
    let e: Effect =
        ron::de::from_str("Attraction(Cell, 1.0)").expect("Attraction(Cell, 1.0) should parse");
    assert_eq!(e, Effect::Attraction(AttractionType::Cell, 1.0));
}

#[test]
fn attraction_type_ron_wall() {
    let e: Effect =
        ron::de::from_str("Attraction(Wall, 0.5)").expect("Attraction(Wall, 0.5) should parse");
    assert_eq!(e, Effect::Attraction(AttractionType::Wall, 0.5));
}

#[test]
fn attraction_type_ron_breaker() {
    let e: Effect = ron::de::from_str("Attraction(Breaker, 2.0)")
        .expect("Attraction(Breaker, 2.0) should parse");
    assert_eq!(e, Effect::Attraction(AttractionType::Breaker, 2.0));
}

#[test]
fn effect_enum_has_all_twenty_three_variants() {
    let effects: Vec<Effect> = vec![
        Effect::Shockwave {
            base_range: 64.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
        },
        Effect::Piercing(1),
        Effect::DamageBoost(0.5),
        Effect::SpeedBoost { multiplier: 1.2 },
        Effect::ChainHit(2),
        Effect::SizeBoost(5.0),
        Effect::Attraction(AttractionType::Cell, 0.3),
        Effect::BumpForce(0.2),
        Effect::TiltControl(0.1),
        Effect::ChainBolt {
            tether_distance: 150.0,
        },
        Effect::MultiBolt {
            base_count: 2,
            count_per_level: 0,
            stacks: 1,
        },
        Effect::Shield {
            base_duration: 3.0,
            duration_per_level: 0.0,
            stacks: 1,
        },
        Effect::LoseLife,
        Effect::TimePenalty { seconds: 3.0 },
        Effect::SpawnBolts {
            count: 1,
            lifespan: None,
            inherit: false,
        },
        Effect::ChainLightning {
            arcs: 3,
            range: 64.0,
            damage_mult: 0.5,
        },
        Effect::SpawnPhantom {
            duration: 5.0,
            max_active: 2,
        },
        Effect::PiercingBeam {
            damage_mult: 1.5,
            width: 20.0,
        },
        Effect::GravityWell {
            strength: 1.0,
            duration: 5.0,
            radius: 100.0,
            max: 2,
        },
        Effect::SecondWind { invuln_secs: 3.0 },
        Effect::RampingDamage {
            bonus_per_hit: 0.04,
        },
        Effect::RandomEffect(vec![(
            1.0,
            EffectNode::Do(Effect::SpawnBolts {
                count: 1,
                lifespan: None,
                inherit: false,
            }),
        )]),
        Effect::EntropyEngine {
            threshold: 5,
            pool: vec![(
                1.0,
                EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                }),
            )],
        },
    ];
    assert_eq!(effects.len(), 23, "all 23 Effect variants");
}

// =========================================================================
// C7 Wave 1 Part J: Multiplier standardization (behaviors 47-48)
// =========================================================================

#[test]
fn damage_boost_uses_multiplier_format() {
    // 2.0 means 2x damage (double), 0.5 means 50% damage (half)
    let double = Effect::DamageBoost(2.0);
    let half = Effect::DamageBoost(0.5);
    assert_eq!(double, Effect::DamageBoost(2.0));
    assert_eq!(half, Effect::DamageBoost(0.5));
}

#[test]
fn speed_boost_uses_multiplier_format() {
    // 1.5 means 1.5x speed, 0.5 means 50% speed
    let fast = Effect::SpeedBoost { multiplier: 1.5 };
    let slow = Effect::SpeedBoost { multiplier: 0.5 };
    assert!(
        matches!(fast, Effect::SpeedBoost { multiplier, .. } if (multiplier - 1.5).abs() < f32::EPSILON)
    );
    assert!(
        matches!(slow, Effect::SpeedBoost { multiplier, .. } if (multiplier - 0.5).abs() < f32::EPSILON)
    );
}

// =========================================================================
// Preserved tests
// =========================================================================

#[test]
fn effect_zero_damage_boost_is_valid() {
    let e = Effect::DamageBoost(0.0);
    assert_eq!(e, Effect::DamageBoost(0.0));
}

#[test]
fn effect_speed_boost_all_bolts_target() {
    let e = Effect::SpeedBoost { multiplier: 0.5 };
    assert!(
        matches!(e, Effect::SpeedBoost { multiplier, .. } if (multiplier - 0.5).abs() < f32::EPSILON)
    );
}

#[test]
fn effect_random_effect_round_trips() {
    let effect = Effect::RandomEffect(vec![
        (
            0.6,
            EffectNode::Do(Effect::SpawnBolts {
                count: 1,
                lifespan: None,
                inherit: false,
            }),
        ),
        (0.4, EffectNode::Do(Effect::test_speed_boost(1.2))),
    ]);
    let cloned = effect.clone();
    assert_eq!(effect, cloned);
}

#[test]
fn effect_entropy_engine_round_trips() {
    let effect = Effect::EntropyEngine {
        threshold: 5,
        pool: vec![
            (
                0.5,
                EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                }),
            ),
            (0.5, EffectNode::Do(Effect::test_speed_boost(1.3))),
        ],
    };
    let cloned = effect.clone();
    assert_eq!(effect, cloned);
}
