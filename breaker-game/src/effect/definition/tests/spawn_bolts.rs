use super::super::*;

// =========================================================================
// SpawnBolts-specific tests
// =========================================================================

#[test]
fn effect_spawn_bolts_full_construction() {
    let e = Effect::SpawnBolts {
        count: 2,
        lifespan: Some(5.0),
        inherit: true,
    };
    match e {
        Effect::SpawnBolts {
            count,
            lifespan,
            inherit,
        } => {
            assert_eq!(count, 2);
            assert_eq!(lifespan, Some(5.0));
            assert!(inherit);
        }
        other => panic!("expected SpawnBolts, got {other:?}"),
    }
}

#[test]
fn effect_spawn_bolts_default_values() {
    let e = Effect::SpawnBolts {
        count: 1,
        lifespan: None,
        inherit: false,
    };
    match e {
        Effect::SpawnBolts {
            count,
            lifespan,
            inherit,
        } => {
            assert_eq!(count, 1);
            assert!(lifespan.is_none());
            assert!(!inherit);
        }
        other => panic!("expected SpawnBolts, got {other:?}"),
    }
}

#[test]
fn effect_spawn_bolts_ron_with_serde_defaults() {
    let ron_str = "SpawnBolts(count: 3)";
    let e: Effect =
        ron::de::from_str(ron_str).expect("SpawnBolts with partial fields should parse");
    match e {
        Effect::SpawnBolts {
            count,
            lifespan,
            inherit,
        } => {
            assert_eq!(count, 3, "count should be 3");
            assert!(lifespan.is_none(), "lifespan should default to None");
            assert!(!inherit, "inherit should default to false");
        }
        other => panic!("expected SpawnBolts, got {other:?}"),
    }
}

#[test]
fn effect_spawn_bolts_ron_full_form() {
    let ron_str = "SpawnBolts(count: 2, lifespan: Some(5.0), inherit: true)";
    let e: Effect = ron::de::from_str(ron_str).expect("SpawnBolts full form should parse");
    assert_eq!(
        e,
        Effect::SpawnBolts {
            count: 2,
            lifespan: Some(5.0),
            inherit: true,
        }
    );
}

#[test]
fn effect_spawn_bolts_ron_bare_parens_defaults_count_to_one() {
    let ron_str = "SpawnBolts()";
    let e: Effect = ron::de::from_str(ron_str).expect("SpawnBolts() bare parens should parse");
    match e {
        Effect::SpawnBolts {
            count,
            lifespan,
            inherit,
        } => {
            assert_eq!(count, 1, "count should default to 1");
            assert!(lifespan.is_none(), "lifespan should default to None");
            assert!(!inherit, "inherit should default to false");
        }
        other => panic!("expected SpawnBolts, got {other:?}"),
    }
}

#[test]
fn effect_spawn_bolts_ron_count_override() {
    let ron_str = "SpawnBolts(count: 5)";
    let e: Effect = ron::de::from_str(ron_str).expect("SpawnBolts(count: 5) should parse");
    match e {
        Effect::SpawnBolts { count, .. } => {
            assert_eq!(count, 5, "count should be overridden to 5");
        }
        other => panic!("expected SpawnBolts, got {other:?}"),
    }
}
